use serde::{Deserialize, Serialize};
use crate::trust::session_scope::SessionScope;
use crate::trust::session_derivation::SessionCredentialProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCredentialRequest {
    pub node_id: String,
    pub certificate_ref: String, // Thumbprint or serial
    pub requested_scope: SessionScope,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCredentialResponse {
    pub credential: SessionCredentialProfile,
    pub issuer_signature: Vec<u8>,
}

pub struct SessionIssuerClient;

impl SessionIssuerClient {
    pub async fn request_credential(
        _node_id: &str,
        _scope: SessionScope,
    ) -> Result<SessionCredentialResponse, String> {
        // Mocked for v1.5
        Err("Not implemented yet: Requires Regional Issuer endpoint".to_string())
    }
}
