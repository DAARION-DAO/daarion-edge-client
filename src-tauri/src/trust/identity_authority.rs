use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdentityType {
    User,
    Agent,
    Node,
    Session,
    WorkerLease,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdentityTrustLevel {
    Guest,      // Unverified, limited connectivity
    Verified,   // Bound to proof of work or identity
    Sovereign,  // Root authority (Sofiia, DAARWIZZ)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityAuthorityProfile {
    pub authority_id: Uuid,
    pub identity_type: IdentityType,
    pub issuer_id: String,
    pub trust_level: IdentityTrustLevel,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
}
