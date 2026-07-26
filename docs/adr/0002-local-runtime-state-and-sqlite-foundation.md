# ADR 0002: Local Runtime State and SQLite Foundation

- Status: Accepted; implementation partial; Phase 1B.1 through Phase 1B.3 merged and fresh-main verified
- Date: 2026-07-04
- Scope: first durable Edge runtime-state store

## Context

The audited Edge snapshot has no durable task/conversation/audit store. Messaging uses in-memory vectors and agent/loop recovery is not possible. Introducing a full six-level memory system at the same time as inference and Supervisor foundations would create an oversized, difficult-to-review change.

## Decision

1. SQLite is the initial durable local runtime store.
2. Phase 1B is separate from inference and Supervisor work.
3. The first schema contains only:
   - schema migrations;
   - conversations;
   - messages;
   - tasks;
   - audit events.
4. Transactions, versioned migrations, restart recovery, retention hooks, deletion, export and migration/corruption tests are mandatory.
5. Storage is accessed through repository interfaces so later memory layers do not leak SQL across runtime modules.
6. Six-level memory is a later phase. Episodes, semantic facts, procedures, entities, relations and embeddings are excluded from the foundation.
7. No external vector database, graph database, Supabase memory store or cloud database is introduced in Phase 1B.
8. Platform at-rest protection/encryption requirements must be resolved in the Phase 1B plan.

## Consequences

- The first durable slice remains reversible and testable.
- Agent Supervisor recovery can rely on a stable task/audit contract in Phase 1C.
- “SQLite foundation” must not be marketed as complete agent memory.
- Schema evolution and deletion/export behavior become release-gated.

## Alternatives rejected

- RAM-only state: rejected because restart recovery and auditability are required.
- Full six-level schema immediately: rejected due scope and truthfulness risk.
- External vector/graph service first: rejected because local-first ownership and MVP complexity favor SQLite.

## Verification gate

See [Durable runtime state gate](../security/SECURITY_GATES.md). The separately
authorized Phase 1B.1 through Phase 1B.3 vertical slices are merged and
fresh-main verified. Phase 1B.1 implements bootstrap and a safe read-only status
projection: exact `rusqlite` 0.40.1, bundled SQLite 3.53.2, the five-table
schema, one bounded Rust owner, explicit bounded application shutdown,
deadline-interrupt and hard-link-aware path controls, strict UUID-v4 schema
constraints, and restart/reopen tests.

Phase 1B.2 adds exactly five crate-private Rust operations for create/get/list
conversations and append/list messages. Its two mutations couple the subject,
global operation-ID evidence, and one privacy-safe audit event in one immediate
SQLite transaction. Reads are bounded and deterministic. Fail-closed 4-GiB
allocation, immutable 16-MiB reserve, operation-growth, and WAL admission
controls passed the executable evidence described below. There is still no
public content Tauri command or frontend content authority.

```text
PHASE_1B_1 = MERGED / FRESH_MAIN_VERIFIED
PHASE_1B2 = MERGED / FRESH_MAIN_VERIFIED
PHASE_1B3 = MERGED / FRESH_MAIN_VERIFIED
PHASE_1B3_IMPLEMENTATION = MERGED
PHASE_1B2_MERGED_IMPLEMENTATION_HEAD = c2fdcc5a234779c7ad886ee5aa0d0762c938a59d
PHASE_1B2_MERGE_COMMIT = ec99bf70d6ada94bc1caae9886cca25ad42852f9
PHASE_1B2_MERGED_AT = 2026-07-25T14:27:32Z
PHASE_1B3_ORIGINAL_IMPLEMENTATION_COMMIT = e62dd44d2bfb88ce7c5ccccad92efcf2e319c45b
PHASE_1B3_MERGED_IMPLEMENTATION_HEAD = 79b14d80a851042a64eff8ef8e4c84f3d6f64e5e
PHASE_1B3_MERGE_COMMIT = dfad7d47745355e09fc8d169568ca6cab4acc48b
PHASE_1B3_MERGED_AT = 2026-07-26T09:26:50Z
STORAGE_BOOTSTRAP = IMPLEMENTED_AND_VERIFIED
STORAGE_RUNTIME_PROJECTION = IMPLEMENTED_AND_VERIFIED_IN_REPOSITORY
PRIVATE_CONVERSATION_STORAGE = IMPLEMENTED_AND_VERIFIED
PRIVATE_MESSAGE_STORAGE = IMPLEMENTED_AND_VERIFIED
CONTENT_OPERATIONS = 5
CONTENT_MUTATIONS = 2
CONTENT_READS = 3
INERT_TASK_STORAGE = IMPLEMENTED_AND_VERIFIED
TYPED_AUDIT_PERSISTENCE = IMPLEMENTED_AND_VERIFIED
TYPED_AUDIT_READBACK = IMPLEMENTED_AND_VERIFIED
PRIVATE_PHASE_1B3_OPERATIONS = 5
TASK_MUTATIONS = 1
TASK_READS = 2
AUDIT_READS = 2
TASK_STATE = created only
TASK_EVENT_TYPE = task.recorded
TASK_IDEMPOTENCY_KEY = SQL NULL / DEFERRED
TASK_EXECUTION = ABSENT
GENERIC_AUDIT_APPEND = 0
PUBLIC_CONTENT_TAURI_COMMANDS = 0
PUBLIC_TASK_TAURI_COMMANDS = 0
PUBLIC_AUDIT_TAURI_COMMANDS = 0
STORAGE_STATUS_TAURI_COMMANDS = 1
FRONTEND_CONTENT_AUTHORITY = 0
DURABLE_RUNTIME_STATE = PARTIALLY_IMPLEMENTED
PHASE_1B = NOT COMPLETE
PHASE_1B4 = NOT AUTHORIZED
PHASE_1C = NOT AUTHORIZED
REAL_DESKTOP_RESTART = NOT VERIFIED
CROSS_PLATFORM_RUNTIME = NOT VERIFIED
REMOTE_CI = NOT PRESENT / NOT CLAIMED
REMOTE_PRODUCTION_WRITES = 0
REAL_USER_PROFILE_WRITES = 0
DEPLOYMENTS = 0
```

Phase 1B.2 repository verification passed 36/36 new repository tests, 100/100
runtime-store tests, 67/67 inference tests and 216/216 full Rust tests with Rust
1.95.0, plus Cargo check/Clippy, 29/29 primary boundary fixtures, 13/13
defense-in-depth fixtures, 46 structural checks, the production build over
1,763 modules, and zero production npm vulnerabilities. The executable growth
proof passed 40/40 runs: maximum create aggregate/WAL growth was
32,960/32,960 bytes and maximum append aggregate/WAL growth was
313,120/313,120 bytes. Runtime-store warning locations were 0. Dev-inclusive
npm audit retained 11 inherited advisories outside the production dependency
set. Inherited RustSec, warning and 94-file rustfmt debt were unchanged. No
remote CI was present.

PR #33 fresh-main verification passed 31/31 focused Phase 1B.3 tests, 131/131
runtime-store tests, 67/67 inference tests and 247/247 full Rust tests. Task
growth passed 20/20 with maximum aggregate/WAL growth of 41,200/41,200 bytes;
the existing create/append regression passed 40/40 with maxima of 32,960 and
313,120 bytes. Cargo check, Clippy, scoped rustfmt for all 11 changed Rust
files, storage contracts, the 1,763-module production build, zero-vulnerability
production npm audit and secret scan passed. These are PR #33 fresh-main
results, not checks rerun by the post-merge documentation reconciliation.

The verified schema invariants are:

```text
MIGRATION_SHA = 62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d
STRUCTURAL_FINGERPRINT = 37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77
TABLES = 5
EXPLICIT_INDEXES = 7
SQLITE_AUTOINDEXES = 7
SQLITE_SEQUENCE = 0
MIGRATION_2 = ABSENT
```

The merged Phase 1B.3 slice adds exactly five crate-private Rust operations: one
atomic inert-task record mutation, two task reads, and two typed audit reads. It
retains `created` as data-only state, writes `task.recorded`, leaves
`tasks.idempotency_key` SQL `NULL`, and closes the previously accepted stringly
audit writer boundary. Operation-local missing-record errors are
`content_task_not_found` and `content_audit_event_not_found`; neither poisons
content intake.

These slices do not implement public content CRUD, executable task semantics,
backup/export, retention/deletion services, six-level memory, Phase 1B.4, or
production/platform acceptance. The full ADR implementation therefore remains
partial.
