# Phase 1B Durable Runtime State — Planning Completion

Status: **PLANNING ARTIFACT CONDITIONAL_PASS / FRESH REVIEW PENDING / IMPLEMENTATION NO_GO**

## Starting state

- Repository: `DAARION-DAO/daarion-edge-client`
- Starting `origin/main`: `62a1d514b93925e8b7098c6db19f8751a70a7bf8`
- Phase 1A reviewed head: `9e8c5d9c8adb4c02bfa9b11e970e33a0bbfd640f`
- Work branch: `phase-01b/durable-runtime-state-plan`
- Phase 1A: `MERGED / FRESH-MAIN VERIFIED / PASS`
- Live Ollama smoke: `NOT PERFORMED / NOT CLAIMED`

Preflight confirmed exact remote main, Phase 1A reachability, a clean source
worktree, no existing Phase 1B branch, and no conflicting active Phase 1B PR.
No runtime SQLite implementation was found. No Critical/High storage finding
prevented a coherent planning document.

## Files audited

Architecture and dependency evidence:

- `AGENTS.md`
- `README.md`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/lib.rs`
- `.github/workflows/release.yml`
- `docs/adr/0001-local-first-inference-and-remote-consent.md`
- `docs/adr/0002-local-runtime-state-and-sqlite-foundation.md`
- `docs/adr/0003-reticulum-lxmf-integration-boundary.md`

Persistence, state, identity, and privacy evidence:

- `src-tauri/src/identity.rs`
- `src-tauri/src/enrollment.rs`
- `src-tauri/src/pairing.rs`
- `src-tauri/src/reset.rs`
- `src-tauri/src/genesis.rs`
- `src-tauri/src/messaging.rs`
- `src-tauri/src/heartbeat.rs`
- `src-tauri/src/worker/mod.rs`
- `src-tauri/src/worker/queue.rs`
- `src-tauri/src/agents/agent_intent.rs`
- `src-tauri/src/agents/agent_execution.rs`
- `src-tauri/src/agents/agent_result_binding.rs`
- `src-tauri/src/registry_client.rs`
- `src/App.tsx`
- `src/components/LocalInferencePanel.tsx`
- `src/components/MessagingPanel.tsx`
- `src/components/GenesisWizard.tsx`
- `src/lib/preflightApi.ts`

Canonical planning/security evidence:

- `docs/planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md`
- `docs/architecture/CAPABILITY_STATUS_MATRIX.md`
- `docs/security/SECURITY_GATES.md`
- existing Phase 0/1A planning and completion reports.

## Repository facts

- The Rust/Tauri application has no SQLite runtime dependency, schema, migration
  runner, repository abstraction, database backup, or durable export.
- Identity, enrollment, pairing, worker mode, and Genesis media use separate
  app-data/keyring/file boundaries. They are not a unified runtime-state store.
- Messaging is mock/in-memory, local chat/UI state is transient, and the worker
  queue is RAM-only/security-gated.
- IndexedDB is used only for a frontend registry cache. Browser `localStorage`
  holds Genesis-adjacent data but is explicitly outside Phase 1B.
- No durable Conversation or structured AuditEvent domain model exists. Existing
  agent/task-shaped types are placeholders and do not define Supervisor behavior.
- Current tests do not include migration, restart, WAL, corruption, deletion,
  or deterministic export coverage.
- `src-tauri/src/lib.rs` is the application composition point and should receive
  only narrow service wiring, never database implementation logic.
- Existing ADR 0002 already selects SQLite and the five-table foundation; no new
  ADR was required by this docs-only task.

## Decisions accepted

- Use one direct `rusqlite` dependency with `bundled`, `limits`, and `backup`,
  owned behind a project-specific dedicated blocking actor. Do not expose
  Tauri SQL.
- Use exactly `schema_migrations`, `conversations`, `messages`, `tasks`, and
  `audit_events` in the first schema.
- Store UUID v4 identifiers as canonical text and UTC timestamps as integer Unix
  epoch milliseconds.
- Use one serialized connection, WAL, `synchronous=FULL`, foreign keys,
  secure-delete, a bounded request queue, and explicit deadlines.
- Use immutable checksummed embedded migrations, transactional exact-once
  application, and fail-closed downgrade/newer-schema behavior.
- Keep Phase 1B tasks inert and defer all state-machine/execution semantics to
  Phase 1C.
- Make export deterministic, versioned, plaintext, bounded, and secret/path-free;
  leave import out of scope.
- Accept standard SQLite for the foundation with explicit residual stolen-disk
  risk acceptance and an OS full-disk-encryption requirement on supported
  production devices. The database, backups, and JSON exports remain plaintext
  at the application layer and must never be described as encrypted.

## Planning decisions resolved

Human decisions HD-01 through HD-09 resolve the plan's storage authority,
SQLite integration, foundation encryption posture, retention, resource limits,
operational pragmas, export contract, and platform boundary. They approve the
plan only. Phase 1B implementation remains `NO_GO`, and Phase 1B.1 is not
authorized by this task.

## Documents created and updated

Created:

- `docs/planning/phases/phase-01b-durable-runtime-state-plan.md`
- `docs/planning/phases/phase-01b-durable-runtime-state-planning-completion.md`

Updated:

- `docs/planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md`
- `docs/architecture/CAPABILITY_STATUS_MATRIX.md`
- `docs/security/SECURITY_GATES.md`

No ADR, Rust/TypeScript source, manifest, lockfile, migration, capability,
configuration, CI, or deployment file was created or changed.

## Focused review correction

The focused Codex review of commit
`65dc740a9780c1fdfad056cb62cc75226d012538` found two substantive documentation
issues:

- P2: `AUTOINCREMENT` would create `sqlite_sequence` and contradict the exact
  five-table boundary. The proposal now uses `INTEGER PRIMARY KEY`, documents
  rowid reuse limits, prohibits public audit deletion, and requires the initial
  migration test to prove exactly five tables with no `sqlite_sequence` or
  other unexpected internal table.
- P3: Rust lockfile evidence now consistently names
  `src-tauri/Cargo.lock`; repository-root `Cargo.lock` is confirmed absent.

The correction remains documentation-only and does not authorize Phase 1B.1.

A subsequent focused review of exact head
`6c7bda635eaefa2878b6acb0e63970c700caafb6` found that the original audited-file
list used shortened, nonexistent ADR filenames. The evidence list now uses the
three exact repository paths. Their ADR numbers and titles match the files, and
ADR 0002 remains the accepted SQLite/five-table foundation.

## Polyglot storage refinement

The final architecture-only refinement from exact head
`74ed00fde00c3c9962bbadec2049ebfbe06c1cab` defines SQLite as the authoritative
local transactional state layer and records semantic, graph, artifact, and
remote storage as future non-authoritative roles. The semantic and graph stores
are rebuildable projections; SQLite remains authoritative for artifact metadata;
and remote systems receive projections, signed summaries, or optional encrypted
backups only.

No engine, dependency, schema, migration, runtime, synchronization loop, or
deployment was added. Qdrant, LanceDB, a graph database, object storage,
SQLCipher, remote replication, and cloud backends remain deferred to future
ADRs. The master execution phase map was not renumbered; the roadmap labels the
requested storage sequence as non-authorizing stream-local placeholders.

## Human decision finalization

The clean Codex review of exact head
`15abe536a98c824c8a19bb5858ed544af2f8e4b2` cleared the docs-only refinement
with zero unresolved substantive threads and no P1/P2 finding. The following
human decisions were then recorded without changing application code:

```text
POLYGLOT_STORAGE_ARCHITECTURE = ACCEPTED
SQLITE_ROLE = AUTHORITATIVE_LOCAL_TRANSACTIONAL_STATE
SEMANTIC_STORE = REPLACEABLE_REBUILDABLE_PROJECTION
GRAPH_STORE = REPLACEABLE_REBUILDABLE_PROJECTION
ARTIFACT_STORE = FILESYSTEM_OR_OBJECT_STORE_FOR_LARGE_BYTES
REMOTE_SYNC = OPTIONAL_NON_AUTHORITATIVE_PROJECTIONS

SQLITE_LIBRARY = rusqlite
FEATURES = bundled, limits, backup
OWNERSHIP = Rust-owned dedicated blocking actor
FRONTEND_SQL_AUTHORITY = FORBIDDEN

PHASE_1B_FOUNDATION_ENCRYPTION = STANDARD_SQLITE_WITH_EXPLICIT_RISK_ACCEPTANCE
CONVERSATIONS_MESSAGES = RETAIN_UNTIL_EXPLICIT_USER_DELETION
INERT_TASKS = RETAIN_UNTIL_EXPLICIT_USER_DELETION
AUDIT_EVENTS = RETAIN_WHILE_RELATED_RUNTIME_STATE_EXISTS
LLM_CONTROLLED_DELETION = FORBIDDEN
VERIFIED_LOCAL_BACKUPS = MAXIMUM_3

CONVERSATION_TITLE_MAX = 512 bytes
MESSAGE_CONTENT_MAX = 256 KiB
AUDIT_METADATA_MAX = 8 KiB
STORAGE_QUEUE_CAPACITY = 128
ORDINARY_OPERATION_DEADLINE = 10 seconds
BUSY_TIMEOUT = 5 seconds
MIGRATION_DEADLINE = 120 seconds
EXPORT_DEADLINE = 120 seconds
DATABASE_WARNING_THRESHOLD = 2 GiB
DATABASE_HARD_LIMIT = 4 GiB
SINGLE_EXPORT_HARD_LIMIT = 4 GiB
VERIFIED_BACKUP_LIMIT = 3

foreign_keys = ON
journal_mode = WAL
synchronous = FULL
secure_delete = ON
trusted_schema = OFF
temp_store = MEMORY
busy_timeout = 5 seconds
shutdown_checkpoint = TRUNCATE
backup_method = SQLite backup API

EXPORT_TRIGGER = EXPLICIT_USER_ACTION
FORMAT = DETERMINISTIC_VERSIONED_JSON
ENCRYPTION = PLAINTEXT_WITH_DISCLOSURE
AUTOMATIC_UPLOAD = FORBIDDEN
REMOTE_SYNC = FORBIDDEN_IN_PHASE_1B
IMPORT = OUT_OF_SCOPE

PHASE_1B_PRIMARY_TARGETS = macOS / Windows / Linux
ANDROID = SEPARATELY_AUTHORIZED_VALIDATION_GATE
IOS = UNSUPPORTED / UNCLAIMED

PHASE_1B_PLAN = APPROVED
PHASE_1B_IMPLEMENTATION = NO_GO
PHASE_1B_1 = NOT_AUTHORIZED_BY_THIS_TASK
```

- HD-01 accepts the polyglot architecture: SQLite is authoritative for local
  transactional state; semantic and graph stores are replaceable rebuildable
  projections; large bytes belong in a filesystem or object store; remote sync
  is optional and non-authoritative.
- HD-02 selects `rusqlite` with `bundled`, `limits`, and `backup`, owned by a
  Rust dedicated blocking actor; frontend SQL authority is forbidden.
- HD-03 accepts standard plaintext SQLite for the foundation with mandatory
  full-disk encryption on supported production devices, no secrets in the
  database, and a separate pre-production SQLCipher decision.
- HD-04 retains conversations, messages, and inert tasks until explicit user
  deletion; retains audit events while related runtime state exists; forbids
  LLM-controlled deletion; and limits verified local backups to three.
- HD-05 accepts 512-byte titles, 262,144-byte messages, 8,192-byte audit
  metadata, queue 128, 10-second ordinary operations, five-second busy timeout,
  120-second migration/export deadlines, 2 GiB warning, 4 GiB database/export
  hard limits, and three verified local backups.
- HD-06 accepts the exact verified pragma, shutdown checkpoint, SQLite backup
  API, no-live-copy, and fail-platform-on-incompatibility policies.
- HD-07 requires explicit user-triggered deterministic versioned plaintext JSON
  export, restrictive permissions, bounded temporary files, cleanup, no
  secrets/paths, no automatic upload/remote sync, and no import in Phase 1B.
- HD-08 limits the shared gate to desktop macOS, Windows, and Linux. Android
  needs separate authorization and validation; iOS is unsupported/unclaimed.
- HD-09 sets `PHASE_1B_PLAN = APPROVED`,
  `PHASE_1B_IMPLEMENTATION = NO_GO`, and
  `PHASE_1B_1 = NOT_AUTHORIZED_BY_THIS_TASK`; all five slices remain separately
  authorized.

No additional ADR is required for this recording because ADR 0002 already
accepts the SQLite/five-table foundation. Concrete future storage engines,
SQLCipher, remote projection, or any material authority change still require a
future ADR.

## Validation results

Validation of the final documentation diff:

- starting exact-head/clean-review readback: `PASS` — PR #25 was open, draft,
  clean/mergeable, unmerged at `15abe536a98c824c8a19bb5858ed544af2f8e4b2`,
  with zero unresolved substantive threads and no P1/P2 finding
- cumulative PR changed-path allowlist: `PASS` — exactly five authorized
  documentation paths
- schema-correction commit scope: `PASS` — only the Phase 1B plan and planning
  completion report changed
- ADR-path correction scope: `PASS` — only the planning completion report changed
- polyglot-refinement scope: `PASS` — only the Phase 1B plan, master roadmap,
  and planning completion report changed
- human-decision-finalization scope: `PASS` — exactly the same five allowlisted
  documentation paths changed
- `git diff --check`: `PASS`
- Markdown link/path validation: `PASS` — all six external references resolved;
  no unresolved local Markdown target was introduced
- repository secret scan: `PASS`
- false-implementation-claim review: `PASS` — durable runtime state remains
  classified `MISSING`, and all implementation status is `NO_GO`
- canonical-doc consistency review: `PASS` — roadmap, capability matrix,
  security gates, plan, and completion report agree
- canonical human-decision consistency: `PASS` — HD-01 through HD-09 values
  match between the plan and completion report; roadmap, capability, and
  security summaries preserve the same authority and authorization boundaries
- storage-invariant review: `PASS` — SQLite owns transactional truth only;
  semantic/graph stores remain rebuildable, artifact bytes remain external by
  default, remote sync remains optional/non-authoritative, and no engine beyond
  SQLite/`rusqlite` was selected
- phase/slice authorization review: `PASS` — six-level memory remains outside
  Phase 1B, implementation is `NO_GO`, and 1B.1 is
  `NOT_AUTHORIZED_BY_THIS_TASK`
- exact-five-table review: `PASS` — the proposed schema has five application
  tables, uses no `AUTOINCREMENT` or custom sequence table, and the test contract
  explicitly rejects `sqlite_sequence` and any unexpected table
- lockfile-path verification: `PASS` — `src-tauri/Cargo.lock` exists and
  repository-root `Cargo.lock` does not
- ADR evidence-path verification: `PASS` — all three exact files exist on the
  PR base and current branch; no duplicate or invented Phase 1B ADR path remains
- polyglot-storage consistency: `PASS` — SQLite alone is authoritative;
  semantic/graph stores are rebuildable, artifact metadata remains in SQLite,
  remote systems are projections only, and no future engine is selected
- clean-worktree verification: required immediately after the final commit and
  push; the current pre-commit diff must contain only allowlisted documentation
  files, while the cumulative PR remains limited to the same five paths
- Rust/frontend builds: `NOT RUN / NOT REQUIRED FOR DOCS-ONLY PLANNING`
- production writes/deployments: `0`

## Final planning classification

```text
PHASE_1A =
MERGED / FRESH-MAIN VERIFIED / PASS

PHASE_1B_PLAN =
APPROVED / HUMAN_DECISIONS_RECORDED

PHASE_1B_IMPLEMENTATION =
NO_GO

PLANNING_ARTIFACT_GATE =
CONDITIONAL_PASS PENDING FRESH EXACT-HEAD REVIEW

FRESH_CODEX_REVIEW =
PENDING AFTER HUMAN DECISION FINALIZATION

POLYGLOT_STORAGE =
ACCEPTED

SQLITE_ROLE =
AUTHORITATIVE_LOCAL_TRANSACTIONAL_STATE

FUTURE_STORES =
DOCUMENTED_ONLY

PHASE_1B_1 =
NOT_AUTHORIZED_BY_THIS_TASK

PRODUCTION_WRITES =
0
```
