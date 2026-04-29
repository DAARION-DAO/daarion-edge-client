mod identity;
mod registry_client;
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
mod provisioning;
mod reset;

use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent}, Manager};
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
        .plugin(tauri_plugin_shell::init())
        .manage(HeartbeatManager {
            status: Arc::new(Mutex::new(HeartbeatStatus::default())),
        })
        .manage(Arc::new(MessagingState::new()))
        .manage(Mutex::new(crate::worker::WorkerModeState::default()))
        .invoke_handler(tauri::generate_handler![
            greet,
            identity::get_identity_status,
            identity::initialize_identity,
            enrollment::get_enrollment_status,
            enrollment::enroll_node,
            enrollment::sync_capabilities,
            heartbeat::get_heartbeat_status,
            capabilities::get_capabilities,
            messaging::get_messaging_status,
            messaging::bootstrap_messaging,
            messaging::send_node_message,
            messaging::get_node_messages,
            messaging::reconnect_messaging,
            models::sync_registry,
            models::detect_ollama,
            models::get_ollama_status,
            models::list_local_models,
            models::pull_model,
            models::run_smoke_inference,
            models::download_model,
            models::load_model,
            models::unload_model,
            models::get_residency_score,
            models::run_local_inference,
            genesis::generate_wallet_keys,
            genesis::record_voice_imprint,
            provisioning::check_beta_slots,
            provisioning::provision_sovereign_genesis,
            provisioning::register_creator_profile,
            reset::factory_reset_local_state,
            trust::hardware_evidence::submit_evidence_handshake,
            crate::worker::toggle_worker_mode,
            crate::worker::get_worker_mode,
            crate::worker::onboarding::check_environment,
            crate::worker::onboarding::check_operator_approval,
            crate::worker::onboarding::activate_octelium_tunnel,
            crate::worker::onboarding::enable_wsl_windows,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            
            // Add System Tray Supervisor Path
            if let Some(icon) = app.default_window_icon().cloned() {
                let _tray = TrayIconBuilder::new()
                    .icon(icon)
                    .tooltip("DAARION Edge")
                    .on_tray_icon_event(|tray, event| match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    })
                    .build(app);
            }
            heartbeat::start_heartbeat_loop(handle.clone());
            
            // Start Worker Runtime (Data Plane) — only if user previously opted in.
            // Load persisted opt-in intent (not runtime state).
            let user_opted_in = worker::load_worker_optin(&handle);
            
            // Update in-memory state to reflect persisted intent
            let worker_state = app.state::<Mutex<crate::worker::WorkerModeState>>();
            {
                let mut state = worker_state.blocking_lock();
                state.opted_in = user_opted_in;
            }
            
            if user_opted_in {
                // Check if relay is actually configured before starting the loop
                if crate::config::resolve_relay_endpoint().is_some() {
                    let app_clone = handle.clone();
                    tokio::spawn(async move {
                        let state = app_clone.state::<Mutex<crate::worker::WorkerModeState>>();
                        let _ = crate::worker::toggle_worker_mode(true, state, app_clone.clone()).await;
                    });
                } else {
                    println!("Worker: User opted in but relay endpoint not configured. Worker loop deferred.");
                    // Update runtime_status so frontend can show honest state
                    let mut state = worker_state.blocking_lock();
                    state.runtime_status = crate::worker::WorkerRuntimeStatus::Unavailable(
                        "Relay endpoint not configured. Worker Mode is deferred until a relay is available.".to_string()
                    );
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
