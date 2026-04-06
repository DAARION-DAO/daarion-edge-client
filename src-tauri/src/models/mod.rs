pub mod registry;
pub mod artifact_store;
pub mod verifier;
pub mod runtime_loader;
pub mod residency_score;
pub mod residency_policy;
pub mod eviction_policy;
pub mod inference_session;
pub mod inference_limits;
pub mod local_inference;
pub mod inference_arbitrator;
pub mod trust_topology;
pub mod heartbeat_topology;
pub mod model_gravity;
pub mod gravity_score;
pub mod placement_policy;
pub mod token_stream;
pub mod inference_stream;
pub mod session_routing;
pub mod session_shard;
pub mod placement_activation;
pub mod placement_recommendation;
pub mod placement_intent;
pub mod collector_signal;
pub mod gravity_collector_sync;
pub mod artifact_holder;
pub mod co_download;
pub mod co_download_intent;

use tauri::{AppHandle, command, State};
use crate::models::registry::{ModelRegistry, ModelRegistryEntry};
use crate::models::artifact_store::ArtifactStore;
use crate::models::residency_score::{ModelResidencyStats, ScoringEngine};
use crate::models::eviction_policy::EvictionPolicy;
use crate::models::inference_session::{LocalInferenceRequest, LocalInferenceResponse};
use crate::models::local_inference::LocalInference;
use crate::models::model_gravity::{ModelGravity, ModelPlacementRecommendation};
use crate::models::gravity_score::ModelGravitySignal;

#[command]
pub async fn get_supported_models() -> Result<Vec<ModelRegistryEntry>, String> {
    Ok(ModelRegistry::get_supported_models())
}

#[command]
pub async fn get_local_models(app: AppHandle) -> Result<Vec<String>, String> {
    ArtifactStore::list_local_models(&app).await
}

#[command]
pub async fn download_model(app: AppHandle, entry: ModelRegistryEntry) -> Result<(), String> {
    ArtifactStore::download_model(app, entry).await
}

#[command]
pub async fn load_model(app: AppHandle, model_id: String) -> Result<(), String> {
    runtime_loader::RuntimeLoader::load_model(app, model_id).await
}

#[command]
pub async fn unload_model(app: AppHandle, model_id: String) -> Result<(), String> {
    runtime_loader::RuntimeLoader::unload_model(app, model_id).await
}

#[command]
pub fn get_residency_score(
    stats: ModelResidencyStats, 
    ram_gb: f32, 
    is_specialized: bool
) -> Result<f32, String> {
    let pressure = EvictionPolicy::get_system_memory_pressure();
    let score = ScoringEngine::calculate_score(&stats, ram_gb, is_specialized, pressure);
    Ok(score.total_score)
}

#[command]
pub async fn run_local_inference(app: AppHandle, request: LocalInferenceRequest) -> Result<LocalInferenceResponse, String> {
    LocalInference::run_chat(app, request).await
}

#[command]
pub fn get_placement_recommendation(model_id: String, signal: ModelGravitySignal) -> Result<ModelPlacementRecommendation, String> {
    Ok(ModelGravity::get_recommendation(model_id, signal))
}
