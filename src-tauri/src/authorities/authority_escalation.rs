use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::authority_protocol::AuthorityLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityEscalationPath {
    pub escalation_id: Uuid,
    pub source_layer: AuthorityLayer,
    pub target_reviewer: String, // e.g., "Human", "GovernanceBoard", "GlobalSovereign"
    pub priority: u8, // 0-255
}
