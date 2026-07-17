use crate::runtime_store::types::{StorageRuntimeErrorCode, StorageRuntimeState};
use rusqlite::ErrorCode;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RuntimeStoreErrorKind {
    PathInvalid,
    PermissionDenied,
    Locked,
    BusyTimeout,
    MigrationMismatch,
    NewerSchema,
    MigrationFailed,
    IntegrityFailed,
    ResourceLimit,
    Unavailable,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeStoreError {
    pub(crate) kind: RuntimeStoreErrorKind,
}

impl RuntimeStoreError {
    pub(crate) const fn new(kind: RuntimeStoreErrorKind) -> Self {
        Self { kind }
    }

    pub(crate) const fn path_invalid() -> Self {
        Self::new(RuntimeStoreErrorKind::PathInvalid)
    }

    pub(crate) const fn permission_denied() -> Self {
        Self::new(RuntimeStoreErrorKind::PermissionDenied)
    }

    pub(crate) const fn migration_mismatch() -> Self {
        Self::new(RuntimeStoreErrorKind::MigrationMismatch)
    }

    pub(crate) const fn newer_schema() -> Self {
        Self::new(RuntimeStoreErrorKind::NewerSchema)
    }

    pub(crate) const fn migration_failed() -> Self {
        Self::new(RuntimeStoreErrorKind::MigrationFailed)
    }

    pub(crate) const fn integrity_failed() -> Self {
        Self::new(RuntimeStoreErrorKind::IntegrityFailed)
    }

    pub(crate) const fn resource_limit() -> Self {
        Self::new(RuntimeStoreErrorKind::ResourceLimit)
    }

    pub(crate) const fn unavailable() -> Self {
        Self::new(RuntimeStoreErrorKind::Unavailable)
    }

    pub(crate) const fn internal() -> Self {
        Self::new(RuntimeStoreErrorKind::Internal)
    }

    pub(crate) fn from_io(error: &std::io::Error) -> Self {
        if error.kind() == std::io::ErrorKind::PermissionDenied {
            Self::permission_denied()
        } else {
            Self::path_invalid()
        }
    }

    pub(crate) fn from_sqlite(error: &rusqlite::Error) -> Self {
        match error.sqlite_error_code() {
            Some(ErrorCode::DatabaseBusy) => Self::new(RuntimeStoreErrorKind::BusyTimeout),
            Some(ErrorCode::DatabaseLocked) => Self::new(RuntimeStoreErrorKind::Locked),
            Some(ErrorCode::DatabaseCorrupt) | Some(ErrorCode::NotADatabase) => {
                Self::integrity_failed()
            }
            Some(ErrorCode::DiskFull) | Some(ErrorCode::TooBig) => Self::resource_limit(),
            Some(ErrorCode::ReadOnly) | Some(ErrorCode::PermissionDenied) => {
                Self::permission_denied()
            }
            _ => Self::internal(),
        }
    }

    pub(crate) fn from_migration_sqlite(error: &rusqlite::Error) -> Self {
        let classified = Self::from_sqlite(error);
        match classified.kind {
            RuntimeStoreErrorKind::Locked
            | RuntimeStoreErrorKind::BusyTimeout
            | RuntimeStoreErrorKind::PermissionDenied
            | RuntimeStoreErrorKind::IntegrityFailed
            | RuntimeStoreErrorKind::ResourceLimit => classified,
            _ => Self::migration_failed(),
        }
    }

    pub(crate) const fn public_state(self) -> StorageRuntimeState {
        match self.kind {
            RuntimeStoreErrorKind::PermissionDenied => StorageRuntimeState::PermissionDenied,
            RuntimeStoreErrorKind::Locked | RuntimeStoreErrorKind::BusyTimeout => {
                StorageRuntimeState::Locked
            }
            RuntimeStoreErrorKind::MigrationMismatch
            | RuntimeStoreErrorKind::NewerSchema
            | RuntimeStoreErrorKind::MigrationFailed => StorageRuntimeState::MigrationFailed,
            RuntimeStoreErrorKind::IntegrityFailed => StorageRuntimeState::IntegrityFailed,
            RuntimeStoreErrorKind::ResourceLimit => StorageRuntimeState::ResourceLimited,
            RuntimeStoreErrorKind::PathInvalid
            | RuntimeStoreErrorKind::Unavailable
            | RuntimeStoreErrorKind::Internal => StorageRuntimeState::Unavailable,
        }
    }

    pub(crate) const fn public_code(self) -> StorageRuntimeErrorCode {
        match self.kind {
            RuntimeStoreErrorKind::PathInvalid => StorageRuntimeErrorCode::PathInvalid,
            RuntimeStoreErrorKind::PermissionDenied => StorageRuntimeErrorCode::PermissionDenied,
            RuntimeStoreErrorKind::Locked => StorageRuntimeErrorCode::Locked,
            RuntimeStoreErrorKind::BusyTimeout => StorageRuntimeErrorCode::BusyTimeout,
            RuntimeStoreErrorKind::MigrationMismatch => StorageRuntimeErrorCode::MigrationMismatch,
            RuntimeStoreErrorKind::NewerSchema => StorageRuntimeErrorCode::NewerSchema,
            RuntimeStoreErrorKind::MigrationFailed => StorageRuntimeErrorCode::MigrationFailed,
            RuntimeStoreErrorKind::IntegrityFailed => StorageRuntimeErrorCode::IntegrityFailed,
            RuntimeStoreErrorKind::ResourceLimit => StorageRuntimeErrorCode::ResourceLimit,
            RuntimeStoreErrorKind::Unavailable | RuntimeStoreErrorKind::Internal => {
                StorageRuntimeErrorCode::Internal
            }
        }
    }

    const fn safe_code(self) -> &'static str {
        match self.kind {
            RuntimeStoreErrorKind::PathInvalid => "runtime_store_path_invalid",
            RuntimeStoreErrorKind::PermissionDenied => "runtime_store_permission_denied",
            RuntimeStoreErrorKind::Locked => "runtime_store_locked",
            RuntimeStoreErrorKind::BusyTimeout => "runtime_store_busy_timeout",
            RuntimeStoreErrorKind::MigrationMismatch => "runtime_store_migration_mismatch",
            RuntimeStoreErrorKind::NewerSchema => "runtime_store_newer_schema",
            RuntimeStoreErrorKind::MigrationFailed => "runtime_store_migration_failed",
            RuntimeStoreErrorKind::IntegrityFailed => "runtime_store_integrity_failed",
            RuntimeStoreErrorKind::ResourceLimit => "runtime_store_resource_limit",
            RuntimeStoreErrorKind::Unavailable => "runtime_store_unavailable",
            RuntimeStoreErrorKind::Internal => "runtime_store_internal",
        }
    }
}

impl fmt::Display for RuntimeStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.safe_code())
    }
}

impl std::error::Error for RuntimeStoreError {}
