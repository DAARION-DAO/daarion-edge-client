use serde::{Deserialize, Serialize};
use super::specialization_activation::DistrictSpecializationActivationIntent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictSpecializationGuardrail {
    pub min_confidence_threshold: f32,
    pub min_evidence_freshness_seconds: u64,
    pub district_capacity_floor: f32,
    pub specialization_saturation_ceiling: f32,
    pub rollback_trigger_anomaly_threshold: f32,
}

impl Default for DistrictSpecializationGuardrail {
    fn default() -> Self {
        Self {
            min_confidence_threshold: 0.8,
            min_evidence_freshness_seconds: 300,
            district_capacity_floor: 0.2,
            specialization_saturation_ceiling: 0.8,
            rollback_trigger_anomaly_threshold: 0.15,
        }
    }
}

pub struct SpecializationSafetyEngine;

impl SpecializationSafetyEngine {
    pub fn validate_intent(intent: &DistrictSpecializationActivationIntent, guard: &DistrictSpecializationGuardrail) -> Result<(), String> {
        if intent.confidence < guard.min_confidence_threshold {
            return Err(format!("Confidence {:.2} below threshold {:.2}", intent.confidence, guard.min_confidence_threshold));
        }
        
        Ok(())
    }
}
