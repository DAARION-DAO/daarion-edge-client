use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetaRiskClass {
    Overconfidence,
    EvidenceSparsity,
    AuthorityDominance,
    StrategicDrift,
    ValueBias,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaRiskProfile {
    pub risk_class: MetaRiskClass,
    pub risk_score: f32, // 0.0 (safe) to 1.0 (critical)
    pub evidence_sufficiency: f32,
    pub authority_balance: f32,
}
