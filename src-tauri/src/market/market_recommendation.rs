use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRecommendation {
    pub recommendation_id: String,
    pub model_id: String,
    pub district_id: String,
    pub action: MarketAction,
    pub replication_level: u32,
    pub gravity_alignment: f32,
    pub confidence: f32,
    pub reason: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketAction {
    IncreaseReplication,
    DecreaseReplication,
    ShiftSpecialization,
    PromoteToPersistent,
    DecommissionArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConfidence {
    pub score: f32,
    pub signal_count: u32,
    pub entropy: f32,
}
