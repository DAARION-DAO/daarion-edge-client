use serde::{Deserialize, Serialize};
use crate::trust::certificate_profile::NodeCertificateProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalIssuerProfile {
    pub issuer_id: String,
    pub region: String,
    pub public_key_der: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerSigningResponse {
    pub certificate_profile: NodeCertificateProfile,
    pub signature: Vec<u8>,
    pub chain: Vec<Vec<u8>>, // DER encoded intermediate/root certs
}
