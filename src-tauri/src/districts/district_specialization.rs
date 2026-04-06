use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DistrictSpecializationClass {
    Routing,   // Low-latency arbitration
    SmallLLM,  // 1B-8B parameter execution
    Embedding, // Vector density
    Vision,    // Image/Video analysis
    Mixed,     // General purpose fallback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictSpecializationRecommendation {
    pub district_id: String,
    pub recommended_class: DistrictSpecializationClass,
    pub confidence: f32,
    pub primary_reason: String,
    pub hardware_fit_score: f32,
    pub demand_alignment: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictSpecializationGap {
    pub class: DistrictSpecializationClass,
    pub gap_delta: f32, // Positive = oversupply, Negative = undersupply
    pub recommendation: String,
}
