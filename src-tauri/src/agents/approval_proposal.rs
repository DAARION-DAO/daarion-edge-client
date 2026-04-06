use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApprovalScope {
    Node,
    District,
    Region,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApprovalAction {
    PromoteToWarm,
    PromoteToPersistent,
    DowngradeToCold,
    AvoidPlacement,
    DistrictCoDownload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalProposal {
    pub proposal_id: String,
    pub agent_id: String,
    pub target_model_id: String,
    pub action: ApprovalAction,
    pub scope: ApprovalScope,
    pub reason_summary: String,
    pub confidence: f32,
    pub telemetry_refs: Vec<String>,
    pub trust_scope: String,
    pub created_at: u64,
}
