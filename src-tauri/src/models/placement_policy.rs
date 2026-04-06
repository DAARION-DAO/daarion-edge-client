use serde::{Deserialize, Serialize};
use crate::models::gravity_score::GravityScore;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlacementDecision {
    Persistent,    // High gravity, essential backbone
    WarmPreferred, // Good gravity, opportunistic warm start
    ColdOnly,      // Low gravity, keep artifact but don't preload
    AvoidPlacement, // Negative gravity or mismatch
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlacementScope {
    Node,
    District,
    Region,
    Global,
}

pub struct PlacementPolicy;

impl PlacementPolicy {
    pub fn decide(score: &GravityScore) -> (PlacementDecision, PlacementScope) {
        let val = score.total_score;
        let decision = if val > 100.0 {
            PlacementDecision::Persistent
        } else if val > 40.0 {
            PlacementDecision::WarmPreferred
        } else if val >= 0.0 {
            PlacementDecision::ColdOnly
        } else {
            PlacementDecision::AvoidPlacement
        };

        // Scope derivation (simplified heuristic)
        let scope = if val > 150.0 {
            PlacementScope::Global
        } else if val > 80.0 {
            PlacementScope::Region
        } else if val > 40.0 {
            PlacementScope::District
        } else {
            PlacementScope::Node
        };

        (decision, scope)
    }
}
