use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::district_specialization::DistrictSpecializationClass;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RolloutState {
    Proposed,
    UnderReview,
    Approved,
    Staged,
    Active,
    Rejected,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictSpecializationActivationIntent {
    pub proposal_id: Uuid,
    pub district_id: String,
    pub region: String,
    pub current_specialization: DistrictSpecializationClass,
    pub proposed_specialization: DistrictSpecializationClass,
    pub confidence: f32,
    pub evidence_basis_ref: Option<Uuid>,
    pub governance_ref: Option<Uuid>,
    pub trust_scope: Option<String>,
    pub rollout_state: RolloutState,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecializationActivationDecision {
    pub decision_id: Uuid,
    pub intent_id: Uuid,
    pub status: crate::authorities::authority_decision::AuthorityDecision,
    pub reason: String,
    pub reviewer_identity: String,
}
