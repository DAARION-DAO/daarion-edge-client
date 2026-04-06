use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionSignalClass {
    DemandShift,
    ResourceAnomaly,
    ValuePressure,
    TrustDecay,
    StructuralDrift,
    PerformanceOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionSignal {
    pub class: EvolutionSignalClass,
    pub source_authority: String,
    pub magnitude: f32, // 0.0 to 1.0 intensity
    pub confidence: f32,
    pub evidence_basis: String,
    pub timestamp: i64,
}
