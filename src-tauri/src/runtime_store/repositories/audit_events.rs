use crate::runtime_store::error::ContentOperationError;
use crate::runtime_store::models::{
    validate_uuid_v4, AuditCursor, AuditEventRecord, AuditEventType, AuditOutcome, AuditPage,
    AuditReasonCode, AuditSubjectType, ContentActor, ListAuditEventsRequest,
};
use rusqlite::{Connection, OptionalExtension, Row};

pub(super) fn load(
    connection: &Connection,
    event_id: &str,
) -> Result<Option<AuditEventRecord>, ContentOperationError> {
    let raw = connection
        .query_row(
            "SELECT sequence_no, event_id, event_type, actor_type, subject_type,
                    subject_id, outcome, reason_code, correlation_id, created_at_ms
             FROM audit_events
             WHERE event_id = ?1",
            [event_id],
            raw_audit_event,
        )
        .optional()
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    raw.map(validate_record).transpose()
}

pub(super) fn list(
    connection: &Connection,
    request: &ListAuditEventsRequest,
) -> Result<AuditPage, ContentOperationError> {
    let fetch_limit = i64::from(request.limit) + 1;
    let after_sequence_no = request.after_sequence_no.unwrap_or(0);
    let mut statement = connection
        .prepare(
            "SELECT sequence_no, event_id, event_type, actor_type, subject_type,
                    subject_id, outcome, reason_code, correlation_id, created_at_ms
             FROM audit_events
             WHERE sequence_no > ?1
             ORDER BY sequence_no ASC
             LIMIT ?2",
        )
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    let rows = statement
        .query_map((after_sequence_no, fetch_limit), raw_audit_event)
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    let mut records = Vec::new();
    for row in rows {
        records.push(validate_record(
            row.map_err(|error| ContentOperationError::from_sqlite(&error))?,
        )?);
    }

    let has_more = records.len() > request.limit as usize;
    if has_more {
        records.pop();
    }
    let next_cursor = if has_more {
        records.last().map(|record| AuditCursor {
            after_sequence_no: record.sequence_no,
        })
    } else {
        None
    };
    Ok(AuditPage {
        items: records,
        next_cursor,
    })
}

#[derive(Debug)]
struct RawAuditEvent {
    sequence_no: i64,
    event_id: String,
    event_type: String,
    actor_type: String,
    subject_type: String,
    subject_id: Option<String>,
    outcome: String,
    reason_code: Option<String>,
    correlation_id: Option<String>,
    created_at_ms: i64,
}

fn raw_audit_event(row: &Row<'_>) -> rusqlite::Result<RawAuditEvent> {
    Ok(RawAuditEvent {
        sequence_no: row.get(0)?,
        event_id: row.get(1)?,
        event_type: row.get(2)?,
        actor_type: row.get(3)?,
        subject_type: row.get(4)?,
        subject_id: row.get(5)?,
        outcome: row.get(6)?,
        reason_code: row.get(7)?,
        correlation_id: row.get(8)?,
        created_at_ms: row.get(9)?,
    })
}

fn validate_record(raw: RawAuditEvent) -> Result<AuditEventRecord, ContentOperationError> {
    validate_persisted_uuid(&raw.event_id)?;
    if let Some(subject_id) = &raw.subject_id {
        validate_persisted_uuid(subject_id)?;
    }
    if let Some(correlation_id) = &raw.correlation_id {
        validate_persisted_uuid(correlation_id)?;
    }
    if raw.sequence_no <= 0 || raw.created_at_ms < 0 || raw.reason_code.is_some() {
        return Err(ContentOperationError::integrity_failure());
    }

    let event_type = AuditEventType::from_database(&raw.event_type)?;
    let subject_type = AuditSubjectType::from_database(&raw.subject_type)?;
    let subject_id_required = match event_type {
        AuditEventType::ConversationCreated | AuditEventType::ConversationDeleted => {
            require_subject(subject_type, AuditSubjectType::Conversation)?;
            true
        }
        AuditEventType::MessageAppended => {
            require_subject(subject_type, AuditSubjectType::Message)?;
            true
        }
        AuditEventType::TaskCreated
        | AuditEventType::TaskRecorded
        | AuditEventType::TaskDeleted => {
            require_subject(subject_type, AuditSubjectType::Task)?;
            true
        }
        AuditEventType::RuntimeContentDeleted => {
            require_subject(subject_type, AuditSubjectType::Runtime)?;
            false
        }
        AuditEventType::ExportCompleted => {
            require_subject(subject_type, AuditSubjectType::Export)?;
            false
        }
        AuditEventType::StorageRecoveryRequired => {
            require_subject(subject_type, AuditSubjectType::Storage)?;
            false
        }
    };
    if subject_id_required && raw.subject_id.is_none() {
        return Err(ContentOperationError::integrity_failure());
    }

    Ok(AuditEventRecord {
        sequence_no: raw.sequence_no,
        event_id: raw.event_id,
        event_type,
        actor: ContentActor::from_database(&raw.actor_type)?,
        subject_type,
        subject_id: raw.subject_id,
        outcome: AuditOutcome::from_database(&raw.outcome)?,
        reason_code: None::<AuditReasonCode>,
        correlation_id: raw.correlation_id,
        created_at_ms: raw.created_at_ms,
    })
}

fn validate_persisted_uuid(value: &str) -> Result<(), ContentOperationError> {
    validate_uuid_v4(value).map_err(|_| ContentOperationError::integrity_failure())
}

fn require_subject(
    actual: AuditSubjectType,
    expected: AuditSubjectType,
) -> Result<(), ContentOperationError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ContentOperationError::integrity_failure())
    }
}
