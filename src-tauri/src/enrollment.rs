use keyring::Entry;
use crate::capabilities::{get_capabilities, CapabilitySummary};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use std::path::PathBuf;
use std::fs;
use crate::identity::load_or_create_identity;
use crate::config::get_config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnrollmentRequest {
    pub local_device_id: String,
    pub public_key: String,
    pub device_kind: String,
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub app_version: String,
    pub bootstrap_grant: String,
    pub capability_summary: CapabilitySummary,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnrollmentResponse {
    pub node_id: String,
    pub status: String,
    pub credential_scope: String,
    pub node_token: String,
    pub heartbeat_interval_sec: u64,
    pub environment: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EnrollmentState {
    pub enrolled: bool,
    pub node_id: Option<String>,
    pub credential_scope: Option<String>,
    pub heartbeat_interval_sec: u64,
    pub environment: Option<String>,
    pub last_enrollment_error: Option<String>,
    
    // Trust-Aware Fields
    pub csr_generated: bool,
    pub csr_submitted: bool,
    pub certificate_issued: bool,
    pub issuer_id: Option<String>,
    pub region_scope: Option<String>,
    pub district_scope: Option<String>,
    pub valid_until: Option<u64>,
    pub next_renewal_at: Option<u64>,
}

const ENROLLMENT_FILE: &str = "enrollment.json";
const SERVICE_NAME: &str = "com.daarion.edge.node";
const TOKEN_KEY: &str = "node_token";

fn get_app_dir(handle: &AppHandle) -> PathBuf {
    handle.path().app_data_dir().expect("Failed to get app data dir")
}

pub fn load_enrollment_state(handle: &AppHandle) -> EnrollmentState {
    let app_dir = get_app_dir(handle);
    let path = app_dir.join(ENROLLMENT_FILE);
    
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(state) = serde_json::from_str(&content) {
                return state;
            }
        }
    }
    EnrollmentState::default()
}

pub fn save_enrollment_state(handle: &AppHandle, state: &EnrollmentState) -> Result<(), String> {
    let app_dir = get_app_dir(handle);
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    }
    let path = app_dir.join(ENROLLMENT_FILE);
    let content = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn get_node_token() -> Result<String, String> {
    let entry = Entry::new(SERVICE_NAME, TOKEN_KEY).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn enroll_node(handle: AppHandle, bootstrap_grant: String) -> Result<EnrollmentState, String> {
    let identity = load_or_create_identity(&handle)?;
    let config = get_config();
    let capabilities = get_capabilities();
    
    let request = EnrollmentRequest {
        local_device_id: identity.node_id.clone(),
        public_key: identity.public_key.clone(),
        device_kind: "edge".to_string(),
        os: capabilities.os.clone(),
        arch: capabilities.arch.clone(),
        hostname: capabilities.hostname.clone(),
        app_version: "0.1.0".to_string(),
        bootstrap_grant,
        capability_summary: capabilities,
    };

    println!("Attempting enrollment with backend: {}", config.backend_url);
    
    // Mocking successful enrollment for the purpose of demonstrating the UI/Lifecycle
    if request.bootstrap_grant == "error-test" {
        return Err("Invalid bootstrap grant".to_string());
    }

    let mock_response = EnrollmentResponse {
        node_id: format!("node-{}", &request.local_device_id[..8]),
        status: "active".to_string(),
        credential_scope: "edge:worker:standard".to_string(),
        node_token: "mock-jwt-node-token-6789".to_string(),
        heartbeat_interval_sec: 30,
        environment: config.environment.clone(),
    };

    // Save token to secure storage
    let entry = Entry::new(SERVICE_NAME, TOKEN_KEY).map_err(|e| e.to_string())?;
    entry.set_password(&mock_response.node_token).map_err(|e| e.to_string())?;

    let mut state = load_enrollment_state(&handle);
    state.enrolled = true;
    state.node_id = Some(mock_response.node_id);
    state.credential_scope = Some(mock_response.credential_scope);
    state.heartbeat_interval_sec = mock_response.heartbeat_interval_sec;
    state.environment = Some(mock_response.environment);
    state.last_enrollment_error = None;

    save_enrollment_state(&handle, &state)?;
    
    Ok(state)
}

#[tauri::command]
pub fn get_enrollment_status(handle: AppHandle) -> EnrollmentState {
    load_enrollment_state(&handle)
}
