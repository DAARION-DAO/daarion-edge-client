//! enrollment.rs — SOFIIA Worker Node Registry enrollment
//!
//! MIGRATION NOTE (2026-04-29):
//!   Legacy endpoint: POST {backend_url}/edge/enroll (identity-service-v2, transitional)
//!   New endpoint:    POST {backend_url}/api/v1/nodes/register (SOFIIA Worker Registry, canonical)
//!
//! The new contract uses camelCase JSON fields and returns a "pending" status
//! until the operator approves the node in the SOFIIA Console Beta Worker Registry.
//!
//! Worker relay mode (worker/mod.rs) is separate from registry enrollment.
//! Registry enrollment does NOT activate the WebSocket relay loop.

use keyring::Entry;
use crate::capabilities::{get_capabilities};
use crate::registry_client::{
    RegistryRegisterRequest, RegistryCapabilitiesRequest,
    call_register, call_capabilities, capabilities_to_registry_json,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use std::path::PathBuf;
use std::fs;
use crate::identity::load_or_create_identity;
use crate::config::resolve_backend_url;
use ed25519_dalek::Signer;

// ─── Structs ──────────────────────────────────────────────────────────────────

/// Preserved for backward compatibility with frontend command interface.
/// Fields that are no longer meaningful from the registry are kept as Option<T>.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EnrollmentState {
    pub enrolled: bool,
    pub node_id: Option<String>,
    pub credential_scope: Option<String>,
    pub heartbeat_interval_sec: u64,
    pub environment: Option<String>,
    pub status: Option<String>,
    pub last_enrollment_error: Option<String>,

    // Registry-specific fields (new in registry integration)
    pub trust_tier: Option<String>,
    pub registry_mode: bool,  // true = using SOFIIA registry, false = legacy

    // Trust-Aware Fields (preserved for future use)
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

// ─── Signing helper ───────────────────────────────────────────────────────────

/// Sign a canonical message with the node's Ed25519 private key.
/// Returns hex-encoded signature. Private key is read from OS keyring and
/// immediately dropped after signing — never stored or printed.
fn sign_payload(handle: &AppHandle, message: &str) -> Result<String, String> {
    let signing_key = crate::identity::get_signing_key(handle)?;
    let sig = signing_key.sign(message.as_bytes());
    Ok(hex::encode(sig.to_bytes()))
}

// ─── Tauri Commands ───────────────────────────────────────────────────────────

/// Enroll this node with the SOFIIA Worker Node Registry.
///
/// Uses POST /api/v1/nodes/register. Registration result is typically "pending"
/// until a SOFIIA operator approves the node in the Beta Worker Registry UI.
///
/// On success, saves node_id, status, and trust_tier to local enrollment.json.
/// On network failure, preserves any existing enrollment state without purging.
///
/// NOTE: This does NOT activate Worker Mode relay. Registry enrollment and
/// relay activation are separate flows.
#[tauri::command]
pub async fn enroll_node(handle: AppHandle, bootstrap_grant: String) -> Result<EnrollmentState, String> {
    let identity = load_or_create_identity(&handle)?;
    let backend_url = resolve_backend_url();
    let capabilities = get_capabilities();

    // Build canonical signature payload: node_id|public_key|invite_code
    // This proves the caller possesses the private key for this public_key.
    let sig_payload = format!("{}|{}|{}", identity.node_id, identity.public_key, bootstrap_grant);
    let signature = sign_payload(&handle, &sig_payload).unwrap_or_else(|e| {
        println!("[enrollment] Signature failed (non-fatal for MVP): {}", e);
        "unsigned".to_string()  // Backend validates but won't reject for beta
    });

    let platform = std::env::consts::OS.to_string();
    let installer_version = "v0.2.0-beta".to_string();

    let request = RegistryRegisterRequest {
        public_key: identity.public_key.clone(),
        invite_code: bootstrap_grant,
        signature,
        capabilities: capabilities_to_registry_json(&capabilities),
        installer_version,
        platform,
    };

    println!("[enrollment] Registering with SOFIIA registry: {}/api/v1/nodes/register", backend_url);

    let mut existing_state = load_enrollment_state(&handle);

    match call_register(&backend_url, &request).await {
        Ok(resp) => {
            println!("[enrollment] Registration OK — node_id={}, status={}", resp.node_id, resp.status);

            // Registry nodes start as "pending" — this is expected, not an error.
            // enrolled=true only when backend says "active".
            let status_lower = resp.status.to_lowercase();
            existing_state.enrolled = status_lower == "active";
            existing_state.node_id = Some(resp.node_id);
            existing_state.status = Some(resp.status);
            existing_state.trust_tier = resp.trust_tier;
            // Backend returns next_heartbeat_interval (not heartbeat_interval_sec)
            existing_state.heartbeat_interval_sec = resp.next_heartbeat_interval.unwrap_or(60);
            existing_state.environment = resp.environment.or_else(|| Some("beta".to_string()));
            existing_state.credential_scope = Some("registry:beta".to_string());
            existing_state.registry_mode = true;
            existing_state.last_enrollment_error = None;
        }
        Err(e) => {
            println!("[enrollment] Registration failed: {}", e);
            // Preserve existing node_id and status — do NOT purge identity on transient failure.
            existing_state.last_enrollment_error = Some(e);
            existing_state.registry_mode = true;  // Intent was registry mode even if failed
        }
    }

    save_enrollment_state(&handle, &existing_state)?;
    Ok(existing_state)
}

#[tauri::command]
pub fn get_enrollment_status(handle: AppHandle) -> EnrollmentState {
    load_enrollment_state(&handle)
}

/// Sync latest hardware capabilities to the SOFIIA registry.
///
/// Should be called:
///   - immediately after successful registration (enrollment.rs calls this internally)
///   - on demand via frontend (e.g. after model install changes worker_ready status)
///   - NOT continuously — capabilities are relatively static hardware facts
///
/// Safe to call on pending nodes. Backend accepts capabilities updates from pending/active nodes.
/// Revoked nodes receive 401 and the error is returned — no local state is purged.
#[tauri::command]
pub async fn sync_capabilities(handle: AppHandle) -> Result<bool, String> {
    let state = load_enrollment_state(&handle);

    let node_id = match &state.node_id {
        Some(id) if !id.is_empty() => id.clone(),
        _ => return Err("sync_capabilities: no node_id — enroll first".to_string()),
    };

    let backend_url = resolve_backend_url();
    let caps = get_capabilities();
    let cap_json = capabilities_to_registry_json(&caps);

    // Sign node_id to prove authenticity
    let signature = crate::identity::get_signing_key(&handle)
        .map(|sk| {
            use ed25519_dalek::Signer;
            hex::encode(sk.sign(node_id.as_bytes()).to_bytes())
        })
        .unwrap_or_else(|_| "unsigned".to_string());

    let req = RegistryCapabilitiesRequest {
        node_id: node_id.clone(),
        capabilities: cap_json,
        signature,
    };

    match call_capabilities(&backend_url, &req).await {
        Ok(resp) => {
            println!("[capabilities] Sync OK for node {}", &node_id[..8.min(node_id.len())]);
            Ok(resp.ack)
        }
        Err(e) => {
            println!("[capabilities] Sync failed: {}", e);
            Err(e)
        }
    }
}
