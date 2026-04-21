pub mod admission;
pub mod queue;
pub mod lease_validator;
pub mod nats_client;
pub mod sandbox_executor;
pub mod result_publisher;
pub mod worker_loop;
pub mod models;
pub mod identity;
pub mod routing_lane;
pub mod relay_client;
pub mod runner;

pub use admission::{AdmissionController, ExecutionDecision, AdmissionInput, JobClass};
pub use queue::LocalQueue;
pub use worker_loop::start_worker_loop;
pub use models::{SpecializationClass, SpecializationProfile, SpecializationPolicy};

use tauri::State;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct WorkerModeState {
    pub enabled: bool,
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
    s.enabled = enabled;
    
    if enabled {
        let identity = crate::identity::load_or_create_identity(&app_handle)?;
        println!("Worker mode ACTIVE. Stable Node ID: {}", identity.node_id);

        // Sprint 3A: Target real SOFIIA Canonical Acceptance Service (was 8181 dev_relay_stub)
        let mut relay: Box<dyn RelayClient> = Box::new(WsRelayClient::new("ws://127.0.0.1:8002/edge/relay"));
        // Or if we want to force mock for testing local interface without server:
        // let mut relay: Box<dyn RelayClient> = Box::new(MockRelayClient::new());

        let hello = WorkerHello {
            event_type: "worker_hello".to_string(),
            payload: WorkerHelloPayload {
                protocol_version: "v0.1-alpha".to_string(),
                worker_uuid: identity.node_id.clone(),
            }
        };

        if let Err(e) = relay.connect().await {
            println!("Relay Connect Failed: {}. Sleep.", e);
            return Ok("Failed".into());
        }

        match relay.send_hello(hello).await {
            Ok(ack) => {
                println!("Hello Ack Received: {:?}", ack);
                let req = EnrollmentRequest {
                    event_type: "enrollment_req".to_string(),
                    payload: EnrollmentReqPayload {
                        worker_uuid: identity.node_id.clone(),
                        pubkey: identity.public_key.clone(),
                    }
                };
                match relay.send_enrollment(req).await {
                    Ok(dec) => {
                        println!("Enrollment Decision: {:?}", dec);
                        
                        // Move 6: Wait for Task
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
                                    
                                    // Sprint 2A Envelope Boundary Call
                                    match crate::worker::runner::execute_ping_math(value).await {
                                        Ok(math_res) => {
                                            println!("Envelope execution SUCCESS: {}", math_res);
                                            let adv = AdvisoryResult {
                                                task_id: task.payload.task_id.clone(),
                                                result: AdvisoryOutput { output: serde_json::json!(math_res) },
                                                execution_ms: 10,
                                            };
                                            
                                            let raw_advisory_json = serde_json::to_string(&adv).unwrap();
                                            let signing_key = crate::identity::get_signing_key(&app_handle).expect("Missing private key");
                                            let sig = signing_key.sign(raw_advisory_json.as_bytes());
                                            
                                            let receipt = ExecutionReceipt {
                                                event_type: "execution_receipt".into(),
                                                payload: ExecutionReceiptPayload {
                                                    worker_id: identity.node_id.clone(),
                                                    lease_id: task.payload.lease_id.clone(),
                                                    raw_advisory_json,
                                                    signature: hex::encode(sig.to_bytes()),
                                                }
                                            };
                                            
                                            println!("Submitting cryptographic ExecutionReceipt...");
                                            let _ = relay.send_receipt(receipt).await;
                                            
                                            println!("Waiting for Canonical VerifyDecision...");
                                            match relay.wait_for_verify().await {
                                                Ok(verify) => println!("Backend Verify Object: {:?}", verify),
                                                Err(e) => println!("Verify Timeout/Error: {}", e),
                                            }
                                        },
                                        Err(envelope_err) => {
                                            println!("Envelope HARD FAIL-CLOSED: {}", envelope_err);
                                            // Fail closed: No ExecutionReceipt generated, daemon discards the lease
                                        }
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
                                            let signing_key = crate::identity::get_signing_key(&app_handle).expect("Missing private key");
                                            let sig = signing_key.sign(raw_advisory_json.as_bytes());
                                            
                                            let receipt = ExecutionReceipt {
                                                event_type: "execution_receipt".into(),
                                                payload: ExecutionReceiptPayload {
                                                    worker_id: identity.node_id.clone(),
                                                    lease_id: task.payload.lease_id.clone(),
                                                    raw_advisory_json,
                                                    signature: hex::encode(sig.to_bytes()),
                                                }
                                            };
                                            
                                            println!("Submitting cryptographic ExecutionReceipt...");
                                            let _ = relay.send_receipt(receipt).await;
                                            
                                            println!("Waiting for Canonical VerifyDecision...");
                                            match relay.wait_for_verify().await {
                                                Ok(verify) => println!("Backend Verify Object: {:?}", verify),
                                                Err(e) => println!("Verify Timeout/Error: {}", e),
                                            }
                                        },
                                        Err(envelope_err) => {
                                            println!("Envelope HARD FAIL-CLOSED: {}", envelope_err);
                                        }
                                    }
                                } else {
                                    println!("HARD REJECT: Unknown work_type {}. Dropping lease.", task.payload.work_type);
                                } else {
                                    println!("Unsupported work_type '{}' rejected before envelope execution.", task.payload.work_type);
                                }
                            },
                            Err(e) => println!("Wait for task failed: {}", e),
                        }
                    },
                    Err(e) => println!("Enrollment Failed: {}", e),
                }
            },
            Err(e) => {
                println!("Relay Hello Failed: {}. Falling back to sleep...", e);
            }
        }
    } else {
        println!("Worker mode changed to: {}", enabled);
    }
    
    Ok(format!("Worker mode changed to: {}", enabled))
}

#[tauri::command]
pub async fn get_worker_mode(state: State<'_, Mutex<WorkerModeState>>) -> Result<bool, String> {
    let s = state.lock().await;
    Ok(s.enabled)
}
