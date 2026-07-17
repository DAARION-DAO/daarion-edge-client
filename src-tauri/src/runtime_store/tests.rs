use super::config::RuntimeStoreConfig;
use super::config::STORAGE_QUEUE_CAPACITY;
use super::connection::RuntimeStoreConnection;
use super::error::RuntimeStoreErrorKind;
use super::migrations::{migrate_and_validate, CURRENT_SCHEMA_VERSION};
use super::types::{
    DatabaseHealth, PersistenceState, StorageRuntimeErrorCode, StorageRuntimeState,
};
use super::worker::RuntimeStoreManager;
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
fn status_read_has_a_bounded_deadline_when_worker_is_busy() {
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
    let fallback = manager.read_status();
    assert!(started_at.elapsed() < Duration::from_millis(120));
    assert!(fallback.initialized);
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
