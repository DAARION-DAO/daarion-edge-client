use crate::runtime_store::config::RuntimeStoreConfig;
use crate::runtime_store::error::RuntimeStoreError;
use crate::runtime_store::path_policy::{
    database_total_size, enforce_sidecar_permissions, prepare_storage_paths, revalidate_database,
    validate_database_after_open, PreparedStoragePaths,
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
}

impl RuntimeStoreConnection {
    pub(crate) fn open(config: &RuntimeStoreConfig) -> Result<Self, RuntimeStoreError> {
        let mut paths = prepare_storage_paths(&config.app_local_data_root)?;
        if database_total_size(&paths.database_path)? > config.database_hard_limit_bytes {
            return Err(RuntimeStoreError::resource_limit());
        }

        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX
            | OpenFlags::SQLITE_OPEN_NOFOLLOW
            | OpenFlags::SQLITE_OPEN_EXRESCODE;
        let connection = Connection::open_with_flags(&paths.database_path, flags)
            .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
        validate_database_after_open(&mut paths)?;
        configure_limits(&connection)?;
        configure_pragmas(&connection, config)?;
        validate_pragmas(&connection, config)?;
        enforce_sidecar_permissions(&paths.database_path)?;

        let persistence_state = if paths.existed_before_open {
            PersistenceState::ReopenedExisting
        } else {
            PersistenceState::CreatedNew
        };
        Ok(Self {
            connection,
            paths,
            persistence_state,
            database_warning_threshold_bytes: config.database_warning_threshold_bytes,
            database_hard_limit_bytes: config.database_hard_limit_bytes,
        })
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

    pub(crate) fn close(self) -> Result<(), RuntimeStoreError> {
        self.revalidate_artifacts()?;
        let busy: i64 = self
            .connection
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))
            .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
        if busy != 0 {
            return Err(RuntimeStoreError::new(
                crate::runtime_store::error::RuntimeStoreErrorKind::BusyTimeout,
            ));
        }
        enforce_sidecar_permissions(&self.paths.database_path)?;
        revalidate_database(&self.paths)?;
        self.connection
            .close()
            .map_err(|(_, error)| RuntimeStoreError::from_sqlite(&error))
    }
}

fn configure_limits(connection: &Connection) -> Result<(), RuntimeStoreError> {
    let limits = [
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
    for (limit, value) in limits {
        connection
            .set_limit(limit, value)
            .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
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
             PRAGMA temp_store=MEMORY;",
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
    let expected_busy_timeout = i64::try_from(config.busy_timeout.as_millis())
        .map_err(|_| RuntimeStoreError::internal())?;
    if foreign_keys != 1
        || journal_mode != "wal"
        || synchronous != 2
        || secure_delete != 1
        || trusted_schema != 0
        || temp_store != 2
        || busy_timeout_ms != expected_busy_timeout
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
