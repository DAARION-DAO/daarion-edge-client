use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub backend_url: String,
    pub environment: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            backend_url: "http://localhost:8080".to_string(), // Dev default
            environment: "development".to_string(),
        }
    }
}

pub fn get_config() -> AppConfig {
    // For now, return default. In future, this could read from a config file or env vars.
    AppConfig::default()
}
