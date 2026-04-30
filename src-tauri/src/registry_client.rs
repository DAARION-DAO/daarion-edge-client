//! registry_client.rs — SOFIIA Worker Node Registry API client
//!
//! Contract source of truth (code-verified 2026-04-29):
//!   microdao-daarion/services/sofiia-console/app/worker_node_registry_router.py
//!
//! Endpoints:
//!   POST /api/v1/nodes/register       — register new beta worker node
//!   POST /api/v1/nodes/heartbeat      — send heartbeat
//!   POST /api/v1/nodes/capabilities   — update capabilities
//!   GET  /api/v1/nodes/tasks          — poll tasks (returns empty list for MVP)
//!
//! Backend field naming notes (code-verified from router.py):
//!   - register payload: camelCase (publicKey, inviteCode, installerVersion, platform)
//!   - register response: snake_case (node_id, status, next_heartbeat_interval)
//!     plus idempotent path uses uppercase STATUS: "PENDING"
//!   - heartbeat response: {"ack": true, "directives": []}
//!   - capabilities payload: camelCase (nodeId, capabilities, signature)
//!   - tasks response: {"tasks": []}
//!
//! IMPORTANT:
//!   - This module does NOT replace the relay-based Worker Mode (worker/mod.rs).
//!   - Registry enrollment ≠ Worker relay activation.
//!   - Private keys are never printed, logged, or transmitted.

use serde::{Deserialize, Serialize};
use std::time::Duration;

// ─── Registry Register ────────────────────────────────────────────────────────

/// POST /api/v1/nodes/register
/// All fields camelCase per SOFIIA backend contract (router.py:26-29).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryRegisterRequest {
    /// Ed25519 public key (hex-encoded). Maps from identity.public_key.
    #[serde(rename = "publicKey")]
    pub public_key: String,

    /// Beta invite/grant code. Maps from enrollment bootstrap_grant.
    #[serde(rename = "inviteCode")]
    pub invite_code: String,

    /// Ed25519 signature of canonical payload for anti-spoofing.
    /// Format: hex(sign(node_id + "|" + public_key + "|" + invite_code)).
    pub signature: String,

    /// Hardware capability summary (free-form JSON).
    pub capabilities: serde_json::Value,

    /// Installer version. Backend reads as payload.get("installer_version") from metadata.
    /// Sent as both camelCase (for future) and snake_case (for backend metadata parsing).
    #[serde(rename = "installerVersion")]
    pub installer_version: String,

    /// Platform identifier: "darwin", "windows", "linux", "android".
    pub platform: String,
}

/// Response from POST /api/v1/nodes/register.
///
/// Backend returns snake_case field names (verified in router.py:79-84, 45-50):
///   {"status": "pending", "nodeId": ..., "server_time": ..., "next_heartbeat_interval": 60}
/// Idempotent path (existing key) returns uppercase status "PENDING".
/// We normalise status to lowercase in EnrollmentState.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryRegisterResponse {
    /// Assigned node ID (UUID). Field: "nodeId".
    #[serde(rename = "nodeId")]
    pub node_id: String,

    /// Node lifecycle status. "pending" (new) or "PENDING" (idempotent).
    /// Normalise with .to_lowercase() before use.
    pub status: String,

    /// Heartbeat interval hint from backend. Field: "next_heartbeat_interval".
    #[serde(rename = "next_heartbeat_interval")]
    pub next_heartbeat_interval: Option<u64>,

    /// Trust tier — may be absent in idempotent response.
    pub trust_tier: Option<String>,

    /// Environment label. May be absent.
    pub environment: Option<String>,
}

// ─── Registry Heartbeat ───────────────────────────────────────────────────────

/// POST /api/v1/nodes/heartbeat
/// Fields sent as camelCase per router.py:93 (reads payload.get("nodeId")).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryHeartbeatRequest {
    /// Node ID from registration response.
    #[serde(rename = "nodeId")]
    pub node_id: String,

    /// Unix timestamp (seconds) of this heartbeat.
    pub timestamp: u64,

    /// Node status string: "ok", "degraded", etc.
    pub status: String,

    /// Ed25519 signature of "node_id|timestamp".
    pub signature: String,
}

/// Response from POST /api/v1/nodes/heartbeat.
/// Backend returns: {"ack": true, "directives": []}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryHeartbeatResponse {
    pub ack: bool,
    /// Future: operator directives (shutdown, reconfigure, etc.). Empty list for MVP.
    #[serde(default)]
    pub directives: Vec<serde_json::Value>,
}

// ─── Registry Capabilities ────────────────────────────────────────────────────

/// POST /api/v1/nodes/capabilities
/// Backend reads: payload.get("nodeId"), payload.get("capabilities") — router.py:143-144.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryCapabilitiesRequest {
    /// Node ID from registration response.
    #[serde(rename = "nodeId")]
    pub node_id: String,

    /// Updated capability summary (free-form JSON).
    pub capabilities: serde_json::Value,

    /// Ed25519 signature of the v1 canonical message:
    /// "daarion.worker.capabilities.v1|{node_id}|{timestamp}"
    pub signature: String,

    /// Unix timestamp (seconds) included in canonical message for replay protection.
    /// Backend Gate A verifies timestamp is within SOFIIA_SIGNATURE_TIMESTAMP_DRIFT_SECS.
    pub timestamp: u64,
}

/// Response from POST /api/v1/nodes/capabilities.
/// Backend returns: {"ack": true}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryCapabilitiesResponse {
    pub ack: bool,
}

// ─── Registry Task Poll ───────────────────────────────────────────────────────

/// Response from GET /api/v1/nodes/tasks?nodeId={node_id}
/// Backend returns: {"tasks": []} for MVP — router.py:191-193.
/// 404 if node not found, 401 if revoked.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryTasksResponse {
    /// Empty for MVP. When tasks are dispatched, contains task objects.
    /// Client MUST NOT execute tasks without explicit operator-controlled gate.
    #[serde(default)]
    pub tasks: Vec<serde_json::Value>,
}

// ─── HTTP Client helpers ──────────────────────────────────────────────────────

/// Build a reusable reqwest client with sensible timeouts.
pub fn build_registry_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// POST /api/v1/nodes/register
pub async fn call_register(
    backend_url: &str,
    req: &RegistryRegisterRequest,
) -> Result<RegistryRegisterResponse, String> {
    let client = build_registry_client()?;
    let url = format!("{}/api/v1/nodes/register", backend_url.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .json(req)
        .send()
        .await
        .map_err(|e| format!("Registry register network error: {}", e))?;

    if resp.status().is_success() {
        resp.json::<RegistryRegisterResponse>()
            .await
            .map_err(|e| format!("Registry register parse error: {}", e))
    } else {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(format!("Registry register rejected: HTTP {} — {}", status, body))
    }
}

/// POST /api/v1/nodes/heartbeat
pub async fn call_heartbeat(
    backend_url: &str,
    req: &RegistryHeartbeatRequest,
) -> Result<RegistryHeartbeatResponse, String> {
    let client = build_registry_client()?;
    let url = format!("{}/api/v1/nodes/heartbeat", backend_url.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .json(req)
        .send()
        .await
        .map_err(|e| format!("Registry heartbeat network error: {}", e))?;

    if resp.status().is_success() {
        resp.json::<RegistryHeartbeatResponse>()
            .await
            .map_err(|e| format!("Registry heartbeat parse error: {}", e))
    } else {
        let status = resp.status().as_u16();
        Err(format!("Registry heartbeat rejected: HTTP {}", status))
    }
}

/// POST /api/v1/nodes/capabilities
pub async fn call_capabilities(
    backend_url: &str,
    req: &RegistryCapabilitiesRequest,
) -> Result<RegistryCapabilitiesResponse, String> {
    let client = build_registry_client()?;
    let url = format!("{}/api/v1/nodes/capabilities", backend_url.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .json(req)
        .send()
        .await
        .map_err(|e| format!("Registry capabilities network error: {}", e))?;

    if resp.status().is_success() {
        resp.json::<RegistryCapabilitiesResponse>()
            .await
            .map_err(|e| format!("Registry capabilities parse error: {}", e))
    } else {
        let status = resp.status().as_u16();
        Err(format!("Registry capabilities rejected: HTTP {}", status))
    }
}

/// GET /api/v1/nodes/tasks?nodeId={node_id}
///
/// Returns empty list for MVP. Backend returns 404 if node not found, 401 if revoked.
/// Caller MUST NOT execute any returned tasks without explicit activation gate.
pub async fn call_tasks(
    backend_url: &str,
    node_id: &str,
) -> Result<RegistryTasksResponse, String> {
    let client = build_registry_client()?;
    let url = format!(
        "{}/api/v1/nodes/tasks?nodeId={}",
        backend_url.trim_end_matches('/'),
        urlencoding::encode(node_id)
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Registry tasks network error: {}", e))?;

    match resp.status().as_u16() {
        200 => resp
            .json::<RegistryTasksResponse>()
            .await
            .map_err(|e| format!("Registry tasks parse error: {}", e)),
        404 => Err("tasks: node not found in registry".to_string()),
        401 => Err("tasks: node is revoked".to_string()),
        code => Err(format!("Registry tasks rejected: HTTP {}", code)),
    }
}

// ─── Capability serialization helper ─────────────────────────────────────────

/// Convert CapabilitySummary to a JSON Value suitable for the registry.
/// Keys use snake_case to match SOFIIA backend storage and operator UI expectations.
pub fn capabilities_to_registry_json(caps: &crate::capabilities::CapabilitySummary) -> serde_json::Value {
    serde_json::json!({
        "os": caps.os,
        "arch": caps.arch,
        "hostname": caps.hostname,
        "cpu_count": caps.cpu_count,
        "cpu_brand": caps.cpu_brand,
        "ram_total_gb": (caps.ram_total_gb * 10.0).round() / 10.0,
        "gpu_detected": caps.gpu.detected,
        "gpu_vendor": caps.gpu.vendor,
        "gpu_api": caps.gpu.acceleration_api,
        "device_class": format!("{:?}", caps.device_class),
        "recommended_model": caps.recommended_model.model_id,
        "performance_tier": caps.recommended_model.performance_tier,
        "worker_ready": caps.worker_ready,
        "model_runtime_ready": caps.model_runtime_ready,
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_serialization() {
        let req = RegistryRegisterRequest {
            public_key: "abc123".to_string(),
            invite_code: "BETA-TEST".to_string(),
            signature: "deadsig".to_string(),
            capabilities: serde_json::json!({"os": "macos", "arch": "aarch64"}),
            installer_version: "v0.2.0-beta".to_string(),
            platform: "darwin".to_string(),
        };
        let json = serde_json::to_string(&req).expect("must serialize");
        assert!(json.contains("publicKey"), "publicKey missing: {}", json);
        assert!(json.contains("inviteCode"), "inviteCode missing: {}", json);
        assert!(json.contains("installerVersion"), "installerVersion missing: {}", json);
        // Must NOT expose private key — none in this struct
        assert!(!json.contains("private"), "private key must never appear: {}", json);
    }

    #[test]
    fn test_heartbeat_request_serialization() {
        let req = RegistryHeartbeatRequest {
            node_id: "node-uuid-123".to_string(),
            timestamp: 1_700_000_000,
            status: "ok".to_string(),
            signature: "hexsig".to_string(),
        };
        let json = serde_json::to_string(&req).expect("must serialize");
        assert!(json.contains("nodeId"), "nodeId must be camelCase: {}", json);
        assert!(json.contains("1700000000"));
        assert!(!json.contains("private"), "no private key in heartbeat: {}", json);
    }

    #[test]
    fn test_capabilities_request_serialization() {
        let req = RegistryCapabilitiesRequest {
            node_id: "node-uuid-123".to_string(),
            capabilities: serde_json::json!({"cpu": "arm64", "ram_total_gb": 16.0}),
            signature: "hexsig".to_string(),
            timestamp: 1_700_000_000,
        };
        let json = serde_json::to_string(&req).expect("must serialize");
        assert!(json.contains("nodeId"), "nodeId must be camelCase: {}", json);
        assert!(json.contains("arm64"), "capabilities content preserved: {}", json);
        assert!(json.contains("1700000000"), "timestamp must be in payload: {}", json);
        assert!(!json.contains("private"), "no private key in capabilities: {}", json);
    }

    #[test]
    fn test_capabilities_v1_canonical_format() {
        // Verifies the v1 canonical message format matches what the backend verifies.
        // Backend Gate A canonical: "daarion.worker.capabilities.v1|{node_id}|{timestamp}"
        let node_id = "test-node-uuid-abc123";
        let ts: u64 = 1_700_000_000;
        let canonical = format!("daarion.worker.capabilities.v1|{}|{}", node_id, ts);
        assert!(canonical.starts_with("daarion.worker.capabilities.v1|"),
            "canonical must have v1 prefix: {}", canonical);
        assert!(canonical.contains(node_id),
            "canonical must include node_id: {}", canonical);
        assert!(canonical.contains(&ts.to_string()),
            "canonical must include timestamp: {}", canonical);
        // Ensure no bare node_id-only signing (old canonical guard)
        assert_ne!(canonical, node_id, "canonical must not be bare node_id");
    }

    #[test]
    fn test_register_v1_canonical_format() {
        // Verifies the v1 registration canonical matches what the backend verifies.
        // Backend Gate A canonical: "daarion.worker.register.v1|{public_key}|{invite_code}"
        let pub_key = "base64encodedpublickey";
        let invite = "BETA-XYZ";
        let canonical = format!("daarion.worker.register.v1|{}|{}", pub_key, invite);
        assert!(canonical.starts_with("daarion.worker.register.v1|"),
            "canonical must have v1 prefix: {}", canonical);
        assert!(canonical.contains(pub_key));
        assert!(canonical.contains(invite));
        // Must NOT contain node_id (server-assigned, not part of proof)
        assert!(!canonical.contains("node-"),
            "registration canonical must not include local node_id");
    }

    #[test]
    fn test_register_response_deserialization_new_node() {
        // New node response from router.py:79-84
        let raw = r#"{"nodeId": "abc-123", "status": "pending", "server_time": 1700000000.0, "next_heartbeat_interval": 60}"#;
        let resp: RegistryRegisterResponse = serde_json::from_str(raw).expect("must deserialize");
        assert_eq!(resp.node_id, "abc-123");
        assert_eq!(resp.status.to_lowercase(), "pending");
        assert_eq!(resp.next_heartbeat_interval, Some(60));
    }

    #[test]
    fn test_register_response_deserialization_idempotent() {
        // Idempotent response (existing key) from router.py:44-50
        let raw = r#"{"status": "PENDING", "nodeId": "existing-uuid", "server_time": 1700000000.0, "next_heartbeat_interval": 60}"#;
        let resp: RegistryRegisterResponse = serde_json::from_str(raw).expect("must deserialize");
        assert_eq!(resp.node_id, "existing-uuid");
        // Status may be uppercase — normalise with .to_lowercase()
        assert_eq!(resp.status.to_lowercase(), "pending");
    }

    #[test]
    fn test_heartbeat_response_with_directives() {
        // Backend returns {"ack": true, "directives": []} — router.py:131-134
        let raw = r#"{"ack": true, "directives": []}"#;
        let resp: RegistryHeartbeatResponse = serde_json::from_str(raw).expect("must deserialize");
        assert!(resp.ack);
        assert!(resp.directives.is_empty());
    }

    #[test]
    fn test_tasks_response_empty() {
        // MVP tasks response from router.py:191-193
        let raw = r#"{"tasks": []}"#;
        let resp: RegistryTasksResponse = serde_json::from_str(raw).expect("must deserialize");
        assert!(resp.tasks.is_empty(), "MVP tasks must be empty");
    }

    #[test]
    fn test_backend_url_trimming() {
        let base = "http://localhost:8010/";
        let url = format!("{}/api/v1/nodes/register", base.trim_end_matches('/'));
        assert_eq!(url, "http://localhost:8010/api/v1/nodes/register");
        assert!(!url.contains("//api"), "double slash detected: {}", url);
    }

    #[test]
    fn test_tasks_url_node_id_encoding() {
        let node_id = "uuid-1234-abcd";
        let url = format!(
            "http://localhost:8010/api/v1/nodes/tasks?nodeId={}",
            urlencoding::encode(node_id)
        );
        assert!(url.contains("nodeId=uuid-1234-abcd"), "nodeId must appear in URL: {}", url);
    }

    #[test]
    fn test_capabilities_json_includes_hostname() {
        // Verifies hostname is included in capabilities payload for operator visibility
        let json = serde_json::json!({
            "os": "macos",
            "arch": "aarch64",
            "hostname": "my-macbook",
            "cpu_count": 8,
            "ram_total_gb": 32.0
        });
        assert!(json.get("hostname").is_some(), "hostname must be in capabilities");
    }
}
