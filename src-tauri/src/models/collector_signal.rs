use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorGravitySignal {
    pub district_id: String,
    pub healthy_node_count: u32,
    pub degraded_node_count: u32,
    pub avg_memory_pressure: f32, // 0.0 to 1.0
    pub specialization_distribution: HashMap<String, u32>, // model_id -> count of "Warm" nodes
    pub anomaly_density: f32, // 0.0 to 1.0
    pub demand_density: HashMap<String, f32>, // model_id -> normalized request volume
    pub collector_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaturationSignal {
    pub model_id: String,
    pub saturation_ratio: f32, // count / healthy_count
    pub penalty_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictDemandSignal {
    pub model_id: String,
    pub normalized_demand: f32,
    pub trend: String, // "Up", "Down", "Stable"
}
