use crate::runtime_store::error::ContentOperationError;
use crate::runtime_store::models::{
    validate_uuid_v4, InertTaskKind, InertTaskState, ListTasksRequest, TaskCursor, TaskPage,
    TaskRecord,
};
use rusqlite::{Connection, OptionalExtension, Row};

pub(super) fn insert(
    connection: &Connection,
    record: &TaskRecord,
) -> Result<(), ContentOperationError> {
    let changed = connection
        .execute(
            "INSERT INTO tasks (
                 id, conversation_id, task_kind, state, idempotency_key,
                 created_at_ms, updated_at_ms, revision
             ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?5, 0)",
            (
                &record.id,
                &record.conversation_id,
                record.task_kind.as_str(),
                record.state.as_str(),
                record.created_at_ms,
            ),
        )
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    if changed != 1 {
        return Err(ContentOperationError::integrity_failure());
    }
    Ok(())
}

pub(super) fn load(
    connection: &Connection,
    task_id: &str,
) -> Result<Option<TaskRecord>, ContentOperationError> {
    let raw = connection
        .query_row(
            "SELECT id, conversation_id, task_kind, state, idempotency_key,
                    created_at_ms, updated_at_ms, revision
             FROM tasks
             WHERE id = ?1",
            [task_id],
            raw_task,
        )
        .optional()
        .map_err(|error| ContentOperationError::from_sqlite(&error))?;
    raw.map(validate_record).transpose()
}

pub(super) fn list(
    connection: &Connection,
    request: &ListTasksRequest,
) -> Result<TaskPage, ContentOperationError> {
    let fetch_limit = i64::from(request.limit) + 1;
    let mut records = match &request.cursor {
        Some(cursor) => {
            let mut statement = connection
                .prepare(
                    "SELECT id, conversation_id, task_kind, state, idempotency_key,
                            created_at_ms, updated_at_ms, revision
                     FROM tasks
                     WHERE state = 'created'
                       AND (updated_at_ms, id) < (?1, ?2)
                     ORDER BY updated_at_ms DESC, id DESC
                     LIMIT ?3",
                )
                .map_err(|error| ContentOperationError::from_sqlite(&error))?;
            let rows = statement
                .query_map((cursor.updated_at_ms, &cursor.id, fetch_limit), raw_task)
                .map_err(|error| ContentOperationError::from_sqlite(&error))?;
            collect_rows(rows)?
        }
        None => {
            let mut statement = connection
                .prepare(
                    "SELECT id, conversation_id, task_kind, state, idempotency_key,
                            created_at_ms, updated_at_ms, revision
                     FROM tasks
                     WHERE state = 'created'
                     ORDER BY updated_at_ms DESC, id DESC
                     LIMIT ?1",
                )
                .map_err(|error| ContentOperationError::from_sqlite(&error))?;
            let rows = statement
                .query_map([fetch_limit], raw_task)
                .map_err(|error| ContentOperationError::from_sqlite(&error))?;
            collect_rows(rows)?
        }
    };

    let has_more = records.len() > request.limit as usize;
    if has_more {
        records.pop();
    }
    let next_cursor = if has_more {
        records.last().map(|record| TaskCursor {
            updated_at_ms: record.updated_at_ms,
            id: record.id.clone(),
        })
    } else {
        None
    };
    Ok(TaskPage {
        items: records,
        next_cursor,
    })
}

#[derive(Debug)]
struct RawTask {
    id: String,
    conversation_id: Option<String>,
    task_kind: String,
    state: String,
    idempotency_key: Option<String>,
    created_at_ms: i64,
    updated_at_ms: i64,
    revision: i64,
}

fn raw_task(row: &Row<'_>) -> rusqlite::Result<RawTask> {
    Ok(RawTask {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        task_kind: row.get(2)?,
        state: row.get(3)?,
        idempotency_key: row.get(4)?,
        created_at_ms: row.get(5)?,
        updated_at_ms: row.get(6)?,
        revision: row.get(7)?,
    })
}

fn collect_rows<F>(
    rows: rusqlite::MappedRows<'_, F>,
) -> Result<Vec<TaskRecord>, ContentOperationError>
where
    F: FnMut(&Row<'_>) -> rusqlite::Result<RawTask>,
{
    let mut records = Vec::new();
    for row in rows {
        records.push(validate_record(
            row.map_err(|error| ContentOperationError::from_sqlite(&error))?,
        )?);
    }
    Ok(records)
}

fn validate_record(raw: RawTask) -> Result<TaskRecord, ContentOperationError> {
    validate_persisted_uuid(&raw.id)?;
    if let Some(conversation_id) = &raw.conversation_id {
        validate_persisted_uuid(conversation_id)?;
    }
    if raw.idempotency_key.is_some()
        || raw.created_at_ms < 0
        || raw.updated_at_ms != raw.created_at_ms
        || raw.revision != 0
    {
        return Err(ContentOperationError::integrity_failure());
    }
    Ok(TaskRecord {
        id: raw.id,
        conversation_id: raw.conversation_id,
        task_kind: InertTaskKind::from_database(raw.task_kind)?,
        state: InertTaskState::from_database(&raw.state)?,
        created_at_ms: raw.created_at_ms,
        updated_at_ms: raw.updated_at_ms,
        revision: raw.revision,
    })
}

fn validate_persisted_uuid(value: &str) -> Result<(), ContentOperationError> {
    validate_uuid_v4(value).map_err(|_| ContentOperationError::integrity_failure())
}
