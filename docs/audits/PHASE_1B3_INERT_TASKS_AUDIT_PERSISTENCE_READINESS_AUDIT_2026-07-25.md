# Phase 1B.3 Inert Tasks and Audit Persistence Readiness Audit

- Date: 2026-07-25
- Canonical source: `255e71b4467fbe7d521c3022c1cd0afc76197ecf`
- Audit mode: source-grounded, docs-only
- Audit result: `COMPLETE`
- Final planning review: `PASS_WITH_NONBLOCKING_FINDINGS`
- Planning result: `CONDITIONAL_GO`
- Implementation: `NOT AUTHORIZED`

## Provenance and Limits

This audit inspects the repository at the exact canonical source above. It
distinguishes executable behavior from schema reservations, documentation
claims, and future architecture. It does not modify Rust, TypeScript, SQL,
migrations, manifests, lockfiles, Tauri capabilities, or production
configuration.

The verified Phase 1B.2 conversation/message service is a preserved dependency,
not missing work. This audit evaluates only the next bounded slice: inert task
recording, task reads, and privacy-safe audit reads. It does not authorize task
execution or any Phase 1C Supervisor behavior.

No runtime tests, production writes, real-user-profile writes, deployments,
commits, pushes, or pull requests are part of this audit.

## Executive Summary

Migration 1 already contains structurally constrained `tasks` and
`audit_events` tables. The `tasks` table permits only state `created`; the
`audit_events` table closes event, actor, subject, and outcome values at the
database boundary. Migration checksum, structural fingerprint, table count,
and index inventory remain the verified Phase 1B.1/1B.2 values.

The runtime does not yet expose task DTOs, task repositories, audit-read
repositories, worker requests, or manager methods. The only implemented
private content operations are the five Phase 1B.2 conversation/message
operations. The only public storage command remains the read-only storage
status projection. Therefore task/audit service capability is
`MISSING`, while schema support is `IMPLEMENTED_AND_VERIFIED`.

The smallest coherent Phase 1B.3 vertical slice contains exactly five
crate-private Rust operations:

1. `record_inert_task`;
2. `get_task`;
3. `list_tasks`;
4. `get_audit_event`;
5. `list_audit_events`.

Only `record_inert_task` mutates state. It creates one immutable inert row in
state `created` and one `task.recorded` audit event in the same
`BEGIN IMMEDIATE` transaction. It does not create executable work.
`task.created` remains a schema reservation for Phase 1C.

The current migration is sufficient. No migration 2, new table, column, index,
trigger, dependency, or public IPC surface is required. The plan must,
however, replace stringly internal audit construction with closed typed values
and operation-specific audit constructors before Phase 1B.3 can be considered
verified. That is an implementation acceptance criterion, not authorization to
change code now.

The planning result is `CONDITIONAL_GO`: the bounded design is source-supported
and has no unresolved Critical or High planning blocker, but implementation
requires separate human authorization and later executable proof of the
selected capacity bounds.

## Capability Status Matrix

| Capability | Status | Evidence |
|---|---|---|
| Runtime-store owner and lifecycle | `IMPLEMENTED_AND_VERIFIED` | `src-tauri/src/runtime_store/worker.rs`; one worker, bounded queue, priority shutdown |
| Five-table schema | `IMPLEMENTED_AND_VERIFIED` | `src-tauri/migrations/runtime_state/0001_runtime_state_initial.sql` |
| Conversations/messages private service | `IMPLEMENTED_AND_VERIFIED` | `src-tauri/src/runtime_store/repositories/mod.rs`, `worker.rs` |
| Task table constraints | `IMPLEMENTED_AND_VERIFIED` | migration 1 `tasks` DDL and migration tests |
| Audit table constraints | `IMPLEMENTED_AND_VERIFIED` | migration 1 `audit_events` DDL and migration tests |
| Task DTO/type model | `MISSING` | no task types in `runtime_store/models.rs` |
| Task repository/service | `MISSING` | no `tasks.rs`; no task manager or worker operation |
| Audit closed Rust model | `PARTIALLY_IMPLEMENTED` | `ContentActor` is closed, but `unit_of_work::AuditEvent` fields are strings |
| Privacy-safe Phase 1B.2 audit writes | `IMPLEMENTED_AND_VERIFIED` | atomic conversation/message mutation code and tests |
| Generic audit-write API | `MISSING` and prohibited | no public API; current internal helper must remain non-generic to callers |
| Audit exact/bounded reads | `MISSING` | only operation-ID replay lookup exists |
| Task execution/transitions | `MISSING` and out of scope | no executor/state transition service |
| Public task/audit Tauri commands | `MISSING` and prohibited | only `get_storage_runtime_status` is registered |
| Frontend task/audit authority | `MISSING` and prohibited | no frontend client or DTO |
| Durable runtime state overall | `PARTIALLY_IMPLEMENTED` | Phase 1B.1/1B.2 complete; 1B.3/1B.4 remain |

## Actual Runtime-Store Architecture

```text
RuntimeStoreManager
  -> bounded RuntimeStoreRequest queue (128)
  -> one connection-owning worker
  -> private repository service
     -> conversations
     -> messages
     -> unit_of_work / capacity / audit coupling
  -> priority shutdown
  -> manager -> reaper -> completed lifecycle

Tauri frontend authority
  -> get_storage_runtime_status only

Missing Phase 1B.3 service boundary
  -> task DTOs/repository/worker requests
  -> audit typed model/read repository
```

The existing architecture already supplies the correct owner, deadline,
shutdown, capacity, and transaction foundation. Phase 1B.3 should extend that
single owner rather than add a second connection, queue, daemon, or generic
repository.

## Source Evidence

| Claim | Source path | Symbol or SQL object | Verdict |
|---|---|---|---|
| One bounded owner exists | `src-tauri/src/runtime_store/worker.rs` | `RuntimeStoreManager`, `RuntimeStoreRequest`, `run_worker` | `SUPPORTED` |
| Queue capacity is 128 | `src-tauri/src/runtime_store/config.rs` | `STORAGE_QUEUE_CAPACITY` | `SUPPORTED` |
| Ordinary deadline is 10 seconds | `src-tauri/src/runtime_store/config.rs` | `ORDINARY_OPERATION_DEADLINE` | `SUPPORTED` |
| Busy timeout is 5 seconds | `src-tauri/src/runtime_store/config.rs` | `BUSY_TIMEOUT` | `SUPPORTED` |
| DB hard limit is 4 GiB | `src-tauri/src/runtime_store/config.rs` | `DATABASE_HARD_LIMIT_BYTES` | `SUPPORTED` |
| Reserve is immutable 16 MiB | `src-tauri/src/runtime_store/config.rs` | `OPERATIONAL_RESERVE_BYTES` and admission code | `SUPPORTED` |
| Phase 1B.2 has five operations | `src-tauri/src/runtime_store/worker.rs` | five content request variants and manager methods | `SUPPORTED` |
| Task table exists | `src-tauri/migrations/runtime_state/0001_runtime_state_initial.sql` | `tasks` | `SUPPORTED` |
| Task state is only `created` | migration 1 | `CHECK (state IN ('created'))` | `SUPPORTED` |
| Task kind is DB-bounded to 64 bytes | migration 1 | `task_kind` checks | `SUPPORTED` |
| Task `idempotency_key` is optional unique | migration 1 | `idempotency_key TEXT UNIQUE` | `SUPPORTED` |
| Audit table exists | migration 1 | `audit_events` | `SUPPORTED` |
| Task audit events are reserved | migration 1 | `task.created`, `task.recorded`, `task.deleted` | `SUPPORTED` |
| Audit actor/subject/outcome values are DB-closed | migration 1 | `CHECK` allowlists | `SUPPORTED` |
| Audit reason code is value-closed in SQL | migration 1 | length check only | `UNSUPPORTED` |
| Task service exists | `runtime_store/models.rs`, `repositories`, `worker.rs` | no task service symbols | `UNSUPPORTED` |
| General audit read service exists | `repositories/unit_of_work.rs` | replay-only `load_audit` | `UNSUPPORTED` |
| Current audit construction is fully typed | `repositories/unit_of_work.rs` | string fields and string arguments | `UNSUPPORTED` |
| Public task/audit IPC exists | `src-tauri/src/lib.rs`, `runtime_store/commands.rs` | no task/audit command | `UNSUPPORTED` |
| A second migration is needed | migration 1 and planned five-operation contract | existing columns/constraints/indexes | `UNSUPPORTED` |

## Schema Sufficiency

The current schema supports the bounded contract without modification:

- `tasks.id` stores an implementation-generated canonical UUID v4;
- `conversation_id` is nullable and is protected by a foreign key;
- `task_kind` stores the selected constrained identifier;
- `state` already rejects every value except `created`;
- `idempotency_key` can remain `NULL`;
- created and updated timestamps plus revision support the immutable initial
  record;
- `tasks_state_updated_idx` supports the selected stable global read;
- the conversation index does not contain `updated_at_ms`, so Phase 1B.3 does
  not claim a bounded conversation-filtered chronological read;
- `audit_events.event_id` is globally unique;
- retained `sequence_no` supports stable forward audit pagination;
- `task.recorded`, actor, task subject, success outcome, and UUID correlation
  are already permitted.

```text
PHASE_1B3_SCHEMA_DECISION = NO_SCHEMA_CHANGE_REQUIRED
MIGRATION_2 = ABSENT / MUST REMAIN ABSENT
TABLES = 5
EXPLICIT_INDEXES = 7
SQLITE_AUTOINDEXES = 7
MIGRATION_SHA =
62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d
STRUCTURAL_FINGERPRINT =
37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77
```

`audit_events.reason_code` is length-bounded but not value-allowlisted in SQL.
Phase 1B.3 does not need a reason code: its only event is successful and stores
`NULL`. The Rust read model must fail closed on non-NULL values until a later
reviewed phase introduces an explicit closed enum variant. This avoids a
migration while preventing arbitrary reason strings from becoming accepted
service data.

## Missing Service Inventory

Required only for a future separately authorized Phase 1B.3 implementation:

- exact inert task request/record/cursor/page types;
- a constrained, non-executable task-kind type;
- task record, exact read, and bounded list repository methods;
- closed audit value types and fail-closed database decoding;
- operation-specific success-audit constructors;
- exact audit-event read and bounded sequence list;
- five manager/worker request paths;
- task-specific capacity mutation kind and proof;
- task/audit error variants;
- repository, concurrency, replay, lifecycle, privacy, and boundary tests.

Explicitly deferred:

- task update and state transitions to Phase 1C;
- task deletion to Phase 1B.4;
- conversation-filtered chronological task listing to a separately indexed
  later contract;
- task execution, planning, verification, retry, cancellation, scheduling, and
  recovery orchestration to Phase 1C or later;
- public task/audit Tauri commands and frontend clients;
- generic audit append, generic SQL, arbitrary filters, and arbitrary metadata;
- memory extraction, tools, models, networks, Reticulum/LXMF, wallet, and
  remote synchronization.

## Findings

### MEDIUM — Internal audit construction is stringly typed

- Evidence: `src-tauri/src/runtime_store/repositories/unit_of_work.rs`,
  `AuditEvent` and `insert_success_audit`.
- Fact: event, actor, subject, outcome, reason, and correlation values are
  represented as strings at the internal repository boundary.
- Impact: the database rejects values outside several allowlists, but valid
  values can still be combined incorrectly—for example a task event with an
  export subject—and a future caller could accidentally turn the helper into
  generic audit authority.
- Remediation: add closed database-decoding types and operation-specific
  constructors for conversation-created, message-appended, and task-recorded
  success events. Read decoding must also validate the event/subject pairing,
  required subject ID, sequence, timestamp, and UUID invariants. Keep the
  low-level SQL insertion function private to the unit-of-work module.
- Development gate: blocks Phase 1B.3 completion, not planning.

### LOW — Current operation error type lacks task/audit NotFound codes

- Evidence: `src-tauri/src/runtime_store/error.rs`,
  `ContentOperationErrorCode`.
- Impact: mapping missing task/audit records through existing codes would be
  misleading or would leak storage details.
- Remediation: minimally extend the proven private error model with
  `TaskNotFound` and `AuditEventNotFound`; preserve current health
  classification and safe-code-only display.
- Development gate: acceptance criterion.

### LOW — No Rust task-kind contract exists

- Evidence: migration 1 bounds only nonempty byte length; no task DTO exists in
  `models.rs`.
- Impact: unrestricted prose could later be misinterpreted as an instruction,
  route, tool, or scheduler command.
- Remediation: use a canonical constrained opaque identifier with exact ASCII
  grammar and no normalization or interpretation.
- Development gate: acceptance criterion.

### INFO — `tasks.idempotency_key` has no current owner

- Evidence: migration 1 includes the nullable unique field; the umbrella plan
  defers deterministic task identity/idempotency semantics to Phase 1C.
- Impact: dual use with operation-ID audit evidence would create ambiguous
  replay authority.
- Remediation: Phase 1B.3 must always store it as `NULL`; Phase 1C must make a
  separate reviewed decision before using it.

### INFO — No task-specific physical growth proof exists

- Evidence: `MutationKind` has only create-conversation and append-message
  variants; config has only their bounds.
- Impact: a task mutation cannot safely enter the hard-reserve path without an
  explicit conservative bound and executable proof.
- Remediation: select 8 MiB aggregate and 2 MiB WAL bounds, then prove them
  across 20 fresh roots with maximum valid task input before completion.
  Task rows are smaller than maximum conversation rows but touch a different
  index set, so source inspection alone is not physical-growth proof.

## Final Planning Review Finding Dispositions

The exact substantive planning patch was independently reviewed with this
final inventory:

```text
CRITICAL 0 / HIGH 0 / MEDIUM 1 / LOW 2 / INFO 2
```

| Severity | Finding | Exact description | Affected contract | Disposition | Implementation gate |
|---|---|---|---|---|---|
| Medium | Internal audit construction is stringly typed | Current internal audit values are strings, so individually allowed values can still form an invalid event/subject combination. | Typed audit writer and read-decoding boundary | `ACCEPTED_NONBLOCKING_PLANNING_RESIDUAL / EXECUTABLE_IMPLEMENTATION_PROOF_REQUIRED` | Phase 1B.3 cannot complete until operation-specific typed constructors, semantic pair validation, and focused tests pass. |
| Low | Current operation error type lacks task/audit NotFound codes | The private error model has no truthful task or audit-event NotFound variants. | Private error and health model | `ACCEPTED_NONBLOCKING_PLANNING_RESIDUAL / EXECUTABLE_IMPLEMENTATION_PROOF_REQUIRED` | Add safe variants and mapping/health tests without leaking storage details. |
| Low | No Rust task-kind contract exists | Migration 1 bounds byte length, but no Rust task DTO currently prevents unrestricted prose from becoming task-kind data. | Canonical inert task input | `ACCEPTED_NONBLOCKING_PLANNING_RESIDUAL / EXECUTABLE_IMPLEMENTATION_PROOF_REQUIRED` | Implement the reviewed opaque ASCII validator and negative tests before accepting a task record. |
| Info | `tasks.idempotency_key` has no current owner | Dual ownership with operation-ID audit evidence would make replay authority ambiguous. | Idempotency ownership | `ACCEPTED_NONBLOCKING_ARCHITECTURAL_LIMITATION / MUST_REMAIN_DOCUMENTED` | Store SQL `NULL` throughout Phase 1B.3; Phase 1C requires a separate reviewed ownership decision. |
| Info | No task-specific physical growth proof exists | Existing create/append measurements do not prove task/index growth. | Aggregate/WAL admission | `ACCEPTED_NONBLOCKING_PLANNING_RESIDUAL / EXECUTABLE_IMPLEMENTATION_PROOF_REQUIRED` | The exact 8 MiB/2 MiB bounds must pass 20 fresh-root runs with zero failures. |

No finding authorizes source changes through this planning package. The Medium
finding is nonblocking only for docs canonicalization; it remains a mandatory
implementation and release gate.

## Security Review

| Threat | Current exposure | Required Phase 1B.3 control |
|---|---|---|
| Executable task injection | no task service exists | opaque canonical identifier; no dispatch/execution |
| Generic audit forgery | no public append; internal strings exist | operation-specific typed constructors and semantic pair validation |
| Free-form audit metadata | schema has no metadata column | no new metadata or generic reason input |
| Operation replay/collision | proven for two Phase 1B.2 mutations | global event-ID evidence for the third mutation |
| Orphan task/audit | no task mutation exists | one immediate transaction plus exact post-write checks |
| Cross-conversation access | no task reads exist | no conversation-content join and no scoped filter in this slice |
| Disk/WAL exhaustion | established content admission | distinct conservative task bounds and 20-run proof |
| Shutdown race | one owner already exists | reuse same queue/deadline/shutdown path |
| SQL injection | no generic SQL boundary | fixed SQL only; typed DTOs; no SQL input |
| Sensitive log leakage | content errors are safe codes | no task kind, content, path, SQL, or raw SQLite text |
| Frontend authority expansion | no task/audit IPC | structural test for zero commands/clients |
| Schema drift | migration fingerprint is verified | migration checksum/fingerprint regression |

No confirmed Critical or High finding exists in the current Phase 1B.3
planning scope. The stringly internal audit boundary is a material Medium
implementation concern and is resolved in the plan by a typed,
operation-specific boundary.

## Test-Gap Matrix

| Area | Existing evidence | Missing future evidence |
|---|---|---|
| Task schema constraints | migration tests | task service validation/replay/read tests |
| Task/audit atomicity | conversation/message mutation tests only | task insert + audit rollback/commit |
| Global operation-ID collision | two mutation kinds | collision across all three kinds |
| Task pagination | indexes only | full-key cursor, tie, restart, filter tests |
| Audit pagination | sequence retained | exact and bounded ascending read tests |
| Privacy | Phase 1B.2 audit assertions | task event/redaction and safe-error assertions |
| Capacity | create/append 40-run proof | 20 task-record fresh-root runs |
| Lifecycle | Phase 1B.1/1B.2 tests | queued/active task and audit-read shutdown tests |
| Public boundary | current inventory | zero task/audit command/client regression |

## Human-Direction Decisions

| Decision | Result | Selected contract |
|---|---|---|
| `HD_1B3_01` exact private operation surface | `ACCEPT` | exactly five: one mutation and four reads; task list is global/index-backed |
| `HD_1B3_02` task event | `ACCEPT` | `task.recorded`; `task.created` reserved for Phase 1C |
| `HD_1B3_03` idempotency ownership | `ACCEPT` | operation ID/audit event owns replay; task idempotency remains `NULL` |
| `HD_1B3_04` task kind | `ACCEPT` | constrained opaque canonical ASCII identifier |
| `HD_1B3_05` update/delete | `ACCEPT` | update/transitions Phase 1C; delete Phase 1B.4 |
| `HD_1B3_06` audit read/type boundary | `ACCEPT` | closed values; exact lookup and sequence pagination; no filters |
| `HD_1B3_07` error domain | `ACCEPT` | minimally extend existing private error model |
| `HD_1B3_08` capacity/WAL bounds | `ACCEPT` | 8 MiB aggregate, 2 MiB WAL; 20-run proof required |
| `HD_1B3_09` public authority | `ACCEPT` | zero Tauri commands, frontend clients, generic SQL/audit append |

## Recommended Bounded Architecture

```text
private RuntimeStoreManager
  record_inert_task
    -> validate canonical data
    -> BEGIN IMMEDIATE
    -> resolve global operation-ID evidence
    -> verify optional conversation
    -> admit aggregate/WAL growth
    -> insert immutable task (created, revision 0, idempotency NULL)
    -> insert operation-specific task.recorded audit
    -> verify and commit

  get_task / list_tasks
    -> typed fixed SQL
    -> stable bounded records

  get_audit_event / list_audit_events
    -> typed fail-closed decoding
    -> exact lookup / retained sequence pagination
```

This design extends the established owner and transaction model without
creating a task engine. Task data remains inert and is never interpreted.

## Readiness Decision

```text
PHASE_1B3_AUDIT = COMPLETE
PHASE_1B3_PLANNING_REVIEW = PASS_WITH_NONBLOCKING_FINDINGS
PLANNING_FINDINGS = CRITICAL 0 / HIGH 0 / MEDIUM 1 / LOW 2 / INFO 2
PLAN_RESULT = CONDITIONAL_GO
PHASE_1B3_SCHEMA_DECISION = NO_SCHEMA_CHANGE_REQUIRED
EXACT_PRIVATE_OPERATIONS = 5
TASK_MUTATIONS = 1
TASK_READS = 2
AUDIT_READS = 2
TASK_EVENT = task.recorded
TASK_UPDATE = DEFERRED_TO_PHASE_1C
TASK_DELETE = DEFERRED_TO_PHASE_1B4
GENERIC_AUDIT_APPEND = 0
PUBLIC_TASK_TAURI_COMMANDS = 0
PUBLIC_AUDIT_TAURI_COMMANDS = 0
PHASE_1B3 = ELIGIBLE_FOR_SEPARATE_IMPLEMENTATION_AUTHORIZATION
PHASE_1B3_IMPLEMENTATION = NOT AUTHORIZED
PHASE_1B4 = NOT AUTHORIZED
PHASE_1C = NOT AUTHORIZED
```

`CONDITIONAL_GO` authorizes only independent review and a later human decision.
It does not authorize source changes.
