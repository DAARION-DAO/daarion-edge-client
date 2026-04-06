use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEscalationRef {
    pub escalation_id: String,
    pub proposal_id: String,
    pub reason: String,
    pub required_roles: Vec<String>,
    pub human_review_triggered: bool,
    pub escalated_at: u64,
}
