use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::authority_protocol::AuthorityLayer;
use super::authority_decision::AuthorityDecision;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityTraceEntry {
    pub layer: AuthorityLayer,
    pub decision: AuthorityDecision,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
}
