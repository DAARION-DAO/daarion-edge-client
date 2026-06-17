/// Sovereign Genesis — Provisioning Module
/// Handles: Thin client communication with Genesis backend
use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::config::resolve_backend_url;
const BETA_MAX_CREATORS: i64 = 10_000;

// ─── Data structures ──────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BetaStatus {
    pub registered: i64,
    pub total: i64,
    pub remaining: i64,
    pub is_open: bool,
    pub slot: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatrixProvisioned {
    pub user_id: String,       // @agentname:daarwizz.space
    pub room_id: String,       // !xxx:daarwizz.space
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProvisioningResult {
    pub beta_slot: i64,
    pub matrix: MatrixProvisioned,
    pub email: String,         // agentname@daarion.city
    pub welcome_sent: bool,
}

// ─── Beta slot check (NODA1 Postgres) ────────────────────────────

#[tauri::command]
pub async fn check_beta_slots() -> Result<BetaStatus, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let backend_url = resolve_backend_url()?;
    let url = format!("{}/genesis/beta-status", backend_url);

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(status) = resp.json::<BetaStatus>().await {
                return Ok(status);
            }
        }
        _ => {}
    }

    // Fallback: optimistic
    Ok(BetaStatus {
        registered: 0,
        total: BETA_MAX_CREATORS,
        remaining: BETA_MAX_CREATORS,
        is_open: true,
        slot: None,
    })
}

// ─── Main provisioning command ────────────────────────────────────

#[tauri::command]
pub async fn provision_sovereign_genesis(
    agent_name: String,
    agent_directive: String,
    solana_pubkey: String,
    evm_address: String,
    device_class: String,
    device_os: String,
    device_ram_gb: f32,
    recommended_model: String,
) -> Result<ProvisioningResult, String> {
    // 1. Check beta slots
    let beta_status = check_beta_slots().await?;
    if !beta_status.is_open {
        return Err("Beta is full. All 10,000 Creator slots have been claimed. Check daarion.city for announcements.".to_string());
    }

    // 2. Prepare payload for Genesis server
    // Note: the backend handles Matrix registration and email provisioning securely
    let registration_payload = serde_json::json!({
        "agent_name": agent_name,
        "agent_directive": agent_directive,
        "solana_pubkey": solana_pubkey,
        "evm_address": evm_address,
        "device_class": device_class,
        "device_os": device_os,
        "device_ram_gb": device_ram_gb,
        "recommended_model": recommended_model,
    });

    let api_client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let backend_url = resolve_backend_url()?;
    // 3. POST to Genesis API
    let resp = match api_client
        .post(format!("{}/genesis/register", backend_url))
        .json(&registration_payload)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => return Err(format!("Server unreachable: {}", e)),
    };

    if !resp.status().is_success() {
        return Err(format!("Genesis API error {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let json: serde_json::Value = resp.json().await.unwrap_or_default();
    
    // Server returns sanitized result
    let slot = json["slot"].as_i64().unwrap_or(1);
    let email = json.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let matrix_room_id = json.get("matrix_room_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let matrix_user_id = json.get("matrix_user_id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    Ok(ProvisioningResult {
        beta_slot: slot,
        matrix: MatrixProvisioned {
            user_id: matrix_user_id,
            room_id: matrix_room_id,
            access_token: "".to_string(), // Strip privileged tokens from client
        },
        email,
        welcome_sent: true,
    })
}

// ─── Creator Profile Registration ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatorProfileRequest {
    pub first_name: String,
    pub last_name: String,
    pub telegram_handle: String,
    pub personal_email: String,
    pub evm_address: String,
    pub agent_name: String,
    pub agent_slot: i64,
}

#[tauri::command]
pub async fn register_creator_profile(profile: CreatorProfileRequest) -> Result<(), String> {
    let api_client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let backend_url = resolve_backend_url()?;
    let url = format!("{}/genesis/creator", backend_url);

    match api_client.post(&url).json(&profile).send().await {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(resp) => Err(format!("Backend rejected creator profile: {}", resp.status())),
        Err(e) => Err(format!("Server unreachable: {}", e)),
    }
}
