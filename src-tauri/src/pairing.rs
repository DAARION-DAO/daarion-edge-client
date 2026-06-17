//! pairing.rs — persisted DAARION backend pairing state
//!
//! This module owns normal user backend configuration. Invite payloads are
//! parsed locally; no discovery service or health probing is introduced here.

use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;
use url::Url;

pub const PAIRING_FILE: &str = "pairing.json";
pub const CONNECTION_NOT_CHECKED: &str = "not_checked";

const DEFAULT_LABEL: &str = "DAARION Backend";
const INVALID_INVITE: &str = "Invitation code is invalid or incomplete.";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PairingLifecycle {
    Paired,
    Unpaired,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PairingSource {
    InvitePayload,
    ManualAdvanced,
    EnvImport,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PairingState {
    pub state: PairingLifecycle,
    pub backend_url: Option<String>,
    pub label: Option<String>,
    pub environment: Option<String>,
    pub source: Option<PairingSource>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub connection_status: String,
    pub message: String,
}

impl PairingState {
    pub fn is_paired(&self) -> bool {
        self.state == PairingLifecycle::Paired && self.backend_url.is_some()
    }

    pub fn unpaired() -> Self {
        Self {
            state: PairingLifecycle::Unpaired,
            backend_url: None,
            label: None,
            environment: None,
            source: None,
            created_at: None,
            updated_at: None,
            connection_status: CONNECTION_NOT_CHECKED.to_string(),
            message: "Pairing required. Enter an invitation code before network operations."
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPairingPayload {
    pub backend_url: String,
    pub label: String,
    pub environment: String,
    pub source: PairingSource,
}

fn get_app_dir(handle: &tauri::AppHandle) -> PathBuf {
    handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir")
}

fn ensure_app_dir(app_dir: &Path) -> Result<(), String> {
    if !app_dir.exists() {
        fs::create_dir_all(app_dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn pairing_path(app_dir: &Path) -> PathBuf {
    app_dir.join(PAIRING_FILE)
}

fn save_pairing_state_at(app_dir: &Path, state: &PairingState) -> Result<(), String> {
    ensure_app_dir(app_dir)?;
    let content = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(pairing_path(app_dir), content).map_err(|e| e.to_string())
}

pub fn load_pairing_state_at(app_dir: &Path) -> Result<PairingState, String> {
    let path = pairing_path(app_dir);
    if !path.exists() {
        return Ok(PairingState::unpaired());
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let state: PairingState = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    validate_pairing_state(&state)?;
    Ok(state)
}

pub fn load_or_import_pairing_state_at(
    app_dir: &Path,
    runtime_env: Option<&str>,
    compile_env: Option<&str>,
) -> Result<PairingState, String> {
    if pairing_path(app_dir).exists() {
        return load_pairing_state_at(app_dir);
    }

    let explicit_backend = runtime_env
        .and_then(normalize_backend_url)
        .or_else(|| compile_env.and_then(normalize_backend_url));

    match explicit_backend {
        Some(backend_url) if !is_loopback_backend(&backend_url) => {
            let state = paired_state_from_backend(
                backend_url,
                None,
                None,
                PairingSource::EnvImport,
                Utc::now(),
            );
            save_pairing_state_at(app_dir, &state)?;
            Ok(state)
        }
        _ => Ok(PairingState::unpaired()),
    }
}

pub fn load_or_import_pairing_state(handle: &tauri::AppHandle) -> Result<PairingState, String> {
    load_or_import_pairing_state_at(
        &get_app_dir(handle),
        std::env::var("DAARION_BACKEND_URL").ok().as_deref(),
        option_env!("DAARION_BACKEND_URL"),
    )
}

pub fn source_label(source: &PairingSource) -> &'static str {
    match source {
        PairingSource::InvitePayload => "invite_payload",
        PairingSource::ManualAdvanced => "manual_advanced",
        PairingSource::EnvImport => "env_import",
    }
}

pub fn normalize_backend_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }

    let mut url = Url::parse(trimmed).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    if url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return None;
    }

    url.set_query(None);
    url.set_fragment(None);
    Some(url.as_str().trim_end_matches('/').to_string())
}

pub fn is_loopback_backend(backend_url: &str) -> bool {
    Url::parse(backend_url)
        .ok()
        .and_then(|url| url.host_str().map(|host| host.to_ascii_lowercase()))
        .is_some_and(|host| host == "localhost" || host == "127.0.0.1" || host == "::1")
}

pub fn parse_pairing_payload(
    input: &str,
    manual_advanced: bool,
) -> Result<ParsedPairingPayload, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(INVALID_INVITE.to_string());
    }

    if manual_advanced {
        if let Some(payload) = parse_manual_backend_url(trimmed) {
            return Ok(payload);
        }
    }

    parse_invite_url(trimmed)
        .or_else(|| parse_prefixed_code(trimmed))
        .or_else(|| parse_json_payload(trimmed))
        .or_else(|| parse_base64_json_payload(trimmed))
        .ok_or_else(|| INVALID_INVITE.to_string())
}

fn validate_pairing_state(state: &PairingState) -> Result<(), String> {
    match state.state {
        PairingLifecycle::Paired => {
            let backend_url = state
                .backend_url
                .as_deref()
                .and_then(normalize_backend_url)
                .ok_or_else(|| "Invalid pairing state: missing backend URL".to_string())?;
            if state.source.is_none() {
                return Err("Invalid pairing state: missing source".to_string());
            }
            if state.created_at.is_none() || state.updated_at.is_none() {
                return Err("Invalid pairing state: missing timestamps".to_string());
            }
            if Some(&backend_url) != state.backend_url.as_ref() {
                return Err("Invalid pairing state: backend URL is not normalized".to_string());
            }
        }
        PairingLifecycle::Unpaired => {}
    }
    Ok(())
}

fn paired_state_from_backend(
    backend_url: String,
    label: Option<String>,
    environment: Option<String>,
    source: PairingSource,
    now: DateTime<Utc>,
) -> PairingState {
    let label = label
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| label_from_backend_url(&backend_url));
    let environment = environment
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| environment_from_backend_url(&backend_url));

    PairingState {
        state: PairingLifecycle::Paired,
        backend_url: Some(backend_url),
        label: Some(label),
        environment: Some(environment),
        source: Some(source),
        created_at: Some(now),
        updated_at: Some(now),
        connection_status: CONNECTION_NOT_CHECKED.to_string(),
        message: "Backend paired. Connection not checked yet.".to_string(),
    }
}

fn parse_manual_backend_url(input: &str) -> Option<ParsedPairingPayload> {
    let backend_url = normalize_backend_url(input)?;
    if !is_dev_or_staging_backend(&backend_url) {
        return None;
    }

    Some(ParsedPairingPayload {
        label: label_from_backend_url(&backend_url),
        environment: environment_from_backend_url(&backend_url),
        backend_url,
        source: PairingSource::ManualAdvanced,
    })
}

fn parse_invite_url(input: &str) -> Option<ParsedPairingPayload> {
    let url = Url::parse(input).ok()?;
    let backend_url = query_value(&url, &["backend_url", "backendUrl", "backend"])?;
    let backend_url = normalize_backend_url(&backend_url)?;
    let label = query_value(&url, &["label", "organization", "profile"])
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| label_from_backend_url(&backend_url));
    let environment = query_value(&url, &["environment", "env"])
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| environment_from_backend_url(&backend_url));

    Some(ParsedPairingPayload {
        backend_url,
        label,
        environment,
        source: PairingSource::InvitePayload,
    })
}

fn parse_prefixed_code(input: &str) -> Option<ParsedPairingPayload> {
    let encoded = input.strip_prefix("daarion-pair:")?;
    parse_base64_json_payload(encoded)
}

fn parse_json_payload(input: &str) -> Option<ParsedPairingPayload> {
    let value: Value = serde_json::from_str(input).ok()?;
    payload_from_json(&value)
}

fn parse_base64_json_payload(input: &str) -> Option<ParsedPairingPayload> {
    for decoded in decode_base64_candidates(input) {
        if let Ok(value) = serde_json::from_slice::<Value>(&decoded) {
            if let Some(payload) = payload_from_json(&value) {
                return Some(payload);
            }
        }
    }
    None
}

fn payload_from_json(value: &Value) -> Option<ParsedPairingPayload> {
    let backend_url = json_string(value, &["backend_url", "backendUrl", "backend"])?;
    let backend_url = normalize_backend_url(backend_url)?;
    let label = json_string(value, &["label", "organization", "profile"])
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| label_from_backend_url(&backend_url));
    let environment = json_string(value, &["environment", "env"])
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| environment_from_backend_url(&backend_url));

    Some(ParsedPairingPayload {
        backend_url,
        label,
        environment,
        source: PairingSource::InvitePayload,
    })
}

fn decode_base64_candidates(input: &str) -> Vec<Vec<u8>> {
    let trimmed = input.trim();
    let mut candidates = Vec::new();

    if let Ok(bytes) = general_purpose::URL_SAFE_NO_PAD.decode(trimmed) {
        candidates.push(bytes);
    }
    if let Ok(bytes) = general_purpose::URL_SAFE.decode(trimmed) {
        candidates.push(bytes);
    }
    if let Ok(bytes) = general_purpose::STANDARD.decode(trimmed) {
        candidates.push(bytes);
    }

    candidates
}

fn query_value(url: &Url, keys: &[&str]) -> Option<String> {
    url.query_pairs().find_map(|(key, value)| {
        keys.iter()
            .any(|candidate| key == *candidate)
            .then(|| value.to_string())
    })
}

fn json_string<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| value.get(*key)?.as_str())
}

fn label_from_backend_url(backend_url: &str) -> String {
    Url::parse(backend_url)
        .ok()
        .and_then(|url| url.host_str().map(ToString::to_string))
        .filter(|host| !host.is_empty())
        .unwrap_or_else(|| DEFAULT_LABEL.to_string())
}

fn environment_from_backend_url(backend_url: &str) -> String {
    let Some(url) = Url::parse(backend_url).ok() else {
        return "production".to_string();
    };
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();

    if is_loopback_backend(backend_url) || host.contains("dev") {
        "development".to_string()
    } else if host.contains("staging") || host.contains("stage") {
        "staging".to_string()
    } else {
        "production".to_string()
    }
}

fn is_dev_or_staging_backend(backend_url: &str) -> bool {
    let Some(url) = Url::parse(backend_url).ok() else {
        return false;
    };
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();

    is_loopback_backend(backend_url)
        || host.contains("dev")
        || host.contains("staging")
        || host.contains("stage")
}

pub fn pair_backend_at(
    app_dir: &Path,
    input: &str,
    manual_advanced: bool,
) -> Result<PairingState, String> {
    let parsed = parse_pairing_payload(input, manual_advanced)?;
    let existing = load_pairing_state_at(app_dir).unwrap_or_else(|_| PairingState::unpaired());
    let now = Utc::now();
    let created_at = existing.created_at.unwrap_or(now);
    let mut state = paired_state_from_backend(
        parsed.backend_url,
        Some(parsed.label),
        Some(parsed.environment),
        parsed.source,
        now,
    );
    state.created_at = Some(created_at);
    save_pairing_state_at(app_dir, &state)?;
    Ok(state)
}

pub fn unpair_backend_at(app_dir: &Path) -> Result<PairingState, String> {
    let state = PairingState::unpaired();
    save_pairing_state_at(app_dir, &state)?;
    Ok(state)
}

#[tauri::command]
pub fn get_pairing_status(handle: tauri::AppHandle) -> Result<PairingState, String> {
    load_or_import_pairing_state(&handle)
}

#[tauri::command]
pub fn pair_backend(
    handle: tauri::AppHandle,
    input: String,
    manual_advanced: Option<bool>,
) -> Result<PairingState, String> {
    pair_backend_at(&get_app_dir(&handle), &input, manual_advanced.unwrap_or(false))
}

#[tauri::command]
pub fn unpair_backend(handle: tauri::AppHandle) -> Result<PairingState, String> {
    unpair_backend_at(&get_app_dir(&handle))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_app_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "daarion-pairing-test-{}-{}",
            name,
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parses_invite_link_payload() {
        let payload = parse_pairing_payload(
            "daarion://pair?backend_url=https%3A%2F%2Fapi.daarion.city%2F&label=DAARION&environment=production",
            false,
        )
        .unwrap();

        assert_eq!(payload.backend_url, "https://api.daarion.city");
        assert_eq!(payload.label, "DAARION");
        assert_eq!(payload.environment, "production");
        assert_eq!(payload.source, PairingSource::InvitePayload);
    }

    #[test]
    fn parses_base64_json_invite_code() {
        let code = general_purpose::URL_SAFE_NO_PAD.encode(
            serde_json::json!({
                "backendUrl": "https://api.daarion.city/",
                "label": "DAARION City"
            })
            .to_string(),
        );
        let payload = parse_pairing_payload(&code, false).unwrap();

        assert_eq!(payload.backend_url, "https://api.daarion.city");
        assert_eq!(payload.label, "DAARION City");
        assert_eq!(payload.environment, "production");
    }

    #[test]
    fn rejects_invalid_payloads() {
        let err = parse_pairing_payload("not-a-short-code-service", false).unwrap_err();
        assert_eq!(err, INVALID_INVITE);
    }

    #[test]
    fn normalizes_backend_urls() {
        let normalized = normalize_backend_url(" https://api.daarion.city///?x=1#frag ").unwrap();
        assert_eq!(normalized, "https://api.daarion.city");
    }

    #[test]
    fn rejects_raw_backend_url_outside_advanced_flow() {
        let err = parse_pairing_payload("https://staging.daarion.city", false).unwrap_err();
        assert_eq!(err, INVALID_INVITE);
    }

    #[test]
    fn accepts_manual_advanced_dev_or_staging_url() {
        let payload = parse_pairing_payload("https://staging.daarion.city/", true).unwrap();

        assert_eq!(payload.backend_url, "https://staging.daarion.city");
        assert_eq!(payload.environment, "staging");
        assert_eq!(payload.source, PairingSource::ManualAdvanced);
    }

    #[test]
    fn rejects_manual_advanced_production_url() {
        let err = parse_pairing_payload("https://api.daarion.city", true).unwrap_err();
        assert_eq!(err, INVALID_INVITE);
    }

    #[test]
    fn persists_paired_and_unpaired_transitions() {
        let dir = temp_app_dir("transitions");
        let paired = pair_backend_at(
            &dir,
            "daarion://pair?backend=https%3A%2F%2Fapi.daarion.city&label=DAARION",
            false,
        )
        .unwrap();
        assert!(paired.is_paired());

        let loaded = load_pairing_state_at(&dir).unwrap();
        assert_eq!(loaded.backend_url.as_deref(), Some("https://api.daarion.city"));

        let unpaired = unpair_backend_at(&dir).unwrap();
        assert_eq!(unpaired.state, PairingLifecycle::Unpaired);
        assert!(!unpaired.is_paired());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn imports_explicit_env_backend_once() {
        let dir = temp_app_dir("env-import");
        let state =
            load_or_import_pairing_state_at(&dir, Some("https://api.daarion.city/"), None).unwrap();

        assert!(state.is_paired());
        assert_eq!(state.source, Some(PairingSource::EnvImport));
        assert_eq!(state.backend_url.as_deref(), Some("https://api.daarion.city"));
        assert!(pairing_path(&dir).exists());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn existing_pairing_state_wins_over_env_import() {
        let dir = temp_app_dir("precedence");
        pair_backend_at(
            &dir,
            "daarion://pair?backend=https%3A%2F%2Fpaired.daarion.city",
            false,
        )
        .unwrap();

        let state =
            load_or_import_pairing_state_at(&dir, Some("https://env.daarion.city"), None).unwrap();

        assert_eq!(state.backend_url.as_deref(), Some("https://paired.daarion.city"));
        assert_eq!(state.source, Some(PairingSource::InvitePayload));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn debug_localhost_is_not_persisted_as_pairing() {
        let dir = temp_app_dir("localhost");
        let state =
            load_or_import_pairing_state_at(&dir, Some("http://localhost:8010"), None).unwrap();

        assert_eq!(state.state, PairingLifecycle::Unpaired);
        assert!(!pairing_path(&dir).exists());

        fs::remove_dir_all(dir).unwrap();
    }
}
