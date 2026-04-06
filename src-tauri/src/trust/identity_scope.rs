use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityScope {
    pub scope_id: Uuid,
    pub identity_id: Uuid,
    pub allowed_namespaces: Vec<String>, // e.g., ["node.*", "agent.sofiia.*"]
    pub forbidden_namespaces: Vec<String>,
}
