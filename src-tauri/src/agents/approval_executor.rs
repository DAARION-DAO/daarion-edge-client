use serde::{Deserialize, Serialize};
use crate::agents::approval_policy::ApprovalPolicyResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedActivationRef {
    pub proposal_id: String,
    pub intent_id: String, // Link to PlacementIntent
    pub approved_at: u64,
}

pub struct ApprovalExecutor;

impl ApprovalExecutor {
    pub fn create_activation_ref(result: &ApprovalPolicyResult) -> Option<ApprovedActivationRef> {
        if result.decision == crate::agents::approval_policy::ApprovalDecision::Approved {
            Some(ApprovedActivationRef {
                proposal_id: result.proposal_id.clone(),
                intent_id: uuid::Uuid::new_v4().to_string(), // In reality, this would initiate a PlacementIntent
                approved_at: chrono::Utc::now().timestamp_millis() as u64,
            })
        } else {
            None
        }
    }
}
