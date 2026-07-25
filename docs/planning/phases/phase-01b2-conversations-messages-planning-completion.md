# Phase 1B.2 Conversations and Messages Planning Completion

Status: **PLANNING REVIEW PASS / CONDITIONAL_GO / IMPLEMENTATION NOT AUTHORIZED**

Date: 2026-07-24

## Scope Completed

This planning-only task audited the durable runtime-state implementation on
canonical `main` and defined a bounded Phase 1B.2 plan for private Rust
conversation and message services.

No application source, migration, manifest, lockfile, capability, dependency,
CI, runtime configuration or production system was changed.

Canonical source:

```text
BASE =
42c5ac0b4fa501e59c83a5b0b395c73af382420d

SOURCE_WORKTREE_AT_START =
CLEAN

PLANNING_BRANCH =
codex/phase-01b2-audit-plan
```

## Repository Evidence

The audit verified:

- Phase 1B.1 owns a real single-connection SQLite runtime, fixed migration,
  startup validation and read-only health projection.
- The version-1 schema already contains constrained conversation, message, task
  and audit tables.
- Production runtime-store requests and manager methods do not provide
  conversation or message content operations.
- Current direct conversation/message SQL exists only in tests and is not a
  production service path.
- The only public storage Tauri command remains the no-user-argument
  `get_storage_runtime_status`.
- `LocalInferencePanel.tsx` keeps chat history in volatile React state.
- `messaging.rs` is an independent mocked RAM-only surface and is not a
  Phase 1B.2 consumer.

The full evidence and capability classifications are in
`docs/audits/PHASE_1B2_CONVERSATIONS_MESSAGES_READINESS_AUDIT_2026-07-24.md`.

## Planning Result

The corrected plan limits a future implementation to five private Rust content
operations:

1. `create_conversation` — mutation;
2. `get_conversation` — read;
3. `list_conversations` — read;
4. `append_message` — mutation;
5. `list_messages` — read.

```text
CONTENT_OPERATIONS = 5
CONTENT_MUTATIONS = 2
CONTENT_READS = 3
```

The future slice must retain the existing schema and dependency graph. It adds
no Tauri CRUD command, frontend consumer, chat persistence, task behavior,
memory promotion, model/tool/network call or transport. This planning task
performs no production write.

```text
PHASE_1B2_SCHEMA_DECISION = NO_SCHEMA_CHANGE_REQUIRED
IDEMPOTENCY_SCHEMA = CURRENT_SCHEMA_SUFFICIENT
MIGRATION_2 = ABSENT
```

## Historical Reviews and Corrections

The first independent review historically returned:

```text
HISTORICAL_REVIEW_1 =
PHASE_1B2_PLANNING_REVIEW_BLOCKED_BY_FINDINGS

HISTORICAL_PLAN_RESULT = NO_GO
HISTORICAL_FINDINGS = CRITICAL 0 / HIGH 0 / MEDIUM 4 / LOW 1 / INFO 2
```

The first correction preserved the accepted no-schema-change and Rust-only
boundaries and addressed:

- **M-01:** exact manager methods, request/output DTOs, per-read NotFound,
  ordering, pagination, restart and error semantics;
- **M-02:** a separate immutable `R = 16 MiB`, exact
  `G_CREATE = 8 MiB` and `G_APPEND = 32 MiB`, and post-lock DB/WAL/SHM
  admission;
- **M-03:** canonical payload comparison, global database-lifetime operation
  IDs, concurrency, restart, retention and inconsistency behavior;
- **M-04:** exact audit fields, rollback/no-audit denial behavior, content error
  codes and global-health effects;
- **L-01:** five operations are now exactly two mutations and three reads.

The exact-patch review of that first correction also historically remained
blocked:

```text
HISTORICAL_REVIEW_2 =
PHASE_1B2_PLANNING_REVIEW_BLOCKED_BY_FINDINGS

OLD_PATCH_SHA256 =
737c6e8066a10a961866382fb36eccd27ad99048152d2fbdc9200003f1482421

HISTORICAL_FINDINGS =
CRITICAL 0 / HIGH 0 / MEDIUM 2 / LOW 0 / INFO 0
```

The two Medium findings were:

1. create replay promised the original result without defining how to
   reconstruct it after append changed the current conversation projection;
2. the reserve design lacked exact auto-checkpoint, live-WAL, retained-file
   and checkpoint-copy controls.

The final docs-only correction defined deterministic create-result
reconstruction and exact WAL/checkpoint controls. The old fingerprint is
preserved only as historical review evidence.

## Canonical Planning Review Result

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

PHASE_1B2 =
ELIGIBLE_FOR_SEPARATE_IMPLEMENTATION_AUTHORIZATION

PHASE_1B2_IMPLEMENTATION =
NOT AUTHORIZED

PHASE_1B3 =
NOT AUTHORIZED
```

These are planning decisions. They do not authorize implementation.

## Security Result

No confirmed Critical or High finding was identified in the current read-only
Phase 1B.1 runtime.

The corrected design gives atomic state/audit ownership to a private two-event
helper, uses the existing unique audit-event ID as global lifetime idempotency
evidence, and reconstructs the original create result from immutable evidence
rather than mutable conversation counters. It separates the immutable
16-MiB operational reserve from fixed mutation and live-WAL growth envelopes,
requires exact auto-checkpoint configuration and bounds checkpoint recovery.
Capacity rejection writes no durable audit row.

Operation-local validation, NotFound, conflict, capacity, busy and deadline
errors do not poison global runtime health. Inconsistent idempotency evidence
deterministically transitions the content runtime to `IntegrityFailed`.

## Corrected Contract Inventory

Manager methods:

```text
create_conversation(CreateConversationRequest) -> ConversationRecord
get_conversation(GetConversationRequest) -> ConversationRecord
list_conversations(ListConversationsRequest) -> ConversationPage
append_message(AppendMessageRequest) -> MessageRecord
list_messages(ListMessagesRequest) -> MessagePage
```

Requests:

```text
CreateConversationRequest =
operation_id / actor / title

GetConversationRequest =
conversation_id

ListConversationsRequest =
limit / typed (updated_at_ms, id) cursor

AppendMessageRequest =
operation_id / actor / conversation_id / role / content

ListMessagesRequest =
conversation_id / limit / after_sequence_no
```

Only the create and append requests contain `operation_id`. The exact output
fields, cursor predicates, NotFound behavior, restart behavior and
cross-conversation isolation are fixed in the plan.

Create replay returns the original deterministic projection:

```text
id = persisted conversation ID
title = persisted immutable Phase 1B.2 title
status = active
created_at_ms = persisted created_at_ms
updated_at_ms = persisted created_at_ms
next_message_sequence = 1
revision = 0
```

It intentionally ignores the current mutable `updated_at_ms`,
`next_message_sequence` and `revision`, so replay after append differs from
`get_conversation`. Migration 1 remains sufficient because the title and
creation timestamp are immutable in this slice, status cannot change, initial
counters are deterministic and the audit event binds operation, actor,
timestamp and subject.

Safe content codes:

```text
InvalidInput
ConversationNotFound
IdempotencyConflict
IdempotencyRecordInconsistent
CapacityExceeded
BusyTimeout
DeadlineExceeded
Unavailable
IntegrityFailure
Internal
```

Reserve contract:

```text
H = 4,294,967,296 bytes
R = 16,777,216 bytes / IMMUTABLE
U = 4,278,190,080 bytes
G_CREATE = 8,388,608 bytes
G_APPEND = 33,554,432 bytes
WAL_AUTOCHECKPOINT_PAGES = 128
WAL_HARD_CEILING = 10,485,760 bytes
WAL_CREATE_GROWTH_BOUND = 2,097,152 bytes
WAL_APPEND_GROWTH_BOUND = 4,194,304 bytes
ADMISSION_1 = current_total + G_OPERATION <= U
ADMISSION_2 =
  current_wal_size + WAL_OPERATION_GROWTH_BOUND <= WAL_HARD_CEILING
CAPACITY_REJECTION_AUDIT = NO DURABLE AUDIT ROW
```

The connection must set and read back
`PRAGMA wal_autocheckpoint = 128`. At page size `P`, its exact byte trigger is
`32 + 128 × (P + 24)`; this trigger is not a hard bound because readers can
delay checkpoint progress. The 10-MiB live-WAL ceiling remains authoritative.

The immutable reserve allocation is exact:

```text
CHECKPOINT_COPY_BUDGET = 10 MiB
SHM_GROWTH_BUDGET = 1 MiB
CHECKPOINT_RECOVERY_OVERHEAD = 2 MiB
UNALLOCATED_SAFETY_MARGIN = 3 MiB
TOTAL = 16 MiB
```

`current_total` counts physical DB, WAL and SHM lengths.
`current_wal_size` is the live uncheckpointed frame obligation used for the
WAL ceiling. If only that inequality fails, the empty write transaction is
rolled back before exactly one bounded `PASSIVE` checkpoint attempt. The
worker then reacquires `BEGIN IMMEDIATE` under the same deadline, revalidates
file identities and remeasures both inequalities. It does not loop or reset
the deadline.

Pre-existing live or physical WAL over the ceiling allows bounded reads after
integrity/path validation but blocks mutations. A single bounded `TRUNCATE`
recovery may run outside a mutation only when the recovery expression fits
under `H`; aggregate use must be at or below `U` and both physical and live WAL
must satisfy their ceilings before mutations resume.

The audit mapping uses `event_id = correlation_id = operation_id`, a closed
`ContentActor`, exact success event types, and an audit `sequence_no` allocated
by omitting the `INTEGER PRIMARY KEY` column from the serialized insert.

## Files Changed

```text
AGENTS.md
docs/audits/PHASE_1B2_CONVERSATIONS_MESSAGES_READINESS_AUDIT_2026-07-24.md
docs/planning/phases/phase-01b2-conversations-messages-plan.md
docs/planning/phases/phase-01b2-conversations-messages-planning-completion.md
```

`AGENTS.md` changed only to replace stale phase-status text with the current
planning-only Phase 1B.2 gate.

## Validation

The final planning diff was checked with:

- changed-path and docs-only scope review;
- `git diff --check`;
- required plan-section review;
- referenced repository-path existence checks;
- Markdown relative-link/path review;
- local-path and secret-like URL review;
- repository secret scan;
- false implementation-claim and phase-status consistency review;
- complete diff review against canonical `main`.

Result:

```text
CHANGED_PATHS = 4 / 4 ALLOWED
APPLICATION_SOURCE_CHANGES = 0
MIGRATION_CHANGES = 0
MANIFEST_OR_LOCKFILE_CHANGES = 0
FRONTEND_OR_TAURI_CAPABILITY_CHANGES = 0
GIT_DIFF_CHECK = PASS
REFERENCED_PATHS = PASS
MARKDOWN_LINKS = PASS / NONE ADDED
SECRET_SCAN = PASS
FALSE_IMPLEMENTATION_CLAIMS = 0
RUNTIME_TESTS = NOT RUN / NOT REQUIRED FOR DOCS-ONLY PLANNING
```

The fresh-main executable evidence inherited from the merged Phase 1B.1 gate
was inspected but not reclassified as a Phase 1B.2 implementation result.

## Mutations and Publication

```text
APPLICATION_CODE_MUTATIONS = 0
SCHEMA_OR_MIGRATION_MUTATIONS = 0
PRODUCTION_WRITES = 0
REAL_USER_PROFILE_WRITES = 0
DEPLOYMENTS = 0
COMMIT = NOT PERFORMED
PUSH = NOT PERFORMED
PULL_REQUEST = NOT CREATED
```

## Final Gate

```text
PHASE_1B2_AUDIT = COMPLETE
PHASE_1B2_PLANNING_REVIEW = PHASE_1B2_PLANNING_REVIEW_PASS
PLANNING_FINDINGS = CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 0 / INFO 0
PHASE_1B2_PLAN = CONDITIONAL_GO
PHASE_1B2_IMPLEMENTATION_AUTHORIZATION = NOT GRANTED
PHASE_1B2 = ELIGIBLE_FOR_SEPARATE_IMPLEMENTATION_AUTHORIZATION
PHASE_1B2_IMPLEMENTATION = NOT AUTHORIZED
PHASE_1B3 = NOT AUTHORIZED

NEXT_AUTHORIZED_ACTION =
SEPARATE HUMAN PHASE 1B.2 IMPLEMENTATION AUTHORIZATION
```
