use crate::runtime_store::error::ContentOperationError;
use crate::runtime_store::models::{
    validate_uuid_v4, ConversationCursor, ConversationPage, ConversationRecord, ConversationStatus,
    ListConversationsRequest, MAX_TITLE_BYTES,
};
use rusqlite::{Connection, OptionalExtension, Row};

pub(super) fn insert(
    connection: &Connection,
    id: &str,
    title: Option<&str>,
    timestamp_ms: i64,
) -> Result<(), ContentOperationError> {
    connection
        .execute(
            "INSERT INTO conversations (
                 id, title, status, created_at_ms, updated_at_ms,
                 next_message_sequence, revision
             ) VALUES (?1, ?2, 'active', ?3, ?3, 1, 0)",
            (id, title, timestamp_ms),
        )
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    Ok(())
}

pub(super) fn load(
    connection: &Connection,
    conversation_id: &str,
) -> Result<Option<ConversationRecord>, ContentOperationError> {
    let raw = connection
        .query_row(
            "SELECT id, title, status, created_at_ms, updated_at_ms,
                    next_message_sequence, revision
             FROM conversations
             WHERE id = ?1",
            [conversation_id],
            raw_conversation,
        )
        .optional()
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    raw.map(validate_record).transpose()
}

pub(super) fn list(
    connection: &Connection,
    request: &ListConversationsRequest,
) -> Result<ConversationPage, ContentOperationError> {
    let fetch_limit = i64::from(request.limit) + 1;
    let mut records = match &request.cursor {
        Some(cursor) => {
            let mut statement = connection
                .prepare(
                    "SELECT id, title, status, created_at_ms, updated_at_ms,
                            next_message_sequence, revision
                     FROM conversations
                     WHERE updated_at_ms < ?1
                        OR (updated_at_ms = ?1 AND id < ?2)
                     ORDER BY updated_at_ms DESC, id DESC
                     LIMIT ?3",
                )
                .map_err(|error| ContentOperationError::from_sqlite(&error))?;
            let rows = statement
                .query_map(
                    (cursor.updated_at_ms, &cursor.id, fetch_limit),
                    raw_conversation,
                )
                .map_err(|error| ContentOperationError::from_sqlite(&error))?;
            collect_rows(rows)?
        }
        None => {
            let mut statement = connection
                .prepare(
                    "SELECT id, title, status, created_at_ms, updated_at_ms,
                            next_message_sequence, revision
                     FROM conversations
                     ORDER BY updated_at_ms DESC, id DESC
                     LIMIT ?1",
                )
                .map_err(|error| ContentOperationError::from_sqlite(&error))?;
            let rows = statement
                .query_map([fetch_limit], raw_conversation)
                .map_err(|error| ContentOperationError::from_sqlite(&error))?;
            collect_rows(rows)?
        }
    };

    let has_more = records.len() > request.limit as usize;
    if has_more {
        records.pop();
    }
    let next_cursor = if has_more {
        records.last().map(|record| ConversationCursor {
            updated_at_ms: record.updated_at_ms,
            id: record.id.clone(),
        })
    } else {
        None
    };
    Ok(ConversationPage {
        items: records,
        next_cursor,
    })
}

pub(super) fn update_after_append(
    connection: &Connection,
    conversation_id: &str,
    expected_sequence: i64,
    timestamp_ms: i64,
) -> Result<(), ContentOperationError> {
    let changed = connection
        .execute(
            "UPDATE conversations
             SET updated_at_ms = ?1,
                 next_message_sequence = next_message_sequence + 1,
                 revision = revision + 1
             WHERE id = ?2
               AND next_message_sequence = ?3",
            (timestamp_ms, conversation_id, expected_sequence),
        )
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    if changed != 1 {
        return Err(ContentOperationError::integrity_failure());
    }
    Ok(())
}

fn collect_rows<F>(
    rows: rusqlite::MappedRows<'_, F>,
) -> Result<Vec<ConversationRecord>, ContentOperationError>
where
    F: FnMut(&Row<'_>) -> rusqlite::Result<RawConversation>,
{
    let mut records = Vec::new();
    for row in rows {
        let raw = row.map_err(|error| ContentOperationError::from_sqlite(&error))?;
        records.push(validate_record(raw)?);
    }
    Ok(records)
}

#[derive(Debug)]
struct RawConversation {
    id: String,
    title: Option<String>,
    status: String,
    created_at_ms: i64,
    updated_at_ms: i64,
    next_message_sequence: i64,
    revision: i64,
}

fn raw_conversation(row: &Row<'_>) -> rusqlite::Result<RawConversation> {
    Ok(RawConversation {
        id: row.get(0)?,
        title: row.get(1)?,
        status: row.get(2)?,
        created_at_ms: row.get(3)?,
        updated_at_ms: row.get(4)?,
        next_message_sequence: row.get(5)?,
        revision: row.get(6)?,
    })
}

fn validate_record(raw: RawConversation) -> Result<ConversationRecord, ContentOperationError> {
    validate_uuid_v4(&raw.id).map_err(|_| ContentOperationError::integrity_failure())?;
    if raw
        .title
        .as_ref()
        .is_some_and(|title| title.len() > MAX_TITLE_BYTES)
        || raw.created_at_ms < 0
        || raw.updated_at_ms < raw.created_at_ms
        || raw.next_message_sequence <= 0
        || raw.revision < 0
    {
        return Err(ContentOperationError::integrity_failure());
    }
    Ok(ConversationRecord {
        id: raw.id,
        title: raw.title,
        status: ConversationStatus::from_database(&raw.status)?,
        created_at_ms: raw.created_at_ms,
        updated_at_ms: raw.updated_at_ms,
        next_message_sequence: raw.next_message_sequence,
        revision: raw.revision,
    })
}
