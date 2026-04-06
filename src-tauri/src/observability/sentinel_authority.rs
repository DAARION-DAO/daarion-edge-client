use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelAuthorityProfile {
    pub authority_id: Uuid,
    pub scope: String, // e.g., "District-12", "Network-Global"
    pub supported_signals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityScope {
    pub scope_id: Uuid,
    pub target_district: String,
    pub hierarchy_level: u8, // node=0, district=1, regional=2
}
