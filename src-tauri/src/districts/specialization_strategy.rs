use crate::districts::district_specialization::{DistrictSpecializationClass, DistrictSpecializationRecommendation, DistrictSpecializationGap};
use crate::districts::specialization_signal::DistrictSpecializationSignal;
use crate::districts::specialization_activation::{DistrictSpecializationActivationIntent, RolloutState};
use uuid::Uuid;
use chrono::Utc;

pub struct SpecializationStrategy;

impl SpecializationStrategy {
    pub fn compute_recommendation(signal: &DistrictSpecializationSignal) -> DistrictSpecializationRecommendation {
        // 1. Detect Hardware Fitness
        let is_gpu_heavy = signal.hardware_vram_total_gb > (signal.node_count as f32 * 8.0);
        let is_ram_heavy = signal.hardware_ram_total_gb > (signal.node_count as f32 * 32.0);

        // 2. Identify highest demand
        let mut best_class = DistrictSpecializationClass::Mixed;
        let mut max_demand = 0.0;
        
        for (class_name, score) in &signal.demand_density {
            if *score > max_demand {
                max_demand = *score;
                best_class = match class_name.as_str() {
                    "vision" => DistrictSpecializationClass::Vision,
                    "llm" => DistrictSpecializationClass::SmallLLM,
                    "embedding" => DistrictSpecializationClass::Embedding,
                    _ => DistrictSpecializationClass::Mixed,
                };
            }
        }

        // 3. Apply Hardware Constraints/Bonuses
        let mut confidence = 0.7;
        let mut reason = format!("High demand for {} detected.", format!("{:?}", best_class));

        if best_class == DistrictSpecializationClass::Vision && !is_gpu_heavy {
            best_class = DistrictSpecializationClass::Mixed;
            confidence = 0.4;
            reason = "Vision demand high but district VRAM is insufficient.".to_string();
        }

        if best_class == DistrictSpecializationClass::SmallLLM && is_ram_heavy {
            confidence += 0.15;
            reason += " Hardware composition is ideal for large LLM context.";
        }

        DistrictSpecializationRecommendation {
            district_id: signal.district_id.clone(),
            recommended_class: best_class,
            confidence: if confidence > 1.0 { 1.0 } else { confidence },
            primary_reason: reason,
            hardware_fit_score: if is_gpu_heavy { 0.9 } else { 0.5 },
            demand_alignment: max_demand,
        }
    }

    pub fn detect_gaps(signal: &DistrictSpecializationSignal) -> Vec<DistrictSpecializationGap> {
        let mut gaps = Vec::new();
        for (class_name, demand) in &signal.demand_density {
            let current = signal.current_distribution.get(class_name).unwrap_or(&0.0);
            let delta = current - demand;
            
            if delta < -0.3 {
                gaps.push(DistrictSpecializationGap {
                    class: match class_name.as_str() {
                        "vision" => DistrictSpecializationClass::Vision,
                        _ => DistrictSpecializationClass::SmallLLM,
                    },
                    gap_delta: delta,
                    recommendation: format!("Increase {} capacity to meet district demand.", class_name),
                });
            }
        }
        gaps
    }

    pub fn create_activation_intent(
        rec: &DistrictSpecializationRecommendation, 
        current: DistrictSpecializationClass
    ) -> DistrictSpecializationActivationIntent {
        DistrictSpecializationActivationIntent {
            proposal_id: Uuid::new_v4(),
            district_id: rec.district_id.clone(),
            region: "unspecified".to_string(), // In reality derived from district metadata
            current_specialization: current,
            proposed_specialization: rec.recommended_class.clone(),
            confidence: rec.confidence,
            evidence_basis_ref: None, // Will be bound during validation
            governance_ref: None,
            trust_scope: None,
            rollout_state: RolloutState::Proposed,
            created_at: Utc::now(),
        }
    }
}
