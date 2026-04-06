use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoritySubrole {
    pub role_id: String,
    pub description: String,
    pub has_veto_rights: bool,
    pub can_approve: bool,
    pub escalation_required: bool,
}
