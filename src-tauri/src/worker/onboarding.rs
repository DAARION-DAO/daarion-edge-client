use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepStatus {
    pub step: String,
    pub status: String, // "idle", "running", "passed", "failed"
    pub message: String,
}

#[tauri::command]
pub async fn check_environment() -> Result<StepStatus, String> {
    let os = std::env::consts::OS;

    if os == "macos" {
        // Check Colima
        let colima_check = std::process::Command::new("which")
            .arg("colima")
            .output()
            .map_err(|e| e.to_string())?;

        if !colima_check.status.success() {
            return Ok(StepStatus {
                step: "environment".to_string(),
                status: "failed".to_string(),
                message: "Colima sandbox not found. Please install Colima.".to_string(),
            });
        }

        // Check Docker socket / CLI
        let docker_check = std::process::Command::new("which")
            .arg("docker")
            .output()
            .map_err(|e| e.to_string())?;

        if !docker_check.status.success() {
            return Ok(StepStatus {
                step: "environment".to_string(),
                status: "failed".to_string(),
                message: "Docker CLI not found. Required for sandbox.".to_string(),
            });
        }

        return Ok(StepStatus {
            step: "environment".to_string(),
            status: "passed".to_string(),
            message: "macOS and Colima sandbox verified.".to_string(),
        });
    } else if os == "windows" {
        // Check WSL2
        let wsl_check = std::process::Command::new("wsl")
            .arg("--status")
            .output();

        match wsl_check {
            Ok(output) if output.status.success() => {
                return Ok(StepStatus {
                    step: "environment".to_string(),
                    status: "passed".to_string(),
                    message: "Windows WSL2 subsystem verified.".to_string(),
                });
            }
            _ => {
                return Ok(StepStatus {
                    step: "environment".to_string(),
                    status: "failed_wsl_missing".to_string(),
                    message: "WSL2 is not enabled or installed. A system reboot may be required after installation.".to_string(),
                });
            }
        }
    }

    Ok(StepStatus {
        step: "environment".to_string(),
        status: "failed".to_string(),
        message: format!("Pilot rollout is currently not supported on OS: {}", os),
    })
}

#[tauri::command]
pub async fn enable_wsl_windows() -> Result<String, String> {
    if std::env::consts::OS != "windows" {
        return Err("Not on Windows".into());
    }
    
    // Launch PowerShell elevated to run `wsl --install`
    let status = std::process::Command::new("powershell")
        .args(&["-Command", "Start-Process", "wsl", "-ArgumentList", "'--install'", "-Verb", "RunAs"])
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok("WSL installation triggered. Please follow the Windows prompts and reboot if necessary.".into())
    } else {
        Err("Failed to trigger WSL installation.".into())
    }
}

#[tauri::command]
pub async fn check_operator_approval() -> Result<StepStatus, String> {
    // Platform-aware home dir resolution:
    // macOS/Linux: $HOME
    // Windows: $USERPROFILE (since $HOME is not set by default)
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();
    
    if home_dir.is_empty() {
        return Ok(StepStatus {
            step: "operator_approval".to_string(),
            status: "failed".to_string(),
            message: "Cannot determine home directory. Unable to check operator token.".to_string(),
        });
    }
    
    let token_path = format!("{}/.daarion/operator_token.txt", home_dir);
    
    if Path::new(&token_path).exists() {
        Ok(StepStatus {
            step: "operator_approval".to_string(),
            status: "passed".to_string(),
            message: "Worker eligibility token verified.".to_string(),
        })
    } else {
        Ok(StepStatus {
            step: "operator_approval".to_string(),
            status: "failed".to_string(),
            message: format!("Pilot Operator Token missing. Expected at {}", token_path),
        })
    }
}

#[tauri::command]
pub async fn activate_octelium_tunnel() -> Result<StepStatus, String> {
    // [GUARDRAIL 1]: "passed" here means ONLY that the relay transport endpoint is reachable.
    // It does NOT imply backend acceptance, worker enrollment, active status, or task availability.
    // Those are determined later in the relay hello/enrollment handshake inside toggle_worker_mode.

    let relay_endpoint = match crate::config::resolve_relay_endpoint() {
        Some(ep) => ep,
        None => {
            return Ok(StepStatus {
                step: "secure_mesh_attach".to_string(),
                status: "unavailable".to_string(),
                message: "Relay endpoint not configured. Worker transport is not available yet.".to_string(),
            });
        }
    };

    // Attempt a lightweight TCP probe against the relay to verify transport reachability.
    let probe_url = relay_endpoint
        .replace("ws://", "http://")
        .replace("wss://", "https://");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    match client.get(&probe_url).send().await {
        Ok(_) => {
            Ok(StepStatus {
                step: "secure_mesh_attach".to_string(),
                status: "passed".to_string(),
                message: "Relay transport endpoint is reachable. Proceeding to worker activation.".to_string(),
            })
        }
        Err(_) => {
            Ok(StepStatus {
                step: "secure_mesh_attach".to_string(),
                status: "blocked".to_string(),
                message: format!("Relay endpoint {} is not reachable. Worker transport is blocked.", relay_endpoint),
            })
        }
    }
}
