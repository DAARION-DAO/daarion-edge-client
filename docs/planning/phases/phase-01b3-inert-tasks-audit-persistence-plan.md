# Phase 1B.3 — Inert Tasks and Audit Persistence Plan

- Plan date: 2026-07-25
- Exact main: `255e71b4467fbe7d521c3022c1cd0afc76197ecf`
- Audit: [Phase 1B.3 readiness audit](../../audits/PHASE_1B3_INERT_TASKS_AUDIT_PERSISTENCE_READINESS_AUDIT_2026-07-25.md)
- Final planning review: `PASS_WITH_NONBLOCKING_FINDINGS`
- Plan result: `CONDITIONAL_GO`
- Implementation: `NOT AUTHORIZED`

## Objective

Define the smallest private Rust vertical slice that can persist and read inert
task records and read privacy-safe audit evidence. Preserve the verified
Phase 1B.1/1B.2 runtime owner, schema, capacity reserve, lifecycle, and
conversation/message behavior.

The slice must create data only. A stored task is not executable, scheduled,
delegated, retried, planned, verified, cancelled, or recovered by an Agent
Supervisor.

## Exact Main

```text
CANONICAL_MAIN =
255e71b4467fbe7d521c3022c1cd0afc76197ecf

PHASE_1B2_IMPLEMENTATION_HEAD =
c2fdcc5a234779c7ad886ee5aa0d0762c938a59d

PHASE_1B2_MERGE =
ec99bf70d6ada94bc1caae9886cca25ad42852f9
```

Implementation may begin only from a fresh clean worktree at the then-current
canonical main after a separate human authorization names the exact reviewed
planning head. This plan does not authorize a branch, commit, push, or PR.

## Current Source Inventory

- `src-tauri/migrations/runtime_state/0001_runtime_state_initial.sql`
  already owns the five-table schema, inert `tasks`, and structured
  `audit_events`.
- `src-tauri/src/runtime_store/worker.rs` owns the one bounded request queue,
  one SQLite worker, absolute deadlines, and priority shutdown.
- `src-tauri/src/runtime_store/repositories/mod.rs` implements the five
  Phase 1B.2 private conversation/message operations.
- `src-tauri/src/runtime_store/repositories/unit_of_work.rs` implements
  global operation-ID lookup, success audit insertion, capacity admission,
  physical-growth verification, and checkpoint handling.
- `src-tauri/src/runtime_store/models.rs` has conversation/message DTOs but no
  task or audit-read DTOs.
- `src-tauri/src/runtime_store/error.rs` has a private safe-code content error
  model but no task/audit NotFound variants.
- `src-tauri/src/runtime_store/config.rs` has proven create/append bounds, not
  a task-record mutation kind.
- `src-tauri/src/runtime_store/commands.rs` and Tauri registration expose only
  the storage status command.

Current Phase 1B.2 behavior is a preserved baseline. It must not be
reimplemented or reclassified as missing.

## Scope

One future implementation commit/package may add:

- exactly one inert task mutation;
- exactly two bounded task reads;
- exactly two bounded audit reads;
- task DTOs and a canonical non-executable task-kind value;
- closed audit values and fail-closed database decoding;
- operation-specific audit construction for all three supported mutation
  events;
- task/audit repository modules;
- five private manager/worker paths;
- task/audit safe error variants;
- one task-record capacity mutation kind;
- focused repository, worker, lifecycle, privacy, and boundary tests;
- truthful architecture/security/status documentation.

## Explicit Non-Goals

- task update, state transitions, execution, planner, executor, verifier;
- retries, cancellation semantics, scheduling, loops, Agent Supervisor;
- task deletion, export, reset, recovery UI, retention;
- tools, models, prompts, memory extraction, embeddings;
- Reticulum, LXMF, network, Supabase, remote synchronization, wallet;
- public task/audit Tauri commands, frontend clients, IPC DTOs;
- generic audit append, generic SQL, arbitrary query/filter language;
- migration 2, new tables, columns, indexes, triggers, dependencies;
- using `tasks.idempotency_key`;
- Phase 1B.4, Phase 1C, or later behavior.

## Repository Ownership

All planned behavior belongs to the private Rust runtime-store boundary in
`daarion-edge-client`. No contract is added to `loval-echoes`, Supabase,
Reticulum/LXMF, a sidecar, or private infrastructure.

The SQLite worker remains the only writer and connection owner. The frontend
receives no new authority or status projection.

## Schema Decision

```text
PHASE_1B3_SCHEMA_DECISION = NO_SCHEMA_CHANGE_REQUIRED
MIGRATION_2 = ABSENT
NEW_TABLES = 0
NEW_COLUMNS = 0
NEW_INDEXES = 0
NEW_TRIGGERS = 0
```

Migration 1 already supports the complete selected slice. Any implementation
discovery that requires a schema change stops the task:

```text
STATUS = SCHEMA_CHANGE_REQUIRED / NOT AUTHORIZED
```

The verified invariants must remain:

```text
MIGRATION_SHA =
62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d
STRUCTURAL_FINGERPRINT =
37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77
TABLES = 5
EXPLICIT_INDEXES = 7
SQLITE_AUTOINDEXES = 7
SQLITE_SEQUENCE = 0
```

## Files and Modules Expected to Change

Expected future source scope:

- `src-tauri/src/runtime_store/models.rs`;
- `src-tauri/src/runtime_store/error.rs`;
- `src-tauri/src/runtime_store/config.rs`;
- `src-tauri/src/runtime_store/worker.rs`;
- `src-tauri/src/runtime_store/repositories/mod.rs`;
- `src-tauri/src/runtime_store/repositories/unit_of_work.rs`;
- new `src-tauri/src/runtime_store/repositories/tasks.rs`;
- new `src-tauri/src/runtime_store/repositories/audit_events.rs`;
- focused existing/new runtime-store test modules.

Expected status/documentation scope:

- this plan and its future implementation completion report;
- capability matrix, security gates, roadmap, and ADR 0002 only where exact
  implementation status changes.

Forbidden future scope:

- SQL migration files;
- Cargo/npm manifests or lockfiles;
- Tauri capabilities/registration for task or audit operations;
- TypeScript/TSX application source;
- production configuration.

## Exact Operation Inventory

| # | Method | Kind | Request | Result |
|---|---|---|---|---|
| 1 | `record_inert_task` | mutation | `RecordInertTaskRequest` | `TaskRecord` |
| 2 | `get_task` | read | `GetTaskRequest` | `TaskRecord` |
| 3 | `list_tasks` | read | `ListTasksRequest` | `TaskPage` |
| 4 | `get_audit_event` | read | `GetAuditEventRequest` | `AuditEventRecord` |
| 5 | `list_audit_events` | read | `ListAuditEventsRequest` | `AuditPage` |

```text
PRIVATE_OPERATIONS = 5
MUTATIONS = 1
TASK_READS = 2
AUDIT_READS = 2
TASK_UPDATES = 0
TASK_DELETES = 0
GENERIC_AUDIT_APPEND = 0
```

No alias, generic executor, generic repository, or caller-supplied SQL may
expand this list.

## DTOs

The exact private model is:

```rust
pub(crate) struct RecordInertTaskRequest {
    pub(crate) operation_id: String,
    pub(crate) actor: ContentActor,
    pub(crate) conversation_id: Option<String>,
    pub(crate) task_kind: InertTaskKind,
}

pub(crate) struct GetTaskRequest {
    pub(crate) task_id: String,
}

pub(crate) struct ListTasksRequest {
    pub(crate) limit: u32,
    pub(crate) cursor: Option<TaskCursor>,
}

pub(crate) struct TaskRecord {
    pub(crate) id: String,
    pub(crate) conversation_id: Option<String>,
    pub(crate) task_kind: InertTaskKind,
    pub(crate) state: InertTaskState,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
    pub(crate) revision: i64,
}

pub(crate) struct TaskCursor {
    pub(crate) updated_at_ms: i64,
    pub(crate) id: String,
}

pub(crate) struct TaskPage {
    pub(crate) items: Vec<TaskRecord>,
    pub(crate) next_cursor: Option<TaskCursor>,
}

pub(crate) enum InertTaskState {
    Created,
}
```

`TaskRecord` intentionally omits `idempotency_key`. Phase 1B.3 must verify the
database value is `NULL` and assigns it no application meaning.

`ListTasksRequest` intentionally omits a conversation filter. Migration 1 has
`tasks_state_updated_idx(state, updated_at_ms, id)` for the selected global
ordering, while `tasks_conversation_idx(conversation_id, id)` does not cover
`updated_at_ms`. Adding a chronological conversation filter would therefore
either introduce an unbounded filtered scan, change ordering/cursor semantics,
or require a new index. All three are outside this no-migration slice.

Audit-read DTOs:

```rust
pub(crate) struct GetAuditEventRequest {
    pub(crate) event_id: String,
}

pub(crate) struct ListAuditEventsRequest {
    pub(crate) limit: u32,
    pub(crate) after_sequence_no: Option<i64>,
}

pub(crate) struct AuditEventRecord {
    pub(crate) sequence_no: i64,
    pub(crate) event_id: String,
    pub(crate) event_type: AuditEventType,
    pub(crate) actor: AuditActor,
    pub(crate) subject_type: AuditSubjectType,
    pub(crate) subject_id: Option<String>,
    pub(crate) outcome: AuditOutcome,
    pub(crate) reason_code: Option<AuditReasonCode>,
    pub(crate) correlation_id: Option<String>,
    pub(crate) created_at_ms: i64,
}

pub(crate) struct AuditCursor {
    pub(crate) after_sequence_no: i64,
}

pub(crate) struct AuditPage {
    pub(crate) items: Vec<AuditEventRecord>,
    pub(crate) next_cursor: Option<AuditCursor>,
}
```

## Task-Kind Contract

`InertTaskKind` is a constrained opaque identifier, not an executable enum and
not free text.

```text
BYTE_LENGTH = 1..=64
CHARSET = ASCII lowercase letters, digits, dot, underscore, hyphen
GRAMMAR = ^[a-z][a-z0-9]*(?:[._-][a-z0-9]+)*$
CASE = lower-case only
NORMALIZATION = none
COMPARISON = exact bytes
```

The request must already be canonical. The runtime does not lowercase, trim,
Unicode-normalize, map, route, dispatch, or otherwise interpret it.

`task_kind` is data only. It is never:

- a prompt or instruction;
- executable code or a function name;
- a tool/model/agent selector;
- a network route;
- a scheduler command;
- a permission or capability token.

No product-specific task kinds are invented by this phase.

## Typed Audit Model

`ContentActor` remains the canonical closed actor enum for existing DTOs.
Phase 1B.3 adds `pub(crate) type AuditActor = ContentActor` so there is one
representation and existing Phase 1B.2 bytes remain unchanged.

`AuditEventType` represents every migration-1 allowed value for fail-closed
read decoding:

- `ConversationCreated` -> `conversation.created`;
- `ConversationDeleted` -> `conversation.deleted`;
- `MessageAppended` -> `message.appended`;
- `TaskCreated` -> `task.created`;
- `TaskRecorded` -> `task.recorded`;
- `TaskDeleted` -> `task.deleted`;
- `RuntimeContentDeleted` -> `runtime.content_deleted`;
- `ExportCompleted` -> `export.completed`;
- `StorageRecoveryRequired` -> `storage.recovery_required`.

`AuditSubjectType` represents:

- `Conversation`;
- `Message`;
- `Task`;
- `Runtime`;
- `Export`;
- `Storage`.

`AuditOutcome` represents `Success`, `Denied`, and `Failed`.

`AuditReasonCode` has no accepted Phase 1B.3 variants. `NULL` decodes to
`None`; a non-NULL database value fails closed as `IntegrityFailure`. A future
phase must add reviewed closed variants before writing or accepting reason
codes. No arbitrary string constructor is permitted.

Schema reservations remain readable as typed events but are not writable
through Phase 1B.3. Write authority is a private closed sum:

```rust
enum SuccessAuditEvent {
    ConversationCreated { /* exact typed fields */ },
    MessageAppended { /* exact typed fields */ },
    TaskRecorded { /* exact typed fields */ },
}
```

The operation-specific variants construct exact event/subject combinations.
The low-level SQL insert remains private to `unit_of_work.rs`; repositories
cannot supply arbitrary event, actor, subject, outcome, reason, or metadata
strings.

Typed database decoding must validate more than individual enum membership:

- `conversation.created` and `conversation.deleted` require subject
  `conversation` and a non-NULL canonical UUID subject ID;
- `message.appended` requires subject `message` and a non-NULL canonical UUID
  subject ID;
- `task.created`, `task.recorded`, and `task.deleted` require subject `task`
  and a non-NULL canonical UUID subject ID;
- `runtime.content_deleted` requires subject `runtime`;
- `export.completed` requires subject `export`;
- `storage.recovery_required` requires subject `storage`;
- `sequence_no` must be positive and `created_at_ms` nonnegative;
- event, subject, and correlation IDs must satisfy the canonical UUID-v4
  contract when present.

An individually valid but semantically incompatible event/subject pair fails
closed as `IntegrityFailure`. Reserved runtime/export/storage event subject-ID
requirements remain exactly as permitted by migration 1; no stronger
identifier meaning is invented in this slice.

## Task Event Decision

```text
PHASE_1B3_TASK_EVENT = task.recorded
TASK_CREATED_EVENT = SCHEMA_RESERVATION / PHASE_1C
```

`task.recorded` truthfully means an inert row was persisted. It does not mean a
goal, executable task, or Supervisor lifecycle object was created.

## Idempotency Contract

`operation_id` is the only Phase 1B.3 mutation idempotency key:

```text
audit_events.event_id = operation_id
audit_events.correlation_id = operation_id
tasks.idempotency_key = NULL
```

The operation ID is a canonical lowercase UUID v4 and is global across
conversation creation, message append, and inert task recording.

- first execution inserts exactly one task and one audit event;
- same operation ID plus same canonical actor, optional conversation, and task
  kind returns the original `TaskRecord`;
- same operation ID plus different actor, conversation, or task kind returns
  `IdempotencyConflict`;
- collision with `conversation.created` or `message.appended` returns
  `IdempotencyConflict`;
- an unsupported event, missing task, wrong subject, wrong outcome, non-NULL
  reason, wrong correlation, mismatched timestamp/state/revision, or non-NULL
  task idempotency value returns `IdempotencyRecordInconsistent` and poisons
  content intake;
- replay after restart returns the original record without a second row/event.

Replay reconstructs the canonical request from the immutable task row and its
audit event. No unstable JSON serialization or check-then-insert outside the
transaction participates.

Rejected mutations—validation, missing conversation, capacity, busy timeout,
or deadline before commit—create neither row and do not consume the operation
ID.

Phase 1C must separately decide whether and how to own
`tasks.idempotency_key`. It must not silently inherit Phase 1B.3 operation
semantics.

## Transaction Contract

`record_inert_task`:

1. validate request before queueing where already consistent with Phase 1B.2;
2. enqueue through the existing absolute-deadline manager path;
3. start one `BEGIN IMMEDIATE`;
4. resolve global operation-ID evidence inside the transaction;
5. if `conversation_id` is present, verify the parent exists;
6. perform final DB/WAL/SHM admission after acquiring the write lock;
7. allocate one canonical lowercase UUID v4 task ID;
8. allocate one runtime-owned logical timestamp;
9. insert task with:
   - state `created`;
   - equal created/updated timestamps;
   - revision `0`;
   - `idempotency_key = NULL`;
10. insert one typed `task.recorded` success event;
11. verify exactly one task and one audit event with matching invariants;
12. enforce the physical growth bounds;
13. commit both records or roll back both.

Audit insert failure, verification failure, deadline, capacity failure, SQLite
error, or panic before commit cannot produce a successful orphan mutation.

## Audit Field Mapping

| Field | Exact Phase 1B.3 value |
|---|---|
| `event_id` | request `operation_id` |
| `event_type` | `task.recorded` |
| `actor_type` | closed request actor |
| `subject_type` | `task` |
| `subject_id` | generated task ID |
| `outcome` | `success` |
| `reason_code` | `NULL` |
| `correlation_id` | request `operation_id` |
| `created_at_ms` | same logical timestamp as task creation |

The event contains no task kind, conversation title, message content, prompt,
payload, model data, path, SQL, raw error, environment value, or metadata blob.

## Read and Pagination Contracts

### `get_task`

- validates canonical lowercase UUID v4;
- fixed query by primary key;
- returns `TaskNotFound` when absent;
- decodes state only as `InertTaskState::Created`;
- verifies `idempotency_key` is `NULL`;
- never joins or returns conversation/message content;
- remains available when ordinary writes are capacity-blocked;
- returns normal lifecycle errors after shutdown intake closes.

### `list_tasks`

- limit range is `1..=100`;
- optional cursor validates nonnegative `updated_at_ms` and canonical UUID v4;
- stable order is `updated_at_ms DESC, id DESC`;
- every query includes `state = 'created'`, using
  `tasks_state_updated_idx(state, updated_at_ms, id)`;
- continuation predicate uses the complete pair:
  `(updated_at_ms, id) < (?, ?)`;
- fetches `limit + 1` to determine `next_cursor`;
- Phase 1B.3 provides a global list only;
- no conversation filter, alternate sort, or caller-supplied predicate exists;
- an empty valid page returns `items = []`, not NotFound;
- result is stable across restart for unchanged data.

### `get_audit_event`

- validates canonical lowercase UUID v4;
- fixed query by unique `event_id`;
- returns `AuditEventNotFound` when absent;
- decodes all values through closed types;
- any unknown or currently unsupported reason code fails as integrity failure;
- returns no joined task/message/conversation content.

### `list_audit_events`

- limit range is `1..=100`;
- optional `after_sequence_no` must be positive;
- stable order is `sequence_no ASC`;
- continuation predicate is `sequence_no > ?`;
- fetches `limit + 1` and returns `AuditCursor`;
- an empty valid page returns `items = []`;
- Phase 1B.3 adds no filters;
- future filters require closed enums and separate review.

All reads use fixed SQL, one worker connection, one absolute deadline, and the
existing intake/shutdown boundary. Operation-local NotFound/invalid errors do
not change global runtime health.

## Error and Health Contract

Select option A: minimally extend the proven private
`ContentOperationError`/`ContentOperationErrorCode` model.

Add:

- `TaskNotFound` -> `content_task_not_found`;
- `AuditEventNotFound` -> `content_audit_event_not_found`.

Retain:

- `InvalidInput`;
- `ConversationNotFound`;
- `IdempotencyConflict`;
- `IdempotencyRecordInconsistent`;
- `CapacityExceeded`;
- `BusyTimeout`;
- `DeadlineExceeded`;
- `Unavailable`;
- `IntegrityFailure`;
- `Internal`.

This avoids a risky Phase 1B.2-wide rename and avoids duplicated task/audit
mapping logic. The content-oriented type name is recorded as naming debt for a
future dedicated refactor, not changed in this slice.

Only `IdempotencyRecordInconsistent` and `IntegrityFailure` poison content
intake. Invalid input, all NotFound variants, expected idempotency conflict,
capacity rejection, busy timeout, and deadline are operation-local.
`Unavailable` and `Internal` retain existing worker health handling.

Display/log output remains safe codes only. No error includes task kind,
content, SQL, paths, environment data, or raw SQLite text.

## Capacity and WAL Contract

Reuse:

- 4 GiB hard allocation limit;
- immutable 16 MiB reserve;
- DB/WAL/SHM aggregate accounting;
- checked fail-closed arithmetic;
- final admission after `BEGIN IMMEDIATE`;
- one-pass PASSIVE checkpoint path;
- oversized-WAL mutation block;
- shutdown TRUNCATE checkpoint;
- read availability where safe.

Add a distinct `MutationKind::RecordInertTask` with:

```text
TASK_RECORD_AGGREGATE_GROWTH_BOUND_BYTES = 8388608
TASK_RECORD_WAL_GROWTH_BOUND_BYTES = 2097152
TASK_GROWTH_PROOF_RUNS = 20
ALLOWED_PROOF_FAILURES = 0
```

The 8 MiB/2 MiB bounds conservatively reuse create-conversation admission.
A maximum task row has a 64-byte kind and no content payload, but task writes
touch a different index set from conversation writes. Therefore the smaller
row size does not prove smaller physical growth. These constants are selected
as deliberately large existing envelopes and remain planning assumptions until
the required fresh-root measurements pass.

Future implementation must measure aggregate and WAL growth with bundled
SQLite, required 4096-byte pages, maximum valid task kind, linked and unlinked
tasks, and repeated fresh roots. Any failure blocks the implementation; it
must not silently increase a constant.

## Actor and Shutdown Contract

Preserve:

- one connection-owning worker;
- queue capacity 128;
- priority shutdown channel;
- one absolute operation deadline;
- manager -> reaper -> completed lifecycle;
- no detached worker;
- no second SQLite connection.

Required behavior:

- queued task/audit request receives the existing closed-intake error after
  shutdown begins;
- an active task transaction commits both task/audit or rolls back;
- no new audit read begins after intake closes;
- operation-local task/audit failure leaves later valid operations available;
- restart readback uses the same single owner;
- Phase 1B.1/1B.2 lifecycle behavior does not regress.

## Privacy Model

- task kind is bounded identifier data and is absent from audit events,
  diagnostic logs, and safe errors;
- audit has no generic metadata/payload field;
- normal logs contain only safe operation/error codes and counts;
- subject IDs and operation IDs stay in local structured storage and are not
  emitted as routine diagnostics;
- audit reads return only typed audit fields;
- no conversation title, message content, prompt, model data, path, SQL,
  environment, secret, token, key, or infrastructure truth enters audit;
- test diagnostics use temporary roots and synthetic identifiers only.

## Public-Authority Boundary

```text
PUBLIC_TASK_TAURI_COMMANDS = 0
PUBLIC_AUDIT_TAURI_COMMANDS = 0
FRONTEND_TASK_CLIENTS = 0
FRONTEND_AUDIT_CLIENTS = 0
GENERIC_AUDIT_APPEND = 0
GENERIC_SQL_AUTHORITY = 0
CALLER_CONTROLLED_DATABASE_PATHS = 0
```

All five methods remain crate-private. Structural tests must fail if a task or
audit method enters `#[tauri::command]`, `generate_handler!`, frontend source,
or a generic IPC DTO.

## Security Considerations

Threats and controls:

- task-as-instruction: canonical opaque grammar and no interpreter/dispatcher;
- SQL injection: fixed SQL and typed parameters only;
- audit forgery: closed types plus operation-specific constructors;
- semantically invalid audit rows: event/subject/required-ID validation;
- replay: database-enforced unique event ID inside one transaction;
- cross-operation collision: global event lookup before insert;
- partial commit: immediate transaction and exact row verification;
- disk/WAL exhaustion: hard reserve, operation bounds, proof, and fail closed;
- cross-conversation data: no content joins and no scoped listing claim;
- lifecycle race: existing queue/deadline/shutdown owner;
- log leakage: safe codes, no task kind/content/raw storage details;
- malformed database values: fail-closed typed decoding and intake poisoning;
- frontend authority: structural zero-surface contract.

Any unresolved Critical, High, or material Medium false-negative in these
controls blocks implementation completion.

## Migration and Compatibility Considerations

- no migration change;
- existing Phase 1B.2 audit rows decode byte- and semantics-identically;
- migration-reserved future event values are readable but not writable;
- existing conversation/message success events retain exact field mapping;
- existing databases reopen without upgrade;
- task rows written by Phase 1B.3 remain compatible with Phase 1C because only
  the schema-approved initial state is used;
- Phase 1C must use a forward migration to add states if authorized;
- any schema correction is forward-only after a database has opened;
- no downgrade or destructive reset is permitted.

## Implementation Steps

These steps are not authorized until a separate human implementation prompt:

1. start from fresh exact main in a clean isolated worktree;
2. re-run source and schema preflight;
3. add exact task/audit DTOs and validation;
4. add closed audit decoding and operation-specific constructors;
5. add fixed task and audit-read repositories;
6. add `RecordInertTask` capacity mutation kind and exact constants;
7. add the five private manager/worker request paths;
8. minimally extend safe error codes;
9. add narrow unit/repository tests first;
10. add replay/concurrency/capacity/lifecycle/privacy/boundary tests;
11. run the task growth proof across 20 fresh roots;
12. run Phase 1B.2 and full regression gates;
13. security-review the full diff;
14. update canonical status documents and completion evidence;
15. stop for exact-head independent review before ready/merge.

## Test Matrix

### Task validation and persistence

- minimum valid task kind;
- maximum 64-byte valid task kind;
- empty, uppercase, Unicode, leading separator, doubled separator, trailing
  separator, invalid character, and 65-byte task kind;
- canonical/noncanonical operation, task, conversation, and cursor UUIDs;
- linked task with existing conversation;
- unlinked task;
- missing optional conversation;
- state is exactly `created`;
- timestamps equal and revision is zero;
- task idempotency field is SQL `NULL`;
- task/audit commit together;
- injected audit failure rolls back task;
- injected task failure creates no audit.

### Idempotency and concurrency

- same operation ID/same request replay;
- same ID/different kind;
- same ID/different conversation;
- same ID/different actor;
- global conflict with conversation-created operation;
- global conflict with message-appended operation;
- concurrent duplicate request creates one task/event;
- audit exists but task missing;
- audit/task subject, actor, correlation, timestamp, state, revision, or
  idempotency mismatch;
- restart then replay;
- rejected mutation leaves operation ID reusable;
- no raw SQLite uniqueness error escapes.

### Task reads

- exact get success and NotFound;
- global list empty/nonempty;
- no conversation-filter field or alternate query path;
- limit 0, 1, 100, and 101;
- stable `updated_at_ms DESC, id DESC` ties;
- complete cursor continuation with no gaps/duplicates;
- restart readback;
- no joined content exposure;
- reads remain available when writes are capacity-blocked.

### Audit reads and typing

- exact lookup success and NotFound;
- decode existing `conversation.created` and `message.appended`;
- decode `task.recorded`;
- decode all migration-reserved event/subject/outcome values;
- unknown event/actor/subject/outcome fails closed;
- valid event and subject enums in an invalid pair fail closed;
- missing required conversation/message/task subject ID fails closed;
- nonpositive sequence or negative timestamp fails closed;
- non-NULL reason code fails closed in Phase 1B.3;
- sequence pagination starts, boundaries, empty page, limit 0/101;
- stable ascending sequence with no gaps/duplicates;
- no arbitrary filter or metadata query;
- audit contains no task kind or content.

### Capacity and WAL

- 8 MiB aggregate and 2 MiB WAL admission constants;
- linked and unlinked maximum task rows;
- 20 fresh-root physical growth runs, zero failures;
- capacity rejection creates no task/audit;
- near-reserve and oversized pre-existing WAL;
- one PASSIVE checkpoint attempt;
- busy external writer;
- checked arithmetic overflow fails closed;
- post-lock admission and post-write physical verification.

### Lifecycle and regression

- queued task request during shutdown;
- active transaction commit-or-rollback during shutdown;
- audit read rejected after intake closes;
- operation-local failure followed by valid operation;
- worker panic/reaper lifecycle unchanged;
- all Phase 1B.2 repository/runtime-store tests;
- all Phase 1A inference tests;
- full Rust suite;
- cargo check and Clippy;
- schema checksum/fingerprint/inventory;
- zero public task/audit command/client;
- frontend contract and production build where repository gate requires;
- production dependency audit and secret scan.

Tests use temporary roots only. No real user profile or production state is
read or written.

## Acceptance Criteria

Phase 1B.3 can pass only when:

1. exact five-operation private surface is present and no broader authority;
2. task remains inert, state `created`, revision zero, timestamps equal;
3. task kind obeys exact opaque grammar and is never interpreted;
4. `task.recorded` is the only Phase 1B.3 task event;
5. `task.created` remains reserved for Phase 1C;
6. operation ID is the sole mutation replay key and task idempotency is NULL;
7. same-request replay returns the original record;
8. cross-operation and changed-payload reuse return controlled conflict;
9. task and audit commit or roll back atomically;
10. audit construction is typed and operation-specific, never generic, and
    read decoding validates semantic event/subject relationships;
11. exact and paginated task/audit reads are bounded and stable;
12. malformed persisted typed values fail closed;
13. safe errors expose no sensitive values;
14. 8 MiB aggregate/2 MiB WAL bounds pass 20/20 fresh-root proof;
15. schema/checksum/fingerprint/dependencies remain unchanged;
16. one owner/deadline/shutdown lifecycle remains;
17. public task/audit Tauri and frontend authority remains zero;
18. all focused and regression tests pass;
19. no unresolved Critical/High or material Medium security finding;
20. documentation distinguishes inert persistence from task execution;
21. no production, real-profile, deployment, or remote write occurs;
22. Phase 1B.4 and Phase 1C remain unauthorized.

No failed required check may be reported as PASS.

## Rollback Strategy

Before merge, revert only the Phase 1B.3 code and preserve the unchanged
schema. After a release writes task rows, disable future callers and forward-fix
code; do not downgrade migration history or delete compatible rows.

Because this slice adds no public authority, rollback does not require frontend
or API compatibility behavior. Audit rows remain append-only evidence. Task
deletion is not a rollback tool and remains Phase 1B.4.

## Documentation Updates

On future verified implementation:

- add a Phase 1B.3 implementation completion report;
- update ADR 0002 implementation status without rewriting history;
- update capability matrix and security gates;
- update master roadmap and Phase 1B umbrella status;
- preserve Phase 1B.1/1B.2 historical evidence;
- keep Phase 1B overall incomplete until later slices close.

## Human Directions

| Direction | Decision | Exact selection |
|---|---|---|
| `HD_1B3_01` | `ACCEPT` | five private operations listed above |
| `HD_1B3_02` | `ACCEPT` | `task.recorded`; `task.created` Phase 1C reservation |
| `HD_1B3_03` | `ACCEPT` | operation-ID replay; task idempotency NULL/deferred |
| `HD_1B3_04` | `ACCEPT` | constrained opaque identifier with exact grammar |
| `HD_1B3_05` | `ACCEPT` | update/transitions Phase 1C; deletion Phase 1B.4 |
| `HD_1B3_06` | `ACCEPT` | closed audit values; exact + sequence reads; no filters |
| `HD_1B3_07` | `ACCEPT` | minimal existing error-domain extension |
| `HD_1B3_08` | `ACCEPT` | 8 MiB aggregate, 2 MiB WAL, 20-run proof |
| `HD_1B3_09` | `ACCEPT` | zero public Tauri/frontend/generic authority |

## Final Planning Review

```text
PHASE_1B3_PLANNING_REVIEW = PASS_WITH_NONBLOCKING_FINDINGS
PLANNING_FINDINGS =
CRITICAL 0 / HIGH 0 / MEDIUM 1 / LOW 2 / INFO 2
```

All five residuals are nonblocking for planning canonicalization and retain
deterministic gates:

| Severity | Finding | Disposition | Gate |
|---|---|---|---|
| Medium | Stringly internal audit construction | `ACCEPTED_NONBLOCKING_PLANNING_RESIDUAL / EXECUTABLE_IMPLEMENTATION_PROOF_REQUIRED` | Typed operation-specific audit construction, semantic decoding, and tests block Phase 1B.3 completion. |
| Low | Missing task/audit NotFound codes | `ACCEPTED_NONBLOCKING_PLANNING_RESIDUAL / EXECUTABLE_IMPLEMENTATION_PROOF_REQUIRED` | Safe error variants and health-mapping tests are required. |
| Low | Missing Rust task-kind contract | `ACCEPTED_NONBLOCKING_PLANNING_RESIDUAL / EXECUTABLE_IMPLEMENTATION_PROOF_REQUIRED` | The reviewed opaque ASCII validator and negative tests are required. |
| Info | Deferred task idempotency-key owner | `ACCEPTED_NONBLOCKING_ARCHITECTURAL_LIMITATION / MUST_REMAIN_DOCUMENTED` | Phase 1B.3 stores SQL `NULL`; Phase 1C owns any future decision. |
| Info | Missing task physical-growth proof | `ACCEPTED_NONBLOCKING_PLANNING_RESIDUAL / EXECUTABLE_IMPLEMENTATION_PROOF_REQUIRED` | 20 fresh-root 8 MiB/2 MiB proof runs must have zero failures. |

The full evidence and exact descriptions are recorded in the readiness audit.
These dispositions do not waive any acceptance criterion or authorize
implementation.

## Open Questions

No material Phase 1B.3 contract is deferred to implementation. The exact
product taxonomy for task kinds, task transition graph, ownership of
`tasks.idempotency_key`, and task deletion UX are intentionally out of scope
and must be decided by their owning later phases.

## GO / CONDITIONAL_GO / NO_GO

```text
PHASE_1B3_AUDIT = COMPLETE
PHASE_1B3_PLANNING_REVIEW = PASS_WITH_NONBLOCKING_FINDINGS
PLANNING_FINDINGS = CRITICAL 0 / HIGH 0 / MEDIUM 1 / LOW 2 / INFO 2
PLAN_RESULT = CONDITIONAL_GO
UNRESOLVED_MATERIAL_PLANNING_DECISIONS = 0
PHASE_1B3_PLAN =
ELIGIBLE_FOR_SEPARATE_IMPLEMENTATION_AUTHORIZATION
PHASE_1B3_IMPLEMENTATION = NOT AUTHORIZED
PHASE_1B4 = NOT AUTHORIZED
PHASE_1C = NOT AUTHORIZED
```

Conditions:

- independent exact-patch planning review passes;
- planning package is separately canonicalized or named by exact reviewed head;
- a later human prompt authorizes only the exact five-operation slice;
- implementation proves the capacity bounds and passes all required gates.

This is not implementation authorization.
