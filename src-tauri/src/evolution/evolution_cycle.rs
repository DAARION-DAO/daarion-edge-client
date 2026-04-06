use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionCycleStage {
    Observation,
    Evaluation,
    Proposal,
    Approval,
    Activation,
    Monitoring,
    Completion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionCycleState {
    pub cycle_id: Uuid,
    pub stage: EvolutionCycleStage,
    pub stability_index: f32,
    pub active_intents: usize,
    pub last_evolution_at: i64,
}
