use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager, Emitter};
use chrono::{DateTime, Utc};
use serde_json::json;
use crate::enrollment::load_enrollment_state;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HeartbeatStatus {
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_error: Option<Option<String>>,
    pub consecutive_failures: u32,
    pub active: bool,
}

pub struct HeartbeatManager {
    pub status: Arc<Mutex<HeartbeatStatus>>,
}

#[tauri::command]
pub async fn get_heartbeat_status(state: tauri::State<'_, HeartbeatManager>) -> Result<HeartbeatStatus, String> {
    let status = state.status.lock().await;
    Ok(status.clone())
}

pub fn start_heartbeat_loop(handle: AppHandle) {
    let state = handle.state::<HeartbeatManager>();
    let status_arc = state.status.clone();
    let handle_clone = handle.clone();

    tokio::spawn(async move {
        loop {
            let enrollment = load_enrollment_state(&handle_clone);
            let node_token = crate::enrollment::get_node_token();
            
            if enrollment.enrolled && node_token.is_ok() {
                let token = node_token.unwrap();
                let interval = enrollment.heartbeat_interval_sec.max(10); // Min 10s
                
                {
                    let mut status = status_arc.lock().await;
                    status.active = true;
                    status.last_attempt_at = Some(Utc::now());
                }

                // Simulate heartbeat request
                // In real implementation: 
                // reqwest::Client::new().post(url).header("Authorization", token).json(payload).send().await
                
                let success = true; // Mocked success for Slice A2

                {
                    let mut status = status_arc.lock().await;
                    if success {
                        status.last_success_at = Some(Utc::now());
                        status.last_error = None;
                        status.consecutive_failures = 0;
                    } else {
                        status.last_error = Some(Some("Network timeout".to_string()));
                        status.consecutive_failures += 1;
                    }
                }

                // Emit event to frontend for real-time updates
                let current_status = {
                    let s = status_arc.lock().await;
                    s.clone()
                };
                let _ = handle_clone.emit("heartbeat-update", current_status);
                
                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            } else {
                {
                    let mut status = status_arc.lock().await;
                    status.active = false;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    });
}
