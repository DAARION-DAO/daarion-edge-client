use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlacementScope {
    Node,
    District,
    Region,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlacementActivationDecision {
    Persistent,
    WarmPreferred,
    ColdOnly,
    AvoidPlacement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementRecommendation {
    pub model_id: String,
    pub scope: PlacementScope,
    pub decision: PlacementActivationDecision,
    pub gravity_score: f32,
    pub confidence: f32,
    pub telemetry_basis: String, // Reason summary
    pub created_at: u64,
}
