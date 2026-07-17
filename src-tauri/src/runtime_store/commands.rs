use crate::runtime_store::types::StorageRuntimeStatus;
use crate::runtime_store::RuntimeStoreManager;
use tauri::Manager;

#[tauri::command]
pub(crate) async fn get_storage_runtime_status(app: tauri::AppHandle) -> StorageRuntimeStatus {
    let manager = app.state::<RuntimeStoreManager>().inner().clone();
    let fallback = manager.internal_failure_status();
    tauri::async_runtime::spawn_blocking(move || manager.read_status())
        .await
        .unwrap_or(fallback)
}
