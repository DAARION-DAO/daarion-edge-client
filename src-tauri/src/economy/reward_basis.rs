use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardBasis {
    pub basis_id: String,
    pub recipient_id: String,
    pub contribution_weight: f64,
    pub integrity_bonus: f64,
    pub duration_minutes: u64,
    pub evidence_ref: String,
}
