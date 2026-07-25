# Phase 1B.2 Conversations and Messages Readiness Audit

Date: 2026-07-24

Status: **AUDIT COMPLETE / PLANNING REVIEW PASS / CONDITIONAL_GO / IMPLEMENTATION NOT AUTHORIZED**

## Provenance and Limits

Repository:

```text
DAARION-DAO/daarion-edge-client
CANONICAL_MAIN = 42c5ac0b4fa501e59c83a5b0b395c73af382420d
WORKTREE_STATE_AT_AUDIT_START = CLEAN
```

This is a source-based repository-readiness audit. It did not modify or run
application behavior, access a real user profile, call a model, use a network
service, deploy, publish or perform production writes.

Existing Phase 1B.1 fresh-main test counts are referenced as repository
evidence; they were not rerun for this planning-only task.

## Executive Summary

Phase 1B.1 provides a strong base for Phase 1B.2:

- the SQLite dependency, path policy, dedicated actor, migration runner,
  lifecycle, five-table schema and read-only health projection are real and
  verified;
- the existing schema already contains the intended conversation, message and
  audit-event fields, constraints and indexes;
- no migration or dependency change is required for the proposed slice.

Phase 1B.2 itself is not implemented:

- there are no production conversation/message domain models;
- there are no repository traits/services or actor requests for content;
- there is no production sequence allocation, bounded listing, idempotent
  retry or state/audit unit of work;
- all current content writes are direct SQL inside tests;
- chat-like frontend state remains in React memory;
- the separate messaging module remains a mocked RAM-only subsystem.

The most important planning issue is an inter-slice boundary conflict. The
accepted Phase 1B contract requires conversation/message state and its audit
event to commit atomically, while the broad AuditStore is assigned to Phase
1B.3. The smallest safe resolution is a private two-event transaction helper
inside Phase 1B.2, leaving general task/audit persistence and audit queries to
Phase 1B.3.

No confirmed Critical or High vulnerability was found in the non-existent
Phase 1B.2 path. Human direction accepts the Rust-only/no-schema-change
boundary and requires exact atomic audit, global idempotency, immutable reserve
and separate implementation authorization. Earlier independent reviews
correctly blocked incomplete contract iterations. The final substantive
planning patch resolves those gaps and passed independent review.
Implementation remains unauthorized until a separate human implementation
decision.

## Historical Planning Review Record

The first independent review historically returned:

```text
HISTORICAL_REVIEW_1 =
PHASE_1B2_PLANNING_REVIEW_BLOCKED_BY_FINDINGS

HISTORICAL_PLAN_RESULT = NO_GO
HISTORICAL_FINDINGS = CRITICAL 0 / HIGH 0 / MEDIUM 4 / LOW 1 / INFO 2
```

| Historical review finding | Required correction | Final disposition |
| --- | --- | --- |
| M-01: five operations lacked exact per-operation contracts, especially the three reads | Exact manager methods, DTOs, NotFound, pagination, restart, error and health semantics | Added and verified by the final substantive review |
| M-02: 16 MiB was used as a transaction-growth budget rather than an immutable reserve | Separate `R = 16 MiB` from exact operation and WAL-growth bounds; lock before DB/WAL/SHM admission; define checkpoint and oversized-WAL behavior | Added with exact auto-checkpoint, live-WAL ceiling, reserve allocation and one-attempt recovery policy; verified by the final substantive review |
| M-03: idempotency normalization, retention and concurrent duplicates were incomplete | Exact canonical payload, global lifetime scope, database uniqueness, replay/conflict/inconsistency/concurrency, including deterministic create-result reconstruction | Added; create replay no longer depends on mutable counters or timestamps; verified by the final substantive review |
| M-04: audit fields and operation/global error effects were incomplete | Exact audit mapping, rollback/no-audit denials, separate content error domain and deterministic health effects | Added and verified by the final substantive review |
| L-01: all five operations were described with mutation semantics | State five operations = two mutations + three reads | Corrected throughout and verified by the final substantive review |

The correction changes planning assurance only. It is not implementation
evidence and does not change any capability classification in this audit.

The next historical exact-patch review of SHA-256
`737c6e8066a10a961866382fb36eccd27ad99048152d2fbdc9200003f1482421`
correctly remained blocked by two Medium findings:

1. create replay promised the original result but did not specify how to
   reconstruct immutable initial counters/timestamps after append mutated the
   current conversation projection;
2. the immutable reserve did not define auto-checkpoint, live-WAL, retained-file
   and checkpoint-copy bounds.

The final docs correction then specified deterministic create-result
reconstruction and exact WAL/checkpoint controls. Re-reviewing `737c6e80…`
could not change that historical gate.

## Current Planning Review Record

The final substantive planning baseline passed independent exact-diff review:

```text
SUBSTANTIVE_PLAN_PATCH_SHA256 =
f1ed3caf34cdcbaffd341f82cd351d443118cf336e44fd2729e9db9c2974a9dd

PHASE_1B2_PLANNING_REVIEW =
PHASE_1B2_PLANNING_REVIEW_PASS

PLANNING_FINDINGS =
CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 0 / INFO 0

PLAN_RESULT =
CONDITIONAL_GO

PHASE_1B2 =
ELIGIBLE_FOR_SEPARATE_IMPLEMENTATION_AUTHORIZATION

PHASE_1B2_IMPLEMENTATION =
NOT AUTHORIZED

PHASE_1B3 =
NOT AUTHORIZED
```

This review approves the planning contracts only. WAL growth bounds and reserve
algebra still require executable proof during a separately authorized
implementation.

## Capability Status Matrix

| Capability | Classification | Source evidence | Limit |
| --- | --- | --- | --- |
| SQLite connection owner | `IMPLEMENTED_AND_VERIFIED` | `src-tauri/src/runtime_store/connection.rs::RuntimeStoreConnection`; `worker.rs::RuntimeStoreManager` | One private actor connection; no content API |
| Migration/schema bootstrap | `IMPLEMENTED_AND_VERIFIED` | `migrations.rs`; `0001_runtime_state_initial.sql`; Phase 1B.1 completion evidence | Version 1 only |
| Conversation table constraints | `IMPLEMENTED_AND_VERIFIED` | migration SQL plus direct schema tests in `runtime_store/tests.rs` | Schema proof is not a production service |
| Message table constraints/order index | `IMPLEMENTED_AND_VERIFIED` | migration SQL plus direct foreign-key/sequence tests | No runtime sequence allocator |
| Audit-event table/allowlists | `IMPLEMENTED_AND_VERIFIED` | migration SQL plus direct allowlist/sequence tests | No production append helper |
| Runtime status projection | `IMPLEMENTED_AND_VERIFIED` | `commands.rs::get_storage_runtime_status` | Read-only metadata only |
| Conversation domain models | `MISSING` | no `runtime_store/models.rs` or equivalent model found | Planned for 1B.2 |
| Conversation create/get/list service | `MISSING` | production `RuntimeStoreRequest` has no content variant | Planned for 1B.2 |
| Message append/list service | `MISSING` | production `RuntimeStoreRequest` has no content variant | Planned for 1B.2 |
| Atomic state/audit unit of work | `MISSING` | no production repository or transaction helper | Required before mutation |
| Content-operation idempotency | `MISSING` | no operation identity/replay contract exists | Required for safe timeout retry |
| Bounded content pagination | `MISSING` | no production query API exists | Required before reads |
| Durable frontend chat history | `MISSING` | `LocalInferencePanel.tsx` uses React `useState` | Explicit 1B.2 non-goal |
| Legacy messaging persistence | `MOCK_OR_PLACEHOLDER` | `messaging.rs::MessagingState.messages` is a RAM vector; session/poll/send are stubs | Must remain separate |
| Six-level memory | `MISSING` | capability matrix and source audit | Not Phase 1B.2 |
| Tasks/general audit repository | `DOCUMENTED_ONLY` | schema and Phase 1B plan only | Phase 1B.3 |
| Deletion/export/backup/recovery service | `DOCUMENTED_ONLY` | Phase 1B plan only | Phase 1B.4 |

## Actual Runtime-Store Architecture

```text
Tauri setup
  -> RuntimeStoreManager::new
  -> bounded sync channel (capacity 128)
  -> dedicated runtime-store worker
  -> RuntimeStoreConnection
  -> path/pragmas/migration/integrity checks
  -> SQLite version-1 schema

Frontend
  -> storageRuntimeClient.ts
  -> get_storage_runtime_status only
  -> StorageRuntimeCard
```

Current production request protocol:

```text
RuntimeStoreRequest
  - Initialize
  - ReadStatus
```

The test-only `Hold`, `Block`, `Panic` and `ExitUnexpectedly` variants are not
production content operations.

## Source Evidence

### Storage owner and lifecycle are real

- `src-tauri/src/runtime_store/worker.rs::RuntimeStoreManager::new` creates the
  bounded channel and dedicated worker.
- `RuntimeStoreManager::start_initialization`, `read_status` and
  `production_shutdown` are the only production manager operations.
- `src-tauri/src/runtime_store/lifecycle.rs::RuntimeStoreLifecycle` integrates
  bounded close behavior with Tauri exit events.
- `src-tauri/src/runtime_store/connection.rs::open_for_initialization` applies
  no-follow open flags, path validation, SQLite limits and verified pragmas.

### Schema support exists

`src-tauri/migrations/runtime_state/0001_runtime_state_initial.sql` contains:

- `conversations` with UUID-v4, status, title byte limit, timestamps, next
  message sequence and revision;
- `messages` with conversation foreign key, sequence uniqueness, role allowlist
  and 256-KiB content boundary;
- `audit_events` with UUID-v4 event ID, allowlisted event/actor/subject/outcome,
  bounded metadata and ordered integer primary key;
- all indexes required by the accepted Phase 1B plan.

The schema checksum and structural fingerprint are:

```text
MIGRATION_SHA =
62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d

STRUCTURAL_FINGERPRINT =
37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77
```

### Content service support does not exist

- `src-tauri/src/runtime_store/mod.rs` declares no model or repository modules.
- `worker.rs::RuntimeStoreRequest` has no create/get/list/append content
  variant.
- `RuntimeStoreManager` has no content method.
- `runtime_store/commands.rs` exposes no content command.
- repository-wide searches found no `ConversationStore`, `MessageStore`,
  `RuntimeUnitOfWork`, `create_conversation` or `append_message` production
  implementation.

### Current capacity and error controls are foundations, not content contracts

- `runtime_store/config.rs` fixes the warning threshold at 2 GiB and hard limit
  at 4 GiB.
- `connection.rs::database_size_bytes` and
  `path_policy.rs::database_total_size` revalidate and sum the database, WAL and
  SHM files.
- `connection.rs::open_for_initialization` checks only whether the existing
  aggregate is already over the hard limit. No content-mutation admission path
  exists.
- `connection.rs` configures WAL and `synchronous=FULL`, but does not currently
  validate `PRAGMA page_size`; the corrected plan therefore records an explicit
  future 4-KiB fail-closed mutation prerequisite.
- `RuntimeStoreErrorKind` and `StorageRuntimeErrorCode` cover path, bootstrap,
  migration, integrity, resource, deadline and availability failures. They do
  not define invalid input, conversation NotFound, idempotency conflict or
  inconsistent idempotency-record semantics.

These facts support a no-schema-change design but do not prove the future hard
reserve or content-operation error domain.

### Current tests are schema tests, not service tests

`runtime_store/tests.rs` directly uses the test-owned `rusqlite::Connection` to
insert conversation, message, task and audit rows. Examples include:

- `conversation_and_message_constraints_reject_invalid_inputs`;
- `foreign_keys_and_unique_message_sequences_are_enforced`;
- `task_and_audit_allowlists_are_enforced`;
- `audit_sequence_uses_integer_primary_key_without_sqlite_sequence`;
- `status_reads_do_not_mutate_migration_or_content_state`.

These tests verify schema constraints and Phase 1B.1 invariants. They do not
verify production input validation, actor dispatch, atomic audit coupling,
idempotency, bounded pagination or restart readback through a service.

### Existing consumers remain out of scope

- `src/components/LocalInferencePanel.tsx` stores `messages` in React state and
  sends the last ten messages directly to the inference client. Phase 1B.2 must
  not silently attach persistence to this UI.
- `src-tauri/src/messaging.rs` creates stub sessions, random fake polling,
  local echo and RAM messages. Treating it as the durable conversation domain
  would conflate transport mocks with local runtime state.

## Missing Durable-State Services Inventory

### Required in Phase 1B.2

1. Validated Rust newtypes/enums and immutable record types.
2. Controlled content-operation errors distinct from storage health projection.
3. Conversation create/get/list repository behavior.
4. Message append/list repository behavior.
5. Actor request/response variants with bounded deadlines.
6. Transaction-scoped message-sequence allocation and conversation revision.
7. Private audit-event lookup/append for exactly two event types.
8. Operation-ID idempotency and mismatch detection.
9. Bounded keyset pagination.
10. Pre-write resource gate and post-operation health/read availability rules.
11. Restart/concurrency/rollback/privacy/denial tests.

### Explicitly deferred

- conversation deletion and delete-all;
- tasks and general audit repositories;
- audit browsing/export;
- backup/export/import/recovery/factory reset;
- frontend/UI persistence;
- Agent Supervisor and task semantics;
- all six-level memory behavior.

## Findings

### MEDIUM — Inter-slice audit ownership is ambiguous and blocks mutation design

Affected:

- `docs/planning/phases/phase-01b-durable-runtime-state-plan.md`, conversation
  invariants, transaction boundaries and slices 1B.2/1B.3.

Evidence:

- conversation creation and message append must include their audit event in
  the same transaction;
- slice 1B.2 owns conversations/messages;
- slice 1B.3 owns broad task/audit persistence.

Impact:

Implementing state writes without audit violates the accepted atomicity
contract. Moving the entire AuditStore into 1B.2 would over-expand the slice.

Planning disposition:

Human direction accepts in principle a private helper for only
`conversation.created` and `message.appended`. The corrected plan defines exact
fields, `INTEGER PRIMARY KEY` allocation, same-transaction rollback and the
absence of durable failure/denial audit rows. General audit APIs remain 1B.3.

Blocks implementation: **YES**, until implementation is separately authorized.

### MEDIUM — Unknown-outcome retry has no idempotency contract

Affected:

- future mutating manager/worker requests.

Evidence:

The existing manager uses deadlines and one-shot replies. A caller can stop
waiting while an already started transaction must still commit or roll back.
No content operation or persistent request identity currently exists.

Impact:

A naïve retry could create a duplicate conversation/message or consume another
sequence after the first request committed but its reply was lost.

Planning disposition:

The corrected plan uses caller-supplied canonical UUID-v4 operation IDs and the
globally unique audit-event ID as the lifetime committed-success record. It now
defines exact normalization, same/different replay, cross-operation reuse,
inconsistent evidence, actor serialization, concurrent duplicate behavior,
restart and no-retention semantics. `create_conversation` replay reconstructs
the original initial projection from immutable `created_at_ms`, immutable
Phase 1B.2 title, the fixed active status, deterministic initial counters and
the matching `conversation.created` audit event. It deliberately does not
return the current mutable `updated_at_ms`, `next_message_sequence` or
`revision` after messages have been appended. A missing subject or an
actor/timestamp/title relationship that cannot reproduce the canonical request
returns `IdempotencyRecordInconsistent`.

Blocks implementation: **YES**, until implementation is separately authorized.

### MEDIUM — Database hard limit is not yet a mutation gate

Affected:

- `runtime_store/config.rs`;
- `runtime_store/types.rs`;
- future content operations.

Evidence:

Phase 1B.1 can classify the runtime as resource-limited from database size, but
there is no content write path on which to enforce the accepted hard gate.

Impact:

Adding writes without an in-transaction/preflight resource policy could exceed
the accepted local-storage limit or convert a denial into a partial failure.

Planning disposition:

The corrected design no longer treats 16 MiB as a transaction budget. It fixes:

```text
H = 4,294,967,296 bytes
R = 16,777,216 bytes IMMUTABLE OPERATIONAL RESERVE
U = 4,278,190,080 bytes
G_CREATE = 8,388,608 bytes
G_APPEND = 33,554,432 bytes
WAL_AUTOCHECKPOINT_PAGES = 128
WAL_HARD_CEILING = 10,485,760 bytes
WAL_CREATE_GROWTH_BOUND = 2,097,152 bytes
WAL_APPEND_GROWTH_BOUND = 4,194,304 bytes
```

The future connection must set and read back
`PRAGMA wal_autocheckpoint = 128`; the exact trigger is
`32 + 128 × (page_size + 24)` bytes and is explicitly not a hard bound. The
reserve is allocated as 10 MiB checkpoint-copy budget, 1 MiB SHM growth,
2 MiB recovery overhead and 3 MiB safety margin.

The future actor must acquire `BEGIN IMMEDIATE`, measure physical DB/WAL/SHM
and live WAL, and admit only when both
`current_total + G_OPERATION <= U` and
`current_wal_size + WAL_OPERATION_GROWTH_BOUND <= WAL_HARD_CEILING`.
A 4-KiB page-size invariant and exact growth assumptions are
implementation/test prerequisites. If only the live-WAL inequality fails, the
actor rolls back the still-empty transaction, makes exactly one bounded
`PASSIVE` checkpoint attempt under the same deadline, reacquires the write
lock, and remeasures both inequalities. There is no retry loop or deadline
reset.

Pre-existing live or physical WAL above the ceiling leaves bounded reads
available after integrity/path checks but blocks mutations. A single bounded
`TRUNCATE` recovery is allowed only when the full recovery expression remains
within `H`; mutation eligibility then requires aggregate use below `U` and
both live and physical WAL at or below the ceiling. Capacity rejection commits
no subject/audit/idempotency state and leaves reads available.

Blocks implementation: **YES** until a separately authorized implementation
provides executable boundary, TOCTOU, checkpoint and maximum-payload proof.

### LOW — Operation-local errors must not poison global store health

Affected:

- `runtime_store/error.rs`;
- `worker.rs::RuntimeStoreManager`.

Evidence:

Current manager failure paths are designed for initialization/status failures
and can disable accepting state. Invalid content, not-found and idempotency
conflict are not store failures.

Impact:

Reusing the status error path could make one bad request disable an otherwise
healthy database.

Planning disposition:

The corrected plan defines `ContentOperationErrorCode` separately, maps every
operation’s errors, preserves health for operation-local failures, and makes
idempotency-record inconsistency deterministically transition the content
runtime to `IntegrityFailed`.

Blocks implementation: **YES** as an acceptance condition; not a current
vulnerability because no content API exists.

### LOW — Runtime-store composition files are already large

Affected:

- `runtime_store/worker.rs` (1,137 lines);
- `runtime_store/tests.rs` (1,909 lines);
- `scripts/validate-storage-runtime-contract.mjs` (1,808 lines).

Impact:

Adding SQL and repository tests directly to these files increases review risk
and makes actor/lifecycle regressions harder to isolate.

Remediation:

Keep worker dispatch thin; place SQL/unit-of-work code and repository tests in
new cohesive modules.

Blocks implementation: **NO**, if the planned module boundary is followed.

### LOW — Active repository instructions contain stale phase-status text

Affected:

- `AGENTS.md`, canonical sovereign-agent baseline.

Evidence:

The instructions still say Phase 1A is the only eligible runtime milestone and
Phase 1B remains `NO_GO`, while canonical main records merged/fresh-main
Phase 1B.1 and authorizes only Phase 1B.2 audit/planning.

Impact:

Future sessions can stop for the wrong reason or misread the current phase.

Remediation:

Reconcile only that status paragraph in this planning-only diff. Preserve the
requirement for a separate Phase 1B.2 implementation authorization.

Blocks implementation: **NO** after the planning correction.

### INFO — Existing frontend validator supports the Rust-only boundary

Affected:

- `scripts/validate-storage-runtime-contract.mjs`.

Evidence:

The validator rejects generic SQL and storage CRUD when exposed as Tauri
commands. It does not require Phase 1B.2 to add such commands.

Impact:

A private Rust service can be implemented without weakening ADR 0006 or the
frontend command choke point.

Remediation:

Keep the Tauri command inventory unchanged and preserve all boundary fixtures.

Blocks implementation: **NO**.

## Security Review

No confirmed Critical or High finding exists in current Phase 1B.1 code for a
Phase 1B.2 API because that API is absent.

The implementation threat model must cover:

- malicious/oversized titles and message content;
- prompt-injection content later read by a model;
- SQL injection and identifier control;
- operation replay and idempotency-key collision;
- concurrent sequence allocation;
- partial state/audit commits;
- audit/log/error content leakage;
- queue, disk, WAL and page-limit exhaustion;
- unhealthy/replaced/locked database behavior;
- cancellation/deadline response races;
- accidental Tauri/frontend exposure;
- plaintext database theft.

Stored messages remain untrusted content. No future consumer may treat them as
instructions merely because they were persisted.

## Test-Gap Matrix

| Required behavior | Current evidence | Phase 1B.2 requirement |
| --- | --- | --- |
| Schema constraints | Direct SQL tests | Preserve and add service-level denial tests |
| Conversation persistence | Direct fixture writes | Create/get/list through actor plus reopen |
| Message ordering | Unique index/direct tests | Atomic allocator, concurrency and restart tests |
| State/audit atomicity | Schema only | Forced-failure transaction tests |
| Idempotent retry | Unique audit-event schema support only | Global lifetime same/different/concurrent/restart/inconsistency tests for exactly two mutations, including create replay after one or more appends and replay/current-projection divergence |
| Bounded pagination | None | Exact get/list NotFound, typed cursor, limit, ordering, isolation and restart tests for three reads |
| Resource write gate | Status projection only | Exact 128-page auto-checkpoint readback, live/physical WAL accounting, 10-MiB ceiling, 2-/4-MiB operation WAL bounds, 10+1+2+3-MiB reserve allocation, one-attempt PASSIVE policy, oversized-WAL recovery, post-lock admission, rejection and read-availability tests |
| Sensitive error/log exclusion | Status contract tests | Content-operation redaction tests |
| Tauri boundary | 29/29 primary, 13/13 defense fixtures | Preserve exact command inventory |
| Real desktop restart | Not verified | Remains unclaimed in Phase 1B.2 unless separately authorized |
| Cross-platform runtime | Not verified | Remains a later platform gate |

## Recommended Bounded Architecture

```text
Trusted Rust caller
  -> validated private request
  -> RuntimeStoreManager
  -> bounded actor request
  -> private repository/unit of work
  -> one SQLite transaction
       - state mutation
       - required privacy-safe audit event
  -> typed controlled result
```

No frontend/Tauri content adapter exists in the recommended Phase 1B.2 slice.

## Readiness Decision

```text
AUDIT_RESULT = COMPLETE
PHASE_1B2_DESIGN_READINESS = CONDITIONAL_GO
PHASE_1B2_PLANNING_REVIEW = PHASE_1B2_PLANNING_REVIEW_PASS
PLANNING_FINDINGS = CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 0 / INFO 0
PLAN_RESULT = CONDITIONAL_GO
PHASE_1B2 = ELIGIBLE_FOR_SEPARATE_IMPLEMENTATION_AUTHORIZATION
PHASE_1B2_IMPLEMENTATION = NOT AUTHORIZED
CONDITIONS =
  SEPARATE IMPLEMENTATION AUTHORIZATION
CONTENT_OPERATIONS = 5
CONTENT_MUTATIONS = 2
CONTENT_READS = 3
PHASE_1B2_SCHEMA_DECISION = NO_SCHEMA_CHANGE_REQUIRED
IDEMPOTENCY_SCHEMA = CURRENT_SCHEMA_SUFFICIENT
HARD_RESERVE = 16 MiB IMMUTABLE
G_CREATE = 8 MiB
G_APPEND = 32 MiB
WAL_AUTOCHECKPOINT_PAGES = 128
WAL_HARD_CEILING = 10 MiB
WAL_CREATE_GROWTH_BOUND = 2 MiB
WAL_APPEND_GROWTH_BOUND = 4 MiB
CREATE_REPLAY_RESULT = DETERMINISTICALLY_RECONSTRUCTED
HD_1B2_01 = ACCEPT
HD_1B2_02 = ACCEPT
HD_1B2_03 = ACCEPT
HD_1B2_04 = ACCEPT / EXECUTABLE PROOF REQUIRED DURING IMPLEMENTATION
HD_1B2_05 = ACCEPT
PHASE_1B3 = NOT AUTHORIZED
REMOTE_PRODUCTION_WRITES = 0
REAL_USER_PROFILE_WRITES = 0
DEPLOYMENTS = 0
```

Relative implementation complexity after authorization: **M**.
