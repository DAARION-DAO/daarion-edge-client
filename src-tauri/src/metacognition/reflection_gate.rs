use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetaCognitiveDecision {
    Continue,
    DowngradeConfidence,
    RequireReview,
    Escalate,
    PauseActivation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionTrigger {
    pub reason: String,
    pub severity: u8,
}
