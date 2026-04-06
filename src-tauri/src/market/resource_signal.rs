use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalResourceSignal {
    pub total_nodes: u32,
    pub total_vram_gb: f32,
    pub total_ram_gb: f32,
    pub regional_demand: HashMap<String, f32>, // region -> score
    pub model_popularity: HashMap<String, f32>, // model_id -> score
    pub avg_latency_ms: f32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDemandSignal {
    pub model_id: String,
    pub region: String,
    pub demand_density: f32,
    pub latency_pressure: f32,
    pub growth_trend: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSupplySignal {
    pub district_id: String,
    pub supply_capacity: f32,
    pub active_models: Vec<String>,
    pub available_vram_gb: f32,
    pub specialization_fit: f32,
}
