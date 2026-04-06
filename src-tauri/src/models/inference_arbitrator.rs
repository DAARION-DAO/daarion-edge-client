use serde::{Deserialize, Serialize};
use crate::models::residency_score::ResidencyScore;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InferenceExecutionDecision {
    LocalExecution,
    RemoteExecution,
    Reject(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationResult {
    pub decision: InferenceExecutionDecision,
    pub reason: String,
    pub estimated_local_latency_ms: u64,
    pub estimated_remote_latency_ms: u64,
}

pub struct InferenceArbitrator;

impl InferenceArbitrator {
    pub fn decide(
        model_id: &str,
        is_warm: bool,
        residency_score: f32,
        device_memory_pressure: f32,
        local_queue_depth: usize,
    ) -> ArbitrationResult {
        let estimated_remote_latency_ms = 800; // Static estimate for v1
        let mut estimated_local_latency_ms = 1200; // Baseline local
        
        if is_warm {
            estimated_local_latency_ms = 500;
        } else {
            // Factor in model loading time based on size/score
            estimated_local_latency_ms += 1500;
        }

        // Add queuing penalty
        estimated_local_latency_ms += (local_queue_depth as u64) * 300;

        // Decision Logic
        if device_memory_pressure > 0.9 {
            return ArbitrationResult {
                decision: InferenceExecutionDecision::RemoteExecution,
                reason: "Extreme memory pressure (offloading)".to_string(),
                estimated_local_latency_ms,
                estimated_remote_latency_ms,
            };
        }

        if is_warm {
            if local_queue_depth < 3 {
                return ArbitrationResult {
                    decision: InferenceExecutionDecision::LocalExecution,
                    reason: "Model is warm and queue is small".to_string(),
                    estimated_local_latency_ms,
                    estimated_remote_latency_ms,
                };
            } else {
                return ArbitrationResult {
                    decision: InferenceExecutionDecision::RemoteExecution,
                    reason: "Local queue depth high, offloading".to_string(),
                    estimated_local_latency_ms,
                    estimated_remote_latency_ms,
                };
            }
        }

        // Model is cold
        if residency_score > 50.0 && device_memory_pressure < 0.6 {
            ArbitrationResult {
                decision: InferenceExecutionDecision::LocalExecution,
                reason: "High residency value compensates for cold start".to_string(),
                estimated_local_latency_ms,
                estimated_remote_latency_ms,
            }
        } else {
            ArbitrationResult {
                decision: InferenceExecutionDecision::RemoteExecution,
                reason: "Cold model / low score (preferring remote)".to_string(),
                estimated_local_latency_ms,
                estimated_remote_latency_ms,
            }
        }
    }
}
