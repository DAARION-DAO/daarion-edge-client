use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanAction {
    AdjustPlacement { model_id: String, target_node: String, reason: String },
    PromoteModel { model_id: String, role: String },
    DemoteModel { model_id: String, reason: String },
    DistrictCoordination { district_id: String, coordination_type: String },
    WorkerSpecialization { lane_id: String, specialization: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationPlan {
    pub plan_id: String,
    pub plan_type: String, // e.g., "PlacementOptimization", "LoadBalancing"
    pub target_scope: String,
    pub recommended_actions: Vec<PlanAction>,
    pub confidence: f32,
    pub telemetry_basis: String,
    pub priority: PlanPriority,
    pub created_at: u64,
}
