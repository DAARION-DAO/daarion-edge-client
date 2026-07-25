use crate::runtime_store::config::ContentStorageLimits;
use crate::runtime_store::connection::RuntimeStoreConnection;
use crate::runtime_store::deadline::{ensure_before, remaining};
use crate::runtime_store::error::{ContentOperationError, ContentOperationErrorCode};
use crate::runtime_store::migrations::{schema_fingerprint, EXPECTED_SCHEMA_FINGERPRINT};
use crate::runtime_store::models::{
    AuditEventRecord, AuditEventType, AuditOutcome, AuditSubjectType, ContentActor,
};
use crate::runtime_store::path_policy::{
    database_artifact_sizes, enforce_sidecar_permissions, revalidate_database,
    PreparedStoragePaths, StorageArtifactSizes,
};
use rusqlite::Connection;
#[cfg(test)]
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime_store) enum MutationKind {
    CreateConversation,
    AppendMessage,
    RecordInertTask,
}

impl MutationKind {
    fn aggregate_growth_bound(self, limits: ContentStorageLimits) -> u64 {
        match self {
            Self::CreateConversation => limits.create_growth_envelope_bytes,
            Self::AppendMessage => limits.append_growth_envelope_bytes,
            Self::RecordInertTask => limits.task_record_growth_envelope_bytes,
        }
    }

    fn wal_growth_bound(self, limits: ContentStorageLimits) -> u64 {
        match self {
            Self::CreateConversation => limits.wal_create_growth_bound_bytes,
            Self::AppendMessage => limits.wal_append_growth_bound_bytes,
            Self::RecordInertTask => limits.wal_task_record_growth_bound_bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SuccessAuditEvent<'a> {
    ConversationCreated {
        event_id: &'a str,
        actor: ContentActor,
        subject_id: &'a str,
        created_at_ms: i64,
    },
    MessageAppended {
        event_id: &'a str,
        actor: ContentActor,
        subject_id: &'a str,
        created_at_ms: i64,
    },
    TaskRecorded {
        event_id: &'a str,
        actor: ContentActor,
        subject_id: &'a str,
        created_at_ms: i64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime_store) struct GrowthObservation {
    pub(in crate::runtime_store) page_size_bytes: u64,
    pub(in crate::runtime_store) before: StorageArtifactSizes,
    pub(in crate::runtime_store) after: StorageArtifactSizes,
    pub(in crate::runtime_store) aggregate_growth_bytes: u64,
    pub(in crate::runtime_store) wal_growth_bytes: u64,
    pub(in crate::runtime_store) aggregate_bound_bytes: u64,
    pub(in crate::runtime_store) wal_bound_bytes: u64,
}

#[derive(Debug)]
pub(in crate::runtime_store) enum Admission {
    Allowed(AdmissionSnapshot),
    CheckpointRequired { oversized: bool },
}

#[derive(Clone, Copy, Debug)]
pub(in crate::runtime_store) struct AdmissionSnapshot {
    before: StorageArtifactSizes,
    page_count: u64,
    page_size: u64,
}

pub(super) fn load_audit(
    connection: &Connection,
    event_id: &str,
) -> Result<Option<AuditEventRecord>, ContentOperationError> {
    super::audit_events::load(connection, event_id).map_err(|error| {
        if error.code == ContentOperationErrorCode::IntegrityFailure {
            ContentOperationError::idempotency_inconsistent()
        } else {
            error
        }
    })
}

pub(super) fn insert_success_audit(
    connection: &Connection,
    event: SuccessAuditEvent<'_>,
) -> Result<(), ContentOperationError> {
    let (event_id, event_type, actor, subject_type, subject_id, outcome, created_at_ms) =
        match event {
            SuccessAuditEvent::ConversationCreated {
                event_id,
                actor,
                subject_id,
                created_at_ms,
            } => (
                event_id,
                AuditEventType::ConversationCreated,
                actor,
                AuditSubjectType::Conversation,
                subject_id,
                AuditOutcome::Success,
                created_at_ms,
            ),
            SuccessAuditEvent::MessageAppended {
                event_id,
                actor,
                subject_id,
                created_at_ms,
            } => (
                event_id,
                AuditEventType::MessageAppended,
                actor,
                AuditSubjectType::Message,
                subject_id,
                AuditOutcome::Success,
                created_at_ms,
            ),
            SuccessAuditEvent::TaskRecorded {
                event_id,
                actor,
                subject_id,
                created_at_ms,
            } => (
                event_id,
                AuditEventType::TaskRecorded,
                actor,
                AuditSubjectType::Task,
                subject_id,
                AuditOutcome::Success,
                created_at_ms,
            ),
        };
    connection
        .execute(
            "INSERT INTO audit_events (
                 event_id, event_type, actor_type, subject_type, subject_id,
                 outcome, reason_code, correlation_id, created_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?1, ?7)",
            (
                event_id,
                event_type.as_str(),
                actor.as_str(),
                subject_type.as_str(),
                subject_id,
                outcome.as_str(),
                created_at_ms,
            ),
        )
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    Ok(())
}

pub(super) fn recover_oversized_wal(
    store: &mut RuntimeStoreConnection,
    deadline: Instant,
) -> Result<(), ContentOperationError> {
    ensure_content_before(deadline)?;
    store
        .revalidate_artifacts()
        .map_err(ContentOperationError::from_runtime)?;
    let before = database_artifact_sizes(&store.paths.database_path)
        .map_err(ContentOperationError::from_runtime)?;
    if before.wal_bytes <= store.content_limits.wal_hard_ceiling_bytes {
        return Ok(());
    }

    let total = before
        .total_bytes()
        .map_err(ContentOperationError::from_runtime)?;
    let recovery_projection = total
        .checked_add(before.wal_bytes)
        .and_then(|value| {
            value.checked_add(store.content_limits.checkpoint_recovery_overhead_bytes)
        })
        .ok_or_else(ContentOperationError::capacity_exceeded)?;
    if recovery_projection > store.database_hard_limit_bytes {
        return Err(ContentOperationError::capacity_exceeded());
    }

    let (busy, _, _) = checkpoint(&store.connection, "TRUNCATE", deadline, store.busy_timeout)?;
    if busy != 0 {
        return Err(ContentOperationError::capacity_exceeded());
    }
    enforce_sidecar_permissions(&store.paths.database_path)
        .map_err(ContentOperationError::from_runtime)?;
    revalidate_database(&store.paths).map_err(ContentOperationError::from_runtime)?;
    let after = database_artifact_sizes(&store.paths.database_path)
        .map_err(ContentOperationError::from_runtime)?;
    let usable_limit = ordinary_usable_limit(
        store.database_hard_limit_bytes,
        store.content_limits.operational_reserve_bytes,
    )?;
    if after
        .total_bytes()
        .map_err(ContentOperationError::from_runtime)?
        > usable_limit
        || after.wal_bytes > store.content_limits.wal_hard_ceiling_bytes
    {
        return Err(ContentOperationError::capacity_exceeded());
    }
    Ok(())
}

pub(in crate::runtime_store) fn admit(
    connection: &Connection,
    paths: &PreparedStoragePaths,
    hard_limit_bytes: u64,
    limits: ContentStorageLimits,
    kind: MutationKind,
    deadline: Instant,
) -> Result<Admission, ContentOperationError> {
    ensure_content_before(deadline)?;
    revalidate_database(paths).map_err(ContentOperationError::from_runtime)?;
    enforce_sidecar_permissions(&paths.database_path)
        .map_err(ContentOperationError::from_runtime)?;
    let before = database_artifact_sizes(&paths.database_path)
        .map_err(ContentOperationError::from_runtime)?;
    let usable_limit = ordinary_usable_limit(hard_limit_bytes, limits.operational_reserve_bytes)?;
    let projected_total = before
        .total_bytes()
        .map_err(ContentOperationError::from_runtime)?
        .checked_add(kind.aggregate_growth_bound(limits))
        .ok_or_else(ContentOperationError::capacity_exceeded)?;
    if projected_total > usable_limit {
        return Err(ContentOperationError::capacity_exceeded());
    }

    let projected_wal = before
        .wal_bytes
        .checked_add(kind.wal_growth_bound(limits))
        .ok_or_else(ContentOperationError::capacity_exceeded)?;
    if projected_wal > limits.wal_hard_ceiling_bytes {
        return Ok(Admission::CheckpointRequired {
            oversized: before.wal_bytes > limits.wal_hard_ceiling_bytes,
        });
    }

    let page_size = pragma_u64(connection, "page_size")?;
    if page_size != u64::from(limits.required_page_size_bytes) {
        return Err(ContentOperationError::integrity_failure());
    }
    let wal_autocheckpoint = pragma_u64(connection, "wal_autocheckpoint")?;
    if wal_autocheckpoint != u64::from(limits.wal_autocheckpoint_pages) {
        return Err(ContentOperationError::integrity_failure());
    }
    Ok(Admission::Allowed(AdmissionSnapshot {
        before,
        page_count: pragma_u64(connection, "page_count")?,
        page_size,
    }))
}

pub(super) fn passive_checkpoint(
    store: &RuntimeStoreConnection,
    deadline: Instant,
) -> Result<(), ContentOperationError> {
    let _ = checkpoint(&store.connection, "PASSIVE", deadline, store.busy_timeout)?;
    store
        .revalidate_artifacts()
        .map_err(ContentOperationError::from_runtime)
}

pub(super) fn verify_precommit(
    connection: &Connection,
    snapshot: AdmissionSnapshot,
    limits: ContentStorageLimits,
    kind: MutationKind,
    deadline: Instant,
) -> Result<(), ContentOperationError> {
    ensure_content_before(deadline)?;
    if schema_fingerprint(connection).map_err(ContentOperationError::from_runtime)?
        != EXPECTED_SCHEMA_FINGERPRINT
    {
        return Err(ContentOperationError::integrity_failure());
    }
    let page_size = pragma_u64(connection, "page_size")?;
    if page_size != snapshot.page_size || page_size != u64::from(limits.required_page_size_bytes) {
        return Err(ContentOperationError::integrity_failure());
    }
    let page_count = pragma_u64(connection, "page_count")?;
    let page_growth = page_count
        .saturating_sub(snapshot.page_count)
        .checked_mul(page_size)
        .ok_or_else(ContentOperationError::integrity_failure)?;
    if page_growth > kind.aggregate_growth_bound(limits) {
        return Err(ContentOperationError::integrity_failure());
    }
    let wal_projection = if page_growth == 0 {
        0
    } else {
        page_count
            .saturating_sub(snapshot.page_count)
            .checked_mul(page_size.saturating_add(24))
            .and_then(|value| value.checked_add(32))
            .ok_or_else(ContentOperationError::integrity_failure)?
    };
    if wal_projection > kind.wal_growth_bound(limits) {
        return Err(ContentOperationError::integrity_failure());
    }
    Ok(())
}

pub(super) fn observe_postcommit(
    store: &mut RuntimeStoreConnection,
    snapshot: AdmissionSnapshot,
    kind: MutationKind,
) -> Result<GrowthObservation, ContentOperationError> {
    store
        .revalidate_artifacts()
        .map_err(ContentOperationError::from_runtime)?;
    let after = database_artifact_sizes(&store.paths.database_path)
        .map_err(ContentOperationError::from_runtime)?;
    let before_total = snapshot
        .before
        .total_bytes()
        .map_err(ContentOperationError::from_runtime)?;
    let after_total = after
        .total_bytes()
        .map_err(ContentOperationError::from_runtime)?;
    let observation = GrowthObservation {
        page_size_bytes: snapshot.page_size,
        before: snapshot.before,
        after,
        aggregate_growth_bytes: after_total.saturating_sub(before_total),
        wal_growth_bytes: after.wal_bytes.saturating_sub(snapshot.before.wal_bytes),
        aggregate_bound_bytes: kind.aggregate_growth_bound(store.content_limits),
        wal_bound_bytes: kind.wal_growth_bound(store.content_limits),
    };
    if observation.aggregate_growth_bytes > observation.aggregate_bound_bytes
        || observation.wal_growth_bytes > observation.wal_bound_bytes
    {
        store.content_integrity_failed = true;
    }
    Ok(observation)
}

fn ordinary_usable_limit(
    hard_limit_bytes: u64,
    operational_reserve_bytes: u64,
) -> Result<u64, ContentOperationError> {
    hard_limit_bytes
        .checked_sub(operational_reserve_bytes)
        .ok_or_else(ContentOperationError::integrity_failure)
}

fn checkpoint(
    connection: &Connection,
    mode: &str,
    deadline: Instant,
    configured_busy_timeout: std::time::Duration,
) -> Result<(i64, i64, i64), ContentOperationError> {
    ensure_content_before(deadline)?;
    let budget = remaining(deadline).map_err(ContentOperationError::from_runtime)?;
    connection
        .busy_timeout(budget.min(configured_busy_timeout))
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    let sql = match mode {
        "PASSIVE" => "PRAGMA wal_checkpoint(PASSIVE)",
        "TRUNCATE" => "PRAGMA wal_checkpoint(TRUNCATE)",
        _ => return Err(ContentOperationError::internal()),
    };
    let result = connection
        .query_row(sql, [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|error| ContentOperationError::from_sqlite(&error));
    let restore = connection
        .busy_timeout(configured_busy_timeout)
        .map_err(|error| ContentOperationError::from_sqlite(&error));
    match (result, restore) {
        (Ok(value), Ok(())) => {
            ensure_content_before(deadline)?;
            Ok(value)
        }
        (Err(error), _) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

fn pragma_u64(connection: &Connection, name: &str) -> Result<u64, ContentOperationError> {
    let value: i64 = connection
        .pragma_query_value(None, name, |row| row.get(0))
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    u64::try_from(value).map_err(|_| ContentOperationError::integrity_failure())
}

fn ensure_content_before(deadline: Instant) -> Result<(), ContentOperationError> {
    ensure_before(deadline).map_err(ContentOperationError::from_runtime)
}

#[cfg(test)]
pub(super) fn artifact_sizes_for_test(
    database_path: &Path,
) -> Result<StorageArtifactSizes, ContentOperationError> {
    database_artifact_sizes(database_path).map_err(ContentOperationError::from_runtime)
}
