mod conversations;
mod messages;
pub(in crate::runtime_store) mod unit_of_work;

use crate::runtime_store::connection::RuntimeStoreConnection;
use crate::runtime_store::deadline::ensure_before;
use crate::runtime_store::error::ContentOperationError;
use crate::runtime_store::models::{
    AppendMessageRequest, ConversationPage, ConversationRecord, ConversationStatus,
    CreateConversationRequest, GetConversationRequest, ListConversationsRequest,
    ListMessagesRequest, MessagePage, MessageRecord,
};
use rusqlite::TransactionBehavior;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
#[cfg(test)]
use unit_of_work::GrowthObservation;
use unit_of_work::{Admission, AuditEvent, MutationKind};
use uuid::Uuid;

#[derive(Debug)]
pub(super) struct MutationExecution<T> {
    pub(super) record: T,
    #[cfg(test)]
    pub(super) growth: GrowthObservation,
}

pub(super) fn create_conversation(
    store: &mut RuntimeStoreConnection,
    request: &CreateConversationRequest,
    deadline: Instant,
) -> Result<MutationExecution<ConversationRecord>, ContentOperationError> {
    request.validate()?;
    ensure_store_readable(store, deadline)?;
    let mut checkpoint_attempted = false;

    loop {
        ensure_content_before(deadline)?;
        let paths = store.paths.clone();
        let hard_limit_bytes = store.database_hard_limit_bytes;
        let limits = store.content_limits;
        let transaction = store
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| ContentOperationError::from_sqlite(&error))?;

        if let Some(audit) = unit_of_work::load_audit(&transaction, &request.operation_id)? {
            let replay = replay_create(&transaction, request, audit)?;
            transaction
                .rollback()
                .map_err(|error| ContentOperationError::from_sqlite(&error))?;
            return Ok(MutationExecution {
                record: replay,
                #[cfg(test)]
                growth: zero_growth(store)?,
            });
        }

        let admission = unit_of_work::admit(
            &transaction,
            &paths,
            hard_limit_bytes,
            limits,
            MutationKind::CreateConversation,
            deadline,
        )?;
        let snapshot = match admission {
            Admission::Allowed(snapshot) => snapshot,
            Admission::CheckpointRequired { oversized } => {
                transaction
                    .rollback()
                    .map_err(|error| ContentOperationError::from_sqlite(&error))?;
                if checkpoint_attempted {
                    return Err(ContentOperationError::capacity_exceeded());
                }
                checkpoint_attempted = true;
                if oversized {
                    unit_of_work::recover_oversized_wal(store, deadline)?;
                } else {
                    unit_of_work::passive_checkpoint(store, deadline)?;
                }
                continue;
            }
        };

        let id = Uuid::new_v4().to_string();
        let timestamp_ms = current_time_ms()?;
        conversations::insert(&transaction, &id, request.title.as_deref(), timestamp_ms)?;
        unit_of_work::insert_success_audit(
            &transaction,
            &request.operation_id,
            "conversation.created",
            request.actor.as_str(),
            "conversation",
            &id,
            timestamp_ms,
        )?;
        verify_exact_counts(&transaction, "conversations", &id, &request.operation_id)?;
        unit_of_work::verify_precommit(
            &transaction,
            snapshot,
            limits,
            MutationKind::CreateConversation,
            deadline,
        )?;
        transaction
            .commit()
            .map_err(|error| ContentOperationError::from_sqlite(&error))?;

        let record = ConversationRecord {
            id,
            title: request.title.clone(),
            status: ConversationStatus::Active,
            created_at_ms: timestamp_ms,
            updated_at_ms: timestamp_ms,
            next_message_sequence: 1,
            revision: 0,
        };
        let growth =
            unit_of_work::observe_postcommit(store, snapshot, MutationKind::CreateConversation)?;
        #[cfg(not(test))]
        let _ = growth;
        return Ok(MutationExecution {
            record,
            #[cfg(test)]
            growth,
        });
    }
}

pub(super) fn get_conversation(
    store: &mut RuntimeStoreConnection,
    request: &GetConversationRequest,
    deadline: Instant,
) -> Result<ConversationRecord, ContentOperationError> {
    request.validate()?;
    ensure_store_readable(store, deadline)?;
    conversations::load(&store.connection, &request.conversation_id)?
        .ok_or_else(ContentOperationError::conversation_not_found)
}

pub(super) fn list_conversations(
    store: &mut RuntimeStoreConnection,
    request: &ListConversationsRequest,
    deadline: Instant,
) -> Result<ConversationPage, ContentOperationError> {
    request.validate()?;
    ensure_store_readable(store, deadline)?;
    conversations::list(&store.connection, request)
}

pub(super) fn append_message(
    store: &mut RuntimeStoreConnection,
    request: &AppendMessageRequest,
    deadline: Instant,
) -> Result<MutationExecution<MessageRecord>, ContentOperationError> {
    request.validate()?;
    ensure_store_readable(store, deadline)?;
    let mut checkpoint_attempted = false;

    loop {
        ensure_content_before(deadline)?;
        let paths = store.paths.clone();
        let hard_limit_bytes = store.database_hard_limit_bytes;
        let limits = store.content_limits;
        let transaction = store
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| ContentOperationError::from_sqlite(&error))?;

        if let Some(audit) = unit_of_work::load_audit(&transaction, &request.operation_id)? {
            let replay = replay_append(&transaction, request, audit)?;
            transaction
                .rollback()
                .map_err(|error| ContentOperationError::from_sqlite(&error))?;
            return Ok(MutationExecution {
                record: replay,
                #[cfg(test)]
                growth: zero_growth(store)?,
            });
        }

        let conversation = conversations::load(&transaction, &request.conversation_id)?
            .ok_or_else(ContentOperationError::conversation_not_found)?;
        let admission = unit_of_work::admit(
            &transaction,
            &paths,
            hard_limit_bytes,
            limits,
            MutationKind::AppendMessage,
            deadline,
        )?;
        let snapshot = match admission {
            Admission::Allowed(snapshot) => snapshot,
            Admission::CheckpointRequired { oversized } => {
                transaction
                    .rollback()
                    .map_err(|error| ContentOperationError::from_sqlite(&error))?;
                if checkpoint_attempted {
                    return Err(ContentOperationError::capacity_exceeded());
                }
                checkpoint_attempted = true;
                if oversized {
                    unit_of_work::recover_oversized_wal(store, deadline)?;
                } else {
                    unit_of_work::passive_checkpoint(store, deadline)?;
                }
                continue;
            }
        };

        let timestamp_ms = current_time_ms()?.max(
            conversation
                .updated_at_ms
                .checked_add(1)
                .ok_or_else(ContentOperationError::integrity_failure)?,
        );
        let record = MessageRecord {
            id: Uuid::new_v4().to_string(),
            conversation_id: request.conversation_id.clone(),
            sequence_no: conversation.next_message_sequence,
            role: request.role,
            content: request.content.clone(),
            created_at_ms: timestamp_ms,
        };
        messages::insert(&transaction, &record)?;
        conversations::update_after_append(
            &transaction,
            &request.conversation_id,
            conversation.next_message_sequence,
            timestamp_ms,
        )?;
        unit_of_work::insert_success_audit(
            &transaction,
            &request.operation_id,
            "message.appended",
            request.actor.as_str(),
            "message",
            &record.id,
            timestamp_ms,
        )?;
        verify_exact_counts(&transaction, "messages", &record.id, &request.operation_id)?;
        unit_of_work::verify_precommit(
            &transaction,
            snapshot,
            limits,
            MutationKind::AppendMessage,
            deadline,
        )?;
        transaction
            .commit()
            .map_err(|error| ContentOperationError::from_sqlite(&error))?;
        let growth =
            unit_of_work::observe_postcommit(store, snapshot, MutationKind::AppendMessage)?;
        #[cfg(not(test))]
        let _ = growth;
        return Ok(MutationExecution {
            record,
            #[cfg(test)]
            growth,
        });
    }
}

pub(super) fn list_messages(
    store: &mut RuntimeStoreConnection,
    request: &ListMessagesRequest,
    deadline: Instant,
) -> Result<MessagePage, ContentOperationError> {
    request.validate()?;
    ensure_store_readable(store, deadline)?;
    if conversations::load(&store.connection, &request.conversation_id)?.is_none() {
        return Err(ContentOperationError::conversation_not_found());
    }
    messages::list(&store.connection, request)
}

fn replay_create(
    connection: &rusqlite::Connection,
    request: &CreateConversationRequest,
    audit: AuditEvent,
) -> Result<ConversationRecord, ContentOperationError> {
    validate_common_audit(&audit, &request.operation_id)?;
    if audit.event_type == "message.appended" {
        validate_other_mutation_audit(&audit, "message")?;
        return Err(ContentOperationError::idempotency_conflict());
    }
    if audit.event_type != "conversation.created" || audit.subject_type != "conversation" {
        return Err(ContentOperationError::idempotency_inconsistent());
    }
    if audit.actor_type != request.actor.as_str() {
        return Err(ContentOperationError::idempotency_inconsistent());
    }
    let subject_id = audit
        .subject_id
        .as_deref()
        .ok_or_else(ContentOperationError::idempotency_inconsistent)?;
    let persisted = conversations::load(connection, subject_id)?
        .ok_or_else(ContentOperationError::idempotency_inconsistent)?;
    if persisted.created_at_ms != audit.created_at_ms
        || persisted.status != ConversationStatus::Active
    {
        return Err(ContentOperationError::idempotency_inconsistent());
    }
    if persisted.title != request.title {
        return Err(ContentOperationError::idempotency_conflict());
    }
    Ok(ConversationRecord {
        id: persisted.id,
        title: persisted.title,
        status: ConversationStatus::Active,
        created_at_ms: persisted.created_at_ms,
        updated_at_ms: persisted.created_at_ms,
        next_message_sequence: 1,
        revision: 0,
    })
}

fn replay_append(
    connection: &rusqlite::Connection,
    request: &AppendMessageRequest,
    audit: AuditEvent,
) -> Result<MessageRecord, ContentOperationError> {
    validate_common_audit(&audit, &request.operation_id)?;
    if audit.event_type == "conversation.created" {
        validate_other_mutation_audit(&audit, "conversation")?;
        return Err(ContentOperationError::idempotency_conflict());
    }
    if audit.event_type != "message.appended" || audit.subject_type != "message" {
        return Err(ContentOperationError::idempotency_inconsistent());
    }
    let subject_id = audit
        .subject_id
        .as_deref()
        .ok_or_else(ContentOperationError::idempotency_inconsistent)?;
    let persisted = messages::load(connection, subject_id)?
        .ok_or_else(ContentOperationError::idempotency_inconsistent)?;
    if persisted.created_at_ms != audit.created_at_ms {
        return Err(ContentOperationError::idempotency_inconsistent());
    }
    if audit.actor_type != request.actor.as_str()
        || persisted.conversation_id != request.conversation_id
        || persisted.role != request.role
        || persisted.content != request.content
    {
        return Err(ContentOperationError::idempotency_conflict());
    }
    Ok(persisted)
}

fn validate_common_audit(
    audit: &AuditEvent,
    operation_id: &str,
) -> Result<(), ContentOperationError> {
    if audit.outcome != "success"
        || audit.reason_code.is_some()
        || audit.correlation_id.as_deref() != Some(operation_id)
        || audit.subject_id.is_none()
        || !matches!(audit.actor_type.as_str(), "user" | "local_runtime")
        || audit.created_at_ms < 0
    {
        return Err(ContentOperationError::idempotency_inconsistent());
    }
    Ok(())
}

fn validate_other_mutation_audit(
    audit: &AuditEvent,
    expected_subject_type: &str,
) -> Result<(), ContentOperationError> {
    if audit.subject_type == expected_subject_type {
        Ok(())
    } else {
        Err(ContentOperationError::idempotency_inconsistent())
    }
}

fn verify_exact_counts(
    connection: &rusqlite::Connection,
    subject_table: &str,
    subject_id: &str,
    operation_id: &str,
) -> Result<(), ContentOperationError> {
    let subject_sql = match subject_table {
        "conversations" => "SELECT COUNT(*) FROM conversations WHERE id = ?1",
        "messages" => "SELECT COUNT(*) FROM messages WHERE id = ?1",
        _ => return Err(ContentOperationError::internal()),
    };
    let subject_count: i64 = connection
        .query_row(subject_sql, [subject_id], |row| row.get(0))
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    let audit_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE event_id = ?1",
            [operation_id],
            |row| row.get(0),
        )
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    if subject_count != 1 || audit_count != 1 {
        return Err(ContentOperationError::integrity_failure());
    }
    Ok(())
}

fn ensure_store_readable(
    store: &RuntimeStoreConnection,
    deadline: Instant,
) -> Result<(), ContentOperationError> {
    ensure_content_before(deadline)?;
    if store.content_integrity_failed {
        return Err(ContentOperationError::integrity_failure());
    }
    store
        .revalidate_artifacts()
        .map_err(ContentOperationError::from_runtime)
}

fn ensure_content_before(deadline: Instant) -> Result<(), ContentOperationError> {
    ensure_before(deadline).map_err(ContentOperationError::from_runtime)
}

fn current_time_ms() -> Result<i64, ContentOperationError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ContentOperationError::internal())?;
    i64::try_from(elapsed.as_millis()).map_err(|_| ContentOperationError::internal())
}

#[cfg(test)]
fn zero_growth(store: &RuntimeStoreConnection) -> Result<GrowthObservation, ContentOperationError> {
    let sizes = unit_of_work::artifact_sizes_for_test(&store.paths.database_path)?;
    Ok(GrowthObservation {
        page_size_bytes: u64::from(store.content_limits.required_page_size_bytes),
        before: sizes,
        after: sizes,
        aggregate_growth_bytes: 0,
        wal_growth_bytes: 0,
        aggregate_bound_bytes: 0,
        wal_bound_bytes: 0,
    })
}
