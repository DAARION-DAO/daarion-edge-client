use serde::{Deserialize, Serialize};
use std::fs;
use keyring::Entry;
use tauri::Manager;

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetResult {
    pub success: bool,
    pub deleted_files: Vec<String>,
    pub deleted_keyring_entries: Vec<String>,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub async fn factory_reset_local_state(handle: tauri::AppHandle) -> Result<ResetResult, String> {
    let mut result = ResetResult {
        success: true,
        deleted_files: vec![],
        deleted_keyring_entries: vec![],
        warnings: vec![],
    };

    let app_dir = handle.path().app_data_dir().expect("Failed to get app data dir");

    // 1. Delete app data files
    let identity_path = app_dir.join("identity.json");
    if identity_path.exists() {
        match fs::remove_file(&identity_path) {
            Ok(_) => result.deleted_files.push("identity.json".to_string()),
            Err(e) => {
                result.success = false;
                result.warnings.push(format!("Failed to delete identity.json: {}", e));
            }
        }
    }

    let enrollment_path = app_dir.join("enrollment.json");
    if enrollment_path.exists() {
        match fs::remove_file(&enrollment_path) {
            Ok(_) => result.deleted_files.push("enrollment.json".to_string()),
            Err(e) => {
                result.success = false;
                result.warnings.push(format!("Failed to delete enrollment.json: {}", e));
            }
        }
    }

    // Best-effort cleanup of voice signatures
    if let Ok(entries) = fs::read_dir(&app_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with("voice_signature_") && file_name.ends_with(".wav") {
                if fs::remove_file(entry.path()).is_ok() {
                    result.deleted_files.push(file_name);
                }
            }
        }
    }

    // 2. Delete Keyring info
    if let Ok(entry) = Entry::new("com.daarion.edge.identity", "node_private_key") {
        if entry.get_password().is_ok() {
            match entry.delete_credential() {
                Ok(_) => result.deleted_keyring_entries.push("com.daarion.edge.identity / node_private_key".to_string()),
                Err(e) => {
                    result.success = false;
                    result.warnings.push(format!("Failed to delete node_private_key from keyring: {}", e));
                }
            }
        }
    }

    if let Ok(entry) = Entry::new("com.daarion.edge.node", "node_token") {
        if entry.get_password().is_ok() {
            match entry.delete_credential() {
                Ok(_) => result.deleted_keyring_entries.push("com.daarion.edge.node / node_token".to_string()),
                Err(e) => {
                    result.success = false;
                    result.warnings.push(format!("Failed to delete node_token from keyring: {}", e));
                }
            }
        }
    }

    Ok(result)
}
