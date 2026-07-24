# Phase 1B.1 — Storage Runtime Vertical Slice Plan

Status: **GO / VISIBLE PRODUCT SLICE / PHASE 1B.2 NOT AUTHORIZED**

Starting `main`: `eb0d7def94675e5668f8a061ecc9e74b493c48c3`

Branch: `phase-01b1/storage-runtime-vertical-slice`

This plan amends only the presentation boundary of the approved Phase 1B.1
bootstrap. The SQLite foundation remains private and empty, but its safe
read-only readiness projection is made visible through the existing Tauri,
TypeScript, and Dashboard architecture.

## 1. Objective

Deliver one bounded end-to-end slice:

```text
SQLite bootstrap
-> Rust-owned storage service
-> read-only Tauri status command
-> typed TypeScript client
-> existing Edge Dashboard storage card
-> deterministic restart/reopen verification
```

The slice proves that the local runtime can create and reopen the approved
version-1 store without exposing SQL, paths, content, or mutation authority to
the frontend.

## 2. Current behavior

- `origin/main` is exactly `eb0d7def94675e5668f8a061ecc9e74b493c48c3`.
- PR #26 is merged and pins Rust 1.95.0; `src-tauri/Cargo.toml` declares
  `rust-version = "1.95"`.
- The repository has no `rusqlite`, SQLite schema, runtime-store module,
  migration runner, storage command, TypeScript storage client, or Dashboard
  storage card.
- `src-tauri/src/lib.rs` is the Tauri composition root and already owns managed
  runtime services and purpose-specific commands.
- `src/lib/inferenceClient.ts` and `LocalInferencePanel` demonstrate the current
  typed client -> Tauri command -> Rust service -> controlled UI-state pattern.
- `src/App.tsx` has one existing Dashboard plus Activation, Inference, and
  Messaging navigation. No second application or navigation layer is needed.
- Messaging and chat history are transient/mock and remain unchanged.
- The merged Phase 1B plan and ADR 0002 authorize exactly five empty runtime
  tables and a Rust-owned actor, but no Phase 1B.2 CRUD.

Capability classification before implementation:

```text
DURABLE_RUNTIME_STATE = MISSING
STORAGE_BOOTSTRAP = MISSING
STORAGE_DASHBOARD_PROJECTION = MISSING
```

## 3. New behavior

- On Tauri setup, an internal runtime-store manager receives only the trusted
  `app.path().app_local_data_dir()` root and starts a named blocking worker.
- The worker creates or safely reopens `runtime-state-v1.sqlite3`, verifies the
  required pragmas, applies/verifies migration 1 transactionally, performs
  integrity checks, and publishes a redacted status snapshot.
- Storage failure remains visible as a controlled status and does not recreate
  or delete the database. The rest of the UI can continue because no state
  consumer exists yet.
- `get_storage_runtime_status` accepts no arguments and returns only the
  allowlisted status DTO.
- The typed TypeScript client is the only frontend invocation path.
- `StorageRuntimeCard` is mounted inside the existing Dashboard and supports a
  read-only refresh that never reopens or reinitializes storage.
- Tests create only generated disposable roots and prove first-open versus
  reopen truth across clean shutdown.

## 4. Scope

- Add exactly `rusqlite = 0.40.1` with `default-features = false` and only
  `bundled`, `limits`, and `backup`.
- Add one immutable embedded migration with exactly the approved five tables.
- Add the private `runtime_store` module with configuration, typed errors and
  public projection types, trusted path policy, connection policy, migrations,
  bounded worker/actor, and one read-only command.
- Add composition-only Tauri managed-state and command registration.
- Add one typed TypeScript client, one Dashboard card, one deterministic
  no-dependency contract validator, and one package script.
- Add deterministic Rust tests and documentation/evidence updates directly
  affected by this slice.

## 5. Explicit non-goals

- No conversation, message, task, or audit-event CRUD or public methods.
- No chat-history persistence or UI integration.
- No semantic, episodic, procedural, graph, vector, embedding, or external
  database implementation.
- No export, import, backup, restore, deletion, reset, or recovery UI/API.
- No Supabase, cloud synchronization, Reticulum/LXMF, Agent Supervisor, Loop
  Runtime, tools, worker persistence, wallet, identity, pairing, inference, or
  deployment changes.
- No generic SQL/query/path/pragma/migration command.
- No new frontend dependency or test framework.
- No Phase 1B.2 behavior.

## 6. Repository ownership

`daarion-edge-client` exclusively owns the local SQLite path, connection,
schema, migrations, lifecycle, and status projection. The browser receives a
minimal read-only projection and no database authority. `loval-echoes`,
Supabase, remote agents, and future transport components receive no raw local
state or storage authority in this slice.

## 7. Expected files

Application and contract paths:

- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/migrations/runtime_state/0001_runtime_state_initial.sql`
- `src-tauri/src/runtime_store/**`
- composition-only `src-tauri/src/lib.rs`
- `src/lib/storageRuntimeClient.ts`
- `src/components/StorageRuntimeCard.tsx`
- bounded `src/App.tsx` import/mount
- `scripts/validate-storage-runtime-contract.mjs`
- package script in `package.json`; `package-lock.json` must remain unchanged

Documentation paths:

- this plan and matching completion report
- `docs/planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md`
- `docs/architecture/CAPABILITY_STATUS_MATRIX.md`
- `docs/security/SECURITY_GATES.md`
- ADR 0002 and README only if implementation evidence/copy requires it

Any unrelated capability, deployment, CI, Tauri capability, reset, inference,
pairing, worker, wallet, transport, or Supabase change is a stop condition.

## 8. Rust storage architecture

- `RuntimeStoreManager` owns a bounded `std::sync::mpsc::SyncSender` with
  capacity 128 and one named blocking thread.
- Only the worker owns `rusqlite::Connection`; no pool, global connection,
  frontend handle, or SQLite type escapes `runtime_store`.
- Typed internal requests are limited to initialize, read status, and shutdown.
- Initialization is sent internally after Tauri resolves the trusted root.
  Tauri setup does not execute SQLite work on the UI/async thread.
- A redacted shared snapshot starts at `initializing`, moves once to the truthful
  initialized/failure projection, and allows status reads even after startup
  failure. Refresh only reads it; it cannot retry initialization.
- Clean shutdown stops intake, drains accepted reads, executes
  `wal_checkpoint(TRUNCATE)`, closes the connection, and joins within the
  service-owned deadline where possible.
- Raw SQL is static and confined to the migration/connection implementation.

## 9. Tauri contract

Register exactly one new purpose-specific command:

```text
get_storage_runtime_status()
```

It accepts no arguments and returns `StorageRuntimeStatus` with allowlisted
state, initialization, schema version, database health/size, backend, SQLite
version, persistence state, start time, and controlled error code. It never
returns paths, filenames, SQL, pragma text, migration SQL/name/checksum, user
content, raw SQLite errors, environment data, or platform identity.

## 10. TypeScript contract

`src/lib/storageRuntimeClient.ts` defines the exact serialized DTO, the single
command constant, a controlled adapter error, and only
`getStorageRuntimeStatus()`. It contains no `any`, path argument, cache,
polling, logging, mutation, or generic invoke surface. No other frontend file
may call the storage command through raw `invoke`.

## 11. Dashboard behavior

`StorageRuntimeCard` is mounted only in the existing Dashboard. It displays the
required state, initialized flag, schema/SQLite versions, database health/size,
and truthful persistence copy. Missing values render as unavailable. Error
copy is mapped from allowlisted codes. A Refresh button only rereads status.
The card displays no local path, filename, SQL, migration metadata, content, or
raw internal/platform error.

## 12. Path and local-data security

- Production root comes only from `app.path().app_local_data_dir()`.
- Database layout is `<app-local-data>/runtime-state/runtime-state-v1.sqlite3`
  with reserved `tmp`, `backups`, and `exports` directories.
- Reject parent traversal, symlinked components/targets, non-regular targets,
  root replacement, and detectable post-open replacement.
- Open with `SQLITE_OPEN_NOFOLLOW`, read-write/create, extended-result codes,
  and serialized/full-mutex safety.
- Canonicalize and verify containment under the trusted root; revalidate file
  identity and metadata after open.
- Unix directories use 0700 and database/WAL/SHM files use 0600. Windows claims
  are limited to truthful writable/current-user LocalAppData behavior until
  real ACL evidence exists.
- Tests use generated temporary roots only and never the real DAARION app-data
  directory.
- Standard SQLite is plaintext; no secret-bearing data is inserted or seeded.

Pre-implementation threats include traversal/symlink replacement, TOCTOU,
database replacement, permission widening, raw error/path disclosure,
migration tampering/newer-schema downgrade, silent corruption recovery,
unbounded queue, blocking UI work, shutdown races, and false persistence copy.
Each receives negative tests or an explicit platform limitation.

## 13. Migration and compatibility

- Migration 1 is an immutable embedded SQL resource identified by sequential ID
  1, name `runtime_state_initial`, and SHA-256 of the exact SQL bytes.
- A `BEGIN IMMEDIATE` transaction creates the schema and inserts its manifest
  row atomically. Applied history is revalidated before and after migration.
- Missing/gapped/duplicate/renamed/checksum-mismatched/unknown-newer history
  fails closed without delete, rename, overwrite, recreate, or downgrade.
- The schema contains exactly `schema_migrations`, `conversations`, `messages`,
  `tasks`, and `audit_events`; no `AUTOINCREMENT`, `sqlite_sequence`, custom
  sequence, seed rows, startup audit row, or sixth table.
- Columns, checks, foreign keys, and indexes follow the merged Phase 1B plan.
- `rusqlite` exact/transitive resolution, bundled SQLite version, license,
  advisories, and duplicate native library state are recorded after locking.
- Existing databases with unsupported/newer history are preserved and refused.

## 14. Restart/reopen semantics

The deterministic integration test uses one disposable root:

```text
initialize new -> capture migration row/checksum/applied_at_ms
-> clean shutdown -> reopen same root -> verify reopened_existing
-> verify unchanged schema/history/database identity -> clean shutdown
```

`created_new` is emitted only after a successful first initialization.
`reopened_existing` is emitted only after the pre-existing trusted database is
successfully reopened and validated. Initialization failures remain `unknown`.

## 15. Error projection

Internal errors map to only:

- `path_invalid`
- `permission_denied`
- `locked`
- `busy_timeout`
- `migration_mismatch`
- `newer_schema`
- `migration_failed`
- `integrity_failed`
- `resource_limit`
- `internal`

The public state distinguishes unavailable, migration/integrity failure,
locked, permission denied, and resource limited without raw diagnostic text.
Storage failure does not prevent the rest of the UI from showing the card and
does not trigger destructive recovery.

## 16. Tests

Rust tests use disposable roots and cover the 36 prompt cases, grouped as:

- dependency/toolchain compile and bundled SQLite evidence;
- fresh bootstrap, exact tables/indexes, no `sqlite_sequence`, manifest row,
  exact-once replay, mismatch/newer/interrupted rollback;
- required pragma/limit readback, WAL, foreign keys, quick/foreign-key checks;
- one connection/worker, capacity 128, concurrent start, lock/busy deadline,
  shutdown/reopen, post-shutdown rejection, read-only status;
- corrupt preservation, traversal, symlink, non-regular target, Unix modes,
  redacted projection, created/reopened truth, unchanged migration timestamp and
  database identity;
- static absence of public CRUD and generic frontend SQL.

Regression gate: 67 inference tests, full Rust tests, cargo check/clippy,
inference contract, storage contract, TypeScript production build, production
npm audit, secret scan, touched-warning check, scoped rustfmt, and diff check.
Repository-wide rustfmt is compared against the known 94-file debt and cannot
be called a clean pass unless that independent debt is resolved.

## 17. Live desktop smoke

Attempt a real Tauri launch only with a provably isolated disposable app-data
profile. Verify first-open and second-open card states plus Inference,
Messaging, and Activation navigation. Do not seed or touch the real user
profile. If isolation and the existing onboarding gate cannot be satisfied
without broader state manipulation, record
`DESKTOP_SMOKE = BLOCKED_BY_SAFE_PROFILE_REQUIREMENT`; do not fake the smoke or
raise `REAL_DESKTOP_RESTART_FLOW` to verified.

## 18. Acceptance criteria

- Exact authorized main/branch and bounded diff.
- Exactly one direct dependency with exact approved version/features; no second
  SQLite library or Critical/High advisory.
- Exactly five tables and approved indexes/constraints; migration checksum and
  exact-once/restart evidence.
- One connection on one named worker, queue 128, no SQLite/UI-thread work.
- Required pragmas, integrity checks, fail-closed migration/corruption/path
  behavior, and clean checkpoint/close.
- Exactly one no-argument read-only Tauri command and matching typed client.
- Dashboard card handles every required state without path/SQL/raw-error leak.
- All applicable Rust/frontend/security/build gates pass; required tests are not
  replaced by mocks.
- No Critical/High or material Medium data-integrity/path/shutdown finding.
- Old evidence worktree fingerprints remain unchanged.
- Draft PR only; no ready, merge, Codex review request, deployment, production
  write, or Phase 1B.2 work.

## 19. Rollback

Before merge, revert the candidate commit or delete the candidate branch while
preserving evidence. After any future merge/open, disable consumers and remove
code only while preserving the database and migration history. Never
downgrade, silently delete, recreate, or raw-copy a valid database. Schema
defects require forward migrations. This slice has no user-content CRUD, but a
created valid store is still preserved.

## 20. Documentation updates

Create the matching completion report with existing/new behavior, regressions,
feature evidence, live-smoke checklist, dependency/lock/migration inventory,
test counts, security findings, platform limits, rollback, and production-write
truth. Update roadmap, capability matrix, security gate, and ADR 0002 only to
the implemented-and-verified repository candidate level. Do not mark Phase 1B
complete.

## 21. Old snapshot reuse ledger

The protected worktree remains byte-for-byte unchanged at
`0e6ff6ada0dd967b6543f3a534f756787c916c42`, tracked diff hash
`8e069857177bf7174981d48e3b053d1ab8f0e7020b88ac938e3fec738e41fbea`.

| Path / symbol | Original purpose | Current-plan compatibility | Security review | Decision |
| --- | --- | --- | --- | --- |
| old Phase 1B.1 plan | Backend-only bootstrap authority | Superseded by the visible-slice authorization | Useful evidence only; cannot authorize current code | `REJECT` as current plan |
| `runtime_store/actor.rs` | Single blocking connection actor | Missing visible failure state and explicit initialize/read/shutdown request model; startup blocks caller and Drop joins without a deadline | Shutdown/startup availability risks | `REWRITE` |
| `runtime_store/connection.rs` | Flags, pragmas, limits | Constants and required pragma set match the accepted plan | Recheck NOFOLLOW, limits, readback, and error mapping | `REWRITE` using reviewed invariants only |
| `runtime_store/error.rs` | Redacted internal errors | Does not provide the required public DTO state/error mapping | Preserve redaction principle; expand controlled classifications | `REWRITE` |
| `runtime_store/migrations.rs` | Manifest/history/schema validation | Transaction/checksum concept matches; hardcoded autoindex inventory is incomplete/brittle | Would reject the approved schema incorrectly and needs failure-state mapping | `REWRITE` |
| old migration SQL | Five-table schema | Mostly matches merged plan; UUID checks are inconsistent and checksum must bind current exact bytes | Review every constraint/index; never copy blindly | `REWRITE` |
| `runtime_store/path_policy.rs` | Trusted path and permissions | Core containment/no-follow/identity concepts match | Rework error specificity, lifecycle, root checks, and generated tests | `REWRITE` |

No old commit, branch, or file is cherry-picked or copied as authority. Any
similar structure is independently reimplemented against current main and this
plan.

## 22. Open questions

- Real Windows ACL, Linux packaging, macOS-x64, Android, and iOS execution are
  unavailable on the current macOS Apple Silicon host and cannot inherit a
  local pass.
- Real desktop Dashboard smoke may remain blocked by the requirement to isolate
  app data while satisfying existing pairing/enrollment gates; repository
  restart/reopen integration remains mandatory regardless.
- Pre-production SQLCipher/key lifecycle remains separately gated.

## 23. Decision

```text
PHASE_1B_1_VERTICAL_SLICE_PLAN = GO
PHASE_1B_2 = NOT_AUTHORIZED
```

The current repository can implement this bounded slice without broader
dependency, capability, schema, deployment, identity, or lifecycle authority.
Stop before implementation if that assessment changes.
