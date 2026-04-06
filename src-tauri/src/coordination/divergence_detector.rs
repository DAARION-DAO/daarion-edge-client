use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DivergenceType {
    SpecializationDrift,
    EconomicImbalance,
    ResourceSilo,
    EvolutionConflict,
    IdentityFragmentation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceMetric {
    pub divergence_type: DivergenceType,
    pub scope_id: String,
    pub divergence_score: f32, // 0.0 (aligned) to 1.0 (highly diverged)
    pub evidence_citation: String,
    pub last_detected_at: i64,
}
