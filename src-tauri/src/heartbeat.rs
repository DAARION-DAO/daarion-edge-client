//! heartbeat.rs — SOFIIA Worker Node Registry heartbeat loop + task polling
//!
//! MIGRATION NOTE (2026-04-29):
//!   Legacy endpoint: POST {backend_url}/edge/heartbeat (bearer token auth)
//!   New endpoint:    POST {backend_url}/api/v1/nodes/heartbeat (nodeId + timestamp + ed25519 sig)
//!
//! Task polling (Gate 1B addition):
//!   GET {backend_url}/api/v1/nodes/tasks — runs after each successful heartbeat
//!   MVP contract: tasks list is always empty. Client DOES NOT execute tasks.
//!   Revoked node heartbeat returns 401 — local state is NOT purged on transient error.
//!
//! Loop lifecycle:
//!   - Waits for node_id before starting (polls every 10s until available)
//!   - Heartbeats every heartbeat_interval_sec (from enrollment, default 60s)
//!   - After successful heartbeat: polls tasks once (safe-empty for MVP)
//!   - 20 consecutive failures: emits WARNING log, never purges identity

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager, Emitter};
use chrono::{DateTime, Utc};
use crate::enrollment::load_enrollment_state;
use crate::registry_client::{RegistryHeartbeatRequest, call_heartbeat, call_tasks};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HeartbeatStatus {
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_error: Option<Option<String>>,
    pub consecutive_failures: u32,
    pub active: bool,
    /// First 8 chars of node_id for diagnostics. Never the full ID or private key.
    pub last_node_id_prefix: Option<String>,
    /// Count of task poll results (empty for MVP — proves polling handshake works).
    pub last_task_count: usize,
    /// Whether the node has been revoked (heartbeat received 401).
    pub revoked: bool,
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

            // Wait until we have a node_id before heartbeating.
            // Pending nodes heartbeat too — backend keeps last_heartbeat_at for operator view.
            let node_id = match &enrollment.node_id {
                Some(id) if !id.is_empty() => id.clone(),
                _ => {
                    {
                        let mut status = status_arc.lock().await;
                        status.active = false;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    continue;
                }
            };

            // Check if already revoked — do not heartbeat revoked nodes
            {
                let status = status_arc.lock().await;
                if status.revoked {
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    continue;
                }
            }

            let interval = enrollment.heartbeat_interval_sec.max(10);

            {
                let mut status = status_arc.lock().await;
                status.active = true;
                status.last_attempt_at = Some(Utc::now());
                status.last_node_id_prefix = Some(node_id.chars().take(8).collect());
            }

            let backend_url = crate::config::resolve_backend_url();

            // Build Ed25519 signature: canonical payload = "node_id|timestamp"
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let sig_payload = format!("{}|{}", node_id, timestamp);
            let signature = crate::identity::get_signing_key(&handle_clone)
                .map(|sk| {
                    use ed25519_dalek::Signer;
                    hex::encode(sk.sign(sig_payload.as_bytes()).to_bytes())
                })
                .unwrap_or_else(|_| "unsigned".to_string());

            let req = RegistryHeartbeatRequest {
                node_id: node_id.clone(),
                timestamp,
                status: "ok".to_string(),
                signature,
            };

            let hb_result = call_heartbeat(&backend_url, &req).await;

            let mut task_count: usize = 0;
            let mut revoked = false;

            match hb_result {
                Ok(resp) if resp.ack => {
                    // ── Successful heartbeat ──────────────────────────────────
                    {
                        let mut status = status_arc.lock().await;
                        status.last_success_at = Some(Utc::now());
                        status.last_error = None;
                        status.consecutive_failures = 0;
                    }

                    // Log directives (safe-empty for MVP, no execution)
                    if !resp.directives.is_empty() {
                        println!("[heartbeat] {} directive(s) received (not executed in MVP)", resp.directives.len());
                    }

                    // ── Task polling after successful heartbeat ───────────────
                    // Contract: tasks list is empty for MVP.
                    // Client DOES NOT execute returned tasks at this gate.
                    // Polling proves the handshake works end-to-end.
                    match call_tasks(&backend_url, &node_id).await {
                        Ok(tasks_resp) => {
                            task_count = tasks_resp.tasks.len();
                            if task_count > 0 {
                                // Log task IDs only — no execution without operator gate
                                println!("[tasks] {} task(s) available (not executed, MVP gate)", task_count);
                            }
                        }
                        Err(e) if e.contains("revoked") => {
                            println!("[tasks] Node revoked signal received: {}", e);
                            revoked = true;
                        }
                        Err(e) => {
                            // Tasks failure is non-fatal — heartbeat already succeeded
                            println!("[tasks] Poll failed (non-fatal): {}", e);
                        }
                    }
                }
                Ok(_) => {
                    // ack == false — treat as soft failure
                    let mut status = status_arc.lock().await;
                    status.last_error = Some(Some("Heartbeat ack=false".to_string()));
                    status.consecutive_failures += 1;
                }
                Err(e) if e.contains("revoked") => {
                    println!("[heartbeat] Node revoked: {}", e);
                    revoked = true;
                }
                Err(e) => {
                    // Transient failure — never purge identity
                    let mut status = status_arc.lock().await;
                    status.last_error = Some(Some(format!("Heartbeat failed: {}", e)));
                    status.consecutive_failures += 1;
                    if status.consecutive_failures == 20 {
                        println!(
                            "[heartbeat] WARNING: 20 consecutive failures for node {}. Backend may be unreachable.",
                            &node_id[..8.min(node_id.len())]
                        );
                    }
                }
            }

            // Apply revocation / task count to status
            {
                let mut status = status_arc.lock().await;
                status.last_task_count = task_count;
                if revoked {
                    status.revoked = true;
                    status.active = false;
                }
            }

            // Emit real-time update to frontend
            let current_status = {
                let s = status_arc.lock().await;
                s.clone()
            };
            let _ = handle_clone.emit("heartbeat-update", current_status);

            tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
        }
    });
}
