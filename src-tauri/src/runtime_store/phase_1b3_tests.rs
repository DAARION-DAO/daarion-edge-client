use super::config::{
    OPERATIONAL_RESERVE_BYTES, TASK_RECORD_GROWTH_ENVELOPE_BYTES,
    WAL_TASK_RECORD_GROWTH_BOUND_BYTES,
};
use super::connection::RuntimeStoreConnection;
use super::error::ContentOperationErrorCode;
use super::migrations::{migrate_and_validate, schema_fingerprint, EXPECTED_SCHEMA_FINGERPRINT};
use super::models::{
    AppendMessageRequest, AuditEventType, AuditOutcome, AuditSubjectType, ContentActor,
    CreateConversationRequest, GetAuditEventRequest, GetTaskRequest, InertTaskKind, InertTaskState,
    ListAuditEventsRequest, ListTasksRequest, MessageRole, RecordInertTaskRequest, TaskCursor,
    MAX_TASK_KIND_BYTES,
};
use super::path_policy::database_artifact_sizes;
use super::repositories;
use super::types::StorageRuntimeState;
use super::{RuntimeStoreConfig, RuntimeStoreManager};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

struct Phase1b3Root {
    path: PathBuf,
}

impl Phase1b3Root {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("daarion-phase-1b3-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).expect("Phase 1B.3 test root must be created");
        let path = fs::canonicalize(path).expect("Phase 1B.3 test root must canonicalize");
        Self { path }
    }

    fn database_path(&self) -> PathBuf {
        self.path
            .join("runtime-state")
            .join("runtime-state-v1.sqlite3")
    }
}

impl Drop for Phase1b3Root {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn initialized_manager(root: &Phase1b3Root) -> RuntimeStoreManager {
    initialized_manager_with(root, RuntimeStoreConfig::for_test(root.path.clone()))
}

fn initialized_manager_with(
    _root: &Phase1b3Root,
    config: RuntimeStoreConfig,
) -> RuntimeStoreManager {
    let manager = RuntimeStoreManager::new();
    let status = manager.initialize_for_test(config);
    assert!(
        status.initialized,
        "Phase 1B.3 manager must initialize: {status:?}"
    );
    manager
}

fn initialized_connection(root: &Phase1b3Root) -> RuntimeStoreConnection {
    let config = RuntimeStoreConfig::for_test(root.path.clone());
    let mut store = RuntimeStoreConnection::open(&config).expect("Phase 1B.3 connection must open");
    migrate_and_validate(&mut store.connection).expect("Phase 1B.3 schema must migrate");
    store
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(2)
}

fn task_kind(value: &str) -> InertTaskKind {
    InertTaskKind::new(value.to_string()).expect("test task kind must be valid")
}

fn task_request(
    operation_id: String,
    actor: ContentActor,
    conversation_id: Option<String>,
    kind: &str,
) -> RecordInertTaskRequest {
    RecordInertTaskRequest {
        operation_id,
        actor,
        conversation_id,
        task_kind: task_kind(kind),
    }
}

fn fresh_task_request(kind: &str) -> RecordInertTaskRequest {
    task_request(
        Uuid::new_v4().to_string(),
        ContentActor::LocalRuntime,
        None,
        kind,
    )
}

fn create_conversation(
    manager: &RuntimeStoreManager,
    operation_id: String,
) -> super::models::ConversationRecord {
    manager
        .create_conversation(CreateConversationRequest {
            operation_id,
            actor: ContentActor::User,
            title: Some("Phase 1B.3 parent".to_string()),
        })
        .expect("parent conversation must create")
}

fn count(database_path: &Path, table: &str) -> i64 {
    let sql = match table {
        "conversations" => "SELECT COUNT(*) FROM conversations",
        "messages" => "SELECT COUNT(*) FROM messages",
        "tasks" => "SELECT COUNT(*) FROM tasks",
        "audit_events" => "SELECT COUNT(*) FROM audit_events",
        _ => panic!("unsupported Phase 1B.3 test table"),
    };
    Connection::open(database_path)
        .expect("count reader must open")
        .query_row(sql, [], |row| row.get(0))
        .expect("count must be readable")
}

fn audit_count(database_path: &Path, operation_id: &str) -> i64 {
    Connection::open(database_path)
        .expect("audit count reader must open")
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE event_id = ?1",
            [operation_id],
            |row| row.get(0),
        )
        .expect("audit count must be readable")
}

fn assert_runtime_accepting(manager: &RuntimeStoreManager) {
    let status = manager.read_status();
    assert!(status.initialized, "runtime must remain initialized");
    assert!(matches!(
        status.state,
        StorageRuntimeState::Healthy | StorageRuntimeState::Warning
    ));
}

#[allow(clippy::too_many_arguments)]
fn insert_audit_fixture(
    connection: &Connection,
    event_type: &str,
    actor_type: &str,
    subject_type: &str,
    subject_id: Option<&str>,
    outcome: &str,
    reason_code: Option<&str>,
    correlation_id: Option<&str>,
    created_at_ms: i64,
) -> String {
    let event_id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO audit_events (
                 event_id, event_type, actor_type, subject_type, subject_id,
                 outcome, reason_code, correlation_id, created_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (
                &event_id,
                event_type,
                actor_type,
                subject_type,
                subject_id,
                outcome,
                reason_code,
                correlation_id,
                created_at_ms,
            ),
        )
        .expect("audit fixture must insert");
    event_id
}

#[test]
fn task_kind_accepts_exact_canonical_grammar_and_byte_bounds() {
    let maximum = format!("a{}", "0".repeat(MAX_TASK_KIND_BYTES - 1));
    for valid in [
        "a",
        "task",
        "task1",
        "task.kind",
        "task_kind",
        "task-kind",
        "a1.b2_c3-d4",
        maximum.as_str(),
    ] {
        let parsed =
            InertTaskKind::new(valid.to_string()).expect("canonical task kind must succeed");
        assert_eq!(parsed.as_str(), valid);
    }
}

#[test]
fn task_kind_rejects_noncanonical_or_instruction_shaped_values() {
    let oversized = format!("a{}", "0".repeat(MAX_TASK_KIND_BYTES));
    for invalid in [
        "",
        "A",
        "Task",
        "1task",
        ".task",
        "_task",
        "-task",
        "task.",
        "task_",
        "task-",
        "task..kind",
        "task__kind",
        "task--kind",
        "task.-kind",
        "task kind",
        "task/kind",
        "task:kind",
        "task;rm",
        "task\nkind",
        "тask",
        oversized.as_str(),
    ] {
        assert!(
            InertTaskKind::new(invalid.to_string()).is_err(),
            "invalid task kind must fail: {invalid:?}"
        );
    }
}

#[test]
fn record_unlinked_task_is_inert_and_audited_without_kind_leakage() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let request = fresh_task_request("local.health_check");
    let record = manager
        .record_inert_task(request.clone())
        .expect("inert task must record");
    assert_eq!(record.conversation_id, None);
    assert_eq!(record.task_kind, request.task_kind);
    assert_eq!(record.state, InertTaskState::Created);
    assert_eq!(record.created_at_ms, record.updated_at_ms);
    assert_eq!(record.revision, 0);

    let connection = Connection::open(root.database_path()).expect("verification reader must open");
    let idempotency_key: Option<String> = connection
        .query_row(
            "SELECT idempotency_key FROM tasks WHERE id = ?1",
            [&record.id],
            |row| row.get(0),
        )
        .expect("task idempotency must read");
    assert_eq!(idempotency_key, None);
    let audit = manager
        .get_audit_event(GetAuditEventRequest {
            event_id: request.operation_id.clone(),
        })
        .expect("task audit must read");
    assert_eq!(audit.event_type, AuditEventType::TaskRecorded);
    assert_eq!(audit.actor, request.actor);
    assert_eq!(audit.subject_type, AuditSubjectType::Task);
    assert_eq!(audit.subject_id.as_deref(), Some(record.id.as_str()));
    assert_eq!(audit.outcome, AuditOutcome::Success);
    assert_eq!(audit.reason_code, None);
    assert_eq!(
        audit.correlation_id.as_deref(),
        Some(request.operation_id.as_str())
    );
    assert_eq!(audit.created_at_ms, record.created_at_ms);
    let audit_text = format!("{audit:?}");
    assert!(!audit_text.contains(request.task_kind.as_str()));
}

#[test]
fn linked_task_requires_an_existing_conversation_and_missing_parent_is_local() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let missing_request = task_request(
        Uuid::new_v4().to_string(),
        ContentActor::User,
        Some(Uuid::new_v4().to_string()),
        "linked.task",
    );
    let missing = manager
        .record_inert_task(missing_request.clone())
        .expect_err("missing parent must fail");
    assert_eq!(
        missing.code,
        ContentOperationErrorCode::ConversationNotFound
    );
    assert_eq!(
        audit_count(&root.database_path(), &missing_request.operation_id),
        0
    );
    assert_runtime_accepting(&manager);

    let conversation = create_conversation(&manager, Uuid::new_v4().to_string());
    let linked = manager
        .record_inert_task(task_request(
            Uuid::new_v4().to_string(),
            ContentActor::User,
            Some(conversation.id.clone()),
            "linked.task",
        ))
        .expect("linked task must record");
    assert_eq!(
        linked.conversation_id.as_deref(),
        Some(conversation.id.as_str())
    );
}

#[test]
fn task_replay_returns_original_and_changed_canonical_request_conflicts() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let conversation = create_conversation(&manager, Uuid::new_v4().to_string());
    let operation_id = Uuid::new_v4().to_string();
    let request = task_request(
        operation_id.clone(),
        ContentActor::User,
        Some(conversation.id.clone()),
        "stable.task",
    );
    let original = manager
        .record_inert_task(request.clone())
        .expect("first task record must succeed");
    assert_eq!(
        manager
            .record_inert_task(request.clone())
            .expect("same request must replay"),
        original
    );
    for conflict in [
        task_request(
            operation_id.clone(),
            ContentActor::LocalRuntime,
            Some(conversation.id.clone()),
            "stable.task",
        ),
        task_request(
            operation_id.clone(),
            ContentActor::User,
            None,
            "stable.task",
        ),
        task_request(
            operation_id.clone(),
            ContentActor::User,
            Some(conversation.id.clone()),
            "different.task",
        ),
    ] {
        assert_eq!(
            manager
                .record_inert_task(conflict)
                .expect_err("changed replay must conflict")
                .code,
            ContentOperationErrorCode::IdempotencyConflict
        );
    }
    assert_eq!(count(&root.database_path(), "tasks"), 1);
    assert_eq!(audit_count(&root.database_path(), &operation_id), 1);
}

#[test]
fn operation_ids_are_global_across_all_three_mutations() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let conversation_operation = Uuid::new_v4().to_string();
    let conversation = create_conversation(&manager, conversation_operation.clone());
    assert_eq!(
        manager
            .record_inert_task(task_request(
                conversation_operation,
                ContentActor::User,
                None,
                "conflict.conversation",
            ))
            .expect_err("conversation operation ID must conflict")
            .code,
        ContentOperationErrorCode::IdempotencyConflict
    );

    let message_operation = Uuid::new_v4().to_string();
    manager
        .append_message(AppendMessageRequest {
            operation_id: message_operation.clone(),
            actor: ContentActor::LocalRuntime,
            conversation_id: conversation.id,
            role: MessageRole::Assistant,
            content: "synthetic content".to_string(),
        })
        .expect("message must append");
    assert_eq!(
        manager
            .record_inert_task(task_request(
                message_operation,
                ContentActor::LocalRuntime,
                None,
                "conflict.message",
            ))
            .expect_err("message operation ID must conflict")
            .code,
        ContentOperationErrorCode::IdempotencyConflict
    );
    assert_eq!(count(&root.database_path(), "tasks"), 0);
}

#[test]
fn concurrent_duplicate_task_request_commits_one_task_and_event() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let request = fresh_task_request("concurrent.task");
    let left = {
        let manager = manager.clone();
        let request = request.clone();
        thread::spawn(move || manager.record_inert_task(request))
    };
    let right = {
        let manager = manager.clone();
        let request = request.clone();
        thread::spawn(move || manager.record_inert_task(request))
    };
    let left = left
        .join()
        .expect("left caller must not panic")
        .expect("left caller must succeed");
    let right = right
        .join()
        .expect("right caller must not panic")
        .expect("right caller must replay");
    assert_eq!(left, right);
    assert_eq!(count(&root.database_path(), "tasks"), 1);
    assert_eq!(audit_count(&root.database_path(), &request.operation_id), 1);
}

#[test]
fn task_replay_survives_clean_restart_without_duplicate_rows() {
    let root = Phase1b3Root::new();
    let request = fresh_task_request("restart.task");
    let first = initialized_manager(&root);
    let original = first
        .record_inert_task(request.clone())
        .expect("task must record");
    first
        .shutdown_for_test(Duration::from_secs(2))
        .expect("first manager must shut down");
    drop(first);

    let second = initialized_manager(&root);
    assert_eq!(
        second
            .record_inert_task(request.clone())
            .expect("replay after restart must succeed"),
        original
    );
    assert_eq!(count(&root.database_path(), "tasks"), 1);
    assert_eq!(audit_count(&root.database_path(), &request.operation_id), 1);
}

#[test]
fn injected_audit_failure_rolls_back_task_and_leaves_operation_reusable() {
    let root = Phase1b3Root::new();
    let mut store = initialized_connection(&root);
    store
        .connection
        .execute_batch(
            "CREATE TEMP TRIGGER fail_task_audit
             BEFORE INSERT ON audit_events
             WHEN NEW.event_type = 'task.recorded'
             BEGIN
               SELECT RAISE(FAIL, 'synthetic');
             END;",
        )
        .expect("test-only audit trigger must install");
    let request = fresh_task_request("atomic.audit");
    repositories::record_inert_task(&mut store, &request, deadline())
        .expect_err("forced audit failure must fail");
    assert_eq!(count(&root.database_path(), "tasks"), 0);
    assert_eq!(audit_count(&root.database_path(), &request.operation_id), 0);
    store
        .connection
        .execute_batch("DROP TRIGGER fail_task_audit;")
        .expect("test-only trigger must drop");
    repositories::record_inert_task(&mut store, &request, deadline())
        .expect("same operation must remain reusable");
}

#[test]
fn injected_task_failure_creates_no_audit_event() {
    let root = Phase1b3Root::new();
    let mut store = initialized_connection(&root);
    store
        .connection
        .execute_batch(
            "CREATE TEMP TRIGGER fail_task_insert
             BEFORE INSERT ON tasks
             BEGIN
               SELECT RAISE(FAIL, 'synthetic');
             END;",
        )
        .expect("test-only task trigger must install");
    let request = fresh_task_request("atomic.task");
    repositories::record_inert_task(&mut store, &request, deadline())
        .expect_err("forced task failure must fail");
    assert_eq!(count(&root.database_path(), "tasks"), 0);
    assert_eq!(audit_count(&root.database_path(), &request.operation_id), 0);
}

#[test]
fn task_get_and_global_list_are_bounded_stable_and_complete() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let mut recorded = Vec::new();
    for kind in ["list.one", "list.two", "list.three"] {
        recorded.push(
            manager
                .record_inert_task(fresh_task_request(kind))
                .expect("task must record"),
        );
    }
    let exact = manager
        .get_task(GetTaskRequest {
            task_id: recorded[1].id.clone(),
        })
        .expect("exact task must read");
    assert_eq!(exact, recorded[1]);
    assert_eq!(
        manager
            .get_task(GetTaskRequest {
                task_id: Uuid::new_v4().to_string(),
            })
            .expect_err("missing task must be truthful")
            .code,
        ContentOperationErrorCode::TaskNotFound
    );
    assert_runtime_accepting(&manager);

    let first = manager
        .list_tasks(ListTasksRequest {
            limit: 1,
            cursor: None,
        })
        .expect("first task page must read");
    assert_eq!(first.items.len(), 1);
    let second = manager
        .list_tasks(ListTasksRequest {
            limit: 100,
            cursor: first.next_cursor.clone(),
        })
        .expect("second task page must read");
    assert_eq!(second.items.len(), 2);
    let mut all = first.items;
    all.extend(second.items);
    for pair in all.windows(2) {
        assert!(
            (pair[0].updated_at_ms, pair[0].id.as_str())
                > (pair[1].updated_at_ms, pair[1].id.as_str())
        );
    }
    let ids = all
        .into_iter()
        .map(|item| item.id)
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(ids.len(), 3);
}

#[test]
fn task_list_tie_uses_full_descending_cursor_without_gaps() {
    let root = Phase1b3Root::new();
    let mut store = initialized_connection(&root);
    let mut ids = vec![
        "11111111-1111-4111-8111-111111111111".to_string(),
        "22222222-2222-4222-8222-222222222222".to_string(),
        "33333333-3333-4333-8333-333333333333".to_string(),
    ];
    for id in &ids {
        store
            .connection
            .execute(
                "INSERT INTO tasks (
                     id, conversation_id, task_kind, state, idempotency_key,
                     created_at_ms, updated_at_ms, revision
                 ) VALUES (?1, NULL, 'tie.task', 'created', NULL, 7, 7, 0)",
                [id],
            )
            .expect("tie fixture must insert");
    }
    ids.sort_by(|left, right| right.cmp(left));
    let first = repositories::list_tasks(
        &mut store,
        &ListTasksRequest {
            limit: 2,
            cursor: None,
        },
        deadline(),
    )
    .expect("first tie page must read");
    assert_eq!(
        first.items.iter().map(|item| &item.id).collect::<Vec<_>>(),
        ids[..2].iter().collect::<Vec<_>>()
    );
    let second = repositories::list_tasks(
        &mut store,
        &ListTasksRequest {
            limit: 2,
            cursor: first.next_cursor,
        },
        deadline(),
    )
    .expect("second tie page must read");
    assert_eq!(second.items.len(), 1);
    assert_eq!(second.items[0].id, ids[2]);
}

#[test]
fn task_and_cursor_validation_rejects_all_invalid_bounds_locally() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    for limit in [0, 101] {
        assert_eq!(
            manager
                .list_tasks(ListTasksRequest {
                    limit,
                    cursor: None,
                })
                .expect_err("invalid task limit must fail")
                .code,
            ContentOperationErrorCode::InvalidInput
        );
    }
    for cursor in [
        TaskCursor {
            updated_at_ms: -1,
            id: Uuid::new_v4().to_string(),
        },
        TaskCursor {
            updated_at_ms: 0,
            id: "not-a-uuid".to_string(),
        },
    ] {
        assert_eq!(
            manager
                .list_tasks(ListTasksRequest {
                    limit: 1,
                    cursor: Some(cursor),
                })
                .expect_err("invalid task cursor must fail")
                .code,
            ContentOperationErrorCode::InvalidInput
        );
    }
    assert_runtime_accepting(&manager);
}

#[test]
fn malformed_persisted_task_invariants_fail_closed_and_poison_intake() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let request = fresh_task_request("integrity.task");
    let task = manager
        .record_inert_task(request)
        .expect("task must record");
    Connection::open(root.database_path())
        .expect("fixture writer must open")
        .execute(
            "UPDATE tasks SET idempotency_key = 'unexpected' WHERE id = ?1",
            [&task.id],
        )
        .expect("fixture must corrupt semantic task invariant");
    assert_eq!(
        manager
            .get_task(GetTaskRequest { task_id: task.id })
            .expect_err("non-NULL task idempotency must fail closed")
            .code,
        ContentOperationErrorCode::IntegrityFailure
    );
    assert!(!manager.read_status().initialized);
}

#[test]
fn task_replay_maps_inconsistent_task_evidence_to_idempotency_inconsistent() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let request = fresh_task_request("replay.integrity");
    let task = manager
        .record_inert_task(request.clone())
        .expect("task must record");
    Connection::open(root.database_path())
        .expect("fixture writer must open")
        .execute(
            "UPDATE tasks SET idempotency_key = 'unexpected' WHERE id = ?1",
            [&task.id],
        )
        .expect("fixture must corrupt replay evidence");
    assert_eq!(
        manager
            .record_inert_task(request)
            .expect_err("inconsistent replay must fail closed")
            .code,
        ContentOperationErrorCode::IdempotencyRecordInconsistent
    );
}

#[test]
fn audit_exact_and_sequence_reads_are_bounded_and_stable() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let conversation_operation = Uuid::new_v4().to_string();
    let conversation = create_conversation(&manager, conversation_operation.clone());
    let message_operation = Uuid::new_v4().to_string();
    manager
        .append_message(AppendMessageRequest {
            operation_id: message_operation.clone(),
            actor: ContentActor::LocalRuntime,
            conversation_id: conversation.id,
            role: MessageRole::Assistant,
            content: "synthetic".to_string(),
        })
        .expect("message must append");
    let task_operation = Uuid::new_v4().to_string();
    manager
        .record_inert_task(task_request(
            task_operation.clone(),
            ContentActor::LocalRuntime,
            None,
            "audit.task",
        ))
        .expect("task must record");

    assert_eq!(
        manager
            .get_audit_event(GetAuditEventRequest {
                event_id: task_operation.clone(),
            })
            .expect("exact audit must read")
            .event_type,
        AuditEventType::TaskRecorded
    );
    assert_eq!(
        manager
            .get_audit_event(GetAuditEventRequest {
                event_id: Uuid::new_v4().to_string(),
            })
            .expect_err("missing audit must be truthful")
            .code,
        ContentOperationErrorCode::AuditEventNotFound
    );
    assert_runtime_accepting(&manager);

    let first = manager
        .list_audit_events(ListAuditEventsRequest {
            limit: 2,
            after_sequence_no: None,
        })
        .expect("first audit page must read");
    assert_eq!(first.items.len(), 2);
    let second = manager
        .list_audit_events(ListAuditEventsRequest {
            limit: 2,
            after_sequence_no: first.next_cursor.map(|cursor| cursor.after_sequence_no),
        })
        .expect("second audit page must read");
    assert_eq!(second.items.len(), 1);
    let sequences = first
        .items
        .iter()
        .chain(second.items.iter())
        .map(|event| event.sequence_no)
        .collect::<Vec<_>>();
    assert!(sequences.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn audit_decoder_covers_every_migration_event_and_closed_outcome() {
    let root = Phase1b3Root::new();
    let mut store = initialized_connection(&root);
    let subject_id = Uuid::new_v4().to_string();
    let fixtures = [
        (
            "conversation.created",
            "conversation",
            Some(subject_id.as_str()),
            "success",
            AuditEventType::ConversationCreated,
            AuditOutcome::Success,
        ),
        (
            "conversation.deleted",
            "conversation",
            Some(subject_id.as_str()),
            "denied",
            AuditEventType::ConversationDeleted,
            AuditOutcome::Denied,
        ),
        (
            "message.appended",
            "message",
            Some(subject_id.as_str()),
            "failed",
            AuditEventType::MessageAppended,
            AuditOutcome::Failed,
        ),
        (
            "task.created",
            "task",
            Some(subject_id.as_str()),
            "success",
            AuditEventType::TaskCreated,
            AuditOutcome::Success,
        ),
        (
            "task.recorded",
            "task",
            Some(subject_id.as_str()),
            "success",
            AuditEventType::TaskRecorded,
            AuditOutcome::Success,
        ),
        (
            "task.deleted",
            "task",
            Some(subject_id.as_str()),
            "success",
            AuditEventType::TaskDeleted,
            AuditOutcome::Success,
        ),
        (
            "runtime.content_deleted",
            "runtime",
            None,
            "success",
            AuditEventType::RuntimeContentDeleted,
            AuditOutcome::Success,
        ),
        (
            "export.completed",
            "export",
            None,
            "success",
            AuditEventType::ExportCompleted,
            AuditOutcome::Success,
        ),
        (
            "storage.recovery_required",
            "storage",
            None,
            "success",
            AuditEventType::StorageRecoveryRequired,
            AuditOutcome::Success,
        ),
    ];
    for (event, subject, subject_id, outcome, expected_event, expected_outcome) in fixtures {
        let event_id = insert_audit_fixture(
            &store.connection,
            event,
            "local_runtime",
            subject,
            subject_id,
            outcome,
            None,
            None,
            1,
        );
        let decoded = repositories::get_audit_event(
            &mut store,
            &GetAuditEventRequest { event_id },
            deadline(),
        )
        .expect("reserved typed audit value must decode");
        assert_eq!(decoded.event_type, expected_event);
        assert_eq!(decoded.outcome, expected_outcome);
    }
}

#[test]
fn audit_decoder_rejects_semantic_pair_missing_subject_reason_and_bad_numbers() {
    let cases = [
        ("conversation.created", "task", Some("subject"), None, 1_i64),
        ("task.recorded", "task", None, None, 1_i64),
        (
            "storage.recovery_required",
            "storage",
            None,
            Some("arbitrary"),
            1_i64,
        ),
        ("runtime.content_deleted", "runtime", None, None, -1_i64),
    ];
    for (event_type, subject_type, subject_mode, reason, created_at_ms) in cases {
        let root = Phase1b3Root::new();
        let mut store = initialized_connection(&root);
        let subject_id = Uuid::new_v4().to_string();
        let event_id = insert_audit_fixture(
            &store.connection,
            event_type,
            "user",
            subject_type,
            subject_mode.map(|_| subject_id.as_str()),
            "success",
            reason,
            None,
            created_at_ms,
        );
        assert_eq!(
            repositories::get_audit_event(
                &mut store,
                &GetAuditEventRequest { event_id },
                deadline(),
            )
            .expect_err("semantically invalid audit must fail closed")
            .code,
            ContentOperationErrorCode::IntegrityFailure
        );
    }
}

#[test]
fn audit_decoder_rejects_unknown_closed_values() {
    for (column, value) in [
        ("event_type", "unknown.event"),
        ("actor_type", "remote"),
        ("subject_type", "unknown"),
        ("outcome", "unknown"),
    ] {
        let root = Phase1b3Root::new();
        let mut store = initialized_connection(&root);
        store
            .connection
            .pragma_update(None, "ignore_check_constraints", true)
            .expect("test fixture must temporarily bypass SQL checks");
        let event_id = Uuid::new_v4().to_string();
        let subject_id = Uuid::new_v4().to_string();
        store
            .connection
            .execute_batch("BEGIN IMMEDIATE;")
            .expect("fixture transaction must begin");
        store
            .connection
            .execute(
                "INSERT INTO audit_events (
                     event_id, event_type, actor_type, subject_type, subject_id,
                     outcome, reason_code, correlation_id, created_at_ms
                 ) VALUES (?1, 'task.recorded', 'user', 'task', ?2,
                           'success', NULL, ?1, 1)",
                (&event_id, &subject_id),
            )
            .expect("base audit fixture must insert");
        store
            .connection
            .execute(
                &format!("UPDATE audit_events SET {column} = ?1 WHERE event_id = ?2"),
                (value, &event_id),
            )
            .expect("unknown value fixture must update");
        store
            .connection
            .execute_batch("COMMIT;")
            .expect("fixture transaction must commit");
        assert_eq!(
            repositories::get_audit_event(
                &mut store,
                &GetAuditEventRequest { event_id },
                deadline(),
            )
            .expect_err("unknown typed value must fail closed")
            .code,
            ContentOperationErrorCode::IntegrityFailure
        );
    }
}

#[test]
fn audit_sequence_and_limit_validation_is_operation_local() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    for request in [
        ListAuditEventsRequest {
            limit: 0,
            after_sequence_no: None,
        },
        ListAuditEventsRequest {
            limit: 101,
            after_sequence_no: None,
        },
        ListAuditEventsRequest {
            limit: 1,
            after_sequence_no: Some(0),
        },
        ListAuditEventsRequest {
            limit: 1,
            after_sequence_no: Some(-1),
        },
    ] {
        assert_eq!(
            manager
                .list_audit_events(request)
                .expect_err("invalid audit read request must fail")
                .code,
            ContentOperationErrorCode::InvalidInput
        );
    }
    assert_runtime_accepting(&manager);
}

#[test]
fn not_found_codes_are_safe_and_do_not_poison_health() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let task_error = manager
        .get_task(GetTaskRequest {
            task_id: Uuid::new_v4().to_string(),
        })
        .expect_err("missing task must fail");
    let audit_error = manager
        .get_audit_event(GetAuditEventRequest {
            event_id: Uuid::new_v4().to_string(),
        })
        .expect_err("missing audit must fail");
    assert_eq!(task_error.to_string(), "content_task_not_found");
    assert_eq!(audit_error.to_string(), "content_audit_event_not_found");
    assert!(!task_error.poisons_content_intake());
    assert!(!audit_error.poisons_content_intake());
    manager
        .record_inert_task(fresh_task_request("after.not_found"))
        .expect("later valid task must succeed");
    assert_runtime_accepting(&manager);
}

#[test]
fn task_capacity_constants_and_schema_invariants_are_exact() {
    let root = Phase1b3Root::new();
    let config = RuntimeStoreConfig::for_test(root.path.clone());
    assert_eq!(
        config.content_limits.task_record_growth_envelope_bytes,
        8_388_608
    );
    assert_eq!(
        config.content_limits.wal_task_record_growth_bound_bytes,
        2_097_152
    );
    let store = initialized_connection(&root);
    assert_eq!(
        schema_fingerprint(&store.connection).expect("fingerprint must compute"),
        EXPECTED_SCHEMA_FINGERPRINT
    );
    assert_eq!(count(&root.database_path(), "tasks"), 0);
}

#[test]
fn task_capacity_rejection_is_atomic_retryable_and_preserves_reads() {
    let root = Phase1b3Root::new();
    let first = initialized_manager(&root);
    let existing = first
        .record_inert_task(fresh_task_request("existing.task"))
        .expect("existing task must record");
    first
        .shutdown_for_test(Duration::from_secs(2))
        .expect("first manager must shut down");
    drop(first);

    let mut limited_config = RuntimeStoreConfig::for_test(root.path.clone());
    limited_config.database_hard_limit_bytes =
        OPERATIONAL_RESERVE_BYTES + TASK_RECORD_GROWTH_ENVELOPE_BYTES;
    let limited = initialized_manager_with(&root, limited_config);
    let rejected = fresh_task_request("capacity.task");
    assert_eq!(
        limited
            .record_inert_task(rejected.clone())
            .expect_err("task must preserve reserve")
            .code,
        ContentOperationErrorCode::CapacityExceeded
    );
    assert_eq!(
        audit_count(&root.database_path(), &rejected.operation_id),
        0
    );
    assert_eq!(
        limited
            .get_task(GetTaskRequest {
                task_id: existing.id.clone(),
            })
            .expect("existing read must remain available"),
        existing
    );
    limited
        .shutdown_for_test(Duration::from_secs(2))
        .expect("limited manager must shut down");
    drop(limited);

    let normal = initialized_manager(&root);
    normal
        .record_inert_task(rejected.clone())
        .expect("rejected operation ID must remain reusable");
    assert_eq!(
        audit_count(&root.database_path(), &rejected.operation_id),
        1
    );
}

#[test]
fn task_growth_proof_passes_ten_unlinked_and_ten_linked_fresh_roots() {
    let maximum_kind = format!("a{}", "0".repeat(MAX_TASK_KIND_BYTES - 1));
    let mut max_aggregate = 0;
    let mut max_wal = 0;
    for run in 0..20 {
        let root = Phase1b3Root::new();
        let mut store = initialized_connection(&root);
        let conversation_id = if run % 2 == 0 {
            None
        } else {
            Some(
                repositories::create_conversation(
                    &mut store,
                    &CreateConversationRequest {
                        operation_id: Uuid::new_v4().to_string(),
                        actor: ContentActor::User,
                        title: Some("growth parent".to_string()),
                    },
                    deadline(),
                )
                .expect("growth parent must create")
                .record
                .id,
            )
        };
        let execution = repositories::record_inert_task(
            &mut store,
            &task_request(
                Uuid::new_v4().to_string(),
                ContentActor::LocalRuntime,
                conversation_id,
                &maximum_kind,
            ),
            deadline(),
        )
        .expect("task growth proof must pass");
        assert_eq!(execution.growth.page_size_bytes, 4096);
        assert!(execution.growth.aggregate_growth_bytes <= execution.growth.aggregate_bound_bytes);
        assert!(execution.growth.wal_growth_bytes <= execution.growth.wal_bound_bytes);
        max_aggregate = max_aggregate.max(execution.growth.aggregate_growth_bytes);
        max_wal = max_wal.max(execution.growth.wal_growth_bytes);
        eprintln!(
            "task-growth run={run} linked={} before={:?} after={:?} aggregate={} wal={}",
            execution.record.conversation_id.is_some(),
            execution.growth.before,
            execution.growth.after,
            execution.growth.aggregate_growth_bytes,
            execution.growth.wal_growth_bytes
        );
    }
    eprintln!("task-growth maxima aggregate={max_aggregate} wal={max_wal}");
    assert!(max_aggregate <= TASK_RECORD_GROWTH_ENVELOPE_BYTES);
    assert!(max_wal <= WAL_TASK_RECORD_GROWTH_BOUND_BYTES);
}

#[test]
fn competing_writer_failure_is_local_and_same_task_operation_retries() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let blocker = Connection::open(root.database_path()).expect("writer fixture must open");
    blocker
        .execute_batch("BEGIN IMMEDIATE;")
        .expect("writer fixture must own lock");
    let request = fresh_task_request("writer.retry");
    assert_eq!(
        manager
            .record_inert_task(request.clone())
            .expect_err("competing writer must time out")
            .code,
        ContentOperationErrorCode::BusyTimeout
    );
    assert_runtime_accepting(&manager);
    blocker
        .execute_batch("ROLLBACK;")
        .expect("writer fixture must release lock");
    manager
        .record_inert_task(request)
        .expect("same operation must retry");
}

#[test]
fn queued_task_and_post_shutdown_audit_read_are_rejected() {
    let root = Phase1b3Root::new();
    let manager = initialized_manager(&root);
    let release = manager
        .block_worker_for_test()
        .expect("worker fixture must block");
    let request = fresh_task_request("queued.task");
    let caller = {
        let manager = manager.clone();
        let request = request.clone();
        thread::spawn(move || manager.record_inert_task(request))
    };
    thread::sleep(Duration::from_millis(20));
    let shutdown = {
        let manager = manager.clone();
        thread::spawn(move || manager.shutdown_for_test(Duration::from_secs(2)))
    };
    for _ in 0..100 {
        if manager.shutdown_requested_for_test() {
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    assert!(manager.shutdown_requested_for_test());
    release.send(()).expect("worker fixture must release");
    shutdown
        .join()
        .expect("shutdown caller must not panic")
        .expect("shutdown must complete");
    assert_eq!(
        caller
            .join()
            .expect("task caller must not panic")
            .expect_err("queued task must not begin")
            .code,
        ContentOperationErrorCode::Unavailable
    );
    assert_eq!(count(&root.database_path(), "tasks"), 0);
    assert_eq!(audit_count(&root.database_path(), &request.operation_id), 0);
    assert_eq!(
        manager
            .get_audit_event(GetAuditEventRequest {
                event_id: Uuid::new_v4().to_string(),
            })
            .expect_err("audit intake must stay closed")
            .code,
        ContentOperationErrorCode::Unavailable
    );
}

#[test]
fn active_task_during_shutdown_commits_both_rows_or_neither() {
    let root = Phase1b3Root::new();
    let mut config = RuntimeStoreConfig::for_test(root.path.clone());
    config.busy_timeout = Duration::from_secs(1);
    config.ordinary_deadline = Duration::from_secs(1);
    let manager = initialized_manager_with(&root, config);
    let writer = Connection::open(root.database_path()).expect("writer fixture must open");
    writer
        .execute_batch("BEGIN IMMEDIATE;")
        .expect("writer fixture must own lock");
    let request = fresh_task_request("shutdown.active");
    let caller = {
        let manager = manager.clone();
        let request = request.clone();
        thread::spawn(move || manager.record_inert_task(request))
    };
    thread::sleep(Duration::from_millis(30));
    let shutdown = {
        let manager = manager.clone();
        thread::spawn(move || manager.shutdown_for_test(Duration::from_secs(2)))
    };
    for _ in 0..100 {
        if manager.shutdown_requested_for_test() {
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    writer
        .execute_batch("ROLLBACK;")
        .expect("writer fixture must release");
    let outcome = caller.join().expect("task caller must not panic");
    shutdown
        .join()
        .expect("shutdown caller must not panic")
        .expect("shutdown must complete");
    match outcome {
        Ok(_) => {
            assert_eq!(count(&root.database_path(), "tasks"), 1);
            assert_eq!(audit_count(&root.database_path(), &request.operation_id), 1);
        }
        Err(error) => {
            assert!(matches!(
                error.code,
                ContentOperationErrorCode::Unavailable
                    | ContentOperationErrorCode::DeadlineExceeded
                    | ContentOperationErrorCode::BusyTimeout
            ));
            assert_eq!(count(&root.database_path(), "tasks"), 0);
            assert_eq!(audit_count(&root.database_path(), &request.operation_id), 0);
        }
    }
}

#[test]
fn typed_audit_writer_has_three_closed_variants_and_no_stringly_authority() {
    let source = include_str!("repositories/unit_of_work.rs");
    for variant in [
        "ConversationCreated {",
        "MessageAppended {",
        "TaskRecorded {",
    ] {
        assert_eq!(
            source.matches(variant).count(),
            2,
            "each closed variant must appear once in the enum and once in its match arm"
        );
    }
    assert!(!source.contains("event_type: &str"));
    assert!(!source.contains("subject_type: &str"));
    assert!(!source.contains("outcome: &str"));
    assert!(!source.contains("pub(crate) fn insert_success_audit"));
    assert!(!source.contains("pub fn insert_success_audit"));
}

#[test]
fn public_task_and_audit_authority_remains_zero() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib = fs::read_to_string(crate_root.join("src/lib.rs")).expect("lib.rs must read");
    let commands = fs::read_to_string(crate_root.join("src/runtime_store/commands.rs"))
        .expect("storage commands must read");
    for name in [
        "record_inert_task",
        "get_task",
        "list_tasks",
        "get_audit_event",
        "list_audit_events",
    ] {
        assert!(
            !lib.contains(name),
            "{name} must not enter Tauri composition"
        );
        assert!(
            !commands.contains(name),
            "{name} must not enter Tauri commands"
        );
    }

    let frontend_root = crate_root
        .parent()
        .expect("crate must have repository parent")
        .join("src");
    let mut stack = vec![frontend_root];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(path).expect("frontend directory must read") {
            let entry = entry.expect("frontend entry must read");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if !matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("ts" | "tsx")
            ) {
                continue;
            }
            let source = fs::read_to_string(&path).expect("frontend source must read");
            for name in [
                "record_inert_task",
                "get_task",
                "list_tasks",
                "get_audit_event",
                "list_audit_events",
            ] {
                assert!(!source.contains(name), "{name} leaked into {path:?}");
            }
        }
    }
}

#[test]
fn schema_dependency_and_physical_artifact_inventory_remain_unchanged() {
    let root = Phase1b3Root::new();
    let store = initialized_connection(&root);
    assert_eq!(
        schema_fingerprint(&store.connection).expect("fingerprint must compute"),
        EXPECTED_SCHEMA_FINGERPRINT
    );
    let tables: i64 = store
        .connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .expect("table inventory must read");
    assert_eq!(tables, 5);
    let sqlite_sequence: i64 = store
        .connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema WHERE name = 'sqlite_sequence'",
            [],
            |row| row.get(0),
        )
        .expect("sqlite_sequence inventory must read");
    assert_eq!(sqlite_sequence, 0);
    let sizes = database_artifact_sizes(&root.database_path()).expect("artifact sizes must read");
    assert_eq!(
        sizes.total_bytes().expect("artifact total must fit"),
        sizes.database_bytes + sizes.wal_bytes + sizes.shm_bytes
    );
}

#[test]
fn task_operations_never_interpret_or_execute_task_kind() {
    let source = include_str!("repositories/tasks.rs");
    let orchestration = include_str!("repositories/mod.rs");
    for forbidden in [
        "Command::new",
        "std::process",
        "reqwest",
        "invoke(",
        "dispatch",
        "schedule",
        "execute_task",
        "tool_call",
    ] {
        assert!(!source.contains(forbidden));
        assert!(!orchestration.contains(forbidden));
    }
}
