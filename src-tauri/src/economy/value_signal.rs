use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueSignal {
    pub value_id: String,
    pub target_id: String, // model_id, node_id, or district_id
    pub usefulness_score: f64,
    pub scarcity_score: f64,
    pub demand_density: f64,
    pub latency_value: f64,
    pub trust_cost: f64,
    pub evidence_basis: String,
    pub created_at: DateTime<Utc>,
}
