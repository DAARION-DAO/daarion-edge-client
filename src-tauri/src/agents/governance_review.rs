use serde::{Deserialize, Serialize};
use crate::agents::governance_role::GovernanceRole;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GovernanceDecision {
    Approve,
    Reject,
    Veto,
    Escalate,
    NeedsHumanReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceReview {
    pub review_id: String,
    pub proposal_id: String,
    pub reviewing_agent_id: String,
    pub governance_role: GovernanceRole,
    pub decision: GovernanceDecision,
    pub confidence: f32,
    pub reason_summary: String,
    pub created_at: u64,
}
