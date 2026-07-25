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
    DeadlineExceeded,
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

    pub(crate) const fn deadline_exceeded() -> Self {
        Self::new(RuntimeStoreErrorKind::DeadlineExceeded)
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
            Some(ErrorCode::OperationInterrupted) => Self::deadline_exceeded(),
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
            | RuntimeStoreErrorKind::ResourceLimit
            | RuntimeStoreErrorKind::DeadlineExceeded => classified,
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
            RuntimeStoreErrorKind::DeadlineExceeded => StorageRuntimeState::Unavailable,
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
            RuntimeStoreErrorKind::DeadlineExceeded => StorageRuntimeErrorCode::Internal,
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
            RuntimeStoreErrorKind::DeadlineExceeded => "runtime_store_deadline_exceeded",
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContentOperationErrorCode {
    InvalidInput,
    ConversationNotFound,
    TaskNotFound,
    AuditEventNotFound,
    IdempotencyConflict,
    IdempotencyRecordInconsistent,
    CapacityExceeded,
    BusyTimeout,
    DeadlineExceeded,
    Unavailable,
    IntegrityFailure,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContentOperationError {
    pub(crate) code: ContentOperationErrorCode,
}

impl ContentOperationError {
    pub(crate) const fn new(code: ContentOperationErrorCode) -> Self {
        Self { code }
    }

    pub(crate) const fn invalid_input() -> Self {
        Self::new(ContentOperationErrorCode::InvalidInput)
    }

    pub(crate) const fn conversation_not_found() -> Self {
        Self::new(ContentOperationErrorCode::ConversationNotFound)
    }

    pub(crate) const fn task_not_found() -> Self {
        Self::new(ContentOperationErrorCode::TaskNotFound)
    }

    pub(crate) const fn audit_event_not_found() -> Self {
        Self::new(ContentOperationErrorCode::AuditEventNotFound)
    }

    pub(crate) const fn idempotency_conflict() -> Self {
        Self::new(ContentOperationErrorCode::IdempotencyConflict)
    }

    pub(crate) const fn idempotency_inconsistent() -> Self {
        Self::new(ContentOperationErrorCode::IdempotencyRecordInconsistent)
    }

    pub(crate) const fn capacity_exceeded() -> Self {
        Self::new(ContentOperationErrorCode::CapacityExceeded)
    }

    pub(crate) const fn deadline_exceeded() -> Self {
        Self::new(ContentOperationErrorCode::DeadlineExceeded)
    }

    pub(crate) const fn unavailable() -> Self {
        Self::new(ContentOperationErrorCode::Unavailable)
    }

    pub(crate) const fn integrity_failure() -> Self {
        Self::new(ContentOperationErrorCode::IntegrityFailure)
    }

    pub(crate) const fn internal() -> Self {
        Self::new(ContentOperationErrorCode::Internal)
    }

    pub(crate) fn from_runtime(error: RuntimeStoreError) -> Self {
        let code = match error.kind {
            RuntimeStoreErrorKind::BusyTimeout | RuntimeStoreErrorKind::Locked => {
                ContentOperationErrorCode::BusyTimeout
            }
            RuntimeStoreErrorKind::DeadlineExceeded => ContentOperationErrorCode::DeadlineExceeded,
            RuntimeStoreErrorKind::IntegrityFailed
            | RuntimeStoreErrorKind::MigrationMismatch
            | RuntimeStoreErrorKind::NewerSchema => ContentOperationErrorCode::IntegrityFailure,
            RuntimeStoreErrorKind::ResourceLimit => ContentOperationErrorCode::CapacityExceeded,
            RuntimeStoreErrorKind::PathInvalid
            | RuntimeStoreErrorKind::PermissionDenied
            | RuntimeStoreErrorKind::Unavailable => ContentOperationErrorCode::Unavailable,
            RuntimeStoreErrorKind::MigrationFailed | RuntimeStoreErrorKind::Internal => {
                ContentOperationErrorCode::Internal
            }
        };
        Self::new(code)
    }

    pub(crate) fn from_sqlite(error: &rusqlite::Error) -> Self {
        match error.sqlite_error_code() {
            Some(ErrorCode::DatabaseBusy) | Some(ErrorCode::DatabaseLocked) => {
                Self::new(ContentOperationErrorCode::BusyTimeout)
            }
            Some(ErrorCode::OperationInterrupted) => Self::deadline_exceeded(),
            Some(ErrorCode::DatabaseCorrupt) | Some(ErrorCode::NotADatabase) => {
                Self::integrity_failure()
            }
            Some(ErrorCode::DiskFull) | Some(ErrorCode::TooBig) => Self::capacity_exceeded(),
            Some(ErrorCode::ReadOnly) | Some(ErrorCode::PermissionDenied) => Self::unavailable(),
            _ => Self::internal(),
        }
    }

    pub(crate) const fn poisons_content_intake(self) -> bool {
        matches!(
            self.code,
            ContentOperationErrorCode::IdempotencyRecordInconsistent
                | ContentOperationErrorCode::IntegrityFailure
        )
    }

    const fn safe_code(self) -> &'static str {
        match self.code {
            ContentOperationErrorCode::InvalidInput => "content_invalid_input",
            ContentOperationErrorCode::ConversationNotFound => "content_conversation_not_found",
            ContentOperationErrorCode::TaskNotFound => "content_task_not_found",
            ContentOperationErrorCode::AuditEventNotFound => "content_audit_event_not_found",
            ContentOperationErrorCode::IdempotencyConflict => "content_idempotency_conflict",
            ContentOperationErrorCode::IdempotencyRecordInconsistent => {
                "content_idempotency_record_inconsistent"
            }
            ContentOperationErrorCode::CapacityExceeded => "content_capacity_exceeded",
            ContentOperationErrorCode::BusyTimeout => "content_busy_timeout",
            ContentOperationErrorCode::DeadlineExceeded => "content_deadline_exceeded",
            ContentOperationErrorCode::Unavailable => "content_unavailable",
            ContentOperationErrorCode::IntegrityFailure => "content_integrity_failure",
            ContentOperationErrorCode::Internal => "content_internal",
        }
    }
}

impl fmt::Display for ContentOperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.safe_code())
    }
}

impl std::error::Error for ContentOperationError {}
