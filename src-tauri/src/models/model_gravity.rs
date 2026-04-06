use serde::{Deserialize, Serialize};
use crate::models::gravity_score::{GravityEngine, ModelGravitySignal, GravityScore};
use crate::models::placement_policy::{PlacementPolicy, PlacementDecision, PlacementScope};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPlacementRecommendation {
    pub model_id: String,
    pub score: GravityScore,
    pub decision: PlacementDecision,
    pub scope: PlacementScope,
    pub reason: String,
}

pub struct ModelGravity;

impl ModelGravity {
    pub fn get_recommendation(model_id: String, signal: ModelGravitySignal) -> ModelPlacementRecommendation {
        let score = GravityEngine::calculate_score(&model_id, &signal);
        let (decision, scope) = PlacementPolicy::decide(&score);

        let reason = match decision {
            PlacementDecision::Persistent => "High regional demand and critical latency benefit.".to_string(),
            PlacementDecision::WarmPreferred => "Good hardware fit and moderate district demand.".to_string(),
            PlacementDecision::ColdOnly => "Low frequent demand; maintaining artifact for cold starts.".to_string(),
            PlacementDecision::AvoidPlacement => "Insufficient hardware capability or trust mismatch.".to_string(),
        };

        ModelPlacementRecommendation {
            model_id,
            score,
            decision,
            scope,
            reason,
        }
    }
}
