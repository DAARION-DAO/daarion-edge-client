use serde::{Deserialize, Serialize};
use super::authority_embodiment::AuthorityEmbodiment;
use super::authority_subrole::AuthoritySubrole;
use super::authority_interface::AuthorityInterfaceSurface;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityTeamProfile {
    pub authority_id: String,
    pub layer: String, // Identity | Security | Observability | Architecture | Orchestration
    pub embodiment: AuthorityEmbodiment,
    pub subroles: Vec<AuthoritySubrole>,
    pub interface_surfaces: Vec<AuthorityInterfaceSurface>,
    pub escalation_rights: bool,
    pub evidence_dependencies: Vec<String>,
}

pub struct AuthorityTeamValidator;

impl AuthorityTeamValidator {
    pub fn validate_team_legitimacy(profile: &AuthorityTeamProfile) -> bool {
        // Institutional authorities must have at least one interface surface
        if profile.interface_surfaces.is_empty() {
            return false;
        }
        
        // Fabrics must be distributed
        if matches!(profile.embodiment.embodiment_type, super::authority_embodiment::AuthorityEmbodimentType::AuthorityFabric) {
            if !profile.embodiment.is_distributed {
                return false;
            }
        }

        true
    }
}
