use crate::models::collector_signal::CollectorGravitySignal;

pub struct GravityCollectorSync;

impl GravityCollectorSync {
    pub fn calculate_saturation_penalty(model_id: &str, signal: &CollectorGravitySignal) -> f32 {
        let warm_count = signal.specialization_distribution.get(model_id).copied().unwrap_or(0);
        let healthy_count = signal.healthy_node_count.max(1);
        
        let ratio = warm_count as f32 / healthy_count as f32;
        
        // Penalize if > 30% of district has this model warm
        if ratio > 0.3 {
            (ratio - 0.3) * 50.0
        } else {
            0.0
        }
    }

    pub fn calculate_specialization_bonus(model_id: &str, signal: &CollectorGravitySignal) -> f32 {
        let warm_count = signal.specialization_distribution.get(model_id).copied().unwrap_or(0);
        let demand = signal.demand_density.get(model_id).copied().unwrap_or(0.0);
        
        // Large bonus if demand is high but 0 nodes have it warm
        if warm_count == 0 && demand > 0.1 {
            demand * 100.0
        } else {
            0.0
        }
    }
}
