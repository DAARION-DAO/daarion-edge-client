use serde::{Deserialize, Serialize};
use super::evolution_signal::EvolutionSignal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionDecision {
    Propose(String), // Proposed Intent Description
    Approve,
    Activate,
    Reinforce,
    Rollback(String), // Reason for rollback
    Defer,
    Decay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionScope {
    Local,
    District,
    Regional,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionPolicy {
    pub scope: EvolutionScope,
    pub min_confidence: f32,
    pub stability_target: f32, // Stability Index threshold (0.0 to 1.0)
    pub auto_activate: bool,
}

impl EvolutionPolicy {
    /// Verify if the current stability index meets the threshold for evolution steps.
    pub fn is_stable(&self, current_stability: f32) -> bool {
        current_stability >= self.stability_target
    }

    /// Hard gateway: Blocks evolution if stability is compromised.
    pub fn can_evolve(&self, current_stability: f32) -> bool {
        self.is_stable(current_stability)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConstraintSet {
    pub identity_required: bool,
    pub safety_gate_enabled: bool,
    pub value_justification_required: bool,
    pub structure_coherence_check: bool,
}
