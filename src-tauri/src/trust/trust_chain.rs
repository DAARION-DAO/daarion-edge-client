use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustScopeNamespace {
    Global,
    Regional(String),
    District(String),
    Worker(String),
    Agent(String),
    Session(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScopeBinding {
    pub namespaces: Vec<TrustScopeNamespace>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustLink {
    pub link_id: Uuid,
    pub parent_link_id: Option<Uuid>,
    pub identity_ref: String, // DAIS Identity ID
    pub credential_ref: Uuid, // Certificate/Session/Lease ID
    pub scope: TrustScopeBinding,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustChain {
    pub root_link: TrustLink,
    pub links: Vec<TrustLink>,
}
