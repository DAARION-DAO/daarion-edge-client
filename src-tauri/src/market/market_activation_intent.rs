use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketResourceType {
    ModelArtifact,
    ComputeCapacity,
    InferenceDemand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketActivationRolloutState {
    Proposed,
    UnderReview,
    Approved,
    Staged,
    Active,
    Rejected,
    RolledBack,
    Frozen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketActivationIntent {
    pub activation_id: Uuid,
    pub target_scope: String,
    pub resource_type: MarketResourceType,
    pub model_id: Option<String>,
    pub district_id: Option<String>,
    pub region: Option<String>,
    pub confidence: f32,
    pub evidence_basis_ref: Option<Uuid>,
    pub trust_chain_ref: Option<Uuid>,
    pub governance_ref: Option<Uuid>,
    pub rollout_state: MarketActivationRolloutState,
    pub created_at: DateTime<Utc>,
}
