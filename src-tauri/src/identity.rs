use base64::Engine;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use keyring::Entry;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeIdentity {
    pub node_id: String,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdentityStatus {
    pub initialized: bool,
    pub node_id: Option<String>,
    pub public_key: Option<String>,
    pub storage_backend: String,
}

pub const SERVICE_NAME: &str = "com.daarion.edge.identity";
pub const KEY_NAME: &str = "node_private_key";
pub const IDENTITY_FILE: &str = "identity.json";

const STORAGE_BACKEND: &str = "OS Secure Storage (keyring)";
const LEGACY_KEY_FIELDS: [&str; 3] = ["private_key", "privateKey", "secret_key"];

trait IdentityKeyStore {
    fn get_private_key(&self) -> Result<Option<String>, String>;
    fn set_private_key(&self, value: &str) -> Result<(), String>;
}

struct SystemKeyStore;

impl IdentityKeyStore for SystemKeyStore {
    fn get_private_key(&self) -> Result<Option<String>, String> {
        let entry = Entry::new(SERVICE_NAME, KEY_NAME).map_err(|e| e.to_string())?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    fn set_private_key(&self, value: &str) -> Result<(), String> {
        let entry = Entry::new(SERVICE_NAME, KEY_NAME).map_err(|e| e.to_string())?;
        entry.set_password(value).map_err(|e| e.to_string())
    }
}

struct IdentityService<S: IdentityKeyStore> {
    key_store: S,
}

impl<S: IdentityKeyStore> IdentityService<S> {
    fn new(key_store: S) -> Self {
        Self { key_store }
    }

    fn load_or_create_identity_at(&self, app_dir: &Path) -> Result<NodeIdentity, String> {
        ensure_app_dir(app_dir)?;

        let identity_path = app_dir.join(IDENTITY_FILE);
        if identity_path.exists() {
            return self.load_existing_identity_at(&identity_path);
        }

        self.create_identity_at(&identity_path)
    }

    fn identity_status_at(&self, app_dir: &Path) -> Result<IdentityStatus, String> {
        let identity_path = app_dir.join(IDENTITY_FILE);
        if !identity_path.exists() {
            return Ok(IdentityStatus {
                initialized: false,
                node_id: None,
                public_key: None,
                storage_backend: STORAGE_BACKEND.to_string(),
            });
        }

        let identity = self.load_existing_identity_at(&identity_path)?;
        Ok(IdentityStatus {
            initialized: true,
            node_id: Some(identity.node_id),
            public_key: Some(identity.public_key),
            storage_backend: STORAGE_BACKEND.to_string(),
        })
    }

    fn sign_payload_at(&self, app_dir: &Path, message: &str) -> Result<String, String> {
        let signing_key = self.signing_key_at(app_dir)?;
        let sig = signing_key.sign(message.as_bytes());
        Ok(hex::encode(sig.to_bytes()))
    }

    fn create_identity_at(&self, identity_path: &Path) -> Result<NodeIdentity, String> {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key: VerifyingKey = signing_key.verifying_key();

        let identity = NodeIdentity {
            node_id: Uuid::new_v4().to_string(),
            public_key: base64::engine::general_purpose::STANDARD.encode(verifying_key.as_bytes()),
            created_at: Utc::now(),
        };

        self.key_store
            .set_private_key(&encode_secret_bytes(&signing_key.to_bytes()))?;
        save_identity_metadata(identity_path, &identity)?;

        Ok(identity)
    }

    fn load_existing_identity_at(&self, identity_path: &Path) -> Result<NodeIdentity, String> {
        let metadata = read_identity_metadata(identity_path)?;
        let identity: NodeIdentity =
            serde_json::from_value(metadata.clone()).map_err(|e| e.to_string())?;

        match self.key_store.get_private_key()? {
            Some(secret) => {
                validate_secret_matches_identity(&secret, &identity)?;
                if has_legacy_private_key(&metadata) {
                    save_identity_metadata(identity_path, &identity)?;
                }
                Ok(identity)
            }
            None => {
                if self.migrate_legacy_private_key(identity_path, &identity, &metadata)? {
                    Ok(identity)
                } else {
                    Err(
                        "Identity metadata exists but private key is missing from secure storage"
                            .to_string(),
                    )
                }
            }
        }
    }

    fn migrate_legacy_private_key(
        &self,
        identity_path: &Path,
        identity: &NodeIdentity,
        metadata: &Value,
    ) -> Result<bool, String> {
        let Some(secret) = legacy_private_key(metadata) else {
            return Ok(false);
        };

        let secret_bytes = decode_secret_bytes(secret)?;
        let canonical_secret = encode_secret_bytes(&secret_bytes);
        validate_secret_matches_identity(&canonical_secret, identity)?;

        self.key_store.set_private_key(&canonical_secret)?;
        save_identity_metadata(identity_path, identity)?;

        Ok(true)
    }

    fn signing_key_at(&self, app_dir: &Path) -> Result<SigningKey, String> {
        match self.key_store.get_private_key()? {
            Some(secret) => signing_key_from_secret(&secret),
            None => {
                let identity_path = app_dir.join(IDENTITY_FILE);
                if identity_path.exists() {
                    let _ = self.load_existing_identity_at(&identity_path)?;
                    let secret = self.key_store.get_private_key()?.ok_or_else(|| {
                        "Private key not found in secure storage after migration".to_string()
                    })?;
                    signing_key_from_secret(&secret)
                } else {
                    Err("Private key not found in secure storage".to_string())
                }
            }
        }
    }
}

fn get_app_dir(handle: &tauri::AppHandle) -> PathBuf {
    handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir")
}

fn system_identity_service() -> IdentityService<SystemKeyStore> {
    IdentityService::new(SystemKeyStore)
}

fn ensure_app_dir(app_dir: &Path) -> Result<(), String> {
    if !app_dir.exists() {
        fs::create_dir_all(app_dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn read_identity_metadata(identity_path: &Path) -> Result<Value, String> {
    let content = fs::read_to_string(identity_path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn save_identity_metadata(identity_path: &Path, identity: &NodeIdentity) -> Result<(), String> {
    let content = serde_json::to_string_pretty(identity).map_err(|e| e.to_string())?;
    fs::write(identity_path, content).map_err(|e| e.to_string())
}

fn legacy_private_key(metadata: &Value) -> Option<&str> {
    LEGACY_KEY_FIELDS
        .iter()
        .find_map(|field| metadata.get(*field)?.as_str())
}

fn has_legacy_private_key(metadata: &Value) -> bool {
    legacy_private_key(metadata).is_some()
}

fn encode_secret_bytes(secret: &[u8; 32]) -> String {
    base64::engine::general_purpose::STANDARD.encode(secret)
}

fn decode_secret_bytes(encoded: &str) -> Result<[u8; 32], String> {
    let encoded = encoded.trim();
    let mut candidates = Vec::new();

    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(encoded) {
        candidates.push(bytes);
    }
    if let Ok(bytes) = hex::decode(encoded) {
        candidates.push(bytes);
    }

    for bytes in candidates {
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            return Ok(arr);
        }
    }

    Err("Invalid Ed25519 private key length".to_string())
}

fn signing_key_from_secret(secret: &str) -> Result<SigningKey, String> {
    let secret_bytes = decode_secret_bytes(secret)?;
    Ok(SigningKey::from_bytes(&secret_bytes))
}

fn verifying_key_from_identity(identity: &NodeIdentity) -> Result<VerifyingKey, String> {
    let public_key_bytes = base64::engine::general_purpose::STANDARD
        .decode(&identity.public_key)
        .map_err(|e| e.to_string())?;
    let public_key_bytes: [u8; 32] = public_key_bytes
        .try_into()
        .map_err(|_| "Invalid Ed25519 public key length".to_string())?;
    VerifyingKey::from_bytes(&public_key_bytes).map_err(|e| e.to_string())
}

fn validate_secret_matches_identity(secret: &str, identity: &NodeIdentity) -> Result<(), String> {
    let signing_key = signing_key_from_secret(secret)?;
    let expected = verifying_key_from_identity(identity)?;
    if signing_key.verifying_key() == expected {
        Ok(())
    } else {
        Err("Identity private key does not match metadata public key".to_string())
    }
}

pub fn load_or_create_identity(handle: &tauri::AppHandle) -> Result<NodeIdentity, String> {
    let app_dir = get_app_dir(handle);
    system_identity_service().load_or_create_identity_at(&app_dir)
}

pub fn sign_payload(handle: &tauri::AppHandle, message: &str) -> Result<String, String> {
    let app_dir = get_app_dir(handle);
    system_identity_service().sign_payload_at(&app_dir, message)
}

#[tauri::command]
pub fn get_identity_status(handle: tauri::AppHandle) -> Result<IdentityStatus, String> {
    let app_dir = get_app_dir(&handle);
    system_identity_service().identity_status_at(&app_dir)
}

#[tauri::command]
pub fn initialize_identity(handle: tauri::AppHandle) -> Result<NodeIdentity, String> {
    load_or_create_identity(&handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, Verifier};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct InMemoryKeyStore {
        entries: Arc<Mutex<HashMap<String, String>>>,
    }

    impl InMemoryKeyStore {
        fn stored_secret(&self) -> Option<String> {
            self.entries.lock().unwrap().get(KEY_NAME).cloned()
        }
    }

    impl IdentityKeyStore for InMemoryKeyStore {
        fn get_private_key(&self) -> Result<Option<String>, String> {
            Ok(self.stored_secret())
        }

        fn set_private_key(&self, value: &str) -> Result<(), String> {
            self.entries
                .lock()
                .unwrap()
                .insert(KEY_NAME.to_string(), value.to_string());
            Ok(())
        }
    }

    fn test_app_dir() -> PathBuf {
        std::env::temp_dir().join(format!("daarion-identity-test-{}", Uuid::new_v4()))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    fn identity_json(path: &Path) -> Value {
        let content = fs::read_to_string(path.join(IDENTITY_FILE)).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    #[test]
    fn generate_identity_keeps_metadata_secret_free() {
        let app_dir = test_app_dir();
        let store = InMemoryKeyStore::default();
        let service = IdentityService::new(store.clone());

        let identity = service.load_or_create_identity_at(&app_dir).unwrap();
        let metadata = identity_json(&app_dir);

        assert_eq!(metadata["node_id"], identity.node_id);
        assert_eq!(metadata["public_key"], identity.public_key);
        assert!(store.stored_secret().is_some());
        assert!(legacy_private_key(&metadata).is_none());

        cleanup(&app_dir);
    }

    #[test]
    fn load_identity_fails_when_secure_key_is_missing() {
        let app_dir = test_app_dir();
        let writer_store = InMemoryKeyStore::default();
        let writer = IdentityService::new(writer_store);
        writer.load_or_create_identity_at(&app_dir).unwrap();

        let reader = IdentityService::new(InMemoryKeyStore::default());
        let err = reader.load_or_create_identity_at(&app_dir).unwrap_err();

        assert!(err.contains("private key is missing from secure storage"));
        cleanup(&app_dir);
    }

    #[test]
    fn sign_payload_returns_verifiable_signature() {
        let app_dir = test_app_dir();
        let store = InMemoryKeyStore::default();
        let service = IdentityService::new(store);
        let identity = service.load_or_create_identity_at(&app_dir).unwrap();

        let signature_hex = service.sign_payload_at(&app_dir, "node|payload").unwrap();
        let signature_bytes = hex::decode(signature_hex).unwrap();
        let signature = Signature::from_slice(&signature_bytes).unwrap();
        let verifying_key = verifying_key_from_identity(&identity).unwrap();

        verifying_key
            .verify("node|payload".as_bytes(), &signature)
            .unwrap();

        cleanup(&app_dir);
    }

    #[test]
    fn migrate_recognized_legacy_private_key_into_key_store() {
        let app_dir = test_app_dir();
        fs::create_dir_all(&app_dir).unwrap();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let identity = NodeIdentity {
            node_id: Uuid::new_v4().to_string(),
            public_key: base64::engine::general_purpose::STANDARD.encode(verifying_key.as_bytes()),
            created_at: Utc::now(),
        };
        let legacy_metadata = serde_json::json!({
            "node_id": identity.node_id,
            "public_key": identity.public_key,
            "created_at": identity.created_at,
            "private_key": encode_secret_bytes(&signing_key.to_bytes())
        });
        fs::write(
            app_dir.join(IDENTITY_FILE),
            serde_json::to_string_pretty(&legacy_metadata).unwrap(),
        )
        .unwrap();

        let store = InMemoryKeyStore::default();
        let service = IdentityService::new(store.clone());
        let migrated = service.load_or_create_identity_at(&app_dir).unwrap();
        let rewritten_metadata = identity_json(&app_dir);

        assert_eq!(migrated.node_id, identity.node_id);
        assert!(store.stored_secret().is_some());
        assert!(legacy_private_key(&rewritten_metadata).is_none());

        cleanup(&app_dir);
    }

    #[test]
    fn missing_secure_key_without_legacy_secret_fails_closed() {
        let app_dir = test_app_dir();
        fs::create_dir_all(&app_dir).unwrap();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let identity = NodeIdentity {
            node_id: Uuid::new_v4().to_string(),
            public_key: base64::engine::general_purpose::STANDARD
                .encode(signing_key.verifying_key().as_bytes()),
            created_at: Utc::now(),
        };
        save_identity_metadata(&app_dir.join(IDENTITY_FILE), &identity).unwrap();

        let store = InMemoryKeyStore::default();
        let service = IdentityService::new(store.clone());
        let err = service.load_or_create_identity_at(&app_dir).unwrap_err();

        assert!(err.contains("private key is missing from secure storage"));
        assert!(store.stored_secret().is_none());
        assert_eq!(identity_json(&app_dir)["node_id"], identity.node_id);

        cleanup(&app_dir);
    }
}
