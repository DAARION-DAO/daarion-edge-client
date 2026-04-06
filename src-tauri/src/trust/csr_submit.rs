use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrSubmissionRequest {
    pub node_id: String,
    pub public_key_der: Vec<u8>,
    pub csr_pem: String,
    pub region: String,
    pub district: String,
    pub tier: String,
    pub specializations: Vec<String>,
    pub capability_hash: String,
    pub enrollment_proof: Vec<u8>,
    pub app_version: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrSubmissionResponse {
    pub certificate_pem: String,
    pub certificate_chain: Vec<String>,
    pub issuer_id: String,
    pub valid_from: u64,
    pub valid_to: u64,
    pub registration_id: String,
    pub session_scope_hints: Vec<String>,
}
