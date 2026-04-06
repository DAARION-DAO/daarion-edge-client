use uuid::Uuid;
use super::market_activation_intent::{MarketActivationIntent, MarketActivationRolloutState};
use super::market_guardrails::{MarketSafetyEngine, MarketGuardrail};

pub struct MarketActivationOrchestrator;

impl MarketActivationOrchestrator {
    pub fn propose_intent(mut intent: MarketActivationIntent) -> MarketActivationIntent {
        let guard = MarketGuardrail::default();
        
        // Mock current signals for normalization demonstration
        // In reality, this would query SENTINEL and GRAVITY
        let mock_saturation = 0.45;
        let mock_anomaly = 0.02;

        if let Err(_e) = MarketSafetyEngine::is_safe_to_activate(
            intent.confidence, 
            mock_saturation, 
            mock_anomaly, 
            &guard
        ) {
            intent.rollout_state = MarketActivationRolloutState::Rejected;
        } else {
            intent.rollout_state = MarketActivationRolloutState::Proposed;
        }
        
        intent
    }

    pub fn advance_rollout(intent: &mut MarketActivationIntent) {
        match intent.rollout_state {
            MarketActivationRolloutState::Proposed => {
                intent.rollout_state = MarketActivationRolloutState::UnderReview;
            },
            MarketActivationRolloutState::Approved => {
                intent.rollout_state = MarketActivationRolloutState::Staged;
            },
            MarketActivationRolloutState::Staged => {
                intent.rollout_state = MarketActivationRolloutState::Active;
            },
            _ => {}
        }
    }
}
