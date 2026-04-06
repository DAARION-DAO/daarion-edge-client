use std::path::{PathBuf, Path};
use tauri::{AppHandle, Manager, Emitter};
use crate::models::registry::ModelRegistryEntry;

pub struct ArtifactStore;

impl ArtifactStore {
    pub fn get_models_dir(app: &AppHandle) -> Result<PathBuf, String> {
        let app_dir = app.path().app_data_dir()
            .map_err(|e| e.to_string())?;
        let models_dir = app_dir.join("models");
        if !models_dir.exists() {
            std::fs::create_dir_all(&models_dir)
                .map_err(|e| e.to_string())?;
        }
        Ok(models_dir)
    }

    pub async fn list_local_models(app: &AppHandle) -> Result<Vec<String>, String> {
        let dir = Self::get_models_dir(app)?;
        let mut models = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            models.push(name.to_string());
                        }
                    }
                }
            }
        }
        Ok(models)
    }

    pub async fn download_model(app: AppHandle, entry: ModelRegistryEntry) -> Result<(), String> {
        let dir = Self::get_models_dir(&app)?;
        let file_path = dir.join(format!("{}.gguf", entry.model_id));

        // M1: Simulate download progress
        println!("Simulating download of {} to {:?}", entry.model_id, file_path);
        
        for i in (0..=100).step_by(10) {
            app.emit("model-download-progress", serde_json::json!({
                "model_id": entry.model_id,
                "progress": i
            })).unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        // Create a dummy file for M1
        std::fs::write(&file_path, "DUMMY MODEL DATA")
            .map_err(|e| e.to_string())?;

        app.emit("model-download-finished", &entry.model_id).unwrap();
        Ok(())
    }
}
