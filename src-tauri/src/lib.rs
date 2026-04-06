mod identity;
mod enrollment;
mod heartbeat;
mod config;
mod capabilities;
mod messaging;
mod worker;
mod models;
mod trust;
mod observability;
mod authorities;
mod agents;
mod districts;
mod market;
mod evolution;
mod coordination;
mod intelligence;
mod metacognition;
mod genesis;

use std::sync::Arc;
use tokio::sync::Mutex;
use heartbeat::{HeartbeatManager, HeartbeatStatus};
use messaging::MessagingState;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(HeartbeatManager {
            status: Arc::new(Mutex::new(HeartbeatStatus::default())),
        })
        .manage(Arc::new(MessagingState::new()))
        .invoke_handler(tauri::generate_handler![
            greet,
            identity::get_identity_status,
            identity::initialize_identity,
            enrollment::get_enrollment_status,
            enrollment::enroll_node,
            heartbeat::get_heartbeat_status,
            capabilities::get_capabilities,
            messaging::get_messaging_status,
            messaging::bootstrap_messaging,
            messaging::send_node_message,
            messaging::get_node_messages,
            messaging::reconnect_messaging,
            models::get_supported_models,
            models::get_local_models,
            models::download_model,
            models::load_model,
            models::unload_model,
            models::get_residency_score,
            models::run_local_inference,
            genesis::generate_wallet_keys,
            genesis::record_voice_imprint,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            heartbeat::start_heartbeat_loop(handle.clone());
            
            // Start Worker Runtime (Data Plane)
            worker::start_worker_loop(handle);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
