use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStateVector {
    pub stability: f32,
    pub value_velocity: f32,
    pub alignment_index: f32,
    pub trust_density: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicDirective {
    pub directive_id: Uuid,
    pub vector_target: SystemStateVector,
    pub suggestion: String,
    pub priority: u8, // 1-10
}

pub struct StrategyVector;
