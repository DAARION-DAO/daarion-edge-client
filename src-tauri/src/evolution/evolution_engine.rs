use serde::{Deserialize, Serialize};
use super::evolution_cycle::EvolutionCycleState;
use super::evolution_signal::EvolutionSignal;
use super::evolution_policy::EvolutionDecision;

pub struct EvolutionEngine {
    pub state: EvolutionCycleState,
}

impl EvolutionEngine {
    pub fn new() -> Self {
        Self {
            state: EvolutionCycleState {
                cycle_id: uuid::Uuid::new_v4(),
                stage: super::evolution_cycle::EvolutionCycleStage::Observation,
                stability_index: 0.95,
                active_intents: 0,
                last_evolution_at: chrono::Utc::now().timestamp(),
            },
        }
    }

    pub fn evaluate_signals(&mut self, signals: Vec<EvolutionSignal>, policy: &super::evolution_policy::EvolutionPolicy) -> EvolutionDecision {
        // Hard gateway: Stability Index Check
        if !policy.can_evolve(self.state.stability_index) {
            return EvolutionDecision::Defer; // Blocked by stability gateway
        }

        if signals.is_empty() {
            return EvolutionDecision::Defer;
        }

        let max_magnitude = signals.iter().map(|s| s.magnitude).fold(0.0, f32::max);
        if max_magnitude > 0.8 {
            EvolutionDecision::Propose("High-intensity signal detected".to_string())
        } else {
            EvolutionDecision::Defer
        }
    }

    pub fn generate_intelligence_signal(&self) -> crate::intelligence::signal_aggregator::IntelligenceSignal {
        use uuid::Uuid;
        use crate::intelligence::signal_aggregator::{IntelligenceSignal, SignalSource};
        use crate::authorities::authority_protocol::AuthorityLayer;

        IntelligenceSignal {
            signal_id: Uuid::new_v4(),
            source: SignalSource::EvolutionStability,
            layer: AuthorityLayer::Architecture, // Evolution is an architectural field
            weight: self.state.stability_index,
            payload_v_hex: format!("0xSTABILITY_{}", self.state.stability_index),
            timestamp: chrono::Utc::now(),
        }
    }
}
