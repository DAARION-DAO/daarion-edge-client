use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallSource {
    pub runtime: String,
    pub upstream_tag: String,
    pub local_alias: String,
    pub estimated_download_gb: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CandidateModel {
    pub id: String,
    pub family: String,
    pub tier: String,
    pub role: String,
    pub stability: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub install_sources: Vec<InstallSource>,
    #[serde(default)]
    pub is_recommended: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryPayload {
    pub schema_version: i32,
    pub registry_version: String,
    #[serde(default)]
    pub generated_at: String,
    #[serde(default)]
    pub families: Vec<String>,
    pub models: Vec<CandidateModel>,
    #[serde(default)]
    pub registry_sha256: String,
}

pub struct ModelRegistry;

impl ModelRegistry {
    /// Save registry to last_known_good_registry.json
    fn save_to_cache(app: &AppHandle, payload: &RegistryPayload) -> Result<(), String> {
        let path = app.path().app_data_dir()
            .map_err(|_| "Failed to get app_data_dir".to_string())?;
        fs::create_dir_all(&path).ok();
        let file_path = path.join("last_known_good_registry.json");
        let content = serde_json::to_string(payload).map_err(|e| e.to_string())?;
        fs::write(&file_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Read registry from last_known_good_registry.json
    fn read_from_cache(app: &AppHandle) -> Result<RegistryPayload, String> {
        let path = app.path().app_data_dir()
            .map_err(|_| "Failed to get app_data_dir".to_string())?;
        let file_path = path.join("last_known_good_registry.json");
        let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    /// Read bundled fallback_registry.json
    fn read_bundled_fallback() -> Result<RegistryPayload, String> {
        // Fallback payload backed directly into binary
        let fallback_str = include_str!("../../../public/fallback_registry.json");
        serde_json::from_str(fallback_str).map_err(|e| e.to_string())
    }

    pub async fn fetch_registry(app: AppHandle) -> Result<RegistryPayload, String> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        // 1. Try Network
        match client.get("https://api.daarion.city/models/registry").send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(payload) = resp.json::<RegistryPayload>().await {
                    println!("[RegistrySync] Successfully fetched from Network");
                    let _ = Self::save_to_cache(&app, &payload);
                    return Ok(payload);
                }
            }
            _ => {}
        }

        // 2. Try DB Cache (last_known_good_registry)
        if let Ok(cached) = Self::read_from_cache(&app) {
            println!("[RegistrySync] Loaded from Local Cache");
            return Ok(cached);
        }

        // 3. Bundled Fallback
        println!("[RegistrySync] Loaded from Bundled Asset (public/fallback_registry.json)");
        Self::read_bundled_fallback()
    }
}
