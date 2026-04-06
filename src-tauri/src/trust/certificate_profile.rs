use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTrustScope {
    pub region: String,
    pub district: String,
    pub tier: String,
    pub specialization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCertificateProfile {
    pub node_id: String,
    pub public_key_der: Vec<u8>,
    pub scope: NodeTrustScope,
    pub issuer_id: String,
    pub valid_from: u64,
    pub valid_to: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateChain {
    pub node_cert: NodeCertificateProfile,
    pub intermediates: Vec<Vec<u8>>,
    pub root_thumbprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationEntry {
    pub node_id: String,
    pub revoked_at: u64,
    pub reason: String,
}
