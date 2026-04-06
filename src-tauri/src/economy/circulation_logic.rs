use serde::{Deserialize, Serialize};
use super::value_signal::ValueSignal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CirculationDecision {
    IncreaseReplication(String), // model_id
    WarmIncentive(String),       // district_id
    CoolDown(String),            // district_id
    AuditRequired(String),       // resource_id
}

pub struct CirculationEngine;

impl CirculationEngine {
    pub fn derived_decisions(signals: &[ValueSignal]) -> Vec<CirculationDecision> {
        let mut decisions = Vec::new();
        
        for signal in signals {
            // Logic: If demand is high and usefulness is high -> Increase Replication
            if signal.demand_density > 0.8 && signal.usefulness_score > 0.7 {
                decisions.push(CirculationDecision::IncreaseReplication(signal.target_id.clone()));
            }
            
            // Logic: If scarcity is high and usefulness is high -> Warm Incentive
            if signal.scarcity_score > 0.9 && signal.usefulness_score > 0.8 {
                decisions.push(CirculationDecision::WarmIncentive(signal.target_id.clone()));
            }
        }
        
        decisions
    }
}
