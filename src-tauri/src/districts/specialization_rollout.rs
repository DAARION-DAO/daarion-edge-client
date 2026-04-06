use uuid::Uuid;
use super::specialization_activation::{DistrictSpecializationActivationIntent, RolloutState};
use super::specialization_transition::DistrictSpecializationTransition;
use super::specialization_guardrails::{SpecializationSafetyEngine, DistrictSpecializationGuardrail};

pub struct SpecializationRolloutOrchestrator;

impl SpecializationRolloutOrchestrator {
    pub fn propose_activation(mut intent: DistrictSpecializationActivationIntent) -> DistrictSpecializationTransition {
        let guard = DistrictSpecializationGuardrail::default();
        
        // 1. Validate against guardrails immediately
        if let Err(e) = SpecializationSafetyEngine::validate_intent(&intent, &guard) {
            intent.rollout_state = RolloutState::Rejected;
            // In a real system, we'd log the reason 'e' here
        } else {
            intent.rollout_state = RolloutState::Proposed;
        }

        DistrictSpecializationTransition {
            transition_id: Uuid::new_v4(),
            intent,
            target_completion_percentage: 100,
            current_completion_percentage: 0,
            last_state_change: chrono::Utc::now(),
        }
    }

    pub fn process_governance_approval(transition: &mut DistrictSpecializationTransition, approved: bool) {
        if approved {
            transition.intent.rollout_state = RolloutState::Approved;
        } else {
            transition.intent.rollout_state = RolloutState::Rejected;
        }
        transition.last_state_change = chrono::Utc::now();
    }

    pub fn step_rollout(transition: &mut DistrictSpecializationTransition) {
        if matches!(transition.intent.rollout_state, RolloutState::Approved) {
            transition.intent.rollout_state = RolloutState::Staged;
        }

        if matches!(transition.intent.rollout_state, RolloutState::Staged) {
            transition.advance_rollout(25); // 25% steps
            if transition.current_completion_percentage >= 100 {
                transition.intent.rollout_state = RolloutState::Active;
            }
        }
    }
}
