pub mod admission;
pub mod queue;
pub mod lease_validator;
pub mod nats_client;
pub mod sandbox_executor;
pub mod result_publisher;
pub mod worker_loop;
pub mod models;
pub mod routing_lane;
pub mod relay_client;

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

        let mut relay: Box<dyn RelayClient> = Box::new(WsRelayClient::new("ws://127.0.0.1:8181/edge/relay"));
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
                                if task.payload.work_type == "ping_math" {
                                    println!("Executing safe sandbox ping_math({})...", task.payload.args.value);
                                    let math_res = task.payload.args.value * 2; // Real computation
                                    
                                    let adv = AdvisoryResult {
                                        task_id: task.payload.task_id.clone(),
                                        result: AdvisoryOutput { output: math_res },
                                        execution_ms: 10,
                                    };
                                    
                                    let raw_advisory_json = serde_json::to_string(&adv).unwrap();
                                    let signing_key = crate::identity::get_signing_key(&app_handle).expect("Missing private key");
                                    let sig = signing_key.sign(raw_advisory_json.as_bytes());
                                    
                                    let receipt = ExecutionReceipt {
                                        event_type: "execution_receipt".into(),
                                        payload: ExecutionReceiptPayload {
                                            worker_id: identity.node_id.clone(),
                                            raw_advisory_json,
                                            signature: hex::encode(sig.to_bytes()), // hex encoding
                                        }
                                    };
                                    
                                    println!("Submitting cryptographic ExecutionReceipt...");
                                    let _ = relay.send_receipt(receipt).await;
                                    
                                    println!("Waiting for Canonical VerifyDecision...");
                                    match relay.wait_for_verify().await {
                                        Ok(verify) => println!("Backend Verify Object: {:?}", verify),
                                        Err(e) => println!("Verify Timeout/Error: {}", e),
                                    }
                                } else {
                                    println!("Unknown workload skipped.");
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
