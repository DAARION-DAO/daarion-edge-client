use crate::market::resource_signal::{GlobalResourceSignal, MarketDemandSignal, MarketSupplySignal};
use crate::market::market_recommendation::{MarketRecommendation, MarketAction};

pub struct ModelMarket;

impl ModelMarket {
    pub fn coordinate_resources(
        global_signal: &GlobalResourceSignal,
        demand_signals: &[MarketDemandSignal],
        supply_signals: &[MarketSupplySignal],
    ) -> Vec<MarketRecommendation> {
        let mut recommendations = Vec::new();

        for demand in demand_signals {
            // Find districts in the same region with capacity
            let local_supply: Vec<&MarketSupplySignal> = supply_signals
                .iter()
                .filter(|s| !s.active_models.contains(&demand.model_id))
                .collect();

            if demand.demand_density > 0.7 && demand.latency_pressure > 0.6 {
                if let Some(best_fit) = local_supply.iter().max_by(|a, b| a.supply_capacity.partial_cmp(&b.supply_capacity).unwrap()) {
                    recommendations.push(MarketRecommendation {
                        recommendation_id: format!("mkt-{}", uuid::Uuid::new_v4()),
                        model_id: demand.model_id.clone(),
                        district_id: best_fit.district_id.clone(),
                        action: MarketAction::IncreaseReplication,
                        replication_level: 3,
                        gravity_alignment: 0.85,
                        confidence: 0.92,
                        reason: format!(
                            "High demand density ({:.2}) and latency pressure ({:.2}) detected for {} in region {}.",
                            demand.demand_density, demand.latency_pressure, demand.model_id, demand.region
                        ),
                        created_at: 1710586000, // Placeholder
                    });
                }
            }
        }

        // Detect oversupply to decommission artifacts
        for supply in supply_signals {
            if supply.supply_capacity < 0.2 && !supply.active_models.is_empty() {
                recommendations.push(MarketRecommendation {
                    recommendation_id: format!("mkt-{}", uuid::Uuid::new_v4()),
                    model_id: supply.active_models[0].clone(),
                    district_id: supply.district_id.clone(),
                    action: MarketAction::DecommissionArtifact,
                    replication_level: 0,
                    gravity_alignment: 0.1,
                    confidence: 0.75,
                    reason: "Low utilization and supply capacity constraints. Recommending artifact cooldown.".to_string(),
                    created_at: 1710586000,
                });
            }
        }

        recommendations
    }
}
