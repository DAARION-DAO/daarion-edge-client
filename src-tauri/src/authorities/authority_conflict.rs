use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::authority_protocol::AuthorityLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityConflict {
    pub conflict_id: Uuid,
    pub layer_a: AuthorityLayer,
    pub layer_b: AuthorityLayer,
    pub conflicting_actions: Vec<String>,
    pub resolution_strategy: String, // e.g., "PrecedenceResolved", "GovernanceEscalation"
}
