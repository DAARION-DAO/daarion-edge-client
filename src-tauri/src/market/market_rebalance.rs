use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketRebalanceAction {
    IncreaseReplication { model_id: String, target_district: String, amount: u32 },
    ReduceReplication { model_id: String, target_district: String, amount: u32 },
    PromoteDistrictHosting { district_id: String, model_class: String },
    ReduceDistrictHosting { district_id: String, model_class: String },
    IncreaseRoutingBias { source_region: String, target_district: String, weight: f32 },
    DecreaseRoutingBias { source_region: String, target_district: String, weight: f32 },
    AdjustArtifactHolderRoles { district_id: String, roles: Vec<String> },
}
