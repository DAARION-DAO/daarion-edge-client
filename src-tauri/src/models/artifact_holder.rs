use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArtifactHolderRole {
    PrimaryHolder, // Always warm, handles inference
    WarmHolder,    // Has artifact, loads on demand
    ColdHolder,    // Has artifact on disk, not prioritised for RAM
    AvoidHolder,   // Explicitly told not to download
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactHolderStatus {
    pub node_id: String,
    pub model_id: String,
    pub role: ArtifactHolderRole,
    pub storage_used_gb: f32,
    pub last_seen: u64,
}
