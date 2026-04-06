use serde::{Deserialize, Serialize};
use crate::models::placement_recommendation::{PlacementRecommendation, PlacementActivationDecision};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlacementApprovalState {
    Pending,
    Approved,
    Rejected,
    AutoExecuted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementIntent {
    pub intent_id: String,
    pub recommendation: PlacementRecommendation,
    pub approval_state: PlacementApprovalState,
    pub approved_by: Option<String>,
    pub updated_at: u64,
}

pub struct PlacementActivation;

impl PlacementActivation {
    pub fn create_intent(recommendation: PlacementRecommendation) -> PlacementIntent {
        PlacementIntent {
            intent_id: uuid::Uuid::new_v4().to_string(),
            recommendation,
            approval_state: PlacementApprovalState::Pending,
            approved_by: None,
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}
