use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketGuardrail {
    pub min_confidence: f32,
    pub max_saturation: f32,
    pub max_replication_step: u32,
    pub anomaly_freeze_threshold: f32,
    pub freshness_requirement_seconds: u64,
    pub rollback_trigger_kpi_drop: f32,
    pub cooldown_window_seconds: u64,
}

impl Default for MarketGuardrail {
    fn default() -> Self {
        Self {
            min_confidence: 0.85,
            max_saturation: 0.8,
            max_replication_step: 3,
            anomaly_freeze_threshold: 0.15,
            freshness_requirement_seconds: 300,
            rollback_trigger_kpi_drop: 0.2,
            cooldown_window_seconds: 3600, // 1 hour anti-oscillation
        }
    }
}

pub struct MarketSafetyEngine;

impl MarketSafetyEngine {
    pub fn is_safe_to_activate(
        confidence: f32, 
        saturation: f32, 
        anomaly_density: f32, 
        guard: &MarketGuardrail
    ) -> Result<(), String> {
        if confidence < guard.min_confidence {
            return Err(format!("Confidence {:.2} < {:.2}", confidence, guard.min_confidence));
        }
        if saturation > guard.max_saturation {
            return Err(format!("District saturation {:.2} > {:.2}", saturation, guard.max_saturation));
        }
        if anomaly_density > guard.anomaly_freeze_threshold {
            return Err(format!("Anomaly density {:.2} > {:.2} (FROZEN)", anomaly_density, guard.anomaly_freeze_threshold));
        }
        Ok(())
    }
}
