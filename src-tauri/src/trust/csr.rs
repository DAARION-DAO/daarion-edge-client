use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCsrRequest {
    pub node_id: String,
    pub public_key_der: Vec<u8>,
    pub region: String,
    pub district: String,
    pub tier: String,
    pub specialization: String,
    pub capability_hash: String,
    pub enrollment_proof: String, // e.g. the initial node_token
    pub created_at: u64,
}
