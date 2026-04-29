use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaModelTag {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaTagsResponse {
    pub models: Vec<OllamaModelTag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaStatus {
    pub is_installed: bool,
    pub is_running: bool,
    pub version: Option<String>,
}

/// Fallback check: uses shell to detect if `ollama -v` is installed
#[tauri::command]
pub async fn detect_ollama(app: AppHandle) -> Result<bool, String> {
    let shell = app.shell();
    let output = shell.command("ollama").args(["-v"]).output().await;
    match output {
        Ok(out) => Ok(out.status.success()),
        Err(_) => Ok(false),
    }
}

/// Primary check: pings localhost:11434 to check if exactly Ollama is listening.
/// Returns status about both HTTP responsiveness and CLI installation.
#[tauri::command]
pub async fn get_ollama_status(app: AppHandle) -> Result<OllamaStatus, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    // Primary check: HTTP ping
    let is_running = match client.get("http://localhost:11434").send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    };

    // Secondary Check: CLI version tracking (only useful if we want version, otherwise running is enough)
    let shell = app.shell();
    let mut is_installed = false;
    let mut version = None;
    if let Ok(output) = shell.command("ollama").args(["-v"]).output().await {
        if output.status.success() {
            is_installed = true;
            if let Ok(v_str) = String::from_utf8(output.stdout) {
                version = Some(v_str.trim().to_string());
            }
        }
    }

    Ok(OllamaStatus {
        is_installed,
        is_running,
        version,
    })
}

/// Fetches local inventory via /api/tags
#[tauri::command]
pub async fn list_local_models() -> Result<Vec<String>, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    match client.get("http://localhost:11434/api/tags").send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(payload) = resp.json::<OllamaTagsResponse>().await {
                let tags = payload.models.into_iter().map(|m| m.name).collect();
                return Ok(tags);
            }
        }
        _ => {}
    }

    Ok(vec![])
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OllamaPullProgress {
    pub status: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}

/// Takes a single upstream_tag (strictly derived from registry[runtime="ollama"].upstream_tag)
#[tauri::command]
pub async fn pull_model(app: AppHandle, upstream_tag: String) -> Result<(), String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(3600)) // pulling can take a long time
        .build()
        .map_err(|e| e.to_string())?;

    let req_body = serde_json::json!({
        "model": upstream_tag,
        "stream": true
    });

    let mut response = client.post("http://localhost:11434/api/pull")
        .json(&req_body)
        .send()
        .await
        .map_err(|e| format!("Failed to initiate pull request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama pull rejected: {}", response.status()));
    }

    while let Ok(Some(chunk)) = response.chunk().await {
        if let Ok(text) = String::from_utf8(chunk.to_vec()) {
            for line in text.lines() {
                if line.trim().is_empty() { continue; }
                if let Ok(progress) = serde_json::from_str::<OllamaPullProgress>(line) {
                    app.emit("ollama-pull-progress", progress).unwrap_or(());
                }
            }
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaGenerateResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
}

/// Executes a bare-minimum inference smoke test. (Bypasses AIP/Arbitration for purely technical edge activation test).
#[tauri::command]
pub async fn run_smoke_inference(upstream_tag: String) -> Result<String, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let req_body = serde_json::json!({
        "model": upstream_tag,
        "prompt": "You are a local edge node. Reply simply 'OK, edge node is operational.'",
        "stream": false
    });

    let response = client.post("http://localhost:11434/api/generate")
        .json(&req_body)
        .send()
        .await
        .map_err(|e| format!("Generate request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Generate rejected: {}", response.status()));
    }

    let result = response.json::<OllamaGenerateResponse>().await
        .map_err(|e| e.to_string())?;

    Ok(result.response)
}
