use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    Untrusted,
    Enrolled,
    Verified,
    HighAssurance, // e.g., signed by Root or HW backed
    Restricted,    // Degraded trust
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScope {
    pub region: String,
    pub zone: Option<String>,
    pub tier: String,
    pub specializations: Vec<String>,
    pub trust_level: TrustLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalIssuer {
    pub issuer_id: String,
    pub region: String,
    pub public_key_der: Vec<u8>,
    pub signature_algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCertificateProfile {
    pub node_id: String,
    pub issuer_id: String,
    pub trust_scope: TrustScope,
    pub valid_from: u64,
    pub valid_to: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCredentialProfile {
    pub session_id: String,
    pub node_id: String,
    pub permissions_mask: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevocationReason {
    KeyCompromise,
    NodeRetired,
    SuspiciousBehavior,
    Maintenance,
    OwnershipTransfer,
}
