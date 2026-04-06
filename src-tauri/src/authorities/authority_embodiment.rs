use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorityEmbodimentType {
    SingleAgent,
    AgentTeam,
    AuthorityFabric,
    ServiceAuthority,
    HybridAuthority,
    EmergentAuthority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityEmbodimentProfile {
    pub authority_id: String,
    pub embodiment_type: AuthorityEmbodimentType,
    pub distributed: bool,
    pub stateful: bool,
    pub requires_consensus: bool,
    pub has_internal_roles: bool,
    pub exposes_unified_interface: bool,
}

impl AuthorityEmbodimentProfile {
    pub fn new_sofia() -> Self {
        Self {
            authority_id: "SOFIIA".to_string(),
            embodiment_type: AuthorityEmbodimentType::HybridAuthority,
            distributed: true,
            stateful: true,
            requires_consensus: true,
            has_internal_roles: true,
            exposes_unified_interface: true,
        }
    }

    pub fn new_aistalk() -> Self {
        Self {
            authority_id: "AISTALK".to_string(),
            embodiment_type: AuthorityEmbodimentType::AgentTeam,
            distributed: false, // Investigative team typically localized for investigation scope
            stateful: true,
            requires_consensus: true,
            has_internal_roles: true,
            exposes_unified_interface: true,
        }
    }

    pub fn new_sentinel() -> Self {
        Self {
            authority_id: "SENTINEL".to_string(),
            embodiment_type: AuthorityEmbodimentType::AuthorityFabric,
            distributed: true,
            stateful: false, // Evidence is transient/flowing
            requires_consensus: false, // Normalized truth is aggregate
            has_internal_roles: false,
            exposes_unified_interface: true,
        }
    }
}
