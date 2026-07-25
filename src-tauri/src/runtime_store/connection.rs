use crate::runtime_store::config::{ContentStorageLimits, RuntimeStoreConfig};
use crate::runtime_store::control::InitializationAttempt;
use crate::runtime_store::deadline::{ensure_before, remaining, SqliteInterruptGuard};
use crate::runtime_store::error::RuntimeStoreError;
use crate::runtime_store::path_policy::{
    capture_sidecar_identities, database_total_size, enforce_sidecar_permissions,
    prepare_storage_paths_until, revalidate_database, validate_database_after_open,
    PreparedStoragePaths,
};
use crate::runtime_store::types::PersistenceState;
use rusqlite::limits::Limit;
use rusqlite::{Connection, OpenFlags};

pub(crate) struct RuntimeStoreConnection {
    pub(crate) connection: Connection,
    pub(crate) paths: PreparedStoragePaths,
    pub(crate) persistence_state: PersistenceState,
    pub(crate) database_warning_threshold_bytes: u64,
    pub(crate) database_hard_limit_bytes: u64,
    pub(crate) content_limits: ContentStorageLimits,
    pub(crate) content_integrity_failed: bool,
    pub(crate) busy_timeout: std::time::Duration,
}

impl RuntimeStoreConnection {
    #[cfg(test)]
    pub(crate) fn open(config: &RuntimeStoreConfig) -> Result<Self, RuntimeStoreError> {
        let deadline = std::time::Instant::now() + config.migration_deadline;
        let control =
            std::sync::Arc::new(crate::runtime_store::control::RuntimeStoreControl::new());
        let attempt = control.begin_initialization()?;
        let (opened, watchdog) = Self::open_for_initialization(config, deadline, &attempt)?;
        if watchdog.finish()? || std::time::Instant::now() >= deadline {
            return Err(RuntimeStoreError::deadline_exceeded());
        }
        Ok(opened)
    }

    pub(crate) fn open_for_initialization(
        config: &RuntimeStoreConfig,
        deadline: std::time::Instant,
        attempt: &InitializationAttempt,
    ) -> Result<(Self, SqliteInterruptGuard), RuntimeStoreError> {
        ensure_initialization_running(deadline, attempt)?;
        #[cfg(test)]
        config
            .initialization_test_hook
            .wait_at(crate::runtime_store::config::InitializationTestStage::BeforePathPreparation)
            .map_err(|_| RuntimeStoreError::internal())?;
        ensure_initialization_running(deadline, attempt)?;
        let mut paths = prepare_storage_paths_until(&config.app_local_data_root, deadline)?;
        ensure_initialization_running(deadline, attempt)?;
        if database_total_size(&paths.database_path)? > config.database_hard_limit_bytes {
            return Err(RuntimeStoreError::resource_limit());
        }
        ensure_initialization_running(deadline, attempt)?;

        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX
            | OpenFlags::SQLITE_OPEN_NOFOLLOW
            | OpenFlags::SQLITE_OPEN_EXRESCODE;
        let connection = Connection::open_with_flags(&paths.database_path, flags)
            .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
        #[cfg(test)]
        config
            .initialization_test_hook
            .wait_at(
                crate::runtime_store::config::InitializationTestStage::BeforeInterruptRegistration,
            )
            .map_err(|_| RuntimeStoreError::internal())?;
        ensure_initialization_running(deadline, attempt)?;
        let watchdog = SqliteInterruptGuard::start_initialization(&connection, deadline, attempt)?;
        #[cfg(test)]
        config
            .initialization_test_hook
            .wait_at(
                crate::runtime_store::config::InitializationTestStage::AfterInterruptRegistration,
            )
            .map_err(|_| RuntimeStoreError::internal())?;
        ensure_initialization_running(deadline, attempt)?;
        validate_database_after_open(&mut paths)?;
        ensure_initialization_running(deadline, attempt)?;
        configure_limits(&connection)?;
        ensure_initialization_running(deadline, attempt)?;
        configure_pragmas(&connection, config)?;
        ensure_initialization_running(deadline, attempt)?;
        validate_pragmas(&connection, config)?;
        ensure_initialization_running(deadline, attempt)?;
        enforce_sidecar_permissions(&paths.database_path)?;
        capture_sidecar_identities(&mut paths)?;
        ensure_initialization_running(deadline, attempt)?;

        let persistence_state = if paths.existed_before_open {
            PersistenceState::ReopenedExisting
        } else {
            PersistenceState::CreatedNew
        };
        Ok((
            Self {
                connection,
                paths,
                persistence_state,
                database_warning_threshold_bytes: config.database_warning_threshold_bytes,
                database_hard_limit_bytes: config.database_hard_limit_bytes,
                content_limits: config.content_limits,
                content_integrity_failed: false,
                busy_timeout: config.busy_timeout,
            },
            watchdog,
        ))
    }

    pub(crate) fn database_size_bytes(&self) -> Result<u64, RuntimeStoreError> {
        self.revalidate_artifacts()?;
        let size = database_total_size(&self.paths.database_path)?;
        self.revalidate_artifacts()?;
        Ok(size)
    }

    pub(crate) fn revalidate_artifacts(&self) -> Result<(), RuntimeStoreError> {
        revalidate_database(&self.paths)?;
        enforce_sidecar_permissions(&self.paths.database_path)
    }

    #[cfg(test)]
    pub(crate) fn close(self) -> Result<(), RuntimeStoreError> {
        self.close_until(std::time::Instant::now() + std::time::Duration::from_secs(2))
    }

    pub(crate) fn close_until(self, deadline: std::time::Instant) -> Result<(), RuntimeStoreError> {
        ensure_before(deadline)?;
        self.revalidate_artifacts()?;
        let checkpoint_budget = remaining(deadline)?;
        self.connection
            .busy_timeout(checkpoint_budget.max(std::time::Duration::from_millis(1)))
            .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
        let watchdog = SqliteInterruptGuard::start(&self.connection, deadline)?;
        let checkpoint: Result<i64, RuntimeStoreError> = self
            .connection
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))
            .map_err(|error| RuntimeStoreError::from_sqlite(&error));
        let checkpoint_expired = watchdog.finish()?;
        let busy = match checkpoint {
            Ok(busy) => busy,
            Err(error)
                if checkpoint_expired
                    || error.kind
                        == crate::runtime_store::error::RuntimeStoreErrorKind::DeadlineExceeded =>
            {
                return Err(RuntimeStoreError::new(
                    crate::runtime_store::error::RuntimeStoreErrorKind::BusyTimeout,
                ));
            }
            Err(error) => return Err(error),
        };
        if checkpoint_expired || busy != 0 {
            return Err(RuntimeStoreError::new(
                crate::runtime_store::error::RuntimeStoreErrorKind::BusyTimeout,
            ));
        }
        ensure_before(deadline)?;
        enforce_sidecar_permissions(&self.paths.database_path)?;
        revalidate_database(&self.paths)?;
        ensure_before(deadline)?;
        self.connection
            .close()
            .map_err(|(_, error)| RuntimeStoreError::from_sqlite(&error))
    }
}

fn ensure_initialization_running(
    deadline: std::time::Instant,
    attempt: &InitializationAttempt,
) -> Result<(), RuntimeStoreError> {
    ensure_before(deadline)?;
    attempt.ensure_running()
}

pub(crate) const REQUIRED_LIMITS: [(Limit, i32); 11] = [
    (Limit::SQLITE_LIMIT_LENGTH, 512 * 1024),
    (Limit::SQLITE_LIMIT_SQL_LENGTH, 1024 * 1024),
    (Limit::SQLITE_LIMIT_COLUMN, 64),
    (Limit::SQLITE_LIMIT_EXPR_DEPTH, 128),
    (Limit::SQLITE_LIMIT_COMPOUND_SELECT, 16),
    (Limit::SQLITE_LIMIT_FUNCTION_ARG, 32),
    (Limit::SQLITE_LIMIT_ATTACHED, 0),
    (Limit::SQLITE_LIMIT_LIKE_PATTERN_LENGTH, 4096),
    (Limit::SQLITE_LIMIT_VARIABLE_NUMBER, 128),
    (Limit::SQLITE_LIMIT_TRIGGER_DEPTH, 8),
    (Limit::SQLITE_LIMIT_WORKER_THREADS, 0),
];

fn configure_limits(connection: &Connection) -> Result<(), RuntimeStoreError> {
    for (limit, value) in REQUIRED_LIMITS {
        connection
            .set_limit(limit, value)
            .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
        let effective = connection
            .limit(limit)
            .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
        if effective != value {
            return Err(RuntimeStoreError::resource_limit());
        }
    }
    Ok(())
}

fn configure_pragmas(
    connection: &Connection,
    config: &RuntimeStoreConfig,
) -> Result<(), RuntimeStoreError> {
    connection
        .busy_timeout(config.busy_timeout)
        .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
    connection
        .execute_batch(
            "PRAGMA foreign_keys=ON;
             PRAGMA journal_mode=WAL;
             PRAGMA synchronous=FULL;
             PRAGMA secure_delete=ON;
             PRAGMA trusted_schema=OFF;
             PRAGMA temp_store=MEMORY;
             PRAGMA wal_autocheckpoint=128;
             BEGIN IMMEDIATE;
             ROLLBACK;",
        )
        .map_err(|error| RuntimeStoreError::from_sqlite(&error))
}

fn validate_pragmas(
    connection: &Connection,
    config: &RuntimeStoreConfig,
) -> Result<(), RuntimeStoreError> {
    let foreign_keys: i64 = pragma_value(connection, "foreign_keys")?;
    let journal_mode: String = pragma_value(connection, "journal_mode")?;
    let synchronous: i64 = pragma_value(connection, "synchronous")?;
    let secure_delete: i64 = pragma_value(connection, "secure_delete")?;
    let trusted_schema: i64 = pragma_value(connection, "trusted_schema")?;
    let temp_store: i64 = pragma_value(connection, "temp_store")?;
    let busy_timeout_ms: i64 = pragma_value(connection, "busy_timeout")?;
    let page_size: i64 = pragma_value(connection, "page_size")?;
    let wal_autocheckpoint: i64 = pragma_value(connection, "wal_autocheckpoint")?;
    let expected_busy_timeout = i64::try_from(config.busy_timeout.as_millis())
        .map_err(|_| RuntimeStoreError::internal())?;
    let expected_page_size = i64::from(config.content_limits.required_page_size_bytes);
    let expected_wal_autocheckpoint = i64::from(config.content_limits.wal_autocheckpoint_pages);
    if foreign_keys != 1
        || journal_mode != "wal"
        || synchronous != 2
        || secure_delete != 1
        || trusted_schema != 0
        || temp_store != 2
        || busy_timeout_ms != expected_busy_timeout
        || page_size != expected_page_size
        || wal_autocheckpoint != expected_wal_autocheckpoint
    {
        return Err(RuntimeStoreError::internal());
    }
    Ok(())
}

fn pragma_value<T: rusqlite::types::FromSql>(
    connection: &Connection,
    name: &str,
) -> Result<T, RuntimeStoreError> {
    connection
        .pragma_query_value(None, name, |row| row.get(0))
        .map_err(|error| RuntimeStoreError::from_sqlite(&error))
}
