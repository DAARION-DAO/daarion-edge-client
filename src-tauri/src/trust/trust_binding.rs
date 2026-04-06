use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustBinding {
    pub binding_ref: Uuid,
    pub source_identity: Uuid,
    pub target_identity: Uuid,
    pub binding_type: String, // e.g., "NodeToAgent", "UserToNode"
    pub confidence_score: f32,
}
