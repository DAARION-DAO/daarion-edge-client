use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictSpecializationSignal {
    pub district_id: String,
    pub region: String,
    pub demand_density: HashMap<String, f32>, // class -> score
    pub current_distribution: HashMap<String, f32>, // class -> score
    pub hardware_vram_total_gb: f32,
    pub hardware_ram_total_gb: f32,
    pub node_count: u32,
    pub trust_scope: String,
    pub created_at: u64,
}
