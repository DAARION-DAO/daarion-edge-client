use crate::runtime_store::deadline::ensure_before;
use crate::runtime_store::error::RuntimeStoreError;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[cfg(test)]
use std::time::Duration;

pub(crate) const CURRENT_SCHEMA_VERSION: u32 = 1;
const INITIAL_MIGRATION_NAME: &str = "runtime_state_initial";
const INITIAL_MIGRATION_SQL: &str =
    include_str!("../../migrations/runtime_state/0001_runtime_state_initial.sql");
pub(crate) const INITIAL_MIGRATION_CHECKSUM: &str =
    "62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d";
pub(crate) const EXPECTED_SCHEMA_FINGERPRINT: &str =
    "37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77";

#[cfg(test)]
pub(crate) fn migrate_and_validate(connection: &mut Connection) -> Result<u32, RuntimeStoreError> {
    migrate_and_validate_until(connection, Instant::now() + Duration::from_secs(120))
}

pub(crate) fn migrate_and_validate_until(
    connection: &mut Connection,
    deadline: Instant,
) -> Result<u32, RuntimeStoreError> {
    migrate_and_validate_inner(connection, deadline, false, false)
}

#[cfg(test)]
pub(crate) fn migrate_and_validate_with_test_interrupt(
    connection: &mut Connection,
    deadline: Instant,
    interrupt_inside_initial_migration: bool,
    interrupt_during_integrity: bool,
) -> Result<u32, RuntimeStoreError> {
    migrate_and_validate_inner(
        connection,
        deadline,
        interrupt_inside_initial_migration,
        interrupt_during_integrity,
    )
}

fn migrate_and_validate_inner(
    connection: &mut Connection,
    deadline: Instant,
    interrupt_inside_initial_migration: bool,
    interrupt_during_integrity: bool,
) -> Result<u32, RuntimeStoreError> {
    ensure_before(deadline)?;
    verify_embedded_checksum()?;
    ensure_before(deadline)?;
    let has_history = object_exists(connection, "table", "schema_migrations")?;
    if !has_history {
        apply_initial_migration(connection, deadline, interrupt_inside_initial_migration)?;
    }
    ensure_before(deadline)?;
    validate_migration_history(connection)?;
    ensure_before(deadline)?;
    validate_schema(connection)?;
    ensure_before(deadline)?;
    #[cfg(test)]
    if interrupt_during_integrity {
        run_long_query(connection)?;
    }
    #[cfg(not(test))]
    let _ = interrupt_during_integrity;
    ensure_before(deadline)?;
    validate_integrity(connection)?;
    ensure_before(deadline)?;
    Ok(CURRENT_SCHEMA_VERSION)
}

fn apply_initial_migration(
    connection: &mut Connection,
    deadline: Instant,
    interrupt_inside_initial_migration: bool,
) -> Result<(), RuntimeStoreError> {
    ensure_before(deadline)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| RuntimeStoreError::from_migration_sqlite(&error))?;
    transaction
        .execute_batch(INITIAL_MIGRATION_SQL)
        .map_err(|error| RuntimeStoreError::from_migration_sqlite(&error))?;
    ensure_before(deadline)?;
    #[cfg(test)]
    if interrupt_inside_initial_migration {
        run_long_query(&transaction)?;
    }
    #[cfg(not(test))]
    let _ = interrupt_inside_initial_migration;
    ensure_before(deadline)?;
    let applied_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RuntimeStoreError::internal())?
        .as_millis();
    let applied_at_ms = i64::try_from(applied_at_ms).map_err(|_| RuntimeStoreError::internal())?;
    transaction
        .execute(
            "INSERT INTO schema_migrations (
                 migration_id, name, checksum_sha256, applied_at_ms
             ) VALUES (?1, ?2, ?3, ?4)",
            (
                i64::from(CURRENT_SCHEMA_VERSION),
                INITIAL_MIGRATION_NAME,
                INITIAL_MIGRATION_CHECKSUM,
                applied_at_ms,
            ),
        )
        .map_err(|error| RuntimeStoreError::from_migration_sqlite(&error))?;
    ensure_before(deadline)?;
    transaction
        .commit()
        .map_err(|error| RuntimeStoreError::from_migration_sqlite(&error))
}

fn validate_migration_history(connection: &Connection) -> Result<(), RuntimeStoreError> {
    let latest: Option<(i64, String, String)> = connection
        .query_row(
            "SELECT migration_id, name, checksum_sha256
             FROM schema_migrations
             ORDER BY migration_id DESC
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|error| RuntimeStoreError::from_migration_sqlite(&error))?;
    let Some((migration_id, name, checksum)) = latest else {
        return Err(RuntimeStoreError::migration_mismatch());
    };
    if migration_id > i64::from(CURRENT_SCHEMA_VERSION) {
        return Err(RuntimeStoreError::newer_schema());
    }
    let history_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .map_err(|error| RuntimeStoreError::from_migration_sqlite(&error))?;
    if migration_id != i64::from(CURRENT_SCHEMA_VERSION)
        || history_count != i64::from(CURRENT_SCHEMA_VERSION)
        || name != INITIAL_MIGRATION_NAME
        || checksum != INITIAL_MIGRATION_CHECKSUM
    {
        return Err(RuntimeStoreError::migration_mismatch());
    }
    Ok(())
}

fn validate_schema(connection: &Connection) -> Result<(), RuntimeStoreError> {
    let fingerprint = schema_fingerprint(connection)?;
    if fingerprint != EXPECTED_SCHEMA_FINGERPRINT {
        return Err(RuntimeStoreError::migration_mismatch());
    }
    Ok(())
}

fn validate_integrity(connection: &Connection) -> Result<(), RuntimeStoreError> {
    let quick_check: String = connection
        .pragma_query_value(None, "quick_check", |row| row.get(0))
        .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
    if quick_check != "ok" {
        return Err(RuntimeStoreError::integrity_failed());
    }
    let foreign_key_violations: i64 = connection
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
    if foreign_key_violations != 0 {
        return Err(RuntimeStoreError::integrity_failed());
    }
    Ok(())
}

pub(crate) fn schema_fingerprint(connection: &Connection) -> Result<String, RuntimeStoreError> {
    let mut statement = connection
        .prepare(
            "SELECT type, name, tbl_name, coalesce(sql, '')
             FROM sqlite_schema
             WHERE name NOT LIKE 'sqlite_%'
                OR name LIKE 'sqlite_autoindex_%'
             ORDER BY type, name",
        )
        .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
    let mut rows = statement
        .query([])
        .map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
    let mut hasher = Sha256::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| RuntimeStoreError::from_sqlite(&error))?
    {
        for value in [
            row.get::<_, String>(0),
            row.get::<_, String>(1),
            row.get::<_, String>(2),
            row.get::<_, String>(3),
        ] {
            let value = value.map_err(|error| RuntimeStoreError::from_sqlite(&error))?;
            hasher.update(value.len().to_le_bytes());
            hasher.update(value.as_bytes());
        }
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
pub(crate) fn run_long_query(connection: &Connection) -> Result<(), RuntimeStoreError> {
    connection
        .query_row(
            "WITH RECURSIVE long_running(value) AS (
                 VALUES(0)
                 UNION ALL
                 SELECT value + 1 FROM long_running WHERE value < 1000000000
             )
             SELECT sum(value) FROM long_running",
            [],
            |_row| Ok(()),
        )
        .map_err(|error| RuntimeStoreError::from_migration_sqlite(&error))
}

fn object_exists(
    connection: &Connection,
    object_type: &str,
    name: &str,
) -> Result<bool, RuntimeStoreError> {
    connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sqlite_schema WHERE type = ?1 AND name = ?2
             )",
            (object_type, name),
            |row| row.get(0),
        )
        .map_err(|error| RuntimeStoreError::from_sqlite(&error))
}

fn verify_embedded_checksum() -> Result<(), RuntimeStoreError> {
    let actual = hex::encode(Sha256::digest(INITIAL_MIGRATION_SQL.as_bytes()));
    if actual != INITIAL_MIGRATION_CHECKSUM {
        return Err(RuntimeStoreError::migration_mismatch());
    }
    Ok(())
}
