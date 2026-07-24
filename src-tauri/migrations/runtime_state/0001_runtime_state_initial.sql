CREATE TABLE schema_migrations (
    migration_id INTEGER PRIMARY KEY CHECK (migration_id > 0),
    name TEXT NOT NULL UNIQUE
        CHECK (length(CAST(name AS BLOB)) BETWEEN 1 AND 128),
    checksum_sha256 TEXT NOT NULL
        CHECK (
            length(checksum_sha256) = 64
            AND checksum_sha256 = lower(checksum_sha256)
            AND checksum_sha256 NOT GLOB '*[^0-9a-f]*'
        ),
    applied_at_ms INTEGER NOT NULL CHECK (applied_at_ms >= 0)
) STRICT;

CREATE TABLE conversations (
    id TEXT PRIMARY KEY
        CHECK (
            length(id) = 36
            AND id = lower(id)
            AND substr(id, 1, 8) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 9, 1) = '-'
            AND substr(id, 10, 4) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 14, 1) = '-'
            AND substr(id, 15, 1) = '4'
            AND substr(id, 16, 3) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 19, 1) = '-'
            AND substr(id, 20, 1) IN ('8', '9', 'a', 'b')
            AND substr(id, 21, 3) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 24, 1) = '-'
            AND substr(id, 25, 12) NOT GLOB '*[^0-9a-f]*'
        ),
    title TEXT
        CHECK (title IS NULL OR length(CAST(title AS BLOB)) <= 512),
    status TEXT NOT NULL CHECK (status IN ('active', 'archived')),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL CHECK (updated_at_ms >= created_at_ms),
    next_message_sequence INTEGER NOT NULL DEFAULT 1
        CHECK (next_message_sequence > 0),
    revision INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0)
) STRICT;

CREATE INDEX conversations_updated_idx
    ON conversations(updated_at_ms, id);

CREATE TABLE messages (
    id TEXT PRIMARY KEY
        CHECK (
            length(id) = 36
            AND id = lower(id)
            AND substr(id, 1, 8) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 9, 1) = '-'
            AND substr(id, 10, 4) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 14, 1) = '-'
            AND substr(id, 15, 1) = '4'
            AND substr(id, 16, 3) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 19, 1) = '-'
            AND substr(id, 20, 1) IN ('8', '9', 'a', 'b')
            AND substr(id, 21, 3) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 24, 1) = '-'
            AND substr(id, 25, 12) NOT GLOB '*[^0-9a-f]*'
        ),
    conversation_id TEXT NOT NULL
        CHECK (
            length(conversation_id) = 36
            AND conversation_id = lower(conversation_id)
            AND substr(conversation_id, 1, 8) NOT GLOB '*[^0-9a-f]*'
            AND substr(conversation_id, 9, 1) = '-'
            AND substr(conversation_id, 10, 4) NOT GLOB '*[^0-9a-f]*'
            AND substr(conversation_id, 14, 1) = '-'
            AND substr(conversation_id, 15, 1) = '4'
            AND substr(conversation_id, 16, 3) NOT GLOB '*[^0-9a-f]*'
            AND substr(conversation_id, 19, 1) = '-'
            AND substr(conversation_id, 20, 1) IN ('8', '9', 'a', 'b')
            AND substr(conversation_id, 21, 3) NOT GLOB '*[^0-9a-f]*'
            AND substr(conversation_id, 24, 1) = '-'
            AND substr(conversation_id, 25, 12) NOT GLOB '*[^0-9a-f]*'
        )
        REFERENCES conversations(id) ON DELETE CASCADE,
    sequence_no INTEGER NOT NULL CHECK (sequence_no > 0),
    role TEXT NOT NULL CHECK (role IN ('system', 'user', 'assistant')),
    content TEXT NOT NULL
        CHECK (length(CAST(content AS BLOB)) BETWEEN 1 AND 262144),
    created_at_ms INTEGER NOT NULL,
    UNIQUE (conversation_id, sequence_no)
) STRICT;

CREATE INDEX messages_conversation_sequence_idx
    ON messages(conversation_id, sequence_no, id);

CREATE TABLE tasks (
    id TEXT PRIMARY KEY
        CHECK (
            length(id) = 36
            AND id = lower(id)
            AND substr(id, 1, 8) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 9, 1) = '-'
            AND substr(id, 10, 4) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 14, 1) = '-'
            AND substr(id, 15, 1) = '4'
            AND substr(id, 16, 3) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 19, 1) = '-'
            AND substr(id, 20, 1) IN ('8', '9', 'a', 'b')
            AND substr(id, 21, 3) NOT GLOB '*[^0-9a-f]*'
            AND substr(id, 24, 1) = '-'
            AND substr(id, 25, 12) NOT GLOB '*[^0-9a-f]*'
        ),
    conversation_id TEXT
        CHECK (
            conversation_id IS NULL
            OR (
                length(conversation_id) = 36
                AND conversation_id = lower(conversation_id)
                AND substr(conversation_id, 1, 8) NOT GLOB '*[^0-9a-f]*'
                AND substr(conversation_id, 9, 1) = '-'
                AND substr(conversation_id, 10, 4) NOT GLOB '*[^0-9a-f]*'
                AND substr(conversation_id, 14, 1) = '-'
                AND substr(conversation_id, 15, 1) = '4'
                AND substr(conversation_id, 16, 3) NOT GLOB '*[^0-9a-f]*'
                AND substr(conversation_id, 19, 1) = '-'
                AND substr(conversation_id, 20, 1) IN ('8', '9', 'a', 'b')
                AND substr(conversation_id, 21, 3) NOT GLOB '*[^0-9a-f]*'
                AND substr(conversation_id, 24, 1) = '-'
                AND substr(conversation_id, 25, 12) NOT GLOB '*[^0-9a-f]*'
            )
        )
        REFERENCES conversations(id) ON DELETE CASCADE,
    task_kind TEXT NOT NULL
        CHECK (length(CAST(task_kind AS BLOB)) BETWEEN 1 AND 64),
    state TEXT NOT NULL DEFAULT 'created' CHECK (state = 'created'),
    idempotency_key TEXT UNIQUE
        CHECK (
            idempotency_key IS NULL
            OR length(CAST(idempotency_key AS BLOB)) <= 128
        ),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL CHECK (updated_at_ms >= created_at_ms),
    revision INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0)
) STRICT;

CREATE INDEX tasks_state_updated_idx
    ON tasks(state, updated_at_ms, id);

CREATE INDEX tasks_conversation_idx
    ON tasks(conversation_id, id);

CREATE TABLE audit_events (
    sequence_no INTEGER PRIMARY KEY,
    event_id TEXT NOT NULL UNIQUE
        CHECK (
            length(event_id) = 36
            AND event_id = lower(event_id)
            AND substr(event_id, 1, 8) NOT GLOB '*[^0-9a-f]*'
            AND substr(event_id, 9, 1) = '-'
            AND substr(event_id, 10, 4) NOT GLOB '*[^0-9a-f]*'
            AND substr(event_id, 14, 1) = '-'
            AND substr(event_id, 15, 1) = '4'
            AND substr(event_id, 16, 3) NOT GLOB '*[^0-9a-f]*'
            AND substr(event_id, 19, 1) = '-'
            AND substr(event_id, 20, 1) IN ('8', '9', 'a', 'b')
            AND substr(event_id, 21, 3) NOT GLOB '*[^0-9a-f]*'
            AND substr(event_id, 24, 1) = '-'
            AND substr(event_id, 25, 12) NOT GLOB '*[^0-9a-f]*'
        ),
    event_type TEXT NOT NULL
        CHECK (
            event_type IN (
                'conversation.created',
                'conversation.deleted',
                'message.appended',
                'task.created',
                'task.recorded',
                'task.deleted',
                'runtime.content_deleted',
                'export.completed',
                'storage.recovery_required'
            )
            AND length(CAST(event_type AS BLOB)) <= 64
        ),
    actor_type TEXT NOT NULL
        CHECK (
            actor_type IN ('user', 'local_runtime')
            AND length(CAST(actor_type AS BLOB)) <= 32
        ),
    subject_type TEXT NOT NULL
        CHECK (
            subject_type IN (
                'conversation', 'message', 'task', 'runtime', 'export', 'storage'
            )
            AND length(CAST(subject_type AS BLOB)) <= 32
        ),
    subject_id TEXT
        CHECK (
            subject_id IS NULL
            OR (
                length(subject_id) = 36
                AND subject_id = lower(subject_id)
                AND substr(subject_id, 1, 8) NOT GLOB '*[^0-9a-f]*'
                AND substr(subject_id, 9, 1) = '-'
                AND substr(subject_id, 10, 4) NOT GLOB '*[^0-9a-f]*'
                AND substr(subject_id, 14, 1) = '-'
                AND substr(subject_id, 15, 1) = '4'
                AND substr(subject_id, 16, 3) NOT GLOB '*[^0-9a-f]*'
                AND substr(subject_id, 19, 1) = '-'
                AND substr(subject_id, 20, 1) IN ('8', '9', 'a', 'b')
                AND substr(subject_id, 21, 3) NOT GLOB '*[^0-9a-f]*'
                AND substr(subject_id, 24, 1) = '-'
                AND substr(subject_id, 25, 12) NOT GLOB '*[^0-9a-f]*'
            )
        ),
    outcome TEXT NOT NULL
        CHECK (
            outcome IN ('success', 'denied', 'failed')
            AND length(CAST(outcome AS BLOB)) <= 16
        ),
    reason_code TEXT
        CHECK (
            reason_code IS NULL
            OR length(CAST(reason_code AS BLOB)) BETWEEN 1 AND 128
        ),
    correlation_id TEXT
        CHECK (
            correlation_id IS NULL
            OR (
                length(correlation_id) = 36
                AND correlation_id = lower(correlation_id)
                AND substr(correlation_id, 1, 8) NOT GLOB '*[^0-9a-f]*'
                AND substr(correlation_id, 9, 1) = '-'
                AND substr(correlation_id, 10, 4) NOT GLOB '*[^0-9a-f]*'
                AND substr(correlation_id, 14, 1) = '-'
                AND substr(correlation_id, 15, 1) = '4'
                AND substr(correlation_id, 16, 3) NOT GLOB '*[^0-9a-f]*'
                AND substr(correlation_id, 19, 1) = '-'
                AND substr(correlation_id, 20, 1) IN ('8', '9', 'a', 'b')
                AND substr(correlation_id, 21, 3) NOT GLOB '*[^0-9a-f]*'
                AND substr(correlation_id, 24, 1) = '-'
                AND substr(correlation_id, 25, 12) NOT GLOB '*[^0-9a-f]*'
            )
        ),
    created_at_ms INTEGER NOT NULL,
    CHECK (
        length(CAST(event_id AS BLOB))
        + length(CAST(event_type AS BLOB))
        + length(CAST(actor_type AS BLOB))
        + length(CAST(subject_type AS BLOB))
        + coalesce(length(CAST(subject_id AS BLOB)), 0)
        + length(CAST(outcome AS BLOB))
        + coalesce(length(CAST(reason_code AS BLOB)), 0)
        + coalesce(length(CAST(correlation_id AS BLOB)), 0)
        <= 8192
    )
) STRICT;

CREATE INDEX audit_events_created_idx
    ON audit_events(created_at_ms, sequence_no);

CREATE INDEX audit_events_type_created_idx
    ON audit_events(event_type, created_at_ms);

CREATE INDEX audit_events_subject_idx
    ON audit_events(subject_type, subject_id, sequence_no);
