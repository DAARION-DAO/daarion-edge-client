use serde::{Deserialize, Serialize};
use ed25519_dalek::{Verifier, VerifyingKey, Signature};
use crate::identity::load_or_create_identity as get_identity;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceLimits {
    pub cpu_limit: f32,
    pub memory_limit_mb: u64,
    pub gpu_allowed: bool,
    pub timeout_sec: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobPayload {
    pub job_type: String,
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lease {
    pub task_id: String,
    pub trace_id: String,
    pub node_target: String,
    pub capabilities_required: Vec<String>,
    pub resource_limits: ResourceLimits,
    pub ttl: i64,
    pub signature: Vec<u8>,
    pub payload: JobPayload,
}

pub struct LeaseValidator;

impl LeaseValidator {
    pub fn validate(app: &tauri::AppHandle, lease: &Lease, public_key_hex: &str) -> Result<(), String> {
        // 1. Check TTL
        let now = chrono::Utc::now().timestamp();
        if lease.ttl < now {
            return Err("Lease expired (TTL check failed)".to_string());
        }

        // 2. Check Node Targeting
        let id_status = get_identity(app).map_err(|e| e.to_string())?;
        if lease.node_target != "*" && (lease.node_target != id_status.node_id) {
            return Err("Lease target mismatch".to_string());
        }

        // 3. Verify Signature
        // The lease content without the signature should be verified.
        // For M1, we assume the signature covers the task_id + payload string.
        let public_key_bytes = hex::decode(public_key_hex)
            .map_err(|_| "Invalid public key hex".to_string())?;
        let verifier = VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap())
            .map_err(|_| "Validating key construction failed".to_string())?;

        let msg = format!("{}:{}", lease.task_id, lease.payload.input);
        let sig = Signature::from_slice(&lease.signature)
            .map_err(|_| "Invalid signature format".to_string())?;

        verifier.verify(msg.as_bytes(), &sig)
            .map_err(|_| "Cryptographic signature verification failed".to_string())?;

        Ok(())
    }
}
