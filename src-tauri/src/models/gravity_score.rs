use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::collector_signal::CollectorGravitySignal;
use crate::models::gravity_collector_sync::GravityCollectorSync;
use crate::observability::sentinel_signal::SentinelSignal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelGravitySignal {
    pub region_demand_15m: u32,
    pub region_demand_24h: u32,
    pub specialization_match: f32, // 0.0 to 1.0
    pub avg_remote_latency_saved_ms: u32,
    pub resident_ram_cost_mb: u32,
    pub artifact_transfer_cost_mb: u32,
    pub trust_scope_match: bool,
    pub node_tier: String,
    pub current_model_presence: bool,
    pub collector_signal: Option<CollectorGravitySignal>,
    pub standardized_signals: Vec<SentinelSignal>,
    pub evidence_basis_ref: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityScore {
    pub total_score: f32,
    pub signals: ModelGravitySignal,
    pub telemetry_basis: String,
    pub timestamp: u64,
}

pub struct GravityEngine;

impl GravityEngine {
    pub fn calculate_score(model_id: &str, signal: &ModelGravitySignal) -> GravityScore {
        let mut score = 0.0;
        let mut basis = Vec::new();

        // 1. Demand multiplier
        let demand_score = (signal.region_demand_15m as f32 * 2.0) + (signal.region_demand_24h as f32 * 0.5);
        score += demand_score;
        basis.push(format!("Region demand contribution: {:.1}", demand_score));

        // 2. Latency benefit
        let latency_bonus = (signal.avg_remote_latency_saved_ms as f32 / 100.0) * 10.0;
        score += latency_bonus;

        // 3. Specialization alignment
        score *= 1.0 + (signal.specialization_match * 0.5);

        // 4. Collector-Sync (New District Signals)
        if let Some(ref collector) = signal.collector_signal {
            // District Saturation Penalty
            let penalty = GravityCollectorSync::calculate_saturation_penalty(model_id, collector);
            if penalty > 0.0 {
                score -= penalty;
                basis.push(format!("District saturation penalty: -{:.1}", penalty));
            }

            // Specialization Gap Bonus
            let bonus = GravityCollectorSync::calculate_specialization_bonus(model_id, collector);
            if bonus > 0.0 {
                score += bonus;
                basis.push(format!("District specialization gap bonus: +{:.1}", bonus));
            }

            // Anomaly Penalty
            if collector.anomaly_density > 0.1 {
                let anomaly_hit = collector.anomaly_density * 20.0;
                score -= anomaly_hit;
                basis.push(format!("District anomaly penalty: -{:.1}", anomaly_hit));
            }
        }

        // 4.5. Standardized Sentinel Evidence
        for signal in &signal.standardized_signals {
            if signal.normalized_value < 0.3 && matches!(signal.signal_class, crate::observability::signal_class::SignalClass::Load) {
                score -= 10.0;
                basis.push(format!("Sentinel load-normalized penalty: -10.0 (Conf: {:.2})", signal.confidence.score));
            }
        }

        // 5. Cost penalties
        let ram_penalty = (signal.resident_ram_cost_mb as f32 / 1024.0) * 5.0;
        score -= ram_penalty;

        // 6. Trust Hard-Filter
        if !signal.trust_scope_match {
            score = -1000.0;
            basis.push("REJECTED: Trust scope mismatch".to_string());
        }

        GravityScore {
            total_score: score.max(-1000.0),
            signals: signal.clone(),
            telemetry_basis: basis.join("; "),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
}
