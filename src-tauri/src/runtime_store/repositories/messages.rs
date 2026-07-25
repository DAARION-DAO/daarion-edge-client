use crate::runtime_store::error::ContentOperationError;
use crate::runtime_store::models::{
    validate_uuid_v4, ListMessagesRequest, MessagePage, MessageRecord, MessageRole,
    MAX_MESSAGE_CONTENT_BYTES,
};
use rusqlite::{Connection, OptionalExtension, Row};

pub(super) fn insert(
    connection: &Connection,
    record: &MessageRecord,
) -> Result<(), ContentOperationError> {
    connection
        .execute(
            "INSERT INTO messages (
                 id, conversation_id, sequence_no, role, content, created_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &record.id,
                &record.conversation_id,
                record.sequence_no,
                record.role.as_str(),
                &record.content,
                record.created_at_ms,
            ),
        )
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    Ok(())
}

pub(super) fn load(
    connection: &Connection,
    message_id: &str,
) -> Result<Option<MessageRecord>, ContentOperationError> {
    let raw = connection
        .query_row(
            "SELECT id, conversation_id, sequence_no, role, content, created_at_ms
             FROM messages
             WHERE id = ?1",
            [message_id],
            raw_message,
        )
        .optional()
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    raw.map(validate_record).transpose()
}

pub(super) fn list(
    connection: &Connection,
    request: &ListMessagesRequest,
) -> Result<MessagePage, ContentOperationError> {
    let fetch_limit = i64::from(request.limit) + 1;
    let after_sequence_no = request.after_sequence_no.unwrap_or(0);
    let mut statement = connection
        .prepare(
            "SELECT id, conversation_id, sequence_no, role, content, created_at_ms
             FROM messages
             WHERE conversation_id = ?1
               AND sequence_no > ?2
             ORDER BY sequence_no ASC, id ASC
             LIMIT ?3",
        )
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    let rows = statement
        .query_map(
            (&request.conversation_id, after_sequence_no, fetch_limit),
            raw_message,
        )
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    let mut records = Vec::new();
    for row in rows {
        let raw = row.map_err(|error| ContentOperationError::from_sqlite(&error))?;
        let record = validate_record(raw)?;
        if record.conversation_id != request.conversation_id {
            return Err(ContentOperationError::integrity_failure());
        }
        records.push(record);
    }
    let has_more = records.len() > request.limit as usize;
    if has_more {
        records.pop();
    }
    let next_after_sequence_no = if has_more {
        records.last().map(|record| record.sequence_no)
    } else {
        None
    };
    Ok(MessagePage {
        items: records,
        next_after_sequence_no,
    })
}

#[derive(Debug)]
struct RawMessage {
    id: String,
    conversation_id: String,
    sequence_no: i64,
    role: String,
    content: String,
    created_at_ms: i64,
}

fn raw_message(row: &Row<'_>) -> rusqlite::Result<RawMessage> {
    Ok(RawMessage {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        sequence_no: row.get(2)?,
        role: row.get(3)?,
        content: row.get(4)?,
        created_at_ms: row.get(5)?,
    })
}

fn validate_record(raw: RawMessage) -> Result<MessageRecord, ContentOperationError> {
    validate_uuid_v4(&raw.id).map_err(|_| ContentOperationError::integrity_failure())?;
    validate_uuid_v4(&raw.conversation_id)
        .map_err(|_| ContentOperationError::integrity_failure())?;
    if raw.sequence_no <= 0
        || raw.content.is_empty()
        || raw.content.len() > MAX_MESSAGE_CONTENT_BYTES
        || raw.created_at_ms < 0
    {
        return Err(ContentOperationError::integrity_failure());
    }
    Ok(MessageRecord {
        id: raw.id,
        conversation_id: raw.conversation_id,
        sequence_no: raw.sequence_no,
        role: MessageRole::from_database(&raw.role)?,
        content: raw.content,
        created_at_ms: raw.created_at_ms,
    })
}
