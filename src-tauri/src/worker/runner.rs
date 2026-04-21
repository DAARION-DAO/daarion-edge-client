use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct PayloadInput {
    value: u64,
}

#[derive(Deserialize)]
struct PayloadOutput {
    output: Option<u64>,
    error: Option<String>,
}

pub async fn execute_ping_math(value: u64) -> Result<u64, String> {
    let input = serde_json::to_string(&PayloadInput { value }).map_err(|e| e.to_string())?;
    
    // In Sprint 2A, we simulate the Colima envelope using an explicitly isolated local subprocess.
    // This bounds the execution away from inline Rust, enforcing exact IPC and STDOUT interfaces
    // that we will cleanly swap for `colima exec` in Sprint 2B/3.
    let mut script_path = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    
    // During dev, cwd is often the rust root or project root:
    if script_path.ends_with("src-tauri") {
        script_path.push("payloads");
        script_path.push("ping_math.py");
    } else {
        script_path.push("daarion-edge-client");
        script_path.push("src-tauri");
        script_path.push("payloads");
        script_path.push("ping_math.py");
    }

    let mut cmd = Command::new("python3");
    cmd.arg(script_path)
       .arg(input)
       .env_clear() // Drop environment variables for sandbox simulation
       .kill_on_drop(true)
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let child = cmd.spawn().map_err(|e| format!("Spawn failed: {}", e))?;
    
    // Enforce 2 second execution timeout limit strictly bounding the compute envelope
    let output = match timeout(Duration::from_secs(2), child.wait_with_output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => return Err(format!("Wait failed: {}", e)),
        Err(_) => return Err("Execution envelope timeout".into()),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Envelope failed closed. Stderr: {}", stderr));
    }

    let stdout_str = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let res: PayloadOutput = serde_json::from_str(&stdout_str).map_err(|e| format!("Invalid envelope JSON: {}", e))?;
    
    if let Some(err) = res.error {
        return Err(format!("Envelope logic error: {}", err));
    }
    
    res.output.ok_or_else(|| "No output produced by envelope".into())
}
