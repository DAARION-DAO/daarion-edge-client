use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialType {
    Heartbeat,
    NatsWorker,
    Collector,
    Messaging,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCredentialRequest {
    pub credential_type: CredentialType,
    pub requested_subjects: Vec<String>, // For NATS mostly
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCredentialProfile {
    pub credential_type: CredentialType,
    pub token: String, // e.g. NATS JWT or scoped token
    pub scope: String, // district or regional scope
    pub valid_from: u64,
    pub valid_to: u64,
    pub issuer_id: String,
    pub node_id: String,
}
