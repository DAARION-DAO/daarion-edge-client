use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationDecision {
    PassiveMonitoring,
    AdvisoryHint(String),
    ActiveConstraint(String),
    EscalatedGovernance(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationScope {
    District,
    Regional,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationPolicy {
    pub scope: CoordinationScope,
    pub divergence_threshold: f32,
    pub advisory_enabled: bool,
    pub enforcement_enabled: bool,
}
