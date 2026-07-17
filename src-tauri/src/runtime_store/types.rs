use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StorageRuntimeState {
    Initializing,
    Healthy,
    Warning,
    Unavailable,
    MigrationFailed,
    IntegrityFailed,
    Locked,
    PermissionDenied,
    ResourceLimited,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DatabaseHealth {
    Ok,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PersistenceState {
    CreatedNew,
    ReopenedExisting,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StorageRuntimeErrorCode {
    PathInvalid,
    PermissionDenied,
    Locked,
    BusyTimeout,
    MigrationMismatch,
    NewerSchema,
    MigrationFailed,
    IntegrityFailed,
    ResourceLimit,
    Internal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct StorageRuntimeStatus {
    pub(crate) state: StorageRuntimeState,
    pub(crate) initialized: bool,
    pub(crate) schema_version: Option<u32>,
    pub(crate) database_health: DatabaseHealth,
    pub(crate) database_size_bytes: Option<u64>,
    pub(crate) storage_backend: String,
    pub(crate) sqlite_version: Option<String>,
    pub(crate) persistence_state: PersistenceState,
    pub(crate) last_start_time_ms: u64,
    pub(crate) error_code: Option<StorageRuntimeErrorCode>,
}

impl StorageRuntimeStatus {
    pub(crate) fn initializing(last_start_time_ms: u64) -> Self {
        Self {
            state: StorageRuntimeState::Initializing,
            initialized: false,
            schema_version: None,
            database_health: DatabaseHealth::Warning,
            database_size_bytes: None,
            storage_backend: "sqlite".to_string(),
            sqlite_version: None,
            persistence_state: PersistenceState::Unknown,
            last_start_time_ms,
            error_code: None,
        }
    }

    pub(crate) fn healthy(
        last_start_time_ms: u64,
        schema_version: u32,
        sqlite_version: String,
        database_size_bytes: u64,
        persistence_state: PersistenceState,
        warning_threshold_bytes: u64,
        hard_limit_bytes: u64,
    ) -> Self {
        let (state, database_health, error_code) = if database_size_bytes > hard_limit_bytes {
            (
                StorageRuntimeState::ResourceLimited,
                DatabaseHealth::Warning,
                Some(StorageRuntimeErrorCode::ResourceLimit),
            )
        } else if database_size_bytes > warning_threshold_bytes {
            (StorageRuntimeState::Warning, DatabaseHealth::Warning, None)
        } else {
            (StorageRuntimeState::Healthy, DatabaseHealth::Ok, None)
        };

        Self {
            state,
            initialized: true,
            schema_version: Some(schema_version),
            database_health,
            database_size_bytes: Some(database_size_bytes),
            storage_backend: "sqlite".to_string(),
            sqlite_version: Some(sqlite_version),
            persistence_state,
            last_start_time_ms,
            error_code,
        }
    }

    pub(crate) fn failed(
        last_start_time_ms: u64,
        state: StorageRuntimeState,
        error_code: StorageRuntimeErrorCode,
    ) -> Self {
        Self {
            state,
            initialized: false,
            schema_version: None,
            database_health: DatabaseHealth::Error,
            database_size_bytes: None,
            storage_backend: "sqlite".to_string(),
            sqlite_version: None,
            persistence_state: PersistenceState::Unknown,
            last_start_time_ms,
            error_code: Some(error_code),
        }
    }
}
