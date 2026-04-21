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
    
    // Sprint 2B: Physical Colima execution boundary
    // We execute via the Colima-backed Docker socket, explicitly passing the python code
    // via `include_str!` to avoid volume mount complexity, and disabling network.
    let mut cmd = Command::new("docker");
    cmd.args([
        "run", 
        "--rm", // Clean up container after run
        "-i", // Interactive/stdin support if needed by docker
        "--network", "none", // Hard bounded execution logic: no network egress
        // To harden this further, we could add --memory="128m" --cpus="0.5"
        "python:3.9-alpine", 
        "python", "-c", 
        include_str!("../../payloads/ping_math.py"), 
        &input
    ])
       .env_clear() 
       .kill_on_drop(true)
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let child = cmd.spawn().map_err(|e| format!("Colima Envelope Spawn failed (check if Colima running): {}", e))?;
    
    // Allow up to 10 seconds for Colima container cold start and compute
    let output = match timeout(Duration::from_secs(10), child.wait_with_output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => return Err(format!("Colima Wait failed: {}", e)),
        Err(_) => return Err("Colima execution envelope timeout".into()),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Colima Envelope failed closed. Stderr: {}", stderr));
    }

    let stdout_str = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let res: PayloadOutput = serde_json::from_str(&stdout_str).map_err(|e| format!("Invalid Colima JSON: {}", e))?;
    
    if let Some(err) = res.error {
        return Err(format!("Colima logic error: {}", err));
    }
    
    res.output.ok_or_else(|| "No output produced by Colima envelope".into())
}

#[derive(Serialize)]
struct HashPayloadInput {
    text: String,
}

#[derive(Deserialize)]
struct HashPayloadOutput {
    output: Option<String>,
    error: Option<String>,
}

pub async fn execute_text_hash(text: String) -> Result<String, String> {
    let input = serde_json::to_string(&HashPayloadInput { text }).map_err(|e| e.to_string())?;
    
    let mut cmd = Command::new("docker");
    cmd.args([
        "run", 
        "--rm", 
        "-i", 
        "--network", "none", 
        "python:3.9-alpine", 
        "python", "-c", 
        include_str!("../../payloads/text_hash.py"), 
        &input
    ])
       .env_clear() 
       .kill_on_drop(true)
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let child = cmd.spawn().map_err(|e| format!("Colima Envelope Spawn failed: {}", e))?;
    
    let output = match timeout(Duration::from_secs(10), child.wait_with_output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => return Err(format!("Colima Wait failed: {}", e)),
        Err(_) => return Err("Colima execution envelope timeout".into()),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Colima Envelope failed closed. Stderr: {}", stderr));
    }

    let stdout_str = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let res: HashPayloadOutput = serde_json::from_str(&stdout_str).map_err(|e| format!("Invalid Colima JSON: {}", e))?;
    
    if let Some(err) = res.error {
        return Err(format!("Colima logic error: {}", err));
    }
    
    res.output.ok_or_else(|| "No output produced by Colima envelope".into())
}
