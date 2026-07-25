# Phase 1B.2 — Conversations and Messages Completion

Status: **IMPLEMENTED IN DRAFT PR / LOCAL GATE PASS / INDEPENDENT REVIEW PENDING / NOT MERGED**

Date: 2026-07-25

## Implementation execution ledger

```text
CANONICAL_BASE =
986e5a36204924dbec5b32999ea50650755891ef

CANONICAL_PLAN =
docs/planning/phases/phase-01b2-conversations-messages-plan.md

CONTENT_OPERATIONS =
5

CONTENT_MUTATIONS =
CREATE_CONVERSATION / APPEND_MESSAGE

CONTENT_READS =
GET_CONVERSATION / LIST_CONVERSATIONS / LIST_MESSAGES

AUTHORITY =
PRIVATE RUST SERVICE ONLY

SCHEMA_CHANGE =
NONE AUTHORIZED

DEPENDENCY_CHANGE =
NONE AUTHORIZED

FRONTEND_OR_TAURI_CONTENT_AUTHORITY =
NONE AUTHORIZED

HARD_RESERVE =
16 MiB IMMUTABLE

G_CREATE =
8 MiB

G_APPEND =
32 MiB

WAL_AUTOCHECKPOINT_PAGES =
128

WAL_HARD_CEILING =
10 MiB

WAL_CREATE_GROWTH_BOUND =
2 MiB

WAL_APPEND_GROWTH_BOUND =
4 MiB

EXECUTABLE_GROWTH_PROOF =
PASS / 20 CREATE RUNS + 20 APPEND RUNS / 0 FAILURES

INDEPENDENT_EXACT_HEAD_REVIEW =
REQUIRED / PENDING AFTER IMPLEMENTATION COMMIT

READY =
NOT AUTHORIZED

MERGE =
NOT AUTHORIZED

PHASE_1B3 =
NOT AUTHORIZED
```

## Pre-implementation evidence

- `origin/main`, the implementation worktree base and canonical merge commit
  are exactly `986e5a36204924dbec5b32999ea50650755891ef`.
- PR #29 is merged and contains the one planning commit
  `fa429412583867be3ac588764b1730e7cb095d20` across four documentation paths.
- The initial migration SHA-256 remains
  `62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d`.
- The expected structural fingerprint remains
  `37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77`.
- Migration 2 is absent and no private production content service exists.
- With the repository rustup proxies first in `PATH`, pinned Rust 1.95.0 ran
  the pre-change runtime-store suite: 64 passed, 0 failed. This includes the
  existing lifecycle and shutdown regressions.

## Stop rules

Stop before commit or push if implementation needs a migration, dependency,
frontend/Tauri content authority, a sixth operation, a second production
connection owner, a weakened shutdown contract, a larger planned growth bound,
or leaves a Critical, High or material Medium security finding unresolved.

Stop after the independent exact-head review. Do not mark the draft PR ready,
merge it, deploy, write the real user profile, or start Phase 1B.3.

## Completion evidence

### Implemented private service

Exactly five crate-private Rust operations now run through the existing
single-owner bounded SQLite worker:

| Operation | Kind | Contract |
| --- | --- | --- |
| `create_conversation` | Mutation | Validates a canonical UUID-v4 operation ID and optional 512-byte title; atomically inserts one conversation plus one privacy-safe audit event |
| `get_conversation` | Read | Returns one exact conversation projection or controlled `ConversationNotFound` |
| `list_conversations` | Read | Returns a bounded stable page ordered by `updated_at_ms DESC, id DESC` |
| `append_message` | Mutation | Validates canonical IDs, closed actor/role enums and 262,144-byte content; atomically inserts the message, advances conversation sequence/revision and appends one audit event |
| `list_messages` | Read | Returns a conversation-scoped bounded page ordered by `sequence_no ASC, id ASC` |

The private DTOs are `CreateConversationRequest`, `GetConversationRequest`,
`ListConversationsRequest`, `AppendMessageRequest` and
`ListMessagesRequest`, with typed record/page results. The safe error surface
is exactly:

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

No raw SQLite error, path, SQL text or message content is returned by that
surface. Validation occurs before queue admission and again at repository
entry. Stored title/message text remains untrusted data and is not interpreted
as instructions.

### Atomicity and idempotency

Both mutations use one `BEGIN IMMEDIATE` transaction for subject mutation and
the unique matching `audit_events.event_id`. The operation ID is global across
the two mutation types. Same-ID/same-canonical-request retries reconstruct the
original result; same-ID/different-request retries return
`IdempotencyConflict`. Missing, mismatched or unreproducible audit/subject
evidence returns `IdempotencyRecordInconsistent`, transitions content intake
to fail-closed integrity state, and preserves safe status reads.

Create replay deliberately returns the immutable initial conversation
projection even after later messages changed the current conversation row.
Forced audit failures roll back the conversation/message, conversation
sequence/revision update and operation-ID record together. Caller timeout is
an unknown-outcome boundary; the same operation ID safely resolves that
outcome without duplication.

### Capacity and WAL evidence

The implementation owns these non-configurable production constants:

```text
HARD_LIMIT = 4,294,967,296 bytes
IMMUTABLE_OPERATIONAL_RESERVE = 16,777,216 bytes
ORDINARY_USABLE_LIMIT = 4,278,190,080 bytes
CREATE_GROWTH_ENVELOPE = 8,388,608 bytes
APPEND_GROWTH_ENVELOPE = 33,554,432 bytes
PAGE_SIZE = 4,096 bytes
WAL_AUTOCHECKPOINT = 128 pages
WAL_AUTO_TRIGGER_BYTES = 527,392 bytes
WAL_HARD_CEILING = 10,485,760 bytes
CREATE_WAL_BOUND = 2,097,152 bytes
APPEND_WAL_BOUND = 4,194,304 bytes
CHECKPOINT_RECOVERY_OVERHEAD = 2,097,152 bytes
```

Admission is actor-owned after `BEGIN IMMEDIATE`, uses checked arithmetic, and
measures the database, WAL and SHM files after write-lock acquisition. A
mutation must fit both its aggregate envelope below `H - reserve` and its WAL
bound below the hard WAL ceiling. At most one `PASSIVE` checkpoint is attempted
under the same deadline. A pre-existing oversized WAL may attempt one bounded
`TRUNCATE` recovery only when the specified transient-copy projection fits;
otherwise mutations fail closed while reads remain available.

The implementation conservatively treats physical WAL file length as
`current_wal_size`; it does not infer live-frame reclamation from allocated
file capacity. This can reject a write after a successful `PASSIVE` checkpoint
that leaves the WAL file allocated, but cannot undercount WAL capacity or fail
open. This conservative limitation is intentional for this bounded slice.

The merge-blocking proof used bundled SQLite, 4,096-byte pages, exact maximum
title/message payloads, 20 fresh temporary roots per operation, and recorded
DB/WAL/SHM before/after values on every run:

| Proof | Observed maximum | Bound | Remaining margin |
| --- | ---: | ---: | ---: |
| Create aggregate growth | 32,960 bytes | 8,388,608 bytes | 8,355,648 bytes |
| Create WAL growth | 32,960 bytes | 2,097,152 bytes | 2,064,192 bytes |
| Append aggregate growth | 313,120 bytes | 33,554,432 bytes | 33,241,312 bytes |
| Append WAL growth | 313,120 bytes | 4,194,304 bytes | 3,881,184 bytes |

All 40 proof executions passed. Tests also cover exact-threshold algebra,
DB/WAL/SHM accounting, repeated maximum writes, busy-reader checkpoint
behavior, external-writer TOCTOU, recoverable and unsafe oversized WAL,
capacity rejection atomicity, reusable operation IDs and safe reads.

### Lifecycle and authority boundary

The five operations reuse the existing queue capacity, ordinary deadline,
priority shutdown channel, interrupt watchdog and single worker connection.
Queued content is rejected once shutdown closes intake. An already active
mutation either commits its complete subject-plus-audit unit or rolls back;
shutdown never observes a partial commit. Clean close retains bounded
`wal_checkpoint(TRUNCATE)` behavior.

```text
CONTENT_OPERATIONS = 5
CONTENT_MUTATIONS = 2
CONTENT_READS = 3
PUBLIC_CONTENT_TAURI_COMMANDS = 0
STORAGE_STATUS_TAURI_COMMANDS = 1
FRONTEND_CONTENT_CLIENTS = 0
FRONTEND_CONTENT_UI = 0
GENERIC_SQL_AUTHORITY = 0
PATH_ARGUMENTS_OVER_IPC = 0
PHASE_1B3_AUTHORITY = 0
```

The dormant private API has narrowly documented `dead_code` allowances because
Phase 1B.2 intentionally provides no production frontend/Tauri consumer.
Runtime-store compiler and Clippy warning locations are zero.

### Schema and dependency invariance

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
MANIFEST_CHANGES = 0
LOCKFILE_CHANGES = 0
DEPENDENCY_GRAPH_CHANGE = 0
```

### Local validation

All checks ran with pinned `rustc`, `rustdoc` and Cargo 1.95.0:

| Check | Result |
| --- | --- |
| Scoped Rust formatting for every changed Rust file | PASS |
| New conversation/message repository tests | 36/36 PASS |
| Full runtime-store suite | 100/100 PASS |
| Phase 1A inference suite | 67/67 PASS |
| Full Rust suite | 216/216 PASS |
| `cargo check --all-targets --locked` | PASS |
| `cargo clippy --all-targets --locked` | PASS |
| New runtime-store warning/Clippy locations | 0 |
| Storage Runtime contract | 29/29 primary, 13/13 defense-in-depth, 46 structural PASS |
| Inference frontend/Rust contract | PASS |
| TypeScript + production build | PASS / 1,763 modules |
| `npm audit --omit=dev` | 0 vulnerabilities |
| Secret scan | PASS |
| Protected Rust warning scan | PASS |
| Executable growth proof | 20 create + 20 append / 0 failures |

The isolated worktree required a disposable `npm ci --ignore-scripts` using the
unchanged lockfile. It created only ignored local dependency/build artifacts.
No dependency declaration or lockfile changed.

### Changed-path inventory

The implementation candidate changes only:

```text
docs/architecture/CAPABILITY_STATUS_MATRIX.md
docs/planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md
docs/planning/phases/phase-01b-durable-runtime-state-plan.md
docs/planning/phases/phase-01b2-conversations-messages-completion.md
docs/security/SECURITY_GATES.md
src-tauri/src/runtime_store/config.rs
src-tauri/src/runtime_store/connection.rs
src-tauri/src/runtime_store/error.rs
src-tauri/src/runtime_store/mod.rs
src-tauri/src/runtime_store/models.rs
src-tauri/src/runtime_store/path_policy.rs
src-tauri/src/runtime_store/repositories/conversations.rs
src-tauri/src/runtime_store/repositories/messages.rs
src-tauri/src/runtime_store/repositories/mod.rs
src-tauri/src/runtime_store/repositories/unit_of_work.rs
src-tauri/src/runtime_store/repository_tests.rs
src-tauri/src/runtime_store/worker.rs
```

The exact implementation head is necessarily created after this document is
included in the single commit; it must be recorded and exact-head protected in
the draft PR body, independent review report and final task report.

### Known limitations and gate

- Phase 1B.2 has no product consumer and adds no public content authority.
- The real desktop restart flow and Windows/Linux runtime remain unverified.
- Remote CI is not present and is not claimed.
- Physical WAL length is the conservative capacity metric; live-frame
  introspection is not claimed.
- Retention/deletion/export/backup, tasks, full recovery/privacy closure and
  the pre-production SQLCipher decision remain open in later separately
  authorized slices.
- Inherited repository-wide Rust warnings, RustSec findings, dev-inclusive npm
  advisories and global rustfmt debt remain visible and unchanged in scope.

```text
PHASE_1B2 =
IMPLEMENTED_IN_DRAFT_PR / LOCAL_GATE_PASS / INDEPENDENT_REVIEW_PENDING

PHASE_1B2_IMPLEMENTATION =
NOT MERGED

PHASE_1B3 =
NOT AUTHORIZED

REMOTE_PRODUCTION_WRITES =
0

REAL_USER_PROFILE_WRITES =
0

DEPLOYMENTS =
0
```
