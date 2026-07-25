# Phase 1B.2 — Conversations and Messages Plan

Status: **CONDITIONAL_GO / PLANNING REVIEW PASS — IMPLEMENTATION NOT AUTHORIZED**

Date: 2026-07-24

## Objective

Define the smallest safe Rust-only vertical slice that turns the already
verified Phase 1B.1 SQLite schema into typed, durable conversation and message
services.

The slice must provide atomic ordered persistence, bounded retrieval,
idempotent retry behavior and privacy-safe audit coupling without adding
frontend, model, tool, network, Supervisor, memory or transport authority.

## Current State

Canonical source:

```text
CANONICAL_MAIN = 42c5ac0b4fa501e59c83a5b0b395c73af382420d
PHASE_1B_1 = MERGED / FRESH_MAIN_VERIFIED / CURRENT_DOCS_RECONCILED
STORAGE_BOOTSTRAP = IMPLEMENTED_AND_VERIFIED
STORAGE_RUNTIME_PROJECTION = IMPLEMENTED_AND_VERIFIED_IN_REPOSITORY
DURABLE_RUNTIME_STATE = PARTIALLY_IMPLEMENTED
PHASE_1B = NOT COMPLETE
PHASE_1B_2_IMPLEMENTATION = NOT AUTHORIZED
```

Planning review evidence:

```text
SUBSTANTIVE_PLAN_PATCH_SHA256 =
f1ed3caf34cdcbaffd341f82cd351d443118cf336e44fd2729e9db9c2974a9dd

PHASE_1B2_PLANNING_REVIEW =
PHASE_1B2_PLANNING_REVIEW_PASS

PLANNING_FINDINGS =
CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 0 / INFO 0
```

Verified repository facts:

- `src-tauri/migrations/runtime_state/0001_runtime_state_initial.sql`
  already creates constrained `conversations`, `messages`, `tasks` and
  `audit_events` tables plus `schema_migrations`.
- `src-tauri/src/runtime_store/worker.rs::RuntimeStoreRequest` currently has
  production variants only for initialization and status reads.
- `RuntimeStoreManager` currently exposes initialization, status and shutdown
  behavior; it exposes no content service.
- `src-tauri/src/runtime_store/commands.rs` contains only the read-only,
  no-user-argument `get_storage_runtime_status` command.
- `src-tauri/src/runtime_store/tests.rs` validates schema behavior by direct
  test-only SQL. Those tests do not prove a production repository/service path.
- `scripts/validate-storage-runtime-contract.mjs` intentionally rejects
  storage CRUD as Tauri commands. Private Rust services do not require a Tauri
  CRUD command.
- `src/components/LocalInferencePanel.tsx` keeps conversation-like chat history
  only in React state.
- `src-tauri/src/messaging.rs` is a separate mocked, RAM-only messaging surface
  and is not a Phase 1B.2 storage consumer.

The detailed source evidence and classifications are recorded in
`docs/audits/PHASE_1B2_CONVERSATIONS_MESSAGES_READINESS_AUDIT_2026-07-24.md`.

## Scope

Phase 1B.2 is limited to:

1. Private Rust domain types for conversations, messages, roles, status,
   validated inputs, bounded page requests and controlled operation results.
2. Private conversation and message repositories executed only on the existing
   single-owner runtime-store worker connection.
3. Exactly five actor requests and manager methods:
   - `create_conversation` — mutation;
   - `get_conversation` — read;
   - `list_conversations` — read;
   - `append_message` — mutation;
   - `list_messages` — read.
4. Runtime-generated conversation/message IDs and service-owned timestamps.
5. One caller-supplied UUID-v4 `operation_id` for each of the two mutations
   only. The same ID is the private audit-event ID and idempotency key.
6. A private transaction helper that writes only the required
   `conversation.created` and `message.appended` events. It is not a public
   `AuditStore` and does not widen Phase 1B.2 into Phase 1B.3.
7. Atomic conversation/audit and message/sequence/conversation-revision/audit
   transactions.
8. Bounded read/write deadlines and queue limits plus a 16-MiB immutable
   operational reserve below the existing database hard limit. Ordinary
   mutations use separate reviewed physical-growth envelopes and can never
   consume the reserve.
9. Restart, concurrency, rollback, idempotency, ordering, privacy and denial
   tests using generated temporary roots only.

Canonical terminology:

```text
CONTENT_OPERATIONS = 5
CONTENT_MUTATIONS = 2
CONTENT_READS = 3

CONTENT_MUTATIONS =
CREATE_CONVERSATION / APPEND_MESSAGE

CONTENT_READS =
GET_CONVERSATION / LIST_CONVERSATIONS / LIST_MESSAGES
```

Only mutations use `operation_id`, idempotency evaluation, `BEGIN IMMEDIATE`,
state-plus-audit atomicity, capacity admission or write-lock acquisition.
Reads use none of those mutation semantics.

## Explicit Non-Goals

- No Tauri content command, generic IPC, frontend adapter, Dashboard CRUD or
  `LocalInferencePanel` persistence.
- No integration with the legacy `messaging.rs` mock.
- No public audit query/append API.
- No task repository, task transition, Agent Supervisor, planner, executor,
  verifier, loop, retry scheduler or checkpoint runtime.
- No deletion, delete-all, retention engine, export, import, backup, restore,
  factory-reset integration or recovery UI. Those remain Phase 1B.4 work.
- No semantic/episodic/procedural/graph memory, extraction, summarization,
  embeddings or vector store.
- No model call, prompt construction, tool use, shell/process access, network
  communication, Supabase, Reticulum/LXMF, wallet or worker authority.
- No migration 2, schema/index/constraint change, dependency, manifest,
  lockfile, Tauri capability or production configuration change.
- No real user-profile data, deployment, publish, smoke or production write.

## Repository Ownership

`daarion-edge-client` exclusively owns this local runtime-state slice.

- Rust owns paths, SQL, IDs, timestamps, transactions, limits and controlled
  errors.
- The frontend, LLM, remote agents and web product own none of these.
- Raw conversations and messages remain local and are not projected to
  `loval-echoes`, Supabase or transport.
- Conversation history is runtime state, not validated long-term memory.

No cross-repository contract changes are allowed.

## Files and Modules Expected to Change

Expected application scope for a later separately authorized implementation:

```text
src-tauri/src/runtime_store/
├── mod.rs
├── config.rs
├── connection.rs
├── models.rs
├── error.rs
├── worker.rs
├── repositories/
│   ├── mod.rs
│   ├── conversations.rs
│   ├── messages.rs
│   └── unit_of_work.rs
└── repository_tests.rs
```

These names are the selected implementation boundary; changing them requires a
plan correction rather than an undocumented alternative. New repository tests
must not further enlarge the already large `runtime_store/tests.rs`
bootstrap/lifecycle suite.

`config.rs` and `connection.rs` are included only for the future immutable
reserve, fixed growth envelopes, page-size validation and actor-owned admission
helper described below. No public configuration field, migration, schema,
dependency or database path changes are allowed.

Expected documentation updates after implementation:

- this plan’s matching implementation completion report;
- ADR 0002 implementation status;
- capability matrix;
- security gates;
- master roadmap;
- README storage claim if the behavior becomes product-relevant.

Not expected to change:

- `src-tauri/migrations/runtime_state/0001_runtime_state_initial.sql`;
- `src-tauri/Cargo.toml`;
- `src-tauri/Cargo.lock`;
- `package.json` or `package-lock.json`;
- `src-tauri/src/lib.rs`;
- `src-tauri/src/runtime_store/commands.rs`;
- frontend source;
- Tauri capabilities or deployment configuration.

If any not-expected path becomes necessary, stop and amend the plan before
implementation.

## Contracts Affected

### Exact Rust-only manager API

The owning service name is fixed as `RuntimeStoreManager`. Phase 1B.2 adds
exactly these private Rust methods:

```rust
RuntimeStoreManager::create_conversation(
    CreateConversationRequest
) -> Result<ConversationRecord, ContentOperationError>

RuntimeStoreManager::get_conversation(
    GetConversationRequest
) -> Result<ConversationRecord, ContentOperationError>

RuntimeStoreManager::list_conversations(
    ListConversationsRequest
) -> Result<ConversationPage, ContentOperationError>

RuntimeStoreManager::append_message(
    AppendMessageRequest
) -> Result<MessageRecord, ContentOperationError>

RuntimeStoreManager::list_messages(
    ListMessagesRequest
) -> Result<MessagePage, ContentOperationError>
```

All methods and DTOs are `pub(crate)` or more private. They are actor messages
and trusted Rust service types only: no DTO is serializable as public Tauri
authority, no new `#[tauri::command]` is added, and `generate_handler!` remains
unchanged.

### Exact DTO inventory

Validated internal newtypes/enums:

```rust
ConversationId       // canonical lowercase UUID-v4
MessageId            // canonical lowercase UUID-v4
OperationId          // canonical lowercase UUID-v4
ConversationStatus::{Active, Archived}
MessageRole::{System, User, Assistant}
ContentActor::{User, LocalRuntime}

ConversationCursor {
    updated_at_ms: i64,
    id: ConversationId,
}
```

Request DTOs:

```rust
CreateConversationRequest {
    operation_id: OperationId,
    actor: ContentActor,
    title: Option<String>,
}

GetConversationRequest {
    conversation_id: ConversationId,
}

ListConversationsRequest {
    limit: u16,
    cursor: Option<ConversationCursor>,
}

AppendMessageRequest {
    operation_id: OperationId,
    actor: ContentActor,
    conversation_id: ConversationId,
    role: MessageRole,
    content: String,
}

ListMessagesRequest {
    conversation_id: ConversationId,
    limit: u16,
    after_sequence_no: Option<i64>,
}
```

Output DTOs:

```rust
ConversationRecord {
    id: ConversationId,
    title: Option<String>,
    status: ConversationStatus,
    created_at_ms: i64,
    updated_at_ms: i64,
    next_message_sequence: i64,
    revision: i64,
}

MessageRecord {
    id: MessageId,
    conversation_id: ConversationId,
    sequence_no: i64,
    role: MessageRole,
    content: String,
    created_at_ms: i64,
}

ConversationPage {
    items: Vec<ConversationRecord>,
    next_cursor: Option<ConversationCursor>,
}

MessagePage {
    items: Vec<MessageRecord>,
    next_after_sequence_no: Option<i64>,
}
```

The conversation cursor is an internal typed DTO, not an encoded opaque string.
No type exposes SQL, paths, SQLite values, transaction handles or raw internal
errors.

### Exact read contracts

#### `get_conversation`

- Validate a canonical lowercase UUID-v4 conversation ID before enqueue and
  before bind.
- Return exactly one complete `ConversationRecord` for that ID.
- Return `ConversationNotFound` for an absent valid ID and `InvalidInput` for
  invalid/noncanonical input.
- Never return a row for another ID.
- Use a bounded read-only query. It has no operation ID, audit event,
  `BEGIN IMMEDIATE` or write-capacity admission.
- Remain available while ordinary mutations are blocked only by the hard
  capacity gate.
- Return the same persisted record after clean restart/reopen.

#### `list_conversations`

- Accept `limit` only in `1..=100`; there is no implicit zero/default request.
- Order by `updated_at_ms DESC, id DESC`.
- Use the complete typed cursor `(updated_at_ms, id)`. Continuation predicate:
  `updated_at_ms < cursor.updated_at_ms OR
  (updated_at_ms = cursor.updated_at_ms AND id < cursor.id)`.
- Fetch at most `limit + 1`, return at most `limit`, and derive
  `next_cursor` from the final returned row only when another row exists. This
  prevents duplicates and skipped rows for an unchanged database.
- An empty database returns an empty page. Invalid limit or cursor returns
  `InvalidInput`.
- Use a bounded read-only query with no operation ID, audit event, write lock or
  capacity admission. Deterministic ordering survives restart.

#### `list_messages`

- Require and validate the canonical lowercase UUID-v4 conversation ID.
- Accept `limit` only in `1..=100`.
- Verify the conversation exists first; an absent valid conversation returns
  `ConversationNotFound`, while an existing conversation with no messages
  returns an empty page.
- Scope every query predicate to the requested conversation and order by
  `sequence_no ASC, id ASC`.
- `after_sequence_no` must be absent or greater than zero and applies only
  inside that conversation. Continuation uses
  `conversation_id = ? AND sequence_no > ?`; the schema makes sequence numbers
  unique per conversation. Fetch at most `limit + 1`, return at most `limit`,
  and set `next_after_sequence_no` to the final returned sequence only when
  another row exists.
- Use a bounded read-only query with no operation ID, audit event, write lock or
  capacity admission. Deterministic results survive restart.

```text
LIST_MESSAGES(conversation_A)
MUST NEVER RETURN
message.conversation_id != conversation_A
```

### Exact mutation contracts

#### `create_conversation`

1. Validate the operation ID, actor and title before enqueue and before bind.
2. On the single connection-owning worker, acquire `BEGIN IMMEDIATE`.
3. If `audit_events.event_id = operation_id` exists, execute the idempotency
   replay contract below without capacity admission.
4. Otherwise run the create growth-envelope admission.
5. Generate a conversation UUID-v4 and one service-owned timestamp `T`.
6. Insert a conversation with `status = active`, `created_at_ms = T`,
   `updated_at_ms = T`, `next_message_sequence = 1`, and `revision = 0`.
7. Insert exactly one `conversation.created` audit event in the same
   transaction.
8. Run the pre-commit envelope verification, then commit or roll back both
   writes.

The title is immutable in Phase 1B.2 because no update operation exists.

#### `append_message`

1. Validate operation ID, actor, conversation ID, role and content before
   enqueue and before bind.
2. On the single connection-owning worker, acquire `BEGIN IMMEDIATE`.
3. If the operation ID already exists, execute the replay contract without
   capacity admission.
4. Verify the conversation exists; otherwise return
   `ConversationNotFound` and roll back the empty transaction.
5. Run append growth-envelope admission.
6. Read the conversation’s `next_message_sequence` and assign it to the new
   message.
7. Generate the message UUID-v4. Let
   `T = max(service_now_ms, conversation.updated_at_ms + 1)`, failing
   controlled on integer overflow.
8. Insert the message, increment `next_message_sequence` and `revision` by one,
   and set the conversation’s `updated_at_ms = T`.
9. Insert exactly one `message.appended` audit event with the same `T`.
10. Run the pre-commit envelope verification, then commit or roll back the
    message, conversation update and audit event together.

Successful message sequences are contiguous per conversation. A rolled-back
append consumes no sequence.

### Canonical payload and validation semantics

IDs:

- all IDs are lowercase canonical UUID-v4 strings;
- noncanonical IDs are rejected rather than rewritten;
- operation IDs are supplied by the trusted Rust caller and are globally
  unique across both mutations for the database lifetime.

Titles:

- `None` maps to SQL `NULL`; `Some("")` maps to an empty string;
- `NULL` and empty string are distinct canonical payloads;
- no trim, case folding or Unicode normalization occurs;
- equality is exact Rust string/UTF-8 equality;
- the migration-1 ceiling is 512 UTF-8 bytes.

Message content:

- content must contain 1 through 262,144 UTF-8 bytes;
- no trim or Unicode normalization occurs;
- the exact string is persisted and compared.

Roles:

- only exact lowercase `system`, `user`, and `assistant` values are valid;
- no aliases or case normalization exist.

Semantically similar but byte-different Unicode strings are different
idempotency payloads.

### Content-operation error domain

Phase 1B.2 defines a separate internal error domain:

```rust
ContentOperationErrorCode {
    InvalidInput,
    ConversationNotFound,
    IdempotencyConflict,
    IdempotencyRecordInconsistent,
    CapacityExceeded,
    BusyTimeout,
    DeadlineExceeded,
    Unavailable,
    IntegrityFailure,
    Internal,
}
```

`ContentOperationError` exposes only this safe code and contains no title,
content, SQL, path or raw SQLite message. It does not reuse
`StorageRuntimeErrorCode` as a content contract.

| Operation | Exact controlled errors |
| --- | --- |
| `create_conversation` | `InvalidInput` for operation ID/actor/title; `IdempotencyConflict`; `IdempotencyRecordInconsistent`; `CapacityExceeded`; `BusyTimeout`; `DeadlineExceeded`; `Unavailable`; `IntegrityFailure`; `Internal` |
| `get_conversation` | `InvalidInput` for ID; `ConversationNotFound`; `BusyTimeout`; `DeadlineExceeded`; `Unavailable`; `IntegrityFailure`; `Internal` |
| `list_conversations` | `InvalidInput` for limit/cursor; `BusyTimeout`; `DeadlineExceeded`; `Unavailable`; `IntegrityFailure`; `Internal` |
| `append_message` | `InvalidInput` for operation/conversation ID, actor, role, empty/oversized content; `ConversationNotFound`; `IdempotencyConflict`; `IdempotencyRecordInconsistent`; `CapacityExceeded`; `BusyTimeout`; `DeadlineExceeded`; `Unavailable`; `IntegrityFailure`; `Internal` |
| `list_messages` | `InvalidInput` for conversation ID, limit or `after_sequence_no`; `ConversationNotFound`; `BusyTimeout`; `DeadlineExceeded`; `Unavailable`; `IntegrityFailure`; `Internal` |

`InvalidInput`, `ConversationNotFound`, `IdempotencyConflict` and
`CapacityExceeded` are operation-local and must not change a healthy/warning
runtime projection. `BusyTimeout` and `DeadlineExceeded` are transient
operation failures and also do not permanently poison health.

Operation-local errors do not poison global runtime health.

`IdempotencyRecordInconsistent` deterministically:

1. returns that code for the current request;
2. transitions global runtime state to `IntegrityFailed`;
3. closes content-operation intake, including content reads and mutations;
4. leaves the safe status projection available;
5. requires reopen/integrity handling before content operations resume.

SQLite integrity failure, connection loss/unavailability, or unrecoverable
worker failure may likewise update global runtime health. `Internal` updates
global health only when it represents an unrecoverable worker/connection
failure; a recoverable operation mapping error is returned locally.

### Complete idempotency contract

```text
audit_events.event_id = mutation operation_id

OPERATION_ID_UNIQUENESS =
GLOBAL ACROSS CREATE_CONVERSATION AND APPEND_MESSAGE /
ENTIRE DATABASE LIFETIME
```

Phase 1B.2 performs no audit retention, pruning or deletion. The current
`audit_events.event_id TEXT NOT NULL UNIQUE` constraint is the database-enforced
atomic uniqueness mechanism; no migration 2 is needed.

For first execution, one `BEGIN IMMEDIATE` transaction performs capacity
admission, subject mutation and audit insert. On any failure both subject and
audit roll back, and the operation ID remains reusable unless a prior committed
event already exists.

For the same ID and same canonical request:

- load the audit event by `event_id`;
- require the expected event type, actor, subject type, outcome `success`,
  `reason_code = NULL`, `correlation_id = operation_id`, and non-null subject;
- load the referenced subject row;
- for append, compare operation kind, actor, conversation ID, role and exact
  content, then return the immutable persisted `MessageRecord`;
- for create, apply the deterministic reconstruction contract below rather
  than returning the current mutable conversation projection;
- create no row, audit event, timestamp/revision change or capacity charge
  beyond bounded reads.

Create replay must:

1. require `event_type = conversation.created`,
   `subject_type = conversation`, `outcome = success`, `reason_code = NULL`,
   and `correlation_id = operation_id`;
2. load the conversation identified by the audit `subject_id`;
3. require the audit actor to equal the request actor;
4. require the audit `created_at_ms` to equal the conversation
   `created_at_ms`;
5. require the persisted status to remain `active`;
6. compare the exact canonical optional title;
7. return the original create result reconstructed as:

```rust
ConversationRecord {
    id: persisted_conversation.id,
    title: persisted_conversation.title,
    status: ConversationStatus::Active,
    created_at_ms: persisted_conversation.created_at_ms,
    updated_at_ms: persisted_conversation.created_at_ms,
    next_message_sequence: 1,
    revision: 0,
}
```

The replay intentionally ignores the conversation’s current mutable
`updated_at_ms`, `next_message_sequence`, and `revision`. Therefore a create
replay after one or more appends differs intentionally from
`get_conversation`, which returns the current projection.

Migration 1 is sufficient because `created_at_ms` and the Phase 1B.2 title are
immutable, status cannot change in this slice, all initial counters are
deterministic, and the audit event identifies the original create operation,
actor, timestamp and subject. A missing subject, actor/timestamp mismatch,
non-active status, or inability to reproduce the title returns
`IdempotencyRecordInconsistent`; no migration 2 or result-snapshot column is
required.

For the same ID and a different canonical request, including a well-formed
prior event for the other approved mutation type, return
`IdempotencyConflict` with no write and no health degradation.

An audit event with a missing subject, an internally impossible event/subject
combination, unsupported event type, non-success outcome, wrong correlation, or
subject unable to reproduce the canonical request returns
`IdempotencyRecordInconsistent` and applies the deterministic integrity-failure
behavior above.

The actor serializes concurrent duplicates. Exactly one physical mutation and
one audit event may commit. A same-payload duplicate replays that result; a
different-payload duplicate returns `IdempotencyConflict`; no raw SQLite unique
error escapes.

### Exact audit contract

`ContentActor` is a closed enum mapping only to `user` or `local_runtime`.
No free-form actor value exists.

| Field | `create_conversation` | `append_message` |
| --- | --- | --- |
| `event_id` | `operation_id` | `operation_id` |
| `event_type` | `conversation.created` | `message.appended` |
| `actor_type` | request actor -> `user` / `local_runtime` | request actor -> `user` / `local_runtime` |
| `subject_type` | `conversation` | `message` |
| `subject_id` | persisted conversation ID | persisted message ID |
| `outcome` | `success` | `success` |
| `reason_code` | `NULL` | `NULL` |
| `correlation_id` | `operation_id` | `operation_id` |
| `created_at_ms` | conversation `created_at_ms` | message `created_at_ms` |

The audit insert omits `sequence_no`; SQLite allocates the
`INTEGER PRIMARY KEY` rowid under the serialized write transaction. There is no
`AUTOINCREMENT` and no custom sequence table. The private service is append-only
and Phase 1B.2 exposes no audit deletion, so sequence values are monotonic for
the retained database.

Validation errors, NotFound, idempotency conflicts and capacity rejection write
no durable audit row. A failed transaction rolls back its audit row. Therefore
committed Phase 1B.2 mutation events use only `outcome = success`;
`denied`/`failed` remain reserved for a separately designed future
security/emergency audit lane. The plan does not claim that every failed request
is durably audited.

### Immutable hard-reserve and growth-envelope design

Production constants:

```text
H = 4 GiB = 4,294,967,296 bytes
R = 16 MiB = 16,777,216 bytes
U = H - R = 4,278,190,080 bytes

G_CREATE = 8 MiB = 8,388,608 bytes
G_APPEND = 32 MiB = 33,554,432 bytes

WAL_AUTOCHECKPOINT_PAGES = 128
WAL_HARD_CEILING = 10 MiB = 10,485,760 bytes
WAL_CREATE_GROWTH_BOUND = 2 MiB = 2,097,152 bytes
WAL_APPEND_GROWTH_BOUND = 4 MiB = 4,194,304 bytes
```

`R` is an immutable operational reserve for shutdown accounting, checkpoint
handling, integrity/recovery handling and mandatory SQLite overhead. It is not
a warning threshold and is not included in either ordinary mutation growth
budget. There is no user override.

Growth-bound derivation:

- Phase 1B.2 must query and require SQLite `PRAGMA page_size = 4096` before
  enabling content mutations. The existing source does not verify page size;
  adding that fail-closed check is an explicit future implementation
  requirement in `connection.rs`, not a schema/migration change.
- The maximum conversation subject image is rounded up to 1,024 bytes
  (512 title bytes plus ID, status, timestamps, counters and record header).
- The maximum message subject image is rounded up to 266,240 bytes
  (262,144 content bytes plus IDs, role, sequence, timestamp and record/overflow
  headers). Its conversation update is separately covered.
- Each approved audit row is bounded to 512 bytes; the actual fixed Phase 1B.2
  field set is below the migration’s 8,192-byte aggregate ceiling.
- Create affects eight table/index B-trees. A 160-page-image allowance covers
  two depth-eight split paths per B-tree plus row/header slack.
- Append affects eleven table/index B-trees. A 320-page-image allowance covers
  the 65-page maximum message body, two depth-eight split paths per B-tree, and
  additional conversation/audit/index slack.
- At 4,096 bytes per page, mirrored DB/WAL page images, 24-byte WAL frame
  headers plus the 32-byte WAL header, possible 32-KiB SHM growth, and
  transaction bookkeeping remain below 2 MiB for create and 4 MiB for append.
- `G_CREATE` applies a 4x conservative multiplier and rounds to 8 MiB.
  `G_APPEND` applies an 8x conservative multiplier and rounds to 32 MiB.
- The create WAL allowance is independently capped at 2 MiB and the append WAL
  allowance at 4 MiB. Those bounds include the operation’s new frames and WAL
  header/frame overhead but not the immutable checkpoint reserve.

The implementation must assert these assumptions with maximum-payload,
page-size, page-count, WAL/SHM and B-tree/index inventory tests. Any unsupported
page size, arithmetic overflow, schema-object drift or measured bound violation
fails closed with `IntegrityFailure`; it does not silently enlarge a bound.

The connection must configure and read back exactly:

```sql
PRAGMA wal_autocheckpoint = 128;
```

For runtime page size `P`:

```text
WAL_AUTO_TRIGGER_BYTES(P) =
32 + 128 × (P + 24)

WAL_AUTO_TRIGGER_BYTES(4096) =
527,392 bytes
```

Auto-checkpoint is a trigger, not a hard bound: active readers can delay
checkpoint progress. The 10-MiB WAL hard ceiling and mutation admission below
remain authoritative.

The plan distinguishes:

```text
wal_file_bytes =
physical WAL file length included in current_total

wal_live_bytes =
32 + live_uncheckpointed_frames × (page_size + 24)

current_wal_size =
wal_live_bytes for the WAL hard-ceiling inequality
```

A complete `PASSIVE` checkpoint may leave the physical WAL file allocated, but
it reduces the live copy obligation and permits SQLite to reuse existing file
capacity. The aggregate quota continues to count the full physical
`wal_file_bytes`; the WAL hard ceiling limits live frames that a checkpoint
may need to copy.

The immutable reserve allocation is exact:

```text
CHECKPOINT_COPY_BUDGET = 10 MiB
SHM_GROWTH_BUDGET = 1 MiB
CHECKPOINT_RECOVERY_OVERHEAD = 2 MiB
UNALLOCATED_SAFETY_MARGIN = 3 MiB
TOTAL = 16 MiB
```

No ordinary mutation may spend any reserve component.

Admission for a first execution:

1. the request reaches the single connection-owning worker;
2. acquire `BEGIN IMMEDIATE`;
3. a competing writer that prevents acquisition until the existing deadline
   returns `BusyTimeout`;
4. revalidate database and sidecar identities;
5. measure physical database, WAL and SHM bytes and the live WAL frame
   obligation;
6. select exact `G_CREATE`/`WAL_CREATE_GROWTH_BOUND` or
   `G_APPEND`/`WAL_APPEND_GROWTH_BOUND`;
7. require both:

   ```text
   current_total + G_OPERATION <= U

   current_wal_size + WAL_OPERATION_GROWTH_BOUND
   <= WAL_HARD_CEILING
   ```

8. if both pass, continue; if only the WAL inequality fails, apply the single
   bounded checkpoint policy below; otherwise roll back the empty transaction
   and return `CapacityExceeded`;
9. execute the bounded state and audit writes;
10. before commit, verify row counts, payload limits, page size, schema-object
    inventory and logical page-count growth remain within the reviewed
    operation and WAL envelopes; revalidate file identities and the projected
    physical DB/WAL/SHM commit bound;
11. commit, then measure actual aggregate growth for safe diagnostics and
    fail closed for later mutations if it exceeds the approved envelope.

Actor serialization closes internal races. `BEGIN IMMEDIATE` acquires the
SQLite write lock before measurement and prevents a competing SQLite writer
from changing the database between admission and commit. DB/WAL mirroring is
already covered by `G_OPERATION`; identity checks fail closed for non-SQLite
artifact replacement. The post-commit check is defense in depth, not the
admission decision.

Capacity rejection is pre-admission: no subject, audit, idempotency record or
SQLite write is committed; the operation ID remains available; the runtime
stays readable; only an existing safe non-durable diagnostic may be emitted.
Reads remain available when only ordinary writes are capacity-blocked.

#### Single bounded checkpoint attempt

SQLite checkpointing is not run inside an active write transaction. If the
post-lock measurement shows that only the WAL inequality cannot fit:

1. roll back the still-empty `BEGIN IMMEDIATE` transaction;
2. without resetting the ordinary-operation deadline, run exactly one
   `PRAGMA wal_checkpoint(PASSIVE)`;
3. require its returned log/checkpointed frame counts to show whether live
   frames were reduced; a busy reader may prevent sufficient progress;
4. reacquire `BEGIN IMMEDIATE` under the same deadline;
5. revalidate identities and remeasure physical DB/WAL/SHM plus live WAL;
6. continue only when both admission inequalities now pass;
7. otherwise roll back and return `CapacityExceeded`.

There is no checkpoint loop and no deadline reset. The PASSIVE attempt may
increase the database file while leaving the WAL file allocated, so the second
`current_total` measurement is authoritative.

#### Pre-existing oversized WAL

```text
NORMAL_IN_CONTRACT_WAL =
wal_live_bytes <= WAL_HARD_CEILING

PRE_EXISTING_OUT_OF_CONTRACT_WAL =
wal_live_bytes > WAL_HARD_CEILING
OR wal_file_bytes > WAL_HARD_CEILING at open
```

After integrity and path checks, bounded reads may remain available for an
out-of-contract WAL, but mutations remain blocked. A single bounded recovery
checkpoint may be attempted outside a mutation only when:

```text
current_total
+ current_wal_size
+ CHECKPOINT_RECOVERY_OVERHEAD
<= H
```

For an oversized physical WAL, recovery uses bounded
`PRAGMA wal_checkpoint(TRUNCATE)` so success can be verified from both returned
frame counts and file length. After recovery, mutation eligibility requires:

```text
current_total <= U
AND wal_live_bytes <= WAL_HARD_CEILING
AND wal_file_bytes <= WAL_HARD_CEILING
```

Otherwise the runtime fails closed for mutations without consuming an
operation ID or claiming that the immutable reserve covers already oversized
state.

Shutdown first closes new admission. No mutation starts after intake closes.
Checkpoint/shutdown handling is not charged against `G_OPERATION`; it may use
`R`. A busy or failed checkpoint returns the existing controlled shutdown error
and never reports a false clean shutdown.

### Five-operation transaction contract

| Exact method | Request -> output | Actor/transaction | Audit | Idempotency | Capacity | NotFound | Ordering/pagination | Restart | Local errors / global health |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `create_conversation` | `CreateConversationRequest -> ConversationRecord` | Single worker; `BEGIN IMMEDIATE`; conversation + audit | One `conversation.created` success event | Global operation ID; exact replay or conflict/inconsistent | `G_CREATE` admission against `U` | Not applicable | Not applicable | Original row and replay survive reopen | Validation/conflict/capacity/busy/deadline are local; inconsistency/integrity/unrecoverable worker failure may update global health |
| `get_conversation` | `GetConversationRequest -> ConversationRecord` | Single worker; bounded read-only query | None | None | None | Missing valid ID -> `ConversationNotFound` | Exact ID only | Same record survives reopen | Invalid/NotFound/busy/deadline local; infrastructure integrity/unavailability may update health |
| `list_conversations` | `ListConversationsRequest -> ConversationPage` | Single worker; bounded read-only query | None | None | None | Empty database -> empty page | `updated_at_ms DESC, id DESC`; typed full-key cursor; `1..=100` | Deterministic for unchanged data after reopen | Invalid/busy/deadline local; infrastructure integrity/unavailability may update health |
| `append_message` | `AppendMessageRequest -> MessageRecord` | Single worker; `BEGIN IMMEDIATE`; message + conversation revision/sequence + audit | One `message.appended` success event | Global operation ID; exact replay or conflict/inconsistent | `G_APPEND` admission against `U` | Missing conversation -> `ConversationNotFound` | Allocates contiguous sequence | Original row/sequence/replay survive reopen | Validation/NotFound/conflict/capacity/busy/deadline local; inconsistency/integrity/unrecoverable worker failure may update global health |
| `list_messages` | `ListMessagesRequest -> MessagePage` | Single worker; bounded read-only query | None | None | None | Missing conversation -> `ConversationNotFound`; existing empty -> empty page | Per conversation; `sequence_no ASC, id ASC`; `after_sequence_no`; `1..=100` | Deterministic after reopen | Invalid/NotFound/busy/deadline local; infrastructure integrity/unavailability may update health |

### Retry and deadline semantics

The runtime owns the existing ten-second ordinary-operation deadline.
Cancellation or timeout before dequeue writes nothing. Once a mutation
transaction starts, it completes or rolls back; result delivery may be
suppressed after caller timeout. A committed mutation is recoverable through
its operation ID. Rolled-back and controlled no-write failures create no
idempotency record and are re-evaluated on retry. The guarantee is exactly-once
committed mutation, not memoized failure responses.

## Security Considerations

### Threat model

| Threat | Required Phase 1B.2 control |
| --- | --- |
| Prompt/memory injection | Stored content is untrusted data; Phase 1B.2 never interprets or promotes it to instructions or memory |
| SQL injection | Static parameterized SQL only; no caller-controlled identifiers |
| Oversized content/resource exhaustion | UTF-8 byte limits before queue/bind, bounded pages, existing queue/deadline limits, hard write gate |
| Orphan/duplicate messages | Foreign keys, actor serialization, transaction-owned sequence allocation and unique constraints |
| Lost response/retry duplication | UUID-v4 operation ID coupled to a unique matching audit event |
| Audit leakage | Audit events contain IDs and allowlisted metadata only, never title/content/path/SQL/error dumps |
| Partial commit | State and required audit event share one transaction |
| Database unhealthy/replaced/locked | Existing path, integrity, migration and health controls fail closed before content operations |
| Hard-limit overrun | Actor-owned `BEGIN IMMEDIATE`, DB/WAL/SHM measurement, exact `G_CREATE`/`G_APPEND` admission below `H - R`, and a separate immutable 16-MiB operational reserve |
| Log leakage | No conversation title/content or raw database error in logs |
| Frontend/LLM authority | No Tauri CRUD command, frontend adapter or model-call path |
| Remote egress | No network or cross-repository code in scope |
| Disk theft | Existing plaintext-SQLite residual risk remains explicit; no encryption claim |

### Blocking security conditions

Any of the following blocks implementation completion:

- state can commit without its required audit event;
- a duplicate `operation_id` can create a second record or sequence;
- title/message content reaches audit metadata, logs or public errors;
- a content operation bypasses queue, deadline, hard-size or health gates;
- an ordinary mutation can enter or consume the immutable 16-MiB reserve;
- page size, DB/WAL/SHM accounting or the reviewed growth envelope is not
  enforced fail closed;
- any raw SQL/path/generic storage authority crosses Tauri or frontend;
- implementation requires a schema/dependency change not approved here;
- any unresolved Critical/High finding.

## Migration and Compatibility Considerations

Phase 1B.2 must use the existing version-1 schema unchanged:

```text
PHASE_1B2_SCHEMA_DECISION = NO_SCHEMA_CHANGE_REQUIRED
IDEMPOTENCY_SCHEMA = CURRENT_SCHEMA_SUFFICIENT

MIGRATION_SHA =
62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d

STRUCTURAL_FINGERPRINT =
37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77

TABLES = 5
EXPLICIT_INDEXES = 7
SQLITE_AUTOINDEXES = 7
SQLITE_SEQUENCE = 0
MIGRATION_2 = ABSENT
```

No new dependency is required. If a safe implementation cannot satisfy the
contract with the existing schema and dependencies, stop. Do not mutate
migration 1 or introduce migration 2 without a separately reviewed plan/ADR.

Rows written by Phase 1B.2 remain compatible with Phase 1B.1 and future
forward-only slices. A code rollback disables consumers and removes repository
code; it never deletes or downgrades the database.

## Implementation Steps

No implementation step is authorized by this planning task. After separate
human authorization:

1. Re-read exact `main`, this plan, the readiness audit and active skills.
2. Confirm migration/dependency/source invariance and create the matching
   implementation completion report skeleton.
3. Add validated private domain types and operation errors.
4. Add private audit idempotency/append helper for only the two approved event
   types.
5. Add conversation repository operations.
6. Add message repository operations and atomic sequence/revision updates.
7. Add the reviewed 4-KiB page-size invariant, immutable 16-MiB operational
   reserve, exact 8-MiB create and 32-MiB append growth envelopes, and
   actor-owned `BEGIN IMMEDIATE` admission without changing the 4-GiB hard
   limit.
8. Extend the existing worker request protocol and manager methods without
   adding Tauri commands.
9. Add split repository tests and negative security fixtures.
10. Run narrow repository tests, then complete Rust/frontend/contract/security
   regression checks.
11. Review the complete diff, update canonical status documents and stop at a
    separate exact-head review/merge gate.

## Tests

The separately authorized implementation must provide this deterministic
matrix using generated temporary roots only.

### Create conversation

- create with `NULL` title;
- create with empty title;
- prove `NULL` and empty title are distinct canonical payloads;
- accept exactly 512 UTF-8 title bytes and reject 513;
- same operation ID/same payload returns the original row;
- same operation ID/different title returns `IdempotencyConflict`;
- byte-different Unicode title returns `IdempotencyConflict`;
- create then replay returns the deterministic initial projection;
- create then append then replay ignores current mutable counters/timestamp;
- create then multiple appends then replay still returns the initial projection;
- restart then replay after append returns the initial projection;
- replay result differs intentionally from the current `get_conversation`
  projection after append;
- missing subject, actor mismatch or audit/conversation timestamp mismatch
  returns `IdempotencyRecordInconsistent`;
- replay creates no second audit event;
- forced audit failure rolls back the conversation and operation ID.

### Get conversation

- return an existing exact DTO projection;
- missing valid ID returns `ConversationNotFound`;
- invalid/noncanonical UUID returns `InvalidInput`;
- restart preserves the same projection;
- operation-local NotFound does not poison health and a later valid read works.

### List conversations

- empty database returns an empty page;
- stable `updated_at_ms DESC, id DESC` ordering;
- equal-timestamp tie-break by descending ID;
- typed full-key cursor continuation;
- no duplicate or skipped rows for unchanged data;
- invalid zero/excessive limit and invalid cursor;
- restart-stable ordering;
- reads work while mutations are capacity-blocked.

### Append message

- first sequence is 1 and later sequences are contiguous;
- conversation timestamp, next sequence and revision update exactly;
- missing conversation returns controlled NotFound with no write;
- invalid role and empty content rejection;
- accept exactly 262,144 UTF-8 content bytes and reject 262,145;
- same operation ID/same payload returns the original message;
- same operation ID/different content returns `IdempotencyConflict`;
- message, conversation update and audit are atomic under forced failures;
- restart and replay returns the original message and sequence.

### List messages

- existing empty conversation returns an empty page;
- missing conversation returns `ConversationNotFound`;
- stable `sequence_no ASC, id ASC` ordering;
- scoped `after_sequence_no` pagination;
- cross-conversation isolation and no message leakage;
- restart-stable results.

### Concurrency and idempotency

- concurrent same ID/same payload produces exactly one subject and one audit
  event and both callers receive the original result;
- concurrent same ID/different payload produces one success and one
  `IdempotencyConflict`;
- no raw unique-constraint error escapes;
- global operation-ID reuse across create and append conflicts;
- inconsistent event type, subject type, correlation, missing subject and
  unreproducible payload each return `IdempotencyRecordInconsistent`, transition
  health to `IntegrityFailed`, and close content intake;
- no retention/pruning/deletion path removes operation records.

### Capacity and reserve

- assert page size is exactly 4,096 bytes before enabling mutations;
- exact admission when `current + G_OPERATION <= U`;
- exact rejection when the inequality would be false;
- preserved-reserve algebra with `H = 4 GiB`, `R = 16 MiB`,
  `G_CREATE = 8 MiB`, and `G_APPEND = 32 MiB`;
- create maximum-payload physical-growth bound;
- append maximum-content physical-growth bound;
- DB + WAL + SHM measurement and identity revalidation;
- configure and read back `wal_autocheckpoint = 128`;
- calculate `WAL_AUTO_TRIGGER_BYTES(P)` from runtime page size;
- maximum create live-WAL growth is at most 2 MiB;
- maximum append live-WAL growth is at most 4 MiB;
- repeated writes cannot exceed the 10-MiB live-WAL ceiling;
- a busy reader prevents sufficient PASSIVE checkpoint progress, blocks writes,
  and preserves reads;
- pre-existing oversized WAL recoverable case;
- pre-existing oversized WAL unsafe case;
- checkpoint transient duplication fits inside the 16-MiB reserve;
- reserve components sum exactly to 16 MiB;
- competing second SQLite writer before `BEGIN IMMEDIATE` returns
  `BusyTimeout`;
- external-writer TOCTOU test proves measurement occurs after the write lock;
- rejection creates no subject or audit event and does not consume operation ID;
- reads remain available after capacity rejection;
- shutdown and checkpoint near the threshold do not report false clean
  shutdown;
- capacity/WAL rejection creates no audit event and consumes no operation ID;
- ordinary mutations never consume the immutable reserve;
- deterministic lowered test limits exercise the same algebra without writing a
  real user profile.

### Error and health separation

For `InvalidInput`, `ConversationNotFound`, `IdempotencyConflict`,
`CapacityExceeded`, `BusyTimeout` and `DeadlineExceeded`, prove:

```text
runtime remains accepting
healthy/warning projection is not replaced by unavailable
later valid operation succeeds
```

Also prove integrity failure, connection loss and unrecoverable worker failure
produce their selected controlled global state without leaking raw errors.

### Persistence and lifecycle

- conversation/message/audit readback survives clean reopen;
- committed WAL content survives restart and a partial transaction does not;
- accepted operations drain or roll back at shutdown;
- no mutation begins after shutdown intake closes;
- queue saturation and deadlines remain bounded;
- schema checksum, structural fingerprint and inventory remain unchanged.

### Boundary regressions

- exactly one storage Tauri command remains:
  `get_storage_runtime_status`;
- no frontend content command, raw SQL, path or generic invoke exists;
- migration checksum/fingerprint and table/index counts remain exact;
- `npm run test:storage-runtime-contract`;
- all runtime-store Rust tests;
- all 67 inference tests;
- full Rust suite;
- `cargo check --all-targets --locked`;
- `cargo clippy --all-targets --locked`;
- scoped rustfmt check;
- TypeScript validation and production build;
- `npm audit --omit=dev`;
- repository secret and touched-warning checks.

Tests use generated temporary roots only. No real user profile, live model,
network, deploy or production system is touched.

## Acceptance Criteria

Phase 1B.2 can be classified `IMPLEMENTED_AND_VERIFIED` only when:

1. The exact reviewed implementation is based on the authorized canonical main.
2. The version-1 migration, schema fingerprint, dependencies and public Tauri
   command inventory remain unchanged.
3. All five private operations are typed, bounded and executed only by the
   Rust-owned actor.
4. Exactly two mutations use `BEGIN IMMEDIATE`, operation IDs, capacity
   admission and atomic privacy-safe audit events; the three reads use none of
   those mutation semantics.
5. Exact committed-success retries are idempotent across restart and concurrent
   duplicates; create replay reconstructs its initial projection after later
   appends; globally mismatched operation-ID reuse fails closed; and no-write
   failures remain retryable.
6. Message sequences are unique, contiguous for successful commits and stable
   across restart.
7. Bounded reads use documented stable ordering and never return raw database
   types or authority.
8. Invalid, oversized, orphaned, resource-limited, timed-out and unhealthy-store
   paths produce no unauthorized write and no sensitive error/log output.
9. The 16-MiB reserve remains immutable; exact 8-MiB create and 32-MiB append
   envelopes, 128-page auto-checkpoint trigger, 10-MiB live-WAL ceiling and
   2-/4-MiB per-operation WAL bounds are enforced with post-lock DB/WAL/SHM
   accounting; capacity rejection writes no audit row.
10. Operation-local errors preserve healthy/warning runtime state, while
    inconsistent idempotency evidence and infrastructure integrity failures
    apply their explicit fail-closed health transition.
11. No Tauri CRUD command, frontend integration, model/tool/network call, task
   behavior, deletion/export or memory promotion exists.
12. Required narrow and full regression checks pass with recorded commands and
    counts.
13. No unresolved Critical/High security finding remains.
14. Architecture, ADR, capability, security and roadmap documents remain
    truthful.
15. An independent exact-head review and separate implementation/merge
    authorization complete.

## Rollback Strategy

Before merge, revert only the Phase 1B.2 implementation diff.

After merge:

- disable/remove consumers without deleting rows;
- preserve migration 1 and existing content;
- forward-fix repository behavior;
- never downgrade, recreate or silently clear the database;
- keep read/export/delete recovery paths available only when their separately
  authorized phases exist.

Because the schema and dependencies do not change, code rollback does not
require a database migration. It also does not erase content already written.

## Documentation Updates

This planning task may create/update only:

- the Phase 1B.2 readiness audit;
- this plan;
- its planning completion report;
- the stale Phase 1B status sentence in `AGENTS.md`.

An authorized implementation later updates ADR 0002, README, capability matrix,
security gates, master roadmap and the implementation completion report based
on executable evidence.

## Risks and Blockers

| ID | Risk/blocker | Current disposition |
| --- | --- | --- |
| B2-01 | Original Phase 1B plan requires state/audit atomicity, while broad AuditStore work is assigned to 1B.3 | Human-approved direction: private append-only helper for only `conversation.created` and `message.appended`; exact atomic contract passed substantive planning review |
| B2-02 | New writes create unknown-outcome retry risk after caller timeout | Global operation-ID contract, canonical comparison, concurrency, restart, retention and inconsistency behavior are exact and passed substantive planning review |
| B2-03 | Current hard limit is status evidence, not yet a content-write gate | Immutable 16-MiB reserve, exact growth bounds and actor-owned admission passed planning review; executable proof remains mandatory during separately authorized implementation |
| B2-04 | `worker.rs` and `runtime_store/tests.rs` are already large | Keep SQL/repository logic in cohesive new modules and split repository tests |
| B2-05 | Active `AGENTS.md` phase-status sentence predates merged Phase 1B.1 | Reconcile in this planning-only diff without authorizing implementation |

## Open Questions

Repository evidence and the passed substantive review resolve the schema,
service ownership, DTO, error, idempotency, audit and reserve design questions
for planning. The remaining authority question is whether a human separately
authorizes implementation with the required executable proof.

Final human-direction table:

```text
HD_1B2_01 =
ACCEPT

HD_1B2_02 =
ACCEPT

HD_1B2_03 =
ACCEPT

HD_1B2_04 =
ACCEPT / EXECUTABLE PROOF REQUIRED DURING IMPLEMENTATION

HD_1B2_05 =
ACCEPT

NO_UPDATE_OPERATION = TRUE
NO_DELETE_OPERATION = TRUE
PUBLIC_TAURI_CONTENT_COMMANDS = NOT AUTHORIZED
```

These directions are planning decisions, not code authority. An implementation
prompt must cite the exact reviewed planning diff and separately authorize its
allowlisted source files, tests and stop rules.

## GO / CONDITIONAL_GO / NO_GO

```text
PHASE_1B2_AUDIT = COMPLETE_IN_PLANNING_DIFF
PHASE_1B2_PLANNING_REVIEW = PHASE_1B2_PLANNING_REVIEW_PASS
PLANNING_FINDINGS = CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 0 / INFO 0
PLAN_RESULT = CONDITIONAL_GO
CONDITIONS =
  SEPARATE HUMAN IMPLEMENTATION AUTHORIZATION
PHASE_1B2_IMPLEMENTATION_AUTHORIZATION = NOT GRANTED
PHASE_1B2 = ELIGIBLE_FOR_SEPARATE_IMPLEMENTATION_AUTHORIZATION
PHASE_1B2_IMPLEMENTATION = NOT AUTHORIZED
PHASE_1B3 = NOT AUTHORIZED
PRODUCTION_WRITES = 0
DEPLOYMENTS = 0
```
