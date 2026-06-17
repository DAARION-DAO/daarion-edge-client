//! backend_health.rs — pairing-aware backend connectivity diagnostics
//!
//! This module answers only whether the configured backend is reachable and
//! compatible with the public health contract. It does not authenticate,
//! mutate pairing, enroll identities, or verify trust.

use crate::config::{get_backend_config_status, BackendConfigStatus};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

const HEALTH_PATH: &str = "/api/v1/edge/health";
const HEALTH_TIMEOUT_SECS: u64 = 5;
const EXPECTED_SERVICE: &str = "daarion-edge-backend";
const SUPPORTED_SCHEMA_VERSION: u64 = 1;
const SUPPORTED_EDGE_PROTOCOL_VERSION: &str = "1.0.0";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackendHealthState {
    PairingRequired,
    Offline,
    ContractInvalid,
    VersionMismatch,
    Online,
    Degraded,
    Maintenance,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BackendHealthStatus {
    pub state: BackendHealthState,
    pub checked_at: DateTime<Utc>,
    pub backend_label: Option<String>,
    pub environment: Option<String>,
    pub http_status: Option<u16>,
    pub backend_status: Option<String>,
    pub backend_version: Option<String>,
    pub edge_protocol_version: Option<String>,
    pub min_edge_client_version: Option<String>,
    pub server_time: Option<String>,
    pub capabilities: Option<HashMap<String, bool>>,
    pub message: String,
}

pub struct BackendHealthManager {
    pub last_status: Arc<Mutex<Option<BackendHealthStatus>>>,
}

impl Default for BackendHealthManager {
    fn default() -> Self {
        Self {
            last_status: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Debug, Deserialize)]
struct BackendHealthResponse {
    schema_version: u64,
    status: BackendReportedStatus,
    service: String,
    environment: BackendEnvironment,
    backend_version: String,
    edge_protocol_version: String,
    min_edge_client_version: String,
    server_time: String,
    capabilities: HashMap<String, bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BackendReportedStatus {
    Ok,
    Degraded,
    Maintenance,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum BackendEnvironment {
    Production,
    Staging,
    Development,
}

impl BackendEnvironment {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Staging => "staging",
            Self::Development => "development",
        }
    }
}

#[tauri::command]
pub async fn get_backend_health_status(
    handle: tauri::AppHandle,
    state: tauri::State<'_, BackendHealthManager>,
) -> Result<BackendHealthStatus, String> {
    run_and_cache_backend_health_check(&handle, &state).await
}

#[tauri::command]
pub async fn check_backend_health(
    handle: tauri::AppHandle,
    state: tauri::State<'_, BackendHealthManager>,
) -> Result<BackendHealthStatus, String> {
    run_and_cache_backend_health_check(&handle, &state).await
}

async fn run_and_cache_backend_health_check(
    handle: &tauri::AppHandle,
    state: &tauri::State<'_, BackendHealthManager>,
) -> Result<BackendHealthStatus, String> {
    let status = run_backend_health_check(handle).await;
    *state.last_status.lock().await = Some(status.clone());
    Ok(status)
}

async fn run_backend_health_check(handle: &tauri::AppHandle) -> BackendHealthStatus {
    let config_status = get_backend_config_status(handle.clone());
    if !config_status.configured {
        return pairing_required_status(&config_status, Utc::now());
    }

    let Some(backend_url) = config_status.backend_url.as_deref() else {
        return pairing_required_status(&config_status, Utc::now());
    };

    perform_health_request(&config_status, backend_url).await
}

async fn perform_health_request(
    config_status: &BackendConfigStatus,
    backend_url: &str,
) -> BackendHealthStatus {
    let checked_at = Utc::now();
    let url = format!("{}{}", backend_url.trim_end_matches('/'), HEALTH_PATH);
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(HEALTH_TIMEOUT_SECS))
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            return basic_status(
                BackendHealthState::Offline,
                config_status,
                checked_at,
                None,
                "Backend health client could not be initialized.",
            )
        }
    };

    let response = match client.get(url).send().await {
        Ok(response) => response,
        Err(error) => {
            let message = if error.is_timeout() {
                "Backend health check timed out."
            } else {
                "Backend health check failed."
            };
            return basic_status(
                BackendHealthState::Offline,
                config_status,
                checked_at,
                None,
                message,
            );
        }
    };

    let http_status = response.status();
    if http_status == StatusCode::UNAUTHORIZED || http_status == StatusCode::FORBIDDEN {
        return basic_status(
            BackendHealthState::ContractInvalid,
            config_status,
            checked_at,
            Some(http_status.as_u16()),
            "Public backend health endpoint must not require identity auth.",
        );
    }

    if !http_status.is_success() {
        return basic_status(
            BackendHealthState::Offline,
            config_status,
            checked_at,
            Some(http_status.as_u16()),
            "Backend health endpoint returned a non-success status.",
        );
    }

    let body = match response.text().await {
        Ok(body) => body,
        Err(_) => {
            return basic_status(
                BackendHealthState::Offline,
                config_status,
                checked_at,
                Some(http_status.as_u16()),
                "Backend health response could not be read.",
            )
        }
    };

    evaluate_health_response(config_status, http_status.as_u16(), &body, checked_at)
}

fn evaluate_health_response(
    config_status: &BackendConfigStatus,
    http_status: u16,
    body: &str,
    checked_at: DateTime<Utc>,
) -> BackendHealthStatus {
    let response = match serde_json::from_str::<BackendHealthResponse>(body) {
        Ok(response) => response,
        Err(_) => {
            return basic_status(
                BackendHealthState::ContractInvalid,
                config_status,
                checked_at,
                Some(http_status),
                "Backend health response did not match the public contract.",
            )
        }
    };

    if response.schema_version != SUPPORTED_SCHEMA_VERSION {
        return health_status_from_response(
            BackendHealthState::VersionMismatch,
            config_status,
            checked_at,
            http_status,
            &response,
            "Backend health schema version is not supported.",
        );
    }

    if response.service != EXPECTED_SERVICE {
        return health_status_from_response(
            BackendHealthState::ContractInvalid,
            config_status,
            checked_at,
            http_status,
            &response,
            "Backend health service identifier is invalid.",
        );
    }

    if !is_edge_protocol_compatible(&response.edge_protocol_version)
        || !is_client_version_supported(&response.min_edge_client_version)
    {
        return health_status_from_response(
            BackendHealthState::VersionMismatch,
            config_status,
            checked_at,
            http_status,
            &response,
            "Backend health contract is not compatible with this client version.",
        );
    }

    if chrono::DateTime::parse_from_rfc3339(&response.server_time).is_err() {
        return health_status_from_response(
            BackendHealthState::ContractInvalid,
            config_status,
            checked_at,
            http_status,
            &response,
            "Backend health server_time is invalid.",
        );
    }

    let state = match response.status {
        BackendReportedStatus::Ok => BackendHealthState::Online,
        BackendReportedStatus::Degraded => BackendHealthState::Degraded,
        BackendReportedStatus::Maintenance => BackendHealthState::Maintenance,
    };

    let message = match state {
        BackendHealthState::Online => "Backend is online and compatible.",
        BackendHealthState::Degraded => "Backend reports degraded service.",
        BackendHealthState::Maintenance => "Backend reports maintenance mode.",
        _ => "Backend health status resolved.",
    };

    health_status_from_response(
        state,
        config_status,
        checked_at,
        http_status,
        &response,
        message,
    )
}

fn is_edge_protocol_compatible(protocol_version: &str) -> bool {
    let Ok(supported) = Version::parse(SUPPORTED_EDGE_PROTOCOL_VERSION) else {
        return false;
    };
    let Ok(protocol) = Version::parse(protocol_version) else {
        return false;
    };

    protocol.major == supported.major
}

fn is_client_version_supported(min_client_version: &str) -> bool {
    let Ok(current) = Version::parse(env!("CARGO_PKG_VERSION")) else {
        return false;
    };
    let Ok(minimum) = Version::parse(min_client_version) else {
        return false;
    };

    current >= minimum
}

fn pairing_required_status(
    config_status: &BackendConfigStatus,
    checked_at: DateTime<Utc>,
) -> BackendHealthStatus {
    basic_status(
        BackendHealthState::PairingRequired,
        config_status,
        checked_at,
        None,
        "Pairing required before backend health can be checked.",
    )
}

fn basic_status(
    state: BackendHealthState,
    config_status: &BackendConfigStatus,
    checked_at: DateTime<Utc>,
    http_status: Option<u16>,
    message: &str,
) -> BackendHealthStatus {
    BackendHealthStatus {
        state,
        checked_at,
        backend_label: config_status.pairing_label.clone(),
        environment: Some(config_status.environment.clone()),
        http_status,
        backend_status: None,
        backend_version: None,
        edge_protocol_version: None,
        min_edge_client_version: None,
        server_time: None,
        capabilities: None,
        message: message.to_string(),
    }
}

fn health_status_from_response(
    state: BackendHealthState,
    config_status: &BackendConfigStatus,
    checked_at: DateTime<Utc>,
    http_status: u16,
    response: &BackendHealthResponse,
    message: &str,
) -> BackendHealthStatus {
    BackendHealthStatus {
        state,
        checked_at,
        backend_label: config_status.pairing_label.clone(),
        environment: Some(response.environment.as_str().to_string()),
        http_status: Some(http_status),
        backend_status: Some(
            match response.status {
                BackendReportedStatus::Ok => "ok",
                BackendReportedStatus::Degraded => "degraded",
                BackendReportedStatus::Maintenance => "maintenance",
            }
            .to_string(),
        ),
        backend_version: Some(response.backend_version.clone()),
        edge_protocol_version: Some(response.edge_protocol_version.clone()),
        min_edge_client_version: Some(response.min_edge_client_version.clone()),
        server_time: Some(response.server_time.clone()),
        capabilities: Some(response.capabilities.clone()),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_status() -> BackendConfigStatus {
        BackendConfigStatus {
            configured: true,
            backend_url: Some("https://backend.daarion.city".to_string()),
            environment: "production".to_string(),
            dev_default: false,
            paired: true,
            pairing_label: Some("Test Backend".to_string()),
            pairing_source: Some("invite_payload".to_string()),
            connection_status: "not_checked".to_string(),
            message: "paired".to_string(),
        }
    }

    fn unconfigured_status() -> BackendConfigStatus {
        BackendConfigStatus {
            configured: false,
            backend_url: None,
            environment: "unconfigured".to_string(),
            dev_default: false,
            paired: false,
            pairing_label: None,
            pairing_source: None,
            connection_status: "not_checked".to_string(),
            message: "Pairing required".to_string(),
        }
    }

    fn body(status: &str) -> String {
        format!(
            r#"{{
                "schema_version": 1,
                "status": "{}",
                "service": "daarion-edge-backend",
                "environment": "production",
                "backend_version": "0.1.0",
                "edge_protocol_version": "1.0.0",
                "min_edge_client_version": "0.2.2-3",
                "server_time": "2026-06-17T00:00:00Z",
                "capabilities": {{
                    "genesis": true,
                    "registry": true
                }}
            }}"#,
            status
        )
    }

    fn evaluate(body: &str) -> BackendHealthStatus {
        evaluate_health_response(&config_status(), 200, body, Utc::now())
    }

    #[test]
    fn valid_ok_maps_to_online() {
        let status = evaluate(&body("ok"));
        assert_eq!(status.state, BackendHealthState::Online);
        assert_eq!(status.backend_status.as_deref(), Some("ok"));
    }

    #[test]
    fn valid_degraded_maps_to_degraded() {
        let status = evaluate(&body("degraded"));
        assert_eq!(status.state, BackendHealthState::Degraded);
        assert_eq!(status.backend_status.as_deref(), Some("degraded"));
    }

    #[test]
    fn valid_maintenance_maps_to_maintenance() {
        let status = evaluate(&body("maintenance"));
        assert_eq!(status.state, BackendHealthState::Maintenance);
        assert_eq!(status.backend_status.as_deref(), Some("maintenance"));
    }

    #[test]
    fn invalid_json_maps_to_contract_invalid() {
        let status = evaluate("{");
        assert_eq!(status.state, BackendHealthState::ContractInvalid);
    }

    #[test]
    fn missing_required_field_maps_to_contract_invalid() {
        let status = evaluate(r#"{"schema_version":1,"status":"ok"}"#);
        assert_eq!(status.state, BackendHealthState::ContractInvalid);
    }

    #[test]
    fn unsupported_schema_maps_to_version_mismatch() {
        let status =
            evaluate(&body("ok").replace(r#""schema_version": 1"#, r#""schema_version": 2"#));
        assert_eq!(status.state, BackendHealthState::VersionMismatch);
    }

    #[test]
    fn incompatible_protocol_maps_to_version_mismatch() {
        let status = evaluate(&body("ok").replace(
            r#""edge_protocol_version": "1.0.0""#,
            r#""edge_protocol_version": "2.0.0""#,
        ));
        assert_eq!(status.state, BackendHealthState::VersionMismatch);
    }

    #[test]
    fn too_high_min_client_maps_to_version_mismatch() {
        let status = evaluate(&body("ok").replace(
            r#""min_edge_client_version": "0.2.2-3""#,
            r#""min_edge_client_version": "999.0.0""#,
        ));
        assert_eq!(status.state, BackendHealthState::VersionMismatch);
    }

    #[test]
    fn unauthorized_maps_to_contract_invalid() {
        let status = basic_status(
            BackendHealthState::ContractInvalid,
            &config_status(),
            Utc::now(),
            Some(401),
            "Public backend health endpoint must not require identity auth.",
        );
        assert_eq!(status.state, BackendHealthState::ContractInvalid);
        assert_eq!(status.http_status, Some(401));
    }

    #[test]
    fn non_success_maps_to_offline() {
        let status = basic_status(
            BackendHealthState::Offline,
            &config_status(),
            Utc::now(),
            Some(503),
            "Backend health endpoint returned a non-success status.",
        );
        assert_eq!(status.state, BackendHealthState::Offline);
        assert_eq!(status.http_status, Some(503));
    }

    #[test]
    fn no_configured_backend_maps_to_pairing_required() {
        let status = pairing_required_status(&unconfigured_status(), Utc::now());
        assert_eq!(status.state, BackendHealthState::PairingRequired);
        assert_eq!(status.http_status, None);
    }
}
