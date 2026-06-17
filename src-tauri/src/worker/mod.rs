pub mod admission;
pub mod queue;
pub mod lease_validator;
pub mod nats_client;
pub mod sandbox_executor;
pub mod result_publisher;
pub mod worker_loop;
pub mod models;
// pub mod identity;
pub mod routing_lane;
pub mod relay_client;
pub mod runner;
pub mod onboarding;

pub use admission::{AdmissionController, ExecutionDecision, AdmissionInput, JobClass};
pub use queue::LocalQueue;
pub use worker_loop::start_worker_loop;
pub use models::{SpecializationClass, SpecializationProfile, SpecializationPolicy};

use tauri::State;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

const WORKER_CRYPTO_GATE_READY: bool = false;
const WORKER_GATE_BLOCKED_MESSAGE: &str =
    "Worker Mode is disabled until cryptographic operator-token validation is implemented.";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct WorkerModeState {
    /// User's explicit opt-in intent. Persisted to disk.
    pub opted_in: bool,
    /// [GUARDRAIL 2]: True ONLY after real relay/backend hello+enrollment acknowledgment.
    /// Not on endpoint presence, TCP probe, WebSocket connect, or local opt-in.
    pub backend_accepted: bool,
    /// Runtime status: whether the worker relay is actually reachable.
    /// NOT persisted — derived at runtime.
    #[serde(skip)]
    pub runtime_status: WorkerRuntimeStatus,
    /// Whether the background daemon loop is actively running.
    /// NOT persisted.
    #[serde(skip)]
    pub loop_running: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "state", content = "reason")]
pub enum WorkerRuntimeStatus {
    #[default]
    Unknown,
    Active,
    Blocked(String),
    Unavailable(String),
}

/// Load persisted worker opt-in intent from app data dir.
/// Returns false if file missing, corrupt, or on any error (fail-safe).
pub fn load_worker_optin(app: &tauri::AppHandle) -> bool {
    if !WORKER_CRYPTO_GATE_READY {
        return false;
    }

    use tauri::Manager;
    let Some(data_dir) = app.path().app_data_dir().ok() else {
        return false;
    };
    let path = data_dir.join("worker_mode.json");
    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            #[derive(Deserialize)]
            struct Persisted { opted_in: bool }
            serde_json::from_str::<Persisted>(&contents)
                .map(|p| p.opted_in)
                .unwrap_or(false)
        }
        Err(_) => false,
    }
}

/// Save worker opt-in intent to app data dir.
/// This persists the user's explicit choice, not the runtime connectivity state.
fn save_worker_optin(app: &tauri::AppHandle, opted_in: bool) {
    use tauri::Manager;
    let Some(data_dir) = app.path().app_data_dir().ok() else {
        eprintln!("Worker: Cannot resolve app data dir for persistence.");
        return;
    };
    let _ = std::fs::create_dir_all(&data_dir);
    let path = data_dir.join("worker_mode.json");
    let json = serde_json::json!({ "opted_in": opted_in });
    if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap_or_default()) {
        eprintln!("Worker: Failed to persist opt-in state: {}", e);
    }
}

use relay_client::{
    RelayClient, MockRelayClient, WsRelayClient, WorkerHello, WorkerHelloPayload,
    EnrollmentRequest, EnrollmentReqPayload, AdvisoryResult, AdvisoryOutput, 
    ExecutionReceipt, ExecutionReceiptPayload
};
use ed25519_dalek::{Signer};

#[tauri::command]
pub async fn toggle_worker_mode(
    enabled: bool, 
    state: State<'_, Mutex<WorkerModeState>>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let mut s = state.lock().await;

    if enabled && !WORKER_CRYPTO_GATE_READY {
        s.opted_in = false;
        s.backend_accepted = false;
        s.runtime_status = WorkerRuntimeStatus::Blocked(WORKER_GATE_BLOCKED_MESSAGE.to_string());
        save_worker_optin(&app_handle, false);
        return Err(WORKER_GATE_BLOCKED_MESSAGE.to_string());
    }

    s.opted_in = enabled;
    
    // Persist the user's explicit opt-in intent (not runtime state)
    save_worker_optin(&app_handle, enabled);
    
    if enabled {
        if s.loop_running {
            return Ok("Worker loop already running".into());
        }

        let identity = crate::identity::load_or_create_identity(&app_handle)?;
        println!("Worker mode OPTED IN. Stable Node ID: {}", identity.node_id);

        s.loop_running = true;
        s.backend_accepted = false;
        
        let app_handle_clone = app_handle.clone();
        let identity_clone = identity.clone();

        tauri::async_runtime::spawn(async move {
            loop {
                // Check if user disabled worker mode
                {
                    use tauri::Manager;
                    let state = app_handle_clone.state::<Mutex<WorkerModeState>>();
                    let s = state.lock().await;
                    if !s.opted_in {
                        println!("Worker loop exiting: user opted out.");
                        break;
                    }
                }

                let relay_endpoint = match crate::config::resolve_relay_endpoint() {
                    Some(ep) => ep,
                    None => {
                        {
                            use tauri::Manager;
                            let state = app_handle_clone.state::<Mutex<WorkerModeState>>();
                            let mut s = state.lock().await;
                            s.runtime_status = WorkerRuntimeStatus::Unavailable(
                                "Relay endpoint not configured. Waiting...".to_string()
                            );
                            s.backend_accepted = false;
                        }
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                };

                let mut relay: Box<dyn RelayClient + Send + Sync> = Box::new(WsRelayClient::new(&relay_endpoint));

                if let Err(e) = relay.connect().await {
                    {
                        use tauri::Manager;
                        let state = app_handle_clone.state::<Mutex<WorkerModeState>>();
                        let mut s = state.lock().await;
                        s.runtime_status = WorkerRuntimeStatus::Unavailable(
                            format!("Network unavailable. Retrying connection... ({})", e)
                        );
                        s.backend_accepted = false;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }

                let hello = WorkerHello {
                    event_type: "worker_hello".to_string(),
                    payload: WorkerHelloPayload {
                        protocol_version: "v0.1-alpha".to_string(),
                        worker_uuid: identity_clone.node_id.clone(),
                    }
                };

                match relay.send_hello(hello).await {
                    Ok(_) => {
                        let req = EnrollmentRequest {
                            event_type: "enrollment_req".to_string(),
                            payload: EnrollmentReqPayload {
                                worker_uuid: identity_clone.node_id.clone(),
                                pubkey: identity_clone.public_key.clone(),
                            }
                        };
                        match relay.send_enrollment(req).await {
                            Ok(dec) => {
                                if dec.payload.status == "approved" {
                                    {
                                        use tauri::Manager;
                                        let state = app_handle_clone.state::<Mutex<WorkerModeState>>();
                                        let mut s = state.lock().await;
                                        s.backend_accepted = true;
                                        s.runtime_status = WorkerRuntimeStatus::Active;
                                    }

                                    loop {
                                        {
                                            use tauri::Manager;
                                            let state = app_handle_clone.state::<Mutex<WorkerModeState>>();
                                            let s = state.lock().await;
                                            if !s.opted_in {
                                                break;
                                            }
                                        }

                                        match relay.wait_for_task().await {
                                        Ok(task) => {
                                            println!("Task Received: {}", task.payload.task_id);
                                            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                                            
                                            if now > task.payload.lease_expires_at {
                                                println!("HARD REJECT [Desktop]: Lease {} arrived already expired. Dropping.", task.payload.lease_id);
                                                continue;
                                            }
                                            
                                            if task.payload.work_type == "ping_math" {
                                                let value = task.payload.args.value.unwrap_or(0);
                                                println!("Handing ping_math payload {} over into Bounded Execution Envelope... [Lease: {}]", value, task.payload.lease_id);
                                                
                                                match crate::worker::runner::execute_ping_math(value).await {
                                                    Ok(math_res) => {
                                                        println!("Envelope execution SUCCESS: {}", math_res);
                                                        let adv = AdvisoryResult {
                                                            task_id: task.payload.task_id.clone(),
                                                            result: AdvisoryOutput { output: serde_json::json!(math_res) },
                                                            execution_ms: 10,
                                                        };
                                                        
                                                        let raw_advisory_json = serde_json::to_string(&adv).unwrap();
                                                        let signing_key = crate::identity::get_signing_key(&app_handle_clone).expect("Missing private key");
                                                        let sig = signing_key.sign(raw_advisory_json.as_bytes());
                                                        
                                                        let receipt = ExecutionReceipt {
                                                            event_type: "execution_receipt".into(),
                                                            payload: ExecutionReceiptPayload {
                                                                worker_id: identity_clone.node_id.clone(),
                                                                lease_id: task.payload.lease_id.clone(),
                                                                raw_advisory_json,
                                                                signature: hex::encode(sig.to_bytes()),
                                                            }
                                                        };
                                                        
                                                        let _ = relay.send_receipt(receipt).await;
                                                        
                                                        match relay.wait_for_verify().await {
                                                            Ok(verify) => println!("Backend Verify Object: {:?}", verify),
                                                            Err(e) => println!("Verify Timeout/Error: {}", e),
                                                        }
                                                    },
                                                    Err(envelope_err) => println!("Envelope HARD FAIL-CLOSED: {}", envelope_err),
                                                }
                                            } else if task.payload.work_type == "text_hash" {
                                                let text_val = task.payload.args.text.clone().unwrap_or_default();
                                                println!("Handing text_hash payload '{}' over into Bounded Execution Envelope... [Lease: {}]", text_val, task.payload.lease_id);
                                                
                                                match crate::worker::runner::execute_text_hash(text_val).await {
                                                    Ok(hash_res) => {
                                                        println!("Envelope execution SUCCESS: {}", hash_res);
                                                        let adv = AdvisoryResult {
                                                            task_id: task.payload.task_id.clone(),
                                                            result: AdvisoryOutput { output: serde_json::json!(hash_res) },
                                                            execution_ms: 15,
                                                        };
                                                        
                                                        let raw_advisory_json = serde_json::to_string(&adv).unwrap();
                                                        let signing_key = crate::identity::get_signing_key(&app_handle_clone).expect("Missing private key");
                                                        let sig = signing_key.sign(raw_advisory_json.as_bytes());
                                                        
                                                        let receipt = ExecutionReceipt {
                                                            event_type: "execution_receipt".into(),
                                                            payload: ExecutionReceiptPayload {
                                                                worker_id: identity_clone.node_id.clone(),
                                                                lease_id: task.payload.lease_id.clone(),
                                                                raw_advisory_json,
                                                                signature: hex::encode(sig.to_bytes()),
                                                            }
                                                        };
                                                        
                                                        let _ = relay.send_receipt(receipt).await;
                                                        
                                                        match relay.wait_for_verify().await {
                                                            Ok(verify) => println!("Backend Verify Object: {:?}", verify),
                                                            Err(e) => println!("Verify Timeout/Error: {}", e),
                                                        }
                                                    },
                                                    Err(envelope_err) => println!("Envelope HARD FAIL-CLOSED: {}", envelope_err),
                                                }
                                            } else {
                                                println!("Unsupported work_type '{}' rejected before envelope execution.", task.payload.work_type);
                                            }
                                        },
                                        Err(e) => {
                                            println!("Wait for task failed or session dropped: {}", e);
                                            use tauri::Manager;
                                            let state = app_handle_clone.state::<Mutex<WorkerModeState>>();
                                            let mut s = state.lock().await;
                                            s.runtime_status = WorkerRuntimeStatus::Unavailable(format!("Connection dropped, retrying: {}", e));
                                            s.backend_accepted = false; // Sync lost, MUST revoke Active status
                                            break;
                                        }
                                    }
                                }
                            } else {
                                {
                                    use tauri::Manager;
                                    let state = app_handle_clone.state::<Mutex<WorkerModeState>>();
                                    let mut s = state.lock().await;
                                    s.backend_accepted = false;
                                    s.runtime_status = WorkerRuntimeStatus::Blocked(format!("Enrollment denied: {}", dec.payload.status));
                                }
                                tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
                            }
                        },
                        Err(e) => {
                            {
                                use tauri::Manager;
                                let state = app_handle_clone.state::<Mutex<WorkerModeState>>();
                                let mut s = state.lock().await;
                                s.runtime_status = WorkerRuntimeStatus::Unavailable(format!("Enrollment failed, retrying: {}", e));
                                s.backend_accepted = false;
                            }
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                    }
                },
                Err(e) => {
                    {
                        use tauri::Manager;
                        let state = app_handle_clone.state::<Mutex<WorkerModeState>>();
                        let mut s = state.lock().await;
                        s.runtime_status = WorkerRuntimeStatus::Unavailable(format!("Relay hello failed, retrying: {}", e));
                        s.backend_accepted = false;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
        
        {
            use tauri::Manager;
            let state = app_handle_clone.state::<Mutex<WorkerModeState>>();
            let mut s = state.lock().await;
            s.loop_running = false;
        }
    });

    } else {
        println!("Worker mode changed to: {}", enabled);
        s.backend_accepted = false;
    }
    
    Ok(format!("Worker mode changed to: {}", enabled))
}

#[tauri::command]
pub async fn get_worker_mode(state: State<'_, Mutex<WorkerModeState>>) -> Result<WorkerModeState, String> {
    let s = state.lock().await;
    Ok(s.clone())
}
