use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rand::rngs::OsRng;
use ed25519_dalek::{SigningKey, VerifyingKey, SecretKey};
use keyring::Entry;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use tauri::Manager;
use base64::Engine;

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

const SERVICE_NAME: &str = "com.daarion.edge.identity";
const KEY_NAME: &str = "node_private_key";
const IDENTITY_FILE: &str = "identity.json";

fn get_app_dir(handle: &tauri::AppHandle) -> PathBuf {
    handle.path().app_data_dir().expect("Failed to get app data dir")
}

pub fn load_or_create_identity(handle: &tauri::AppHandle) -> Result<NodeIdentity, String> {
    let app_dir = get_app_dir(handle);
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    }

    let identity_path = app_dir.join(IDENTITY_FILE);
    
    if identity_path.exists() {
        // Try to load
        let content = fs::read_to_string(&identity_path).map_err(|e| e.to_string())?;
        let identity: NodeIdentity = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        
        // Verify key exists in keyring
        let entry = Entry::new(SERVICE_NAME, KEY_NAME).map_err(|e| e.to_string())?;
        match entry.get_password() {
            Ok(_) => Ok(identity),
            Err(_) => Err("Identity metadata exists but private key is missing from secure storage".to_string()),
        }
    } else {
        // Create new
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        
        let node_id = Uuid::new_v4().to_string();
        let pub_key_base64 = base64::engine::general_purpose::STANDARD.encode(verifying_key.as_bytes());
        
        let identity = NodeIdentity {
            node_id,
            public_key: pub_key_base64,
            created_at: Utc::now(),
        };

        // Save private key to keyring
        let entry = Entry::new(SERVICE_NAME, KEY_NAME).map_err(|e| e.to_string())?;
        entry.set_password(&base64::engine::general_purpose::STANDARD.encode(signing_key.to_bytes())).map_err(|e| e.to_string())?;

        // Save metadata
        let content = serde_json::to_string_pretty(&identity).map_err(|e| e.to_string())?;
        fs::write(identity_path, content).map_err(|e| e.to_string())?;

        Ok(identity)
    }
}

#[tauri::command]
pub fn get_identity_status(handle: tauri::AppHandle) -> Result<IdentityStatus, String> {
    let app_dir = get_app_dir(&handle);
    let identity_path = app_dir.join(IDENTITY_FILE);

    if identity_path.exists() {
        let content = fs::read_to_string(&identity_path).map_err(|e| e.to_string())?;
        let identity: NodeIdentity = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        
        Ok(IdentityStatus {
            initialized: true,
            node_id: Some(identity.node_id),
            public_key: Some(identity.public_key),
            storage_backend: "OS Secure Storage (keyring)".to_string(),
        })
    } else {
        Ok(IdentityStatus {
            initialized: false,
            node_id: None,
            public_key: None,
            storage_backend: "OS Secure Storage (keyring)".to_string(),
        })
    }
}

#[tauri::command]
pub fn initialize_identity(handle: tauri::AppHandle) -> Result<NodeIdentity, String> {
    load_or_create_identity(&handle)
}
