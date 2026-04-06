use serde::{Deserialize, Serialize};
use crate::models::artifact_holder::ArtifactHolderRole;
use crate::models::co_download::CoDownloadStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoDownloadIntent {
    pub intent_id: String,
    pub request_id: String,
    pub model_id: String,
    pub assigned_role: ArtifactHolderRole,
    pub status: CoDownloadStatus,
    pub approval_ref: Option<String>, // Link to AgentApproval if required
    pub confirmed_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoDownloadDecision {
    pub decision_id: String,
    pub intent_id: String,
    pub action: String, // e.g., "START_DOWNLOAD", "SKIP_DOWNLOAD"
    pub reason: String,
}
