pub mod artifact_holder;
pub mod artifact_store;
pub mod co_download;
pub mod co_download_intent;
pub mod collector_signal;
pub mod eviction_policy;
pub mod gravity_collector_sync;
pub mod gravity_score;
pub mod heartbeat_topology;
pub mod inference_stream;
pub mod model_gravity;
pub mod placement_activation;
pub mod placement_intent;
pub mod placement_policy;
pub mod placement_recommendation;
pub mod registry;
pub mod residency_policy;
pub mod residency_score;
pub mod runtime_loader;
pub mod session_routing;
pub mod session_shard;
pub mod token_stream;
pub mod trust_topology;
pub mod verifier;

use crate::models::eviction_policy::EvictionPolicy;
use crate::models::registry::{ModelRegistry, RegistryPayload};
use crate::models::residency_score::{ModelResidencyStats, ScoringEngine};
use tauri::{command, AppHandle};

#[command]
pub async fn sync_registry(app: AppHandle) -> Result<RegistryPayload, String> {
    ModelRegistry::fetch_registry(app).await
}

#[command]
pub fn get_residency_score(
    stats: ModelResidencyStats,
    ram_gb: f32,
    is_specialized: bool,
) -> Result<f32, String> {
    let pressure = EvictionPolicy::get_system_memory_pressure();
    let score = ScoringEngine::calculate_score(&stats, ram_gb, is_specialized, pressure);
    Ok(score.total_score)
}
