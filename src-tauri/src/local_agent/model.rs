use futures_util::StreamExt;
use serde::Deserialize;
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::Duration,
};

pub struct ModelProcess(Child);
impl ModelProcess {
    pub fn installed_binary() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        let candidates = vec![
            PathBuf::from("/opt/homebrew/bin/ollama"),
            PathBuf::from("/usr/local/bin/ollama"),
            PathBuf::from("/Applications/Ollama.app/Contents/Resources/ollama"),
        ];
        #[cfg(target_os = "windows")]
        let candidates = vec![std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_default()
            .join("Programs/Ollama/ollama.exe")];
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let candidates = vec![
            PathBuf::from("/usr/local/bin/ollama"),
            PathBuf::from("/usr/bin/ollama"),
        ];
        candidates
            .into_iter()
            .find(|p| p.is_absolute() && p.is_file())
    }

    pub fn start() -> Result<Self, String> {
        let listener = std::net::TcpListener::bind("127.0.0.1:19435").map_err(|_| {
            "Локальний порт пілота зайнятий. Чужий процес не буде використано.".to_string()
        })?;
        let binary = Self::installed_binary()
            .ok_or("Ollama не встановлена у стандартному місці. Огляд папки працює без моделі.")?;
        let mut command = Command::new(binary);
        command
            .arg("serve")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("OLLAMA_") {
                command.env_remove(key);
            }
        }
        command
            .env("OLLAMA_HOST", "127.0.0.1:19435")
            .env("OLLAMA_NO_CLOUD", "1")
            .env("OLLAMA_KEEP_ALIVE", "0")
            .env("OLLAMA_MAX_LOADED_MODELS", "1")
            .env("OLLAMA_NUM_PARALLEL", "1");
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        drop(listener);
        command
            .spawn()
            .map(Self)
            .map_err(|_| "Не вдалося запустити локальну модель.".into())
    }
    pub fn alive(&mut self) -> bool {
        matches!(self.0.try_wait(), Ok(None))
    }
}
impl Drop for ModelProcess {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(None)) { return; }
        // Only the process tree created by this pilot is terminated.
        #[cfg(unix)]
        {
            let _ = Command::new("/bin/kill")
                .args(["-TERM", "--", &format!("-{}", self.0.id())])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        #[cfg(target_os = "windows")]
        {
            if let Some(root) = std::env::var_os("SystemRoot") {
                let _ = Command::new(PathBuf::from(root).join("System32/taskkill.exe"))
                    .args(["/PID", &self.0.id().to_string(), "/T", "/F"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[derive(Deserialize)]
struct Tags {
    models: Vec<Tag>,
}
#[derive(Deserialize)]
struct Tag {
    name: String,
    #[serde(default)]
    remote_host: String,
    #[serde(default)]
    remote_model: String,
    details: Details,
}
#[derive(Deserialize)]
struct Details {
    family: String,
}

pub async fn discover(process: &mut ModelProcess) -> Result<Vec<(String, String)>, String> {
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|_| "Локальна модель недоступна.")?;
    for _ in 0..40 {
        if !process.alive() {
            return Err("Локальний процес моделі завершився.".into());
        }
        if let Ok(response) = client.get("http://127.0.0.1:19435/api/tags").send().await {
            if response.status().is_success() {
                let mut stream = response.bytes_stream();
                let mut bytes = Vec::new();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk.map_err(|_| "Модель повернула неповну відповідь.")?;
                    if bytes.len() + chunk.len() > 1024 * 1024 {
                        return Err("Список моделей завеликий.".into());
                    }
                    bytes.extend_from_slice(&chunk);
                }
                let tags: Tags = serde_json::from_slice(&bytes)
                    .map_err(|_| "Не вдалося прочитати список моделей.")?;
                if tags.models.len() > 32 {
                    return Err("Пілот підтримує до 32 локальних моделей.".into());
                }
                return Ok(tags
                    .models
                    .into_iter()
                    .filter(|t| t.remote_host.is_empty() && t.remote_model.is_empty())
                    .map(|t| (t.name, t.details.family))
                    .collect());
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err("Локальна модель не відповіла вчасно.".into())
}
