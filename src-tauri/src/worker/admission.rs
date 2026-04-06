use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum JobClass {
    Interactive,
    Background,
    Bulk,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ExecutionDecision {
    ExecuteNow,
    DeferLocal { delay_ms: u64 },
    NakWithDelay { delay_ms: u64, reason: String },
    Reject { reason: String },
}

use crate::worker::models::{SpecializationProfile, SpecializationPolicy};

#[derive(Debug, Clone)]
pub struct AdmissionInput {
    pub job_class: JobClass,
    pub job_type: String, // e.g., "reasoning", "embedding"
    pub estimated_latency_ms: u32,
    pub cpu_pressure: f32, // 0.0 to 1.0
    pub memory_pressure: f32, 
    pub gpu_available: bool,
    pub on_battery: bool,
    pub queue_depth: usize,
    pub specialization: SpecializationProfile,
}

pub struct AdmissionController;

impl AdmissionController {
    pub fn decide(input: AdmissionInput) -> ExecutionDecision {
        // 1. Fast-path for Interactive work
        if input.job_class == JobClass::Interactive {
            return ExecutionDecision::ExecuteNow;
        }

        // 2. High pressure rejection
        if input.cpu_pressure > 0.9 || input.memory_pressure > 0.9 {
            return ExecutionDecision::NakWithDelay { 
                delay_ms: 120000, 
                reason: "Extreme system pressure".to_string() 
            };
        }

        // 3. Specialization Affinity Bias
        let is_warm = SpecializationPolicy::is_affinity_match(&input.specialization, &input.job_type);
        if is_warm {
            return ExecutionDecision::ExecuteNow;
        }

        // 4. Power-aware Bulk handling
        if input.job_class == JobClass::Bulk && input.on_battery {
            return ExecutionDecision::NakWithDelay { 
                delay_ms: 300000, 
                reason: "Conserving energy for user".to_string() 
            };
        }

        // 5. Deferral if not warm and system is moderately busy
        if input.cpu_pressure > 0.4 {
            return ExecutionDecision::DeferLocal { delay_ms: 1000 };
        }

        // Default: Proceed
        ExecutionDecision::ExecuteNow
    }
}
