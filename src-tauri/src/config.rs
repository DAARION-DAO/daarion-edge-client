//! config.rs — DAARION Edge Client configuration resolution
//!
//! Backend URL priority chain (for registry and Genesis integration):
//!   1. DAARION_BACKEND_URL runtime env var (dev/operator override)
//!   2. DAARION_BACKEND_URL compile-time env var (release build config)
//!   3. "http://localhost:8010" only in debug/dev builds
//!   4. None in production builds (pairing/config required)
//!
//! Relay endpoint priority chain (for Worker relay mode, unchanged):
//!   1. DAARION_RELAY_ENDPOINT env var
//!   2. AppConfig.relay_endpoint
//!   3. None (honest "not configured" state)

use serde::{Deserialize, Serialize};

const DEV_BACKEND_URL: &str = "http://localhost:8010";
const BACKEND_NOT_CONFIGURED: &str =
    "DAARION backend is not configured. Pair this client with a DAARION backend before network operations.";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub backend_url: String,
    pub environment: String,
    /// Relay endpoint for Worker Mode. Empty string means not configured.
    pub relay_endpoint: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let backend_url = resolve_backend_url_from_sources(
            std::env::var("DAARION_BACKEND_URL").ok().as_deref(),
            option_env!("DAARION_BACKEND_URL"),
            cfg!(debug_assertions),
        )
        .unwrap_or_default();
        let environment = if backend_url == DEV_BACKEND_URL {
            "development"
        } else if backend_url.is_empty() {
            "unconfigured"
        } else {
            "production"
        };

        Self {
            backend_url,
            environment: environment.to_string(),
            relay_endpoint: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackendConfigStatus {
    pub configured: bool,
    pub backend_url: Option<String>,
    pub environment: String,
    pub dev_default: bool,
    pub message: String,
}

pub fn get_config() -> AppConfig {
    AppConfig::default()
}

fn normalize_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_end_matches('/').to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn resolve_backend_url_from_sources(
    runtime_env: Option<&str>,
    compile_env: Option<&str>,
    debug_build: bool,
) -> Option<String> {
    runtime_env
        .and_then(normalize_url)
        .or_else(|| compile_env.and_then(normalize_url))
        .or_else(|| debug_build.then(|| DEV_BACKEND_URL.to_string()))
}

/// Resolve the SOFIIA registry backend URL using the approved priority chain.
///
/// Priority:
///   1. DAARION_BACKEND_URL runtime env var (dev/operator override)
///   2. DAARION_BACKEND_URL compile-time env var (release build config)
///   3. debug/dev fallback to http://localhost:8010
///
/// Production builds without an explicit backend return an error.
pub fn resolve_backend_url() -> Result<String, String> {
    let config = get_config();
    normalize_url(&config.backend_url).ok_or_else(|| BACKEND_NOT_CONFIGURED.to_string())
}

#[tauri::command]
pub fn get_backend_config_status() -> BackendConfigStatus {
    let config = get_config();
    let backend_url = normalize_url(&config.backend_url);
    let dev_default = backend_url.as_deref() == Some(DEV_BACKEND_URL) && cfg!(debug_assertions);
    let configured = backend_url.is_some();
    let message = if dev_default {
        "Using local development backend.".to_string()
    } else if configured {
        "Backend configured.".to_string()
    } else {
        BACKEND_NOT_CONFIGURED.to_string()
    };

    BackendConfigStatus {
        configured,
        backend_url,
        environment: config.environment,
        dev_default,
        message,
    }
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
    fn test_backend_url_debug_default() {
        std::env::remove_var("DAARION_BACKEND_URL");
        let url = resolve_backend_url().expect("debug builds should use local backend");
        assert!(
            !url.ends_with('/'),
            "URL must not have trailing slash: {}",
            url
        );
    }

    #[test]
    fn test_backend_url_release_requires_explicit_config() {
        let url = resolve_backend_url_from_sources(None, None, false);
        assert!(url.is_none());
    }

    #[test]
    fn test_backend_url_env_wins_and_trims() {
        let url = resolve_backend_url_from_sources(
            Some("https://api.daarion.city/"),
            Some("https://compile.example"),
            false,
        )
        .expect("env should resolve");
        assert_eq!(url, "https://api.daarion.city");
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
