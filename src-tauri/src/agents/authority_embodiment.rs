use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorityEmbodimentType {
    SingleAuthorityAgent,
    AgentTeam,
    AuthorityFabric,
    ServiceAuthority,
    HybridAuthority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityEmbodiment {
    pub embodiment_id: String,
    pub embodiment_type: AuthorityEmbodimentType,
    pub is_distributed: bool,
    pub trust_scope: String,
}
