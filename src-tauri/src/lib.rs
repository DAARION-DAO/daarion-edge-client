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
use tauri::Manager;
#[cfg(desktop)]
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use heartbeat::{HeartbeatManager, HeartbeatStatus};
use messaging::MessagingState;

// ── Boot Logging ──────────────────────────────────────────────────────────────
// Durable file-based logging for release builds where windows_subsystem = "windows"
// hides all stdout/stderr.  Writes to: {app_data_dir}/logs/boot.log
// ──────────────────────────────────────────────────────────────────────────────

use std::io::Write;
use std::path::PathBuf;

fn boot_log_path() -> PathBuf {
    // Use platform-specific app data dir without requiring AppHandle (not yet available)
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("APPDATA")
            .unwrap_or_else(|_| std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()));
        PathBuf::from(base).join("DAARION Edge").join("logs")
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".daarion-edge").join("logs")
    }
}

fn boot_log(msg: &str) {
    let dir = boot_log_path();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("boot.log");
    
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
    let line = format!("[{}] {}\n", timestamp, msg);
    
    // Append to log file
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
    
    // Also print to stdout for dev/debug builds
    print!("{}", line);
}

fn show_fatal_error(message: &str) {
    boot_log(&format!("FATAL: {}", message));
    
    // On Windows release builds, stdout is invisible. Show a native dialog.
    #[cfg(target_os = "windows")]
    {
        // Use Windows MessageBox via raw winapi to avoid extra dependency
        // For now, the boot.log file is the primary diagnostic channel
        eprintln!("FATAL: {}", message);
    }
    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("FATAL: {}", message);
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    boot_log("=== DAARION Edge boot sequence started ===");
    boot_log(&format!("Version: {}", env!("CARGO_PKG_VERSION")));
    boot_log(&format!("OS: {}", std::env::consts::OS));
    boot_log(&format!("Arch: {}", std::env::consts::ARCH));
    
    boot_log("Initializing Tauri builder...");
    
    let builder = tauri::Builder::default();
    boot_log("  Tauri builder created");
    
    let builder = builder.plugin(tauri_plugin_opener::init());
    boot_log("  Plugin: opener initialized");
    
    let builder = builder.plugin(tauri_plugin_shell::init());
    boot_log("  Plugin: shell initialized");
    
    boot_log("  Managing state: HeartbeatManager, MessagingState, WorkerModeState");
    let builder = builder
        .manage(HeartbeatManager {
            status: Arc::new(Mutex::new(HeartbeatStatus::default())),
        })
        .manage(Arc::new(MessagingState::new()))
        .manage(Mutex::new(crate::worker::WorkerModeState::default()));
    
    boot_log("  Registering invoke handlers...");
    let builder = builder.invoke_handler(tauri::generate_handler![
            greet,
            identity::get_identity_status,
            identity::initialize_identity,
            config::get_backend_config_status,
            enrollment::get_enrollment_status,
            enrollment::enroll_node,
            enrollment::sync_capabilities,
            heartbeat::get_heartbeat_status,
            capabilities::get_capabilities,
            capabilities::get_device_capability_profile,
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
        ]);
    boot_log("  Invoke handlers registered");
    
    boot_log("  Configuring setup()...");
    let builder = builder.setup(|app| {
            boot_log("  setup() entered");
            let handle = app.handle().clone();
            
            // Add System Tray Supervisor Path
            #[cfg(desktop)]
            {
                boot_log("  Setting up system tray...");
                if let Some(icon) = app.default_window_icon().cloned() {
                    match TrayIconBuilder::new()
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
                        .build(app)
                    {
                        Ok(_) => boot_log("  System tray: OK"),
                        Err(e) => boot_log(&format!("  System tray: FAILED (non-fatal) — {}", e)),
                    }
                } else {
                    boot_log("  System tray: SKIPPED (no default window icon)");
                }
            }
            #[cfg(mobile)]
            {
                boot_log("  System tray: SKIPPED (mobile build)");
            }

            boot_log("  Starting heartbeat loop...");
            heartbeat::start_heartbeat_loop(handle.clone());
            boot_log("  Heartbeat loop started");
            
            // Start Worker Runtime (Data Plane) — only if user previously opted in.
            // Load persisted opt-in intent (not runtime state).
            let user_opted_in = worker::load_worker_optin(&handle);
            boot_log(&format!("  Worker opt-in loaded: {}", user_opted_in));
            
            // Update in-memory state to reflect persisted intent
            let worker_state = app.state::<Mutex<crate::worker::WorkerModeState>>();
            {
                let mut state = worker_state.blocking_lock();
                state.opted_in = user_opted_in;
            }
            
            if user_opted_in {
                // Check if relay is actually configured before starting the loop
                match crate::config::resolve_relay_endpoint() {
                    Some(ep) => {
                        boot_log(&format!("  Worker relay endpoint found: {}", ep));
                        let app_clone = handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let state = app_clone.state::<Mutex<crate::worker::WorkerModeState>>();
                            let _ = crate::worker::toggle_worker_mode(true, state, app_clone.clone()).await;
                        });
                        boot_log("  Worker loop spawned");
                    }
                    None => {
                        boot_log("  Worker: opted in but relay endpoint not configured. Deferred.");
                        let mut state = worker_state.blocking_lock();
                        state.runtime_status = crate::worker::WorkerRuntimeStatus::Unavailable(
                            "Relay endpoint not configured. Worker Mode is deferred until a relay is available.".to_string()
                        );
                    }
                }
            }

            boot_log("  setup() completed successfully");
            Ok(())
        });
    
    boot_log("Running Tauri application...");
    
    match builder.run(tauri::generate_context!()) {
        Ok(()) => {
            boot_log("Tauri application exited cleanly");
        }
        Err(e) => {
            let msg = format!("Tauri application failed to start: {}", e);
            show_fatal_error(&msg);
        }
    }
}
