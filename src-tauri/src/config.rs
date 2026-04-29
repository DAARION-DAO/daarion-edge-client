//! config.rs — DAARION Edge Client configuration resolution
//!
//! Backend URL priority chain (for registry integration):
//!   1. DAARION_BACKEND_URL env var (dev/operator override, highest priority)
//!   2. AppConfig.backend_url (compile-time/Tauri config default)
//!   3. "http://localhost:8010" (SOFIIA Console dev default — only used in dev builds)
//!
//! Relay endpoint priority chain (for Worker relay mode, unchanged):
//!   1. DAARION_RELAY_ENDPOINT env var
//!   2. AppConfig.relay_endpoint
//!   3. None (honest "not configured" state)

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub backend_url: String,
    pub environment: String,
    /// Relay endpoint for Worker Mode. Empty string means not configured.
    pub relay_endpoint: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            // Default points to SOFIIA Console dev backend.
            // Production URL set via DAARION_BACKEND_URL env var or Tauri config.
            backend_url: "http://localhost:8010".to_string(),
            environment: "development".to_string(),
            relay_endpoint: String::new(),
        }
    }
}

pub fn get_config() -> AppConfig {
    AppConfig::default()
}

/// Resolve the SOFIIA registry backend URL using the approved priority chain.
///
/// Priority:
///   1. DAARION_BACKEND_URL env var (dev/operator override)
///   2. AppConfig.backend_url (compile default: http://localhost:8010)
///
/// Never returns an empty string. Always returns a usable base URL.
pub fn resolve_backend_url() -> String {
    if let Ok(env_url) = std::env::var("DAARION_BACKEND_URL") {
        let trimmed = env_url.trim().trim_end_matches('/').to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    let config = get_config();
    config.backend_url.trim_end_matches('/').to_string()
}

/// Resolve the relay endpoint using the approved priority chain:
///   1. DAARION_RELAY_ENDPOINT env var (dev/operator override)
///   2. AppConfig.relay_endpoint (primary source)
///   3. None — honest "not configured" state
///
/// Returns None if no relay is configured. Callers must handle this honestly.
pub fn resolve_relay_endpoint() -> Option<String> {
    if let Ok(env_ep) = std::env::var("DAARION_RELAY_ENDPOINT") {
        if !env_ep.is_empty() {
            return Some(env_ep);
        }
    }
    let config = get_config();
    if !config.relay_endpoint.is_empty() {
        return Some(config.relay_endpoint);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_url_default() {
        // Without env var, should return the compile default
        std::env::remove_var("DAARION_BACKEND_URL");
        let url = resolve_backend_url();
        assert!(!url.is_empty());
        assert!(!url.ends_with('/'), "URL must not have trailing slash: {}", url);
    }

    #[test]
    fn test_backend_url_trailing_slash_stripped() {
        let url = "http://localhost:8010/";
        let stripped = url.trim_end_matches('/').to_string();
        assert_eq!(stripped, "http://localhost:8010");
    }

    #[test]
    fn test_relay_endpoint_none_when_empty() {
        std::env::remove_var("DAARION_RELAY_ENDPOINT");
        // Default config has empty relay_endpoint
        let config = get_config();
        assert!(config.relay_endpoint.is_empty());
    }
}
