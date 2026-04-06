use tauri::AppHandle;
use crate::models::runtime_loader::RuntimeLoader;

pub struct EvictionPolicy;

impl EvictionPolicy {
    pub async fn trigger_eviction(app: AppHandle, model_id: String) -> Result<(), String> {
        println!("EvictionPolicy: Trimming model {} from RAM", model_id);
        
        // In M1, eviction is just unloading
        RuntimeLoader::unload_model(app, model_id).await
    }

    pub fn get_system_memory_pressure() -> f32 {
        // M1: Simulated pressure
        // In production this would use sys-info or similar
        0.2 // 20% pressure
    }
}
