use serde::{Deserialize, Serialize};
use crate::models::artifact_holder::ArtifactHolderRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoDownloadRequest {
    pub request_id: String,
    pub model_id: String,
    pub district_id: String,
    pub proposed_role: ArtifactHolderRole,
    pub gravity_score: f32,
    pub telemetry_basis: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoDownloadStatus {
    Requested,
    Coordinating,
    Confirmed,
    Rejected,
}
