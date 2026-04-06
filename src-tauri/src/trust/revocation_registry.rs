use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationRecord {
    pub identity_id: Uuid,
    pub revoked_at: DateTime<Utc>,
    pub revocation_reason: String,
    pub issuer_id: String,
}
