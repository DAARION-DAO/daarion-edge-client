use std::time::{Instant, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResidencyStats {
    pub hits_5m: u32,
    pub hits_15m: u32,
    pub hits_60m: u32,
    pub last_hit_at: Option<u64>,
    pub load_count: u32,
    pub total_warm_time_sec: u64,
    pub avg_load_ms: u32,
}

impl Default for ModelResidencyStats {
    fn default() -> Self {
        Self {
            hits_5m: 0,
            hits_15m: 0,
            hits_60m: 0,
            last_hit_at: None,
            load_count: 0,
            total_warm_time_sec: 0,
            avg_load_ms: 0,
        }
    }
}

pub struct ResidencyScore {
    pub total_score: f32,
    pub utility_component: f32,
    pub cost_penalty: f32,
    pub specialization_bonus: f32,
}

pub struct ScoringEngine;

impl ScoringEngine {
    pub fn calculate_score(
        stats: &ModelResidencyStats,
        ram_gb: f32,
        is_specialized: bool,
        device_memory_pressure: f32,
    ) -> ResidencyScore {
        // Utility: recent hits weight
        let utility = (stats.hits_5m as f32 * 10.0) 
            + (stats.hits_15m as f32 * 5.0) 
            + (stats.hits_60m as f32 * 2.0);
        
        // Specialization bonus
        let spec_bonus = if is_specialized { 50.0 } else { 0.0 };
        
        // Load time persistence bonus (keep hard-to-load models longer)
        let load_bonus = (stats.avg_load_ms as f32 / 100.0).min(30.0);
        
        // Cost: RAM footprint penalty
        // Higher memory pressure increases the penalty
        let cost_base = ram_gb * 15.0;
        let pressure_multiplier = 1.0 + (device_memory_pressure * 2.0);
        let penalty = cost_base * pressure_multiplier;
        
        let total = utility + spec_bonus + load_bonus - penalty;
        
        ResidencyScore {
            total_score: total,
            utility_component: utility,
            cost_penalty: penalty,
            specialization_bonus: spec_bonus,
        }
    }
}
