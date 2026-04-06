use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBasis {
    pub evidence_ref: Uuid,
    pub authority_id: Uuid,
    pub signal_type: String,
    pub normalized_value: f32,
    pub confidence: f32,
    pub evidence_signature: String, // Proof of normalization
    pub created_at: DateTime<Utc>,
}
