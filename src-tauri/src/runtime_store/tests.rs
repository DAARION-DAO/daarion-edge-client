use super::config::{
    InitializationTestHook, InitializationTestStage, RuntimeStoreConfig, STORAGE_QUEUE_CAPACITY,
};
use super::connection::{RuntimeStoreConnection, REQUIRED_LIMITS};
use super::deadline::{active_watchdogs, watchdog_counts};
use super::error::RuntimeStoreErrorKind;
use super::lifecycle::RuntimeStoreLifecycle;
use super::migrations::{
    migrate_and_validate, schema_fingerprint, CURRENT_SCHEMA_VERSION, EXPECTED_SCHEMA_FINGERPRINT,
    INITIAL_MIGRATION_CHECKSUM,
};
use super::path_policy::{regular_file_identity_for_test, sidecar_path_for_test};
use super::types::{
    DatabaseHealth, PersistenceState, StorageRuntimeErrorCode, StorageRuntimeState,
};
use super::worker::{RuntimeStoreManager, WorkerExit, WorkerJoinOwnership};
use rusqlite::{Connection, TransactionBehavior};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

struct TestRoot {
    path: PathBuf,
}

impl TestRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("daarion-runtime-store-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).expect("test root must be created");
        let path = fs::canonicalize(path).expect("test root must canonicalize");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn database_path(&self) -> PathBuf {
        self.path
            .join("runtime-state")
            .join("runtime-state-v1.sqlite3")
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn initialized_store(root: &TestRoot) -> RuntimeStoreConnection {
    let config = RuntimeStoreConfig::for_test(root.path().to_path_buf());
    let mut store = RuntimeStoreConnection::open(&config).expect("store must open");
    assert_eq!(
        migrate_and_validate(&mut store.connection).expect("migration must validate"),
        CURRENT_SCHEMA_VERSION
    );
    store
}

#[test]
fn first_open_creates_a_healthy_store() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    let status = manager.initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()));
    assert_eq!(status.state, StorageRuntimeState::Healthy);
    assert!(status.initialized);
    assert_eq!(status.schema_version, Some(1));
    assert_eq!(status.database_health, DatabaseHealth::Ok);
    assert_eq!(status.persistence_state, PersistenceState::CreatedNew);
    assert_eq!(status.storage_backend, "sqlite");
    assert!(status.sqlite_version.is_some());
    assert!(status.database_size_bytes.is_some_and(|size| size > 0));
}

#[test]
fn clean_restart_reopens_the_same_database_without_reapplying_migration() {
    let root = TestRoot::new();
    let first = RuntimeStoreManager::new();
    let first_status = first.initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()));
    assert_eq!(first_status.persistence_state, PersistenceState::CreatedNew);
    let before = migration_evidence(&root.database_path());
    first
        .shutdown_for_test(Duration::from_secs(2))
        .expect("first worker must shut down");
    drop(first);

    let second = RuntimeStoreManager::new();
    let second_status = second.initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()));
    assert_eq!(second_status.state, StorageRuntimeState::Healthy);
    assert_eq!(
        second_status.persistence_state,
        PersistenceState::ReopenedExisting
    );
    assert_eq!(migration_evidence(&root.database_path()), before);
    second
        .shutdown_for_test(Duration::from_secs(2))
        .expect("second worker must shut down");
}

#[test]
fn initial_migration_creates_exactly_five_application_tables() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    let mut statement = store
        .connection
        .prepare(
            "SELECT name FROM sqlite_schema
             WHERE type = 'table'
             ORDER BY name",
        )
        .expect("table inventory must prepare");
    let tables: Vec<String> = statement
        .query_map([], |row| row.get(0))
        .expect("table inventory must query")
        .collect::<Result<_, _>>()
        .expect("table names must decode");
    assert_eq!(
        tables,
        vec![
            "audit_events",
            "conversations",
            "messages",
            "schema_migrations",
            "tasks"
        ]
    );
    assert!(!tables.iter().any(|table| table == "sqlite_sequence"));
}

#[test]
fn initial_migration_creates_the_approved_index_inventory() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    let mut statement = store
        .connection
        .prepare("SELECT name FROM sqlite_schema WHERE type = 'index' ORDER BY name")
        .expect("index inventory must prepare");
    let indexes: Vec<String> = statement
        .query_map([], |row| row.get(0))
        .expect("index inventory must query")
        .collect::<Result<_, _>>()
        .expect("index names must decode");
    assert_eq!(
        indexes,
        vec![
            "audit_events_created_idx",
            "audit_events_subject_idx",
            "audit_events_type_created_idx",
            "conversations_updated_idx",
            "messages_conversation_sequence_idx",
            "sqlite_autoindex_audit_events_1",
            "sqlite_autoindex_conversations_1",
            "sqlite_autoindex_messages_1",
            "sqlite_autoindex_messages_2",
            "sqlite_autoindex_schema_migrations_1",
            "sqlite_autoindex_tasks_1",
            "sqlite_autoindex_tasks_2",
            "tasks_conversation_idx",
            "tasks_state_updated_idx"
        ]
    );
}

#[test]
fn required_pragmas_are_applied_and_read_back() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    let integer_pragma = |name: &str| {
        store
            .connection
            .pragma_query_value(None, name, |row| row.get::<_, i64>(0))
            .expect("integer pragma must be readable")
    };
    assert_eq!(integer_pragma("foreign_keys"), 1);
    assert_eq!(integer_pragma("synchronous"), 2);
    assert_eq!(integer_pragma("secure_delete"), 1);
    assert_eq!(integer_pragma("trusted_schema"), 0);
    assert_eq!(integer_pragma("temp_store"), 2);
    assert_eq!(integer_pragma("busy_timeout"), 150);
    let journal: String = store
        .connection
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .expect("journal mode must be readable");
    assert_eq!(journal, "wal");
}

#[test]
fn startup_writes_only_the_migration_history_row() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    for table in ["conversations", "messages", "tasks", "audit_events"] {
        let count: i64 = store
            .connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("startup content table count must be readable");
        assert_eq!(count, 0, "startup must not seed {table}");
    }
    let migration_count: i64 = store
        .connection
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .expect("migration count must be readable");
    assert_eq!(migration_count, 1);
}

#[test]
fn migration_checksum_mismatch_fails_closed() {
    let root = TestRoot::new();
    let mut store = initialized_store(&root);
    store
        .connection
        .execute(
            "UPDATE schema_migrations SET checksum_sha256 = ?1 WHERE migration_id = 1",
            ["0".repeat(64)],
        )
        .expect("test must tamper migration checksum");
    let error = migrate_and_validate(&mut store.connection).expect_err("tamper must fail");
    assert_eq!(
        error.public_code(),
        StorageRuntimeErrorCode::MigrationMismatch
    );
}

#[test]
fn migration_name_mismatch_fails_closed() {
    let root = TestRoot::new();
    let mut store = initialized_store(&root);
    store
        .connection
        .execute(
            "UPDATE schema_migrations SET name = 'unexpected_name' WHERE migration_id = 1",
            [],
        )
        .expect("test must tamper migration name");
    let error = migrate_and_validate(&mut store.connection).expect_err("tamper must fail");
    assert_eq!(
        error.public_code(),
        StorageRuntimeErrorCode::MigrationMismatch
    );
}

#[test]
fn interrupted_initial_migration_rolls_back_schema_and_history() {
    let root = TestRoot::new();
    fs::create_dir_all(
        root.database_path()
            .parent()
            .expect("database must have parent"),
    )
    .expect("database parent must exist");
    let mut connection =
        Connection::open(root.database_path()).expect("fixture database must open");
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .expect("migration transaction must begin");
    transaction
        .execute_batch(include_str!(
            "../../migrations/runtime_state/0001_runtime_state_initial.sql"
        ))
        .expect("migration SQL must execute inside transaction");
    transaction
        .rollback()
        .expect("interrupted migration must roll back");
    let table_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table'",
            [],
            |row| row.get(0),
        )
        .expect("schema inventory must remain readable");
    assert_eq!(table_count, 0);
}

#[test]
fn newer_schema_fails_closed() {
    let root = TestRoot::new();
    let mut store = initialized_store(&root);
    store
        .connection
        .execute("UPDATE schema_migrations SET migration_id = 2", [])
        .expect("test must create newer history");
    let error = migrate_and_validate(&mut store.connection).expect_err("newer schema must fail");
    assert_eq!(error.public_code(), StorageRuntimeErrorCode::NewerSchema);
}

#[test]
fn schema_tampering_fails_structural_fingerprint_validation() {
    let root = TestRoot::new();
    let mut store = initialized_store(&root);
    store
        .connection
        .execute("CREATE TABLE unexpected_table (id INTEGER) STRICT", [])
        .expect("test must create unexpected table");
    let error = migrate_and_validate(&mut store.connection).expect_err("schema tamper must fail");
    assert_eq!(
        error.public_code(),
        StorageRuntimeErrorCode::MigrationMismatch
    );
}

#[test]
fn quick_check_and_foreign_key_check_are_clean() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    let quick_check: String = store
        .connection
        .pragma_query_value(None, "quick_check", |row| row.get(0))
        .expect("quick check must execute");
    let foreign_key_violations: i64 = store
        .connection
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .expect("foreign-key check must execute");
    assert_eq!(quick_check, "ok");
    assert_eq!(foreign_key_violations, 0);
}

#[test]
fn corrupt_database_is_preserved_and_reported_without_raw_details() {
    let root = TestRoot::new();
    fs::create_dir_all(
        root.database_path()
            .parent()
            .expect("database must have parent"),
    )
    .expect("database parent must exist");
    let original = b"not a sqlite database";
    fs::write(root.database_path(), original).expect("corrupt fixture must be written");
    let before = hex::encode(Sha256::digest(original));
    let manager = RuntimeStoreManager::new();
    let status = manager.initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()));
    assert!(!status.initialized);
    assert!(matches!(
        status.state,
        StorageRuntimeState::IntegrityFailed | StorageRuntimeState::MigrationFailed
    ));
    let after_bytes = fs::read(root.database_path()).expect("corrupt fixture must remain");
    assert_eq!(hex::encode(Sha256::digest(after_bytes)), before);
    let serialized = serde_json::to_string(&status).expect("status must serialize");
    assert!(!serialized.contains(root.path.to_string_lossy().as_ref()));
    assert!(!serialized.contains("sqlite database"));
}

#[test]
fn relative_and_parent_traversal_roots_are_rejected() {
    for invalid in [
        PathBuf::from("relative"),
        PathBuf::from("/tmp/../tmp/daarion"),
    ] {
        let result = RuntimeStoreConnection::open(&RuntimeStoreConfig::for_test(invalid));
        assert!(result.is_err());
    }
}

#[cfg(unix)]
#[test]
fn symlinked_runtime_directory_is_rejected() {
    use std::os::unix::fs::symlink;
    let root = TestRoot::new();
    let outside = TestRoot::new();
    symlink(outside.path(), root.path().join("runtime-state"))
        .expect("test symlink must be created");
    let result = RuntimeStoreConnection::open(&RuntimeStoreConfig::for_test(root.path.clone()));
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn symlinked_database_target_is_rejected() {
    use std::os::unix::fs::symlink;
    let root = TestRoot::new();
    let outside = TestRoot::new();
    fs::create_dir_all(
        root.database_path()
            .parent()
            .expect("database must have parent"),
    )
    .expect("database parent must exist");
    let outside_database = outside.path().join("outside.sqlite3");
    fs::write(&outside_database, []).expect("outside fixture must be created");
    symlink(outside_database, root.database_path()).expect("database symlink must be created");
    let result = RuntimeStoreConnection::open(&RuntimeStoreConfig::for_test(root.path.clone()));
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn symlinked_app_root_component_is_rejected() {
    use std::os::unix::fs::symlink;
    let target = TestRoot::new();
    let link = target
        .path()
        .parent()
        .expect("test root must have parent")
        .join(format!("daarion-root-link-{}", Uuid::new_v4()));
    symlink(target.path(), &link).expect("test root symlink must be created");
    let result = RuntimeStoreConnection::open(&RuntimeStoreConfig::for_test(link.clone()));
    fs::remove_file(link).expect("test root symlink must be removed");
    assert!(result.is_err());
}

#[test]
fn non_regular_database_target_is_rejected() {
    let root = TestRoot::new();
    fs::create_dir_all(root.database_path()).expect("directory fixture must be created");
    let result = RuntimeStoreConnection::open(&RuntimeStoreConfig::for_test(root.path.clone()));
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn unix_runtime_paths_use_private_modes() {
    use std::os::unix::fs::PermissionsExt;
    let root = TestRoot::new();
    let store = initialized_store(&root);
    assert_eq!(
        fs::metadata(root.path().join("runtime-state"))
            .expect("runtime directory metadata must exist")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(root.database_path())
            .expect("database metadata must exist")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    store.close().expect("store must close cleanly");
}

#[test]
fn conversation_and_message_constraints_reject_invalid_inputs() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    assert!(store
        .connection
        .execute(
            "INSERT INTO conversations (
                 id, title, status, created_at_ms, updated_at_ms
             ) VALUES ('invalid-id', NULL, 'active', 1, 1)",
            [],
        )
        .is_err());
    let conversation_id = Uuid::new_v4().to_string();
    store
        .connection
        .execute(
            "INSERT INTO conversations (
                 id, title, status, created_at_ms, updated_at_ms
             ) VALUES (?1, NULL, 'active', 1, 1)",
            [&conversation_id],
        )
        .expect("valid conversation must insert");
    assert!(store
        .connection
        .execute(
            "INSERT INTO messages (
                 id, conversation_id, sequence_no, role, content, created_at_ms
             ) VALUES (?1, ?2, 1, 'tool', 'content', 1)",
            [Uuid::new_v4().to_string(), conversation_id],
        )
        .is_err());
}

#[test]
fn foreign_keys_and_unique_message_sequences_are_enforced() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    assert!(store
        .connection
        .execute(
            "INSERT INTO messages (
                 id, conversation_id, sequence_no, role, content, created_at_ms
             ) VALUES (?1, ?2, 1, 'user', 'content', 1)",
            [Uuid::new_v4().to_string(), Uuid::new_v4().to_string()],
        )
        .is_err());
    let conversation_id = Uuid::new_v4().to_string();
    store
        .connection
        .execute(
            "INSERT INTO conversations (
                 id, title, status, created_at_ms, updated_at_ms
             ) VALUES (?1, NULL, 'active', 1, 1)",
            [&conversation_id],
        )
        .expect("valid conversation must insert");
    for attempt in 0..2 {
        let result = store.connection.execute(
            "INSERT INTO messages (
                 id, conversation_id, sequence_no, role, content, created_at_ms
             ) VALUES (?1, ?2, 1, 'user', 'content', 1)",
            [Uuid::new_v4().to_string(), conversation_id.clone()],
        );
        assert_eq!(result.is_ok(), attempt == 0);
    }
}

#[test]
fn task_and_audit_allowlists_are_enforced() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    assert!(store
        .connection
        .execute(
            "INSERT INTO tasks (
                 id, task_kind, state, created_at_ms, updated_at_ms
             ) VALUES (?1, 'health', 'running', 1, 1)",
            [Uuid::new_v4().to_string()],
        )
        .is_err());
    assert!(store
        .connection
        .execute(
            "INSERT INTO audit_events (
                 event_id, event_type, actor_type, subject_type, outcome, created_at_ms
             ) VALUES (?1, 'unknown.event', 'local_runtime', 'runtime', 'success', 1)",
            [Uuid::new_v4().to_string()],
        )
        .is_err());
}

#[test]
fn audit_sequence_uses_integer_primary_key_without_sqlite_sequence() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    for _ in 0..2 {
        store
            .connection
            .execute(
                "INSERT INTO audit_events (
                     event_id, event_type, actor_type, subject_type, outcome, created_at_ms
                 ) VALUES (?1, 'storage.recovery_required', 'local_runtime',
                           'storage', 'success', 1)",
                [Uuid::new_v4().to_string()],
            )
            .expect("valid audit event must insert");
    }
    let sequences: Vec<i64> = store
        .connection
        .prepare("SELECT sequence_no FROM audit_events ORDER BY sequence_no")
        .expect("audit query must prepare")
        .query_map([], |row| row.get(0))
        .expect("audit query must execute")
        .collect::<Result<_, _>>()
        .expect("audit sequence values must decode");
    assert_eq!(sequences, vec![1, 2]);
    let sqlite_sequence_exists: bool = store
        .connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sqlite_schema WHERE name = 'sqlite_sequence'
             )",
            [],
            |row| row.get(0),
        )
        .expect("internal table check must run");
    assert!(!sqlite_sequence_exists);
}

#[test]
fn concurrent_initialization_is_idempotent() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let manager = manager.clone();
            let root = root.path.clone();
            thread::spawn(move || manager.initialize_for_test(RuntimeStoreConfig::for_test(root)))
        })
        .collect();
    let statuses: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().expect("initializer must not panic"))
        .collect();
    assert!(statuses.iter().all(|status| status.initialized));
    assert_eq!(migration_evidence(&root.database_path()).0, 1);
}

#[test]
fn worker_contract_has_one_named_owner_and_bounded_queue() {
    assert_eq!(STORAGE_QUEUE_CAPACITY, 128);
    let worker_source = include_str!("worker.rs");
    assert!(worker_source.contains(".name(\"daarion-runtime-store\".to_string())"));
    assert_eq!(
        worker_source
            .matches("let mut connection: Option<RuntimeStoreConnection>")
            .count(),
        1
    );
}

#[test]
fn locked_database_returns_controlled_status_within_busy_deadline() {
    let root = TestRoot::new();
    fs::create_dir_all(
        root.database_path()
            .parent()
            .expect("database must have parent"),
    )
    .expect("database parent must exist");
    let blocker = Connection::open(root.database_path()).expect("lock fixture must open");
    blocker
        .execute_batch("BEGIN EXCLUSIVE;")
        .expect("exclusive lock must be acquired");
    let manager = RuntimeStoreManager::new();
    let started_at = Instant::now();
    let status = manager.initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()));
    assert!(started_at.elapsed() < Duration::from_secs(2));
    assert!(!status.initialized);
    assert_eq!(status.state, StorageRuntimeState::Locked);
    assert!(matches!(
        status.error_code,
        Some(StorageRuntimeErrorCode::BusyTimeout | StorageRuntimeErrorCode::Locked)
    ));
    blocker
        .execute_batch("ROLLBACK;")
        .expect("exclusive lock must release");
}

#[test]
fn status_read_timeout_returns_fresh_unavailable_instead_of_stale_healthy() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.ordinary_deadline = Duration::from_millis(20);
    assert!(manager.initialize_for_test(config).initialized);
    let holding_manager = manager.clone();
    let hold = thread::spawn(move || {
        holding_manager
            .hold_worker_for_test(Duration::from_millis(150))
            .expect("test hold must complete");
    });
    thread::sleep(Duration::from_millis(10));
    let started_at = Instant::now();
    let failure = manager.read_status();
    assert!(started_at.elapsed() < Duration::from_millis(120));
    assert!(!failure.initialized);
    assert_eq!(failure.state, StorageRuntimeState::Unavailable);
    assert_eq!(failure.error_code, Some(StorageRuntimeErrorCode::Internal));
    hold.join().expect("hold thread must not panic");
}

#[test]
fn shutdown_rejects_new_status_work_and_preserves_store_for_reopen() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    manager
        .shutdown_for_test(Duration::from_secs(2))
        .expect("shutdown must complete");
    let wal_path = PathBuf::from(format!("{}-wal", root.database_path().display()));
    assert!(
        !wal_path.exists()
            || fs::metadata(wal_path)
                .expect("WAL metadata must be readable")
                .len()
                == 0
    );
    let rejected = manager.read_status();
    assert_eq!(rejected.state, StorageRuntimeState::Unavailable);
    drop(manager);
    let reopened = RuntimeStoreManager::new();
    assert_eq!(
        reopened
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .persistence_state,
        PersistenceState::ReopenedExisting
    );
}

#[test]
fn shutdown_propagates_a_busy_checkpoint_failure() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );

    let reader = Connection::open(root.database_path()).expect("reader must open");
    reader
        .execute_batch("BEGIN; SELECT COUNT(*) FROM schema_migrations;")
        .expect("reader snapshot must start");
    let writer = Connection::open(root.database_path()).expect("writer must open");
    writer
        .execute(
            "INSERT INTO conversations (
                 id, title, status, created_at_ms, updated_at_ms
             ) VALUES (?1, NULL, 'active', 1, 1)",
            [Uuid::new_v4().to_string()],
        )
        .expect("fixture write must commit to the WAL");

    let error = manager
        .shutdown_for_test(Duration::from_secs(2))
        .expect_err("busy checkpoint must fail closed");
    assert_eq!(error.kind, RuntimeStoreErrorKind::BusyTimeout);
    reader
        .execute_batch("ROLLBACK;")
        .expect("reader snapshot must release");
}

#[test]
fn status_reads_do_not_mutate_migration_or_content_state() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    let before = migration_evidence(&root.database_path());
    for _ in 0..10 {
        assert!(manager.read_status().initialized);
    }
    assert_eq!(migration_evidence(&root.database_path()), before);
    let connection = Connection::open(root.database_path()).expect("content check must open");
    for table in ["conversations", "messages", "tasks", "audit_events"] {
        let count: i64 = connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("content count must be readable");
        assert_eq!(count, 0);
    }
}

#[cfg(unix)]
#[test]
fn database_replacement_after_open_fails_closed() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    let preserved = root.database_path().with_extension("preserved");
    fs::rename(root.database_path(), &preserved).expect("database fixture must be moved");
    fs::write(root.database_path(), b"replacement").expect("replacement fixture must be created");
    let status = manager.read_status();
    assert!(!status.initialized);
    assert_eq!(status.state, StorageRuntimeState::Unavailable);
    assert_eq!(
        status.error_code,
        Some(StorageRuntimeErrorCode::PathInvalid)
    );
    assert!(preserved.exists());
}

#[cfg(unix)]
#[test]
fn runtime_directory_replacement_with_same_database_inode_fails_closed() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    let runtime_root = root.path.join("runtime-state");
    let preserved_root = root.path.join("runtime-state-preserved");
    fs::rename(&runtime_root, &preserved_root).expect("runtime directory must move");
    fs::create_dir(&runtime_root).expect("replacement runtime directory must be created");
    fs::hard_link(
        preserved_root.join("runtime-state-v1.sqlite3"),
        runtime_root.join("runtime-state-v1.sqlite3"),
    )
    .expect("same-inode database fixture must be linked");

    let status = manager.read_status();
    assert!(!status.initialized);
    assert_eq!(status.state, StorageRuntimeState::Unavailable);
    assert_eq!(
        status.error_code,
        Some(StorageRuntimeErrorCode::PathInvalid)
    );
}

#[cfg(unix)]
#[test]
fn database_identity_is_preserved_across_clean_reopen() {
    use std::os::unix::fs::MetadataExt;
    let root = TestRoot::new();
    let first = RuntimeStoreManager::new();
    assert!(
        first
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    let before = fs::metadata(root.database_path()).expect("database metadata must exist");
    first
        .shutdown_for_test(Duration::from_secs(2))
        .expect("first worker must shut down");
    drop(first);
    let second = RuntimeStoreManager::new();
    assert!(
        second
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    let after = fs::metadata(root.database_path()).expect("database metadata must exist");
    assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
}

#[test]
fn hard_database_limit_fails_closed() {
    let root = TestRoot::new();
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.database_hard_limit_bytes = 1;
    let manager = RuntimeStoreManager::new();
    let status = manager.initialize_for_test(config);
    assert!(!status.initialized);
    assert_eq!(status.state, StorageRuntimeState::ResourceLimited);
    assert_eq!(
        status.error_code,
        Some(StorageRuntimeErrorCode::ResourceLimit)
    );
}

#[test]
fn status_read_reapplies_resource_limits_after_database_growth() {
    let root = TestRoot::new();
    initialized_store(&root)
        .close()
        .expect("bootstrap store must close");
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.database_warning_threshold_bytes = 1024 * 1024;
    config.database_hard_limit_bytes = 2 * 1024 * 1024;
    let manager = RuntimeStoreManager::new();
    assert_eq!(
        manager.initialize_for_test(config).state,
        StorageRuntimeState::Healthy
    );

    let writer = Connection::open(root.database_path()).expect("growth writer must open");
    writer
        .execute_batch("PRAGMA foreign_keys=ON;")
        .expect("fixture foreign keys must enable");
    let conversation_id = Uuid::new_v4().to_string();
    writer
        .execute(
            "INSERT INTO conversations (
                 id, title, status, created_at_ms, updated_at_ms
             ) VALUES (?1, NULL, 'active', 1, 1)",
            [&conversation_id],
        )
        .expect("fixture conversation must insert");
    let content = "x".repeat(256 * 1024);
    for sequence_no in 1_i64..=12 {
        writer
            .execute(
                "INSERT INTO messages (
                     id, conversation_id, sequence_no, role, content, created_at_ms
                 ) VALUES (?1, ?2, ?3, 'user', ?4, 1)",
                (
                    Uuid::new_v4().to_string(),
                    &conversation_id,
                    sequence_no,
                    &content,
                ),
            )
            .expect("fixture message must grow the database");
    }

    let status = manager.read_status();
    assert!(!status.initialized);
    assert_eq!(status.state, StorageRuntimeState::ResourceLimited);
    assert_eq!(
        status.error_code,
        Some(StorageRuntimeErrorCode::ResourceLimit)
    );
}

#[test]
fn configured_sqlite_limits_are_all_read_back_exactly() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    for (limit, expected) in REQUIRED_LIMITS {
        assert_eq!(
            store
                .connection
                .limit(limit)
                .expect("configured SQLite limit must be readable"),
            expected,
            "effective limit must match the required value for {limit:?}"
        );
    }
}

#[test]
fn migration_checksum_and_structural_fingerprint_match_exact_final_sql() {
    let sql = include_str!("../../migrations/runtime_state/0001_runtime_state_initial.sql");
    assert_eq!(
        hex::encode(Sha256::digest(sql.as_bytes())),
        INITIAL_MIGRATION_CHECKSUM
    );
    let root = TestRoot::new();
    let store = initialized_store(&root);
    assert_eq!(
        schema_fingerprint(&store.connection).expect("schema fingerprint must calculate"),
        EXPECTED_SCHEMA_FINGERPRINT
    );
}

#[test]
fn strict_uuid_v4_checks_cover_every_uuid_bearing_column() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    let valid_conversation_id = Uuid::new_v4().to_string();
    store
        .connection
        .execute(
            "INSERT INTO conversations (
                 id, title, status, created_at_ms, updated_at_ms
             ) VALUES (?1, NULL, 'active', 1, 1)",
            [&valid_conversation_id],
        )
        .expect("valid lowercase UUID v4 conversation must insert");

    let malformed = [
        "-2345678-1234-4abc-8abc-123456789abc",
        "12345678--234-4abc-8abc-123456789abc",
        "12345678-1234-4-bc-8abc-123456789abc",
        "12345678-1234-4abc-8-bc-123456789abc",
        "12345678-1234-4abc-8abc--23456789abc",
        "g2345678-1234-4abc-8abc-123456789abc",
        "A2345678-1234-4abc-8abc-123456789abc",
        "12345678-1234-4abc-8abc-123456789ab",
        "12345678a1234-4abc-8abc-123456789abc",
        "1234567-81234-4abc-8abc-123456789abc",
        "12345678-1234-5abc-8abc-123456789abc",
        "12345678-1234-4abc-7abc-123456789abc",
    ];

    for value in malformed {
        assert_check_constraint(
            store.connection.execute(
                "INSERT INTO conversations (
                     id, title, status, created_at_ms, updated_at_ms
                 ) VALUES (?1, NULL, 'active', 1, 1)",
                [value],
            ),
            "conversations.id",
        );
        assert_check_constraint(
            store.connection.execute(
                "INSERT INTO messages (
                     id, conversation_id, sequence_no, role, content, created_at_ms
                 ) VALUES (?1, ?2, 1, 'user', 'content', 1)",
                (value, &valid_conversation_id),
            ),
            "messages.id",
        );
        assert_check_constraint(
            store.connection.execute(
                "INSERT INTO messages (
                     id, conversation_id, sequence_no, role, content, created_at_ms
                 ) VALUES (?1, ?2, 1, 'user', 'content', 1)",
                (Uuid::new_v4().to_string(), value),
            ),
            "messages.conversation_id",
        );
        assert_check_constraint(
            store.connection.execute(
                "INSERT INTO tasks (
                     id, task_kind, state, created_at_ms, updated_at_ms
                 ) VALUES (?1, 'health', 'created', 1, 1)",
                [value],
            ),
            "tasks.id",
        );
        assert_check_constraint(
            store.connection.execute(
                "INSERT INTO tasks (
                     id, conversation_id, task_kind, state, created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, 'health', 'created', 1, 1)",
                (Uuid::new_v4().to_string(), value),
            ),
            "tasks.conversation_id",
        );
        assert_check_constraint(
            store.connection.execute(
                "INSERT INTO audit_events (
                     event_id, event_type, actor_type, subject_type, outcome, created_at_ms
                 ) VALUES (?1, 'storage.recovery_required', 'local_runtime',
                           'storage', 'success', 1)",
                [value],
            ),
            "audit_events.event_id",
        );
        assert_check_constraint(
            store.connection.execute(
                "INSERT INTO audit_events (
                     event_id, event_type, actor_type, subject_type, subject_id,
                     outcome, created_at_ms
                 ) VALUES (?1, 'storage.recovery_required', 'local_runtime',
                           'storage', ?2, 'success', 1)",
                (Uuid::new_v4().to_string(), value),
            ),
            "audit_events.subject_id",
        );
        assert_check_constraint(
            store.connection.execute(
                "INSERT INTO audit_events (
                     event_id, event_type, actor_type, subject_type, outcome,
                     correlation_id, created_at_ms
                 ) VALUES (?1, 'storage.recovery_required', 'local_runtime',
                           'storage', 'success', ?2, 1)",
                (Uuid::new_v4().to_string(), value),
            ),
            "audit_events.correlation_id",
        );
    }

    let valid_message_id = Uuid::new_v4().to_string();
    store
        .connection
        .execute(
            "INSERT INTO messages (
                 id, conversation_id, sequence_no, role, content, created_at_ms
             ) VALUES (?1, ?2, 1, 'user', 'content', 1)",
            (&valid_message_id, &valid_conversation_id),
        )
        .expect("valid lowercase UUID v4 message and foreign key must insert");
    store
        .connection
        .execute(
            "INSERT INTO tasks (
                 id, conversation_id, task_kind, state, created_at_ms, updated_at_ms
             ) VALUES (?1, NULL, 'health', 'created', 1, 1)",
            [Uuid::new_v4().to_string()],
        )
        .expect("nullable task conversation UUID must accept NULL");
    store
        .connection
        .execute(
            "INSERT INTO audit_events (
                 event_id, event_type, actor_type, subject_type, subject_id,
                 outcome, correlation_id, created_at_ms
             ) VALUES (?1, 'storage.recovery_required', 'local_runtime',
                       'storage', NULL, 'success', NULL, 1)",
            [Uuid::new_v4().to_string()],
        )
        .expect("nullable audit UUID columns must accept NULL");
}

#[test]
fn production_lifecycle_helper_performs_one_idempotent_shutdown() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    let lifecycle = RuntimeStoreLifecycle::new();
    assert_eq!(lifecycle.shutdown_once(&manager), Some(Ok(())));
    assert_eq!(lifecycle.shutdown_once(&manager), None);
    assert_eq!(manager.shutdown_for_test(Duration::from_millis(50)), Ok(()));
    assert_eq!(
        manager.worker_exit_for_test(),
        Some(WorkerExit::CleanShutdown)
    );
    assert!(manager.joined_after_exit_for_test());
}

#[test]
fn tauri_composition_registers_the_explicit_storage_shutdown_lifecycle() {
    let root = include_str!("../lib.rs");
    let lifecycle = include_str!("lifecycle.rs");
    assert!(root.contains("runtime_store::RuntimeStoreLifecycle::new()"));
    assert!(root.contains("storage_lifecycle.on_run_event"));
    assert!(root.contains("builder.build(tauri::generate_context!())"));
    assert!(root.contains("app.run(move |app_handle, event|"));
    assert!(lifecycle.contains("tauri::RunEvent::ExitRequested"));
    assert!(lifecycle.contains("tauri::RunEvent::Exit"));
    assert!(lifecycle.contains("manager.production_shutdown()"));
}

#[test]
fn busy_checkpoint_is_bounded_by_the_shutdown_deadline() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    let reader = Connection::open(root.database_path()).expect("reader must open");
    reader
        .execute_batch("BEGIN; SELECT COUNT(*) FROM schema_migrations;")
        .expect("reader snapshot must start");
    let writer = Connection::open(root.database_path()).expect("writer must open");
    writer
        .execute(
            "INSERT INTO conversations (
                 id, title, status, created_at_ms, updated_at_ms
             ) VALUES (?1, NULL, 'active', 1, 1)",
            [Uuid::new_v4().to_string()],
        )
        .expect("fixture write must commit to WAL");
    let started = Instant::now();
    let error = manager
        .shutdown_for_test(Duration::from_millis(80))
        .expect_err("busy checkpoint must fail within the service deadline");
    assert!(started.elapsed() < Duration::from_millis(350));
    assert!(matches!(
        error.kind,
        RuntimeStoreErrorKind::BusyTimeout | RuntimeStoreErrorKind::Unavailable
    ));
    reader
        .execute_batch("ROLLBACK;")
        .expect("reader must release");
}

#[test]
fn missing_worker_exit_signal_never_causes_an_unbounded_join() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    manager.suppress_exit_signal_for_test();
    let started = Instant::now();
    let error = manager
        .shutdown_for_test(Duration::from_millis(60))
        .expect_err("missing worker-exit proof must fail closed");
    assert!(started.elapsed() < Duration::from_millis(300));
    assert_eq!(error.kind, RuntimeStoreErrorKind::Unavailable);
    assert!(!manager.joined_after_exit_for_test());
    let status = manager.read_status();
    let serialized = serde_json::to_string(&status).expect("failure status must serialize");
    assert!(!status.initialized);
    assert_eq!(status.error_code, Some(StorageRuntimeErrorCode::Internal));
    assert!(!serialized.contains(root.path().to_string_lossy().as_ref()));
    assert!(!serialized.contains("PRAGMA"));
}

#[test]
fn worker_panic_after_healthy_disables_intake_and_removes_stale_success() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    manager
        .trigger_worker_panic_for_test()
        .expect("test panic request must enqueue");
    assert_eq!(wait_for_worker_exit(&manager), WorkerExit::Panic);
    let status = manager.read_status();
    assert!(!status.initialized);
    assert_eq!(status.state, StorageRuntimeState::Unavailable);
    assert_eq!(status.error_code, Some(StorageRuntimeErrorCode::Internal));
    let started = Instant::now();
    assert!(manager
        .shutdown_for_test(Duration::from_millis(200))
        .is_err());
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(manager.joined_after_exit_for_test());
}

#[test]
fn unexpected_worker_exit_uses_the_same_safe_unavailable_projection() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    manager
        .trigger_unexpected_exit_for_test()
        .expect("unexpected-exit request must enqueue");
    assert_eq!(wait_for_worker_exit(&manager), WorkerExit::UnexpectedExit);
    let status = manager.read_status();
    assert!(!status.initialized);
    assert_eq!(status.state, StorageRuntimeState::Unavailable);
    assert_eq!(status.error_code, Some(StorageRuntimeErrorCode::Internal));
    assert!(manager
        .shutdown_for_test(Duration::from_millis(200))
        .is_err());
    assert!(manager.joined_after_exit_for_test());
}

#[test]
fn real_sqlite_initialization_deadline_interrupts_long_query_without_late_health() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.migration_deadline = Duration::from_millis(50);
    config.initialization_test_hook = InitializationTestHook::LongQueryBeforeMigration;
    let started = Instant::now();
    let status = manager.initialize_for_test(config);
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(!status.initialized);
    assert_eq!(status.state, StorageRuntimeState::Unavailable);
    thread::sleep(Duration::from_millis(80));
    let later = manager.read_status();
    assert!(!later.initialized);
    assert_ne!(later.state, StorageRuntimeState::Healthy);
    let shutdown_started = Instant::now();
    let _controlled_shutdown = manager.shutdown_for_test(Duration::from_millis(300));
    assert!(shutdown_started.elapsed() < Duration::from_secs(1));
    assert!(manager.joined_after_exit_for_test());
}

#[test]
fn interrupted_initialization_transaction_leaves_no_partial_schema_or_history() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.migration_deadline = Duration::from_millis(50);
    config.initialization_test_hook = InitializationTestHook::LongQueryInsideMigration;
    let status = manager.initialize_for_test(config);
    assert!(!status.initialized);
    let connection =
        Connection::open(root.database_path()).expect("interrupted database must open");
    let tables: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table'",
            [],
            |row| row.get(0),
        )
        .expect("interrupted schema inventory must read");
    assert_eq!(tables, 0);
    let migration_history: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema
             WHERE type = 'table' AND name = 'schema_migrations'",
            [],
            |row| row.get(0),
        )
        .expect("migration-history presence must read");
    assert_eq!(migration_history, 0);
    let shutdown_started = Instant::now();
    let _controlled_shutdown = manager.shutdown_for_test(Duration::from_millis(300));
    assert!(shutdown_started.elapsed() < Duration::from_secs(1));
    assert!(manager.joined_after_exit_for_test());
}

#[test]
fn shutdown_cancellation_interrupts_active_sqlite_initialization_within_100_ms() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(wait_until(Duration::from_secs(1), || {
        manager.active_worker_count_for_test() == 1
    }));
    let watchdog_before = watchdog_counts();
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.initialization_test_hook = InitializationTestHook::LongQueryBeforeMigration;
    let initializing = manager.clone();
    let initialization = thread::spawn(move || initializing.initialize_for_test(config));
    assert!(wait_until(Duration::from_secs(1), || {
        manager.active_initialization_for_test()
    }));

    let shutdown_budget = Duration::from_millis(100);
    let started = Instant::now();
    manager
        .shutdown_for_test(shutdown_budget)
        .expect("active SQLite initialization must be interruptible");
    let elapsed = started.elapsed();
    assert!(
        elapsed < shutdown_budget,
        "shutdown used the complete 100 ms budget: {elapsed:?}"
    );
    eprintln!(
        "shutdown_budget_ms=100 elapsed_ms={} margin_ms={}",
        elapsed.as_millis(),
        shutdown_budget.saturating_sub(elapsed).as_millis()
    );
    let status = initialization
        .join()
        .expect("initialization observer must not panic");
    assert!(!status.initialized);
    assert!(!manager.read_status().initialized);
    assert!(!manager.active_initialization_for_test());
    assert_eq!(manager.active_watchdog_count_for_test(), 0);
    assert_eq!(
        manager.worker_join_ownership_for_test(),
        WorkerJoinOwnership::Completed
    );
    assert!(manager.joined_after_exit_for_test());
    assert_eq!(manager.active_worker_count_for_test(), 0);
    let watchdog_after = watchdog_counts();
    assert!(watchdog_after.0 > watchdog_before.0);
    assert!(watchdog_after.1 > watchdog_before.1);
    assert_interrupted_database_has_no_schema(&root);
}

#[test]
fn shutdown_cancellation_before_interrupt_registration_retains_join_ownership() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    let (hook, entered, release) = InitializationTestHook::blocking(
        InitializationTestStage::BeforeInterruptRegistration,
        false,
    );
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.initialization_test_hook = hook;
    let initializing = manager.clone();
    let initialization = thread::spawn(move || initializing.initialize_for_test(config));
    entered
        .recv_timeout(Duration::from_secs(1))
        .expect("initialization must pause before interrupt registration");
    assert!(!manager.active_initialization_for_test());

    let error = manager
        .shutdown_for_test(Duration::from_millis(50))
        .expect_err("noninterruptible test stage must time out closed");
    assert_eq!(error.kind, RuntimeStoreErrorKind::Unavailable);
    assert_eq!(
        manager.worker_join_ownership_for_test(),
        WorkerJoinOwnership::Reaper
    );
    assert_eq!(manager.active_worker_count_for_test(), 1);
    release
        .try_send(())
        .expect("blocked initialization must be released");
    assert!(
        !initialization
            .join()
            .expect("initialization observer must return")
            .initialized
    );
    assert_reaper_eventually_completes(&manager);
    assert!(!manager.read_status().initialized);
}

#[test]
fn shutdown_cancellation_after_interrupt_registration_prevents_late_health() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    let (hook, entered, release) = InitializationTestHook::blocking(
        InitializationTestStage::AfterInterruptRegistration,
        false,
    );
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.initialization_test_hook = hook;
    let initializing = manager.clone();
    let initialization = thread::spawn(move || initializing.initialize_for_test(config));
    entered
        .recv_timeout(Duration::from_secs(1))
        .expect("initialization must pause after interrupt registration");
    assert!(manager.active_initialization_for_test());

    let error = manager
        .shutdown_for_test(Duration::from_millis(50))
        .expect_err("blocked registered initialization must time out closed");
    assert_eq!(error.kind, RuntimeStoreErrorKind::Unavailable);
    assert_eq!(
        manager.worker_join_ownership_for_test(),
        WorkerJoinOwnership::Reaper
    );
    release
        .try_send(())
        .expect("blocked initialization must be released");
    assert!(
        !initialization
            .join()
            .expect("initialization observer must return")
            .initialized
    );
    assert_reaper_eventually_completes(&manager);
    thread::sleep(Duration::from_millis(25));
    assert!(!manager.read_status().initialized);
    assert!(!manager.active_initialization_for_test());
}

#[test]
fn shutdown_cancellation_rolls_back_an_interrupted_migration_transaction() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.initialization_test_hook = InitializationTestHook::LongQueryInsideMigration;
    let initializing = manager.clone();
    let initialization = thread::spawn(move || initializing.initialize_for_test(config));
    assert!(wait_until(Duration::from_secs(1), || {
        manager.active_initialization_for_test()
    }));
    let shutdown = manager.shutdown_for_test(Duration::from_millis(100));
    assert!(
        shutdown.is_ok()
            || shutdown
                .as_ref()
                .is_err_and(|error| error.kind == RuntimeStoreErrorKind::Unavailable),
        "interrupted migration must return a bounded shutdown result: {shutdown:?}"
    );
    assert!(
        !initialization
            .join()
            .expect("initialization observer must return")
            .initialized
    );
    assert_reaper_eventually_completes(&manager);
    assert_interrupted_database_has_no_schema(&root);
}

#[test]
fn shutdown_cancellation_interrupts_integrity_validation_without_schema_damage() {
    let root = TestRoot::new();
    let first = RuntimeStoreManager::new();
    assert!(
        first
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    let migration_before = migration_evidence(&root.database_path());
    first
        .shutdown_for_test(Duration::from_secs(1))
        .expect("fixture store must close");

    let manager = RuntimeStoreManager::new();
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.initialization_test_hook = InitializationTestHook::LongQueryDuringIntegrity;
    let initializing = manager.clone();
    let initialization = thread::spawn(move || initializing.initialize_for_test(config));
    assert!(wait_until(Duration::from_secs(1), || {
        manager.active_initialization_for_test()
    }));
    manager
        .shutdown_for_test(Duration::from_millis(100))
        .expect("integrity query must be interruptible");
    assert!(
        !initialization
            .join()
            .expect("initialization observer must return")
            .initialized
    );
    assert_eq!(migration_evidence(&root.database_path()), migration_before);
    assert_eq!(manager.active_worker_count_for_test(), 0);
}

#[test]
fn shutdown_cancellation_preempts_status_and_ordinary_queue_work() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    let release = manager
        .block_worker_for_test()
        .expect("worker must enter deterministic ordinary work");
    let first = manager
        .try_enqueue_status_for_test()
        .expect("status request must queue");
    let second = manager
        .try_enqueue_status_for_test()
        .expect("second status request must queue");
    let shutting_down = manager.clone();
    let shutdown =
        thread::spawn(move || shutting_down.shutdown_for_test(Duration::from_millis(500)));
    assert!(wait_until(Duration::from_secs(1), || {
        manager.shutdown_requested_for_test()
    }));
    release.try_send(()).expect("ordinary work must release");
    shutdown
        .join()
        .expect("shutdown observer must not panic")
        .expect("priority shutdown must complete");
    assert!(first.recv_timeout(Duration::from_millis(100)).is_err());
    assert!(second.recv_timeout(Duration::from_millis(100)).is_err());
    assert!(!manager.read_status().initialized);
    assert_eq!(
        manager.worker_join_ownership_for_test(),
        WorkerJoinOwnership::Completed
    );
    assert_eq!(manager.active_worker_count_for_test(), 0);
}

#[test]
fn shutdown_cancellation_panic_finalization_keeps_worker_accounted() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    let (hook, entered, release) =
        InitializationTestHook::blocking(InitializationTestStage::AfterInterruptRegistration, true);
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.initialization_test_hook = hook;
    let initializing = manager.clone();
    let initialization = thread::spawn(move || initializing.initialize_for_test(config));
    entered
        .recv_timeout(Duration::from_secs(1))
        .expect("initialization must pause before test panic");
    let shutting_down = manager.clone();
    let shutdown =
        thread::spawn(move || shutting_down.shutdown_for_test(Duration::from_millis(500)));
    assert!(wait_until(Duration::from_secs(1), || {
        manager.shutdown_requested_for_test()
    }));
    release
        .try_send(())
        .expect("cancelled initialization must reach the panic hook");
    assert!(
        !initialization
            .join()
            .expect("initialization observer must return")
            .initialized
    );
    assert!(shutdown
        .join()
        .expect("shutdown observer must return")
        .is_err());
    assert_eq!(manager.worker_exit_for_test(), Some(WorkerExit::Panic));
    assert!(manager.joined_after_exit_for_test());
    assert_eq!(
        manager.worker_join_ownership_for_test(),
        WorkerJoinOwnership::Completed
    );
    assert_eq!(manager.active_worker_count_for_test(), 0);
    assert!(!manager.active_initialization_for_test());
}

#[test]
fn shutdown_cancellation_missing_reply_still_joins_after_exit_proof() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    manager.suppress_shutdown_reply_for_test();
    let error = manager
        .shutdown_for_test(Duration::from_millis(200))
        .expect_err("missing shutdown reply must fail closed");
    assert_eq!(error.kind, RuntimeStoreErrorKind::Unavailable);
    assert!(manager.joined_after_exit_for_test());
    assert_eq!(
        manager.worker_join_ownership_for_test(),
        WorkerJoinOwnership::Completed
    );
    assert_eq!(manager.active_worker_count_for_test(), 0);
}

#[test]
fn shutdown_cancellation_delayed_exit_signal_transfers_to_reaper_then_joins() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    manager.delay_exit_signal_for_test(Duration::from_millis(150));
    let error = manager
        .shutdown_for_test(Duration::from_millis(50))
        .expect_err("delayed exit proof must exhaust the caller deadline");
    assert_eq!(error.kind, RuntimeStoreErrorKind::Unavailable);
    assert_eq!(
        manager.worker_join_ownership_for_test(),
        WorkerJoinOwnership::Reaper
    );
    assert_reaper_eventually_completes(&manager);
}

#[test]
fn shutdown_cancellation_noninterruptible_preopen_stage_has_one_accounted_worker() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    let (hook, entered, release) =
        InitializationTestHook::blocking(InitializationTestStage::BeforePathPreparation, false);
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.initialization_test_hook = hook;
    let initializing = manager.clone();
    let initialization = thread::spawn(move || initializing.initialize_for_test(config));
    entered
        .recv_timeout(Duration::from_secs(1))
        .expect("initialization must pause before path preparation");
    assert_eq!(manager.active_worker_count_for_test(), 1);
    assert!(!manager.active_initialization_for_test());
    let error = manager
        .shutdown_for_test(Duration::from_millis(50))
        .expect_err("pre-open test stage must return a controlled timeout");
    assert_eq!(error.kind, RuntimeStoreErrorKind::Unavailable);
    assert_eq!(
        manager.worker_join_ownership_for_test(),
        WorkerJoinOwnership::Reaper
    );
    assert_eq!(manager.active_worker_count_for_test(), 1);
    assert!(!root.database_path().exists());
    release
        .try_send(())
        .expect("pre-open test stage must be released");
    assert!(
        !initialization
            .join()
            .expect("initialization observer must return")
            .initialized
    );
    assert_reaper_eventually_completes(&manager);
    assert!(!root.database_path().exists());
}

#[test]
fn shutdown_cancellation_test_hooks_are_not_tauri_authority() {
    let root = include_str!("../lib.rs");
    let commands = include_str!("commands.rs");
    let config = include_str!("config.rs");
    assert!(config.contains("#[cfg(test)]"));
    for forbidden in [
        "InitializationTestHook",
        "InitializationTestStage",
        "shutdown_for_test",
        "delay_exit_signal_for_test",
        "suppress_shutdown_reply_for_test",
    ] {
        assert!(
            !root.contains(forbidden),
            "{forbidden} must not be registered"
        );
        assert!(
            !commands.contains(forbidden),
            "{forbidden} must not be public IPC"
        );
    }
}

#[test]
fn successful_initialization_disarms_and_joins_every_watchdog() {
    let before = watchdog_counts();
    for _ in 0..5 {
        let root = TestRoot::new();
        let manager = RuntimeStoreManager::new();
        assert!(
            manager
                .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
                .initialized
        );
        manager
            .shutdown_for_test(Duration::from_secs(1))
            .expect("successful manager must shut down");
    }
    let after = watchdog_counts();
    assert!(after.0 >= before.0 + 10);
    assert!(after.1 >= before.1 + 10);
    assert!(active_watchdogs() <= after.0.saturating_sub(after.1));
}

#[cfg(unix)]
#[test]
fn external_hard_link_at_database_path_is_rejected_without_chmod_or_byte_change() {
    use std::os::unix::fs::PermissionsExt;
    let root = TestRoot::new();
    let outside = TestRoot::new();
    fs::create_dir_all(
        root.database_path()
            .parent()
            .expect("database parent must exist"),
    )
    .expect("database parent must be created");
    let outside_file = outside.path().join("external.sqlite3");
    let bytes = b"external-file-must-remain-unchanged";
    fs::write(&outside_file, bytes).expect("external fixture must be written");
    fs::set_permissions(&outside_file, fs::Permissions::from_mode(0o640))
        .expect("external fixture mode must be set");
    fs::hard_link(&outside_file, root.database_path()).expect("hard link must be created");
    let before_mode = fs::metadata(&outside_file)
        .expect("external metadata must exist")
        .permissions()
        .mode()
        & 0o777;
    let result = RuntimeStoreConnection::open(&RuntimeStoreConfig::for_test(root.path.clone()));
    assert!(result.is_err());
    assert_eq!(
        fs::read(&outside_file).expect("external bytes must read"),
        bytes
    );
    assert_eq!(
        fs::metadata(&outside_file)
            .expect("external metadata must remain")
            .permissions()
            .mode()
            & 0o777,
        before_mode
    );
}

#[cfg(unix)]
#[test]
fn hard_link_created_after_healthy_open_is_detected_on_status_read() {
    let root = TestRoot::new();
    let outside = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    fs::hard_link(root.database_path(), outside.path().join("database-link"))
        .expect("post-open hard link must be created");
    let status = manager.read_status();
    assert!(!status.initialized);
    assert_eq!(status.state, StorageRuntimeState::Unavailable);
    assert_eq!(
        status.error_code,
        Some(StorageRuntimeErrorCode::PathInvalid)
    );
}

#[cfg(unix)]
#[test]
fn hard_linked_wal_and_shm_sidecars_are_rejected() {
    for suffix in ["-wal", "-shm"] {
        let root = TestRoot::new();
        let outside = TestRoot::new();
        let manager = RuntimeStoreManager::new();
        assert!(
            manager
                .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
                .initialized
        );
        let sidecar = sidecar_path_for_test(&root.database_path(), suffix);
        assert!(
            sidecar.exists(),
            "{suffix} sidecar must exist while WAL is open"
        );
        fs::hard_link(&sidecar, outside.path().join(format!("sidecar{suffix}")))
            .expect("sidecar hard link must be created");
        let status = manager.read_status();
        assert!(!status.initialized);
        assert_eq!(
            status.error_code,
            Some(StorageRuntimeErrorCode::PathInvalid)
        );
    }
}

#[cfg(unix)]
#[test]
fn stable_regular_file_identity_matches_only_the_same_single_link_file() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    let first = regular_file_identity_for_test(&root.database_path())
        .expect("first stable identity must read");
    let second = regular_file_identity_for_test(&root.database_path())
        .expect("second stable identity must read");
    assert_eq!(first, second);
    let replacement = root.path().join("replacement.sqlite3");
    fs::write(&replacement, b"replacement").expect("replacement must be written");
    let replacement_identity =
        regular_file_identity_for_test(&replacement).expect("replacement identity must read");
    assert_ne!(first, replacement_identity);
    store.close().expect("store must close");
}

#[cfg(windows)]
#[test]
fn windows_uses_stable_file_identity_and_rejects_multiple_links() {
    let root = TestRoot::new();
    let store = initialized_store(&root);
    let first = regular_file_identity_for_test(&root.database_path())
        .expect("Windows file identity must read");
    let second = regular_file_identity_for_test(&root.database_path())
        .expect("same Windows file identity must reread");
    assert_eq!(first, second);
    let replacement = root.path().join("replacement.sqlite3");
    fs::write(&replacement, b"replacement").expect("replacement must be written");
    assert_ne!(
        first,
        regular_file_identity_for_test(&replacement).expect("replacement identity must read")
    );
    fs::hard_link(root.database_path(), root.path().join("database-hard-link"))
        .expect("Windows hard link must be created");
    assert!(regular_file_identity_for_test(&root.database_path()).is_err());
    drop(store);
}

#[test]
fn non_unix_identity_policy_contains_no_timestamp_or_length_fallback() {
    let source = include_str!("path_policy.rs");
    assert!(!source.contains("metadata.created()"));
    assert!(!source.contains("metadata.modified()"));
    assert!(!source.contains("length: metadata.len()"));
    assert!(source.contains("volume_serial_number"));
    assert!(source.contains("file_index"));
    assert!(source.contains("number_of_links"));
}

#[test]
fn actual_capacity_128_queue_rejects_the_next_request_and_shuts_down_cleanly() {
    let root = TestRoot::new();
    let manager = RuntimeStoreManager::new();
    assert!(
        manager
            .initialize_for_test(RuntimeStoreConfig::for_test(root.path.clone()))
            .initialized
    );
    let release = manager
        .block_worker_for_test()
        .expect("worker must enter deterministic block");
    let mut responses = Vec::with_capacity(STORAGE_QUEUE_CAPACITY);
    for _ in 0..STORAGE_QUEUE_CAPACITY {
        responses.push(
            manager
                .try_enqueue_status_for_test()
                .expect("all 128 queue slots must accept work"),
        );
    }
    let overflow = manager
        .try_enqueue_status_for_test()
        .expect_err("the 129th queued request must be rejected");
    assert_eq!(overflow.kind, RuntimeStoreErrorKind::BusyTimeout);
    release.try_send(()).expect("blocked worker must release");
    for response in responses {
        assert!(
            response
                .recv_timeout(Duration::from_secs(1))
                .expect("queued status response must arrive")
                .initialized
        );
    }
    manager
        .shutdown_for_test(Duration::from_secs(1))
        .expect("manager must shut down without a leaked worker");
    assert!(manager.joined_after_exit_for_test());
}

fn assert_check_constraint(result: rusqlite::Result<usize>, column: &str) {
    match result.expect_err(column) {
        rusqlite::Error::SqliteFailure(error, _) => assert_eq!(
            error.extended_code,
            rusqlite::ffi::SQLITE_CONSTRAINT_CHECK,
            "{column} must fail its lexical CHECK before any foreign-key fallback"
        ),
        other => panic!("{column} returned unexpected error: {other}"),
    }
}

fn wait_for_worker_exit(manager: &RuntimeStoreManager) -> WorkerExit {
    for _ in 0..100 {
        if let Some(exit) = manager.worker_exit_for_test() {
            return exit;
        }
        thread::sleep(Duration::from_millis(5));
    }
    panic!("worker exit must be observed within the bounded test window");
}

fn wait_until(timeout: Duration, predicate: impl Fn() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if predicate() {
            return true;
        }
        thread::sleep(Duration::from_millis(2));
    }
    predicate()
}

fn assert_reaper_eventually_completes(manager: &RuntimeStoreManager) {
    assert!(wait_until(Duration::from_secs(2), || {
        manager.worker_join_ownership_for_test() == WorkerJoinOwnership::Completed
            && manager.reaper_completed_for_test()
            && manager.joined_after_exit_for_test()
            && manager.active_worker_count_for_test() == 0
            && manager.active_watchdog_count_for_test() == 0
    }));
}

fn assert_interrupted_database_has_no_schema(root: &TestRoot) {
    let connection =
        Connection::open(root.database_path()).expect("interrupted database must remain readable");
    let objects: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema
             WHERE type IN ('table', 'index', 'trigger', 'view')",
            [],
            |row| row.get(0),
        )
        .expect("interrupted schema inventory must read");
    assert_eq!(objects, 0);
}

fn migration_evidence(database_path: &Path) -> (i64, String, String, i64) {
    let connection = Connection::open(database_path).expect("evidence database must open");
    connection
        .query_row(
            "SELECT migration_id, name, checksum_sha256, applied_at_ms
             FROM schema_migrations",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("migration evidence must be readable")
}
