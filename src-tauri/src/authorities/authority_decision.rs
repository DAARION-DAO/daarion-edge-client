use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::authority_protocol::AuthorityLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorityDecision {
    Allow,
    Block,
    Veto,
    RequireReview,
    Escalate,
    Defer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityLayerDecision {
    pub authority_layer: AuthorityLayer,
    pub decision: AuthorityDecision,
    pub reason_summary: String,
    pub evidence_ref: Option<Uuid>,
    pub trust_scope: String,
    pub created_at: DateTime<Utc>,
}
