# Phase 1B.3 Inert Tasks and Audit Persistence Completion

- Canonical implementation base:
  `3fbdef767d5ca26d198f6e16faba705790e66db9`
- Implementation head: the single commit containing this report; its exact SHA
  is recorded in the draft PR body and completion readback because a commit
  cannot embed its own final object ID.
- Worktree: isolated implementation checkout
- Branch: `phase-01b3/inert-tasks-audit-persistence`
- Status:
  `IMPLEMENTED_IN_DRAFT_PR / LOCAL_GATE_PASS / INDEPENDENT_REVIEW_PENDING`
- Implementation merge: `NOT PERFORMED`
- Ready: `NOT AUTHORIZED`
- Phase 1B.4: `NOT AUTHORIZED`
- Phase 1C: `NOT AUTHORIZED`

## Scope Delivered

The candidate implements exactly five crate-private Rust operations:

| Operation | Class | Contract |
| --- | --- | --- |
| `record_inert_task` | mutation | Atomically writes one inert `created` task and one typed `task.recorded` success event |
| `get_task` | read | Exact UUID-v4 lookup with closed decoding and safe NotFound |
| `list_tasks` | read | Bounded global keyset page ordered by `updated_at_ms DESC, id DESC` |
| `get_audit_event` | read | Exact event UUID-v4 lookup with closed fail-closed decoding |
| `list_audit_events` | read | Bounded keyset page ordered by `sequence_no ASC` |

```text
PRIVATE_OPERATIONS = 5
TASK_MUTATIONS = 1
TASK_READS = 2
AUDIT_READS = 2
TASK_EVENT = task.recorded
TASK_STATE = created
tasks.idempotency_key = SQL NULL
GENERIC_AUDIT_APPEND = 0
PUBLIC_TASK_TAURI_COMMANDS = 0
PUBLIC_AUDIT_TAURI_COMMANDS = 0
FRONTEND_TASK_OR_AUDIT_AUTHORITY = 0
```

The implementation deliberately excludes task execution, transition, update,
delete, retry, scheduling, planner/executor/verifier behavior, tools, network,
generic SQL, generic audit append, frontend persistence, and public IPC.

## Changed Paths

The single implementation candidate contains 18 paths: 11 scoped Rust paths
and 7 current-status documentation paths.

```text
AGENTS.md
docs/adr/0002-local-runtime-state-and-sqlite-foundation.md
docs/architecture/CAPABILITY_STATUS_MATRIX.md
docs/planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md
docs/planning/phases/phase-01b-durable-runtime-state-plan.md
docs/planning/phases/phase-01b3-inert-tasks-audit-persistence-completion.md
docs/security/SECURITY_GATES.md
src-tauri/src/runtime_store/config.rs
src-tauri/src/runtime_store/error.rs
src-tauri/src/runtime_store/mod.rs
src-tauri/src/runtime_store/models.rs
src-tauri/src/runtime_store/phase_1b3_tests.rs
src-tauri/src/runtime_store/repositories/audit_events.rs
src-tauri/src/runtime_store/repositories/mod.rs
src-tauri/src/runtime_store/repositories/tasks.rs
src-tauri/src/runtime_store/repositories/unit_of_work.rs
src-tauri/src/runtime_store/repository_tests.rs
src-tauri/src/runtime_store/worker.rs
```

Application changes outside `src-tauri/src/runtime_store`, migrations,
manifests, lockfiles, Tauri capabilities, frontend source, production
configuration, and dependency declarations remain unchanged.

## Typed Boundary Evidence

### Task DTO and enum inventory

- `RecordInertTaskRequest`, `GetTaskRequest`, `ListTasksRequest`;
- `TaskRecord`, `TaskCursor`, `TaskPage`;
- `InertTaskKind`;
- closed `InertTaskState::Created`.

`InertTaskKind` accepts exact, already-canonical ASCII bytes matching:

```text
^[a-z][a-z0-9]*(?:[._-][a-z0-9]+)*$
BYTE_LENGTH = 1..=64
NORMALIZATION = none
INTERPRETATION = none
```

The focused matrix covers minimum and maximum accepted values plus empty,
overlong, uppercase, Unicode, whitespace, repeated/trailing/leading separator,
and punctuation failures.

### Audit DTO and enum inventory

- `GetAuditEventRequest`, `ListAuditEventsRequest`;
- `AuditEventRecord`, `AuditCursor`, `AuditPage`;
- `AuditActor` as the existing closed `ContentActor`;
- closed `AuditEventType` for all nine migration-1 values;
- closed `AuditSubjectType` for all six migration-1 values;
- closed `AuditOutcome` for all three migration-1 values;
- empty closed `AuditReasonCode`, so only SQL NULL is accepted in Phase 1B.3.

Database decoding validates UUIDs, positive audit sequence, nonnegative
timestamp, event/subject semantic compatibility, required subject IDs, closed
actors/outcomes, and NULL reason codes. Unknown or incompatible persisted
values fail closed.

The only success-audit write surface is the private closed sum
`SuccessAuditEvent::{ConversationCreated, MessageAppended, TaskRecorded}`.
The SQL insertion derives event, subject and outcome from the selected variant;
callers cannot supply arbitrary event, actor, subject, outcome, reason, or
metadata strings.

```text
STRINGLY_AUDIT_AUTHORITY = 0
TASK_AUDIT_WRITER = CLOSED / OPERATION_SPECIFIC
RESERVED_AUDIT_VALUES = READABLE / NOT WRITABLE BY PHASE_1B3
```

## Atomicity and Idempotency

`record_inert_task` uses the existing single bounded worker and one
`BEGIN IMMEDIATE` transaction:

```text
task row
+ global operation-ID evidence
+ typed task.recorded event
= one commit
```

The audit event uses:

```text
event_id = operation_id
correlation_id = operation_id
subject_id = generated task ID
reason_code = NULL
tasks.idempotency_key = NULL
```

Same-ID/same-request replay returns the original task after concurrency or
restart without another task/event. Actor, optional conversation, or task-kind
drift returns `IdempotencyConflict`. Cross-operation collisions with
conversation/message mutations also conflict. Missing, malformed, or
semantically inconsistent replay evidence fails closed as
`IdempotencyRecordInconsistent`.

Injected task and audit failures prove rollback and operation-ID reuse.
Capacity and writer-busy rejection likewise leave no partial task or audit.

## Errors and Privacy

The existing private error model adds only:

- `TaskNotFound` / `content_task_not_found`;
- `AuditEventNotFound` / `content_audit_event_not_found`.

No raw SQLite error, path, task kind, content, title, prompt, payload,
environment value, or audit metadata is exposed through these errors or normal
diagnostic output. Operation-local validation and NotFound failures do not
poison global runtime health.

## Schema, Dependency, and Authority Invariance

```text
MIGRATION_SHA =
62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d

STRUCTURAL_FINGERPRINT =
37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77

TABLES = 5
EXPLICIT_INDEXES = 7
SQLITE_AUTOINDEXES = 7
SQLITE_SEQUENCE = 0
MIGRATION_2 = ABSENT
DEPENDENCY_CHANGE = NONE
TAURI_COMMAND_INVENTORY_CHANGE = NONE
FRONTEND_AUTHORITY_CHANGE = NONE
PRODUCTION_CONFIGURATION_CHANGE = NONE
```

The locked Rust and npm dependency graphs are byte-for-byte unchanged.

## Test and Validation Evidence

Pinned toolchain:

```text
rustc = 1.95.0
rustdoc = 1.95.0
cargo = 1.95.0
active toolchain = 1.95.0-aarch64-apple-darwin
```

| Check | Result |
| --- | --- |
| Phase 1B.3 focused tests | `31/31 PASS` |
| Complete runtime-store tests | `131/131 PASS` |
| Inference regression | `67/67 PASS` |
| Full Rust tests | `247/247 PASS` |
| Cargo check, all targets, locked | `PASS` |
| Cargo Clippy, all targets, locked | `PASS` |
| Scoped Rustfmt | `PASS` |
| Storage boundary validator | `29/29` primary, `13/13` defense-in-depth, 46 structural checks |
| Inference frontend/Rust contract | `PASS` |
| TypeScript and production build | `PASS`, 1,763 modules |
| Production npm audit | `0 vulnerabilities` |
| Secret scan | `PASS` |
| Changed runtime-store warning locations | `0` |
| Migration checksum and schema fingerprint | `PASS` |

### Executable physical-growth evidence

Twenty fresh roots, alternating ten unlinked and ten linked tasks, passed:

```text
TASK_GROWTH_RUNS = 20/20 PASS
MAX_TASK_AGGREGATE_GROWTH = 41,200 bytes
MAX_TASK_WAL_GROWTH = 41,200 bytes
TASK_AGGREGATE_BOUND = 8 MiB
TASK_WAL_BOUND = 2 MiB
```

The existing fresh-root regression also passed unchanged:

```text
CREATE_GROWTH_RUNS = 20/20 PASS
MAX_CREATE_AGGREGATE_GROWTH = 32,960 bytes
MAX_CREATE_WAL_GROWTH = 32,960 bytes
APPEND_GROWTH_RUNS = 20/20 PASS
MAX_APPEND_AGGREGATE_GROWTH = 313,120 bytes
MAX_APPEND_WAL_GROWTH = 313,120 bytes
```

## Security Review

The scoped self-review covered SQL injection, unbounded reads, task-kind
interpretation, forged enum values, invalid event/subject pairs, arbitrary
audit writes, replay conflicts, duplicate/concurrent operations, partial
commit, parent ownership, log leakage, capacity/WAL exhaustion, lock
contention, shutdown races, public IPC expansion, frontend authority, schema
drift, dependency drift, and real-profile writes.

```text
SCOPED_FINDINGS =
CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 0 / INFO 5
```

Accepted informational residuals:

1. real desktop restart remains unverified;
2. cross-platform runtime remains unverified;
3. the pre-production SQLCipher decision remains open;
4. 13 RustSec vulnerabilities and 22 RustSec warnings are inherited with an
   unchanged lockfile;
5. 94 legacy Rust files remain outside full-repository rustfmt compliance.

The dev-inclusive npm audit retains 11 inherited advisories; the production
dependency audit is zero. Existing repository warnings remain inherited and
the changed runtime-store warning count is zero.

No Critical, High, material Medium, scope, schema, dependency, IPC, privacy, or
capacity blocker was found by the local gate.

## Disposable Local Writes

Only isolated worktree build/test artifacts, temporary SQLite roots, npm
`node_modules`, Rust `target`, and temporary audit/format logs were produced.

```text
REMOTE_PRODUCTION_WRITES = 0
REAL_USER_PROFILE_READS = 0
REAL_USER_PROFILE_WRITES = 0
DEPLOYMENTS = 0
OLLAMA_DOWNLOADS = 0
REAL_PROMPTS = 0
```

## Release Gate

```text
SCOPED_PHASE_1B3_GATE = PASS
REPOSITORY_BASELINE_GATE = CONDITIONAL_PASS
INDEPENDENT_EXACT_HEAD_REVIEW = PENDING
PHASE_1B3_IMPLEMENTATION = NOT MERGED
READY = NOT PERFORMED
MERGE = NOT PERFORMED
PHASE_1B4 = NOT AUTHORIZED
PHASE_1C = NOT AUTHORIZED
```

The `CONDITIONAL_PASS` repository baseline reflects inherited RustSec,
warning, and 94-file formatting debt only; none was introduced or changed by
this slice. A separate clean detached exact-head review is mandatory before any
ready/merge decision.
