use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::specialization_activation::{DistrictSpecializationActivationIntent, RolloutState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictSpecializationTransition {
    pub transition_id: Uuid,
    pub intent: DistrictSpecializationActivationIntent,
    pub target_completion_percentage: u32,
    pub current_completion_percentage: u32,
    pub last_state_change: chrono::DateTime<chrono::Utc>,
}

impl DistrictSpecializationTransition {
    pub fn is_complete(&self) -> bool {
        self.current_completion_percentage >= 100 && matches!(self.intent.rollout_state, RolloutState::Active)
    }
    
    pub fn advance_rollout(&mut self, step: u32) {
        self.current_completion_percentage = (self.current_completion_percentage + step).min(100);
        self.last_state_change = chrono::Utc::now();
    }
}
