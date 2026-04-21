pub mod admission;
pub mod queue;
pub mod lease_validator;
pub mod nats_client;
pub mod sandbox_executor;
pub mod result_publisher;
pub mod worker_loop;
pub mod models;
pub mod routing_lane;

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

#[derive(Serialize)]
pub struct WorkerRegistrationRequest {
    pub event_type: String,
    pub payload: EnrollmentReqPayload,
}

#[derive(Serialize)]
pub struct EnrollmentReqPayload {
    pub worker_uuid: String,
    pub pubkey: String,
}

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
        
        let req = WorkerRegistrationRequest {
            event_type: "enrollment_req".to_string(),
            payload: EnrollmentReqPayload {
                worker_uuid: identity.node_id.clone(),
                pubkey: identity.public_key.clone(),
            }
        };
        
        let dump = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        println!("Worker mode ACTIVE. Bootstrapping Identity...");
        println!("Stable Identity Node ID: {}", identity.node_id);
        println!("Mock Registration Payload Compiled: {}", dump);
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
