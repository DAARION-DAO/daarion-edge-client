//! config.rs — DAARION Edge Client configuration resolution
//!
//! Backend URL priority chain (for registry and Genesis integration):
//!   1. Persisted PairingState from pairing.json
//!   2. One-time import of explicit DAARION_BACKEND_URL runtime/compile config
//!   3. "http://localhost:8010" only in debug/dev builds, never persisted
//!   4. None in production builds (pairing required)
//!
//! Relay endpoint priority chain (for Worker relay mode, unchanged):
//!   1. DAARION_RELAY_ENDPOINT env var
//!   2. AppConfig.relay_endpoint
//!   3. None (honest "not configured" state)

use crate::pairing::{
    load_or_import_pairing_state_at, normalize_backend_url, source_label, PairingState,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Manager;

const DEV_BACKEND_URL: &str = "http://localhost:8010";
const BACKEND_NOT_CONFIGURED: &str =
    "Pairing required. Enter a DAARION invitation code before network operations.";
const BACKEND_OVERRIDE_FLAG: &str = "DAARION_BACKEND_OVERRIDE";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub backend_url: String,
    pub environment: String,
    /// Relay endpoint for Worker Mode. Empty string means not configured.
    pub relay_endpoint: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let backend_url = resolve_ephemeral_backend_url_from_sources(
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
    pub paired: bool,
    pub pairing_label: Option<String>,
    pub pairing_source: Option<String>,
    pub connection_status: String,
    pub message: String,
}

pub fn get_config() -> AppConfig {
    AppConfig::default()
}

fn resolve_ephemeral_backend_url_from_sources(
    runtime_env: Option<&str>,
    compile_env: Option<&str>,
    debug_build: bool,
) -> Option<String> {
    runtime_env
        .and_then(normalize_backend_url)
        .or_else(|| compile_env.and_then(normalize_backend_url))
        .or_else(|| debug_build.then(|| DEV_BACKEND_URL.to_string()))
}

fn get_app_dir(handle: &tauri::AppHandle) -> PathBuf {
    handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir")
}

fn developer_override_enabled() -> bool {
    std::env::var(BACKEND_OVERRIDE_FLAG)
        .ok()
        .is_some_and(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

fn status_from_backend(
    backend_url: String,
    environment: String,
    dev_default: bool,
    paired: bool,
    pairing_label: Option<String>,
    pairing_source: Option<String>,
    connection_status: String,
    message: String,
) -> BackendConfigStatus {
    BackendConfigStatus {
        configured: true,
        backend_url: Some(backend_url),
        environment,
        dev_default,
        paired,
        pairing_label,
        pairing_source,
        connection_status,
        message,
    }
}

fn unconfigured_status(message: String) -> BackendConfigStatus {
    BackendConfigStatus {
        configured: false,
        backend_url: None,
        environment: "unconfigured".to_string(),
        dev_default: false,
        paired: false,
        pairing_label: None,
        pairing_source: None,
        connection_status: crate::pairing::CONNECTION_NOT_CHECKED.to_string(),
        message,
    }
}

fn paired_backend_status(pairing: PairingState) -> Option<BackendConfigStatus> {
    if !pairing.is_paired() {
        return None;
    }

    let backend_url = pairing
        .backend_url
        .as_deref()
        .and_then(normalize_backend_url)?;
    let source = pairing.source.as_ref().map(source_label).map(str::to_string);
    Some(status_from_backend(
        backend_url,
        pairing
            .environment
            .clone()
            .unwrap_or_else(|| "production".to_string()),
        false,
        true,
        pairing.label.clone(),
        source,
        pairing.connection_status,
        pairing.message,
    ))
}

fn resolve_backend_status_at(
    app_dir: &Path,
    runtime_env: Option<&str>,
    compile_env: Option<&str>,
    debug_build: bool,
    override_enabled: bool,
) -> BackendConfigStatus {
    if override_enabled {
        if let Some(backend_url) = runtime_env.and_then(normalize_backend_url) {
            return status_from_backend(
                backend_url,
                "development".to_string(),
                false,
                false,
                Some("Developer override".to_string()),
                Some("developer_override".to_string()),
                crate::pairing::CONNECTION_NOT_CHECKED.to_string(),
                "Using explicit developer backend override.".to_string(),
            );
        }
    }

    let pairing = match load_or_import_pairing_state_at(app_dir, runtime_env, compile_env) {
        Ok(state) => state,
        Err(e) => return unconfigured_status(format!("Pairing state invalid: {}", e)),
    };

    if let Some(status) = paired_backend_status(pairing) {
        return status;
    }

    if debug_build {
        return status_from_backend(
            DEV_BACKEND_URL.to_string(),
            "development".to_string(),
            true,
            false,
            Some("Local development backend".to_string()),
            Some("debug_default".to_string()),
            crate::pairing::CONNECTION_NOT_CHECKED.to_string(),
            "Using local development backend. This is not persisted as pairing.".to_string(),
        );
    }

    unconfigured_status(BACKEND_NOT_CONFIGURED.to_string())
}

/// Resolve the registry/Genesis backend URL from persisted pairing state.
///
/// Production builds without pairing return an error. Debug builds may use the
/// local fallback, but the localhost fallback is never persisted as user state.
pub fn resolve_backend_url(handle: &tauri::AppHandle) -> Result<String, String> {
    let status = resolve_backend_status_at(
        &get_app_dir(handle),
        std::env::var("DAARION_BACKEND_URL").ok().as_deref(),
        option_env!("DAARION_BACKEND_URL"),
        cfg!(debug_assertions),
        developer_override_enabled(),
    );

    status.backend_url.ok_or(status.message)
}

#[tauri::command]
pub fn get_backend_config_status(handle: tauri::AppHandle) -> BackendConfigStatus {
    resolve_backend_status_at(
        &get_app_dir(&handle),
        std::env::var("DAARION_BACKEND_URL").ok().as_deref(),
        option_env!("DAARION_BACKEND_URL"),
        cfg!(debug_assertions),
        developer_override_enabled(),
    )
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
    use crate::pairing::{pair_backend_at, PAIRING_FILE};
    use std::fs;

    fn temp_app_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "daarion-config-test-{}-{}",
            name,
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_backend_url_debug_default() {
        let dir = temp_app_dir("debug-default");
        let status = resolve_backend_status_at(&dir, None, None, true, false);

        assert_eq!(status.backend_url.as_deref(), Some(DEV_BACKEND_URL));
        assert!(status.dev_default);
        assert!(!dir.join(PAIRING_FILE).exists());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_backend_url_release_requires_explicit_config() {
        let dir = temp_app_dir("release-unconfigured");
        let status = resolve_backend_status_at(&dir, None, None, false, false);

        assert!(!status.configured);
        assert!(status.message.contains("Pairing required"));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_backend_url_env_imports_and_trims() {
        let dir = temp_app_dir("env-import");
        let status = resolve_backend_status_at(
            &dir,
            Some("https://api.daarion.city/"),
            Some("https://compile.example"),
            false,
            false,
        );

        assert!(status.configured);
        assert_eq!(status.backend_url.as_deref(), Some("https://api.daarion.city"));
        assert_eq!(status.pairing_source.as_deref(), Some("env_import"));
        assert!(dir.join(PAIRING_FILE).exists());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_pairing_state_wins_over_env() {
        let dir = temp_app_dir("pairing-precedence");
        pair_backend_at(
            &dir,
            "daarion://pair?backend=https%3A%2F%2Fpaired.daarion.city",
            false,
        )
        .unwrap();

        let status = resolve_backend_status_at(
            &dir,
            Some("https://env.daarion.city"),
            Some("https://compile.example"),
            false,
            false,
        );

        assert_eq!(status.backend_url.as_deref(), Some("https://paired.daarion.city"));
        assert_eq!(status.pairing_source.as_deref(), Some("invite_payload"));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_debug_localhost_env_is_not_imported_as_pairing() {
        let dir = temp_app_dir("localhost");
        let status =
            resolve_backend_status_at(&dir, Some("http://localhost:8010"), None, true, false);

        assert_eq!(status.backend_url.as_deref(), Some(DEV_BACKEND_URL));
        assert!(status.dev_default);
        assert!(!dir.join(PAIRING_FILE).exists());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_backend_url_trailing_slash_stripped() {
        let url = normalize_backend_url("http://localhost:8010/").unwrap();
        assert_eq!(url, "http://localhost:8010");
    }

    #[test]
    fn test_invalid_pairing_state_fails_closed() {
        let dir = temp_app_dir("invalid-state");
        fs::write(dir.join(PAIRING_FILE), "{}").unwrap();

        let status = resolve_backend_status_at(&dir, None, None, true, false);
        assert!(!status.configured);
        assert!(status.message.contains("Pairing state invalid"));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_developer_override_requires_flag() {
        let dir = temp_app_dir("override");
        let without_flag =
            resolve_backend_status_at(&dir, Some("https://dev.daarion.city"), None, true, false);
        let with_flag =
            resolve_backend_status_at(&dir, Some("https://dev.daarion.city"), None, true, true);

        assert_eq!(without_flag.pairing_source.as_deref(), Some("env_import"));
        assert_eq!(with_flag.pairing_source.as_deref(), Some("developer_override"));
        assert!(!with_flag.paired);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_relay_endpoint_none_when_empty() {
        std::env::remove_var("DAARION_RELAY_ENDPOINT");
        // Default config has empty relay_endpoint
        let config = get_config();
        assert!(config.relay_endpoint.is_empty());
    }
}
