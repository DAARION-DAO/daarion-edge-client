# Phase 1B.1 — Storage Runtime Vertical Slice Completion

Status: **IMPLEMENTED / REPOSITORY VERIFIED / SECURITY REVIEWED / DRAFT PR GATE**

Starting `main`: `eb0d7def94675e5668f8a061ecc9e74b493c48c3`

Branch: `phase-01b1/storage-runtime-vertical-slice`

Date: 2026-07-17

This completion report covers only the separately authorized Phase 1B.1
vertical slice. It does not authorize or claim Phase 1B.2, public content CRUD,
full memory, production readiness, deployment, or live-user-profile execution.

## 1. Existing behavior before task

- Canonical `main` contained the merged Phase 1A local-only inference
  foundation and the merged Phase 1B planning/toolchain documents.
- The exact Rust 1.95.0 toolchain was pinned, but no `rusqlite` dependency,
  local SQLite schema, runtime-store owner, migration runner, storage status
  command, typed storage client, or Dashboard storage projection existed.
- Conversation, message, task, and audit data had no durable local service.
- The existing Dashboard, Inference, Messaging, Activation, Genesis, and
  pairing flows were already present and remained the composition target.

Capability classification before this task:

```text
DURABLE_RUNTIME_STATE = MISSING
STORAGE_BOOTSTRAP = MISSING
STORAGE_DASHBOARD_PROJECTION = MISSING
```

## 2. New behavior after task

The candidate implements this bounded chain:

```text
app.path().app_local_data_dir()
-> private runtime-state path policy
-> one Rust-owned SQLite connection
-> embedded migration 1
-> bounded status actor
-> get_storage_runtime_status()
-> typed TypeScript client
-> existing Edge Dashboard Storage Runtime card
```

The Rust runtime creates or reopens the empty version-1 SQLite schema, verifies
its migration evidence and structural fingerprint, applies and reads back the
required pragmas, runs integrity checks, and exposes only a safe read-only
projection. The Dashboard renders the required status, initialization, schema,
SQLite version, database health/size, and persistence fields without receiving
a path, SQL, migration content, raw error, or user data.

Capability classification after repository verification:

```text
DURABLE_RUNTIME_STATE = PARTIALLY_IMPLEMENTED
STORAGE_BOOTSTRAP = IMPLEMENTED_AND_VERIFIED_IN_REPOSITORY
STORAGE_DASHBOARD_PROJECTION = IMPLEMENTED_AND_VERIFIED_IN_REPOSITORY
REAL_DESKTOP_RESTART_FLOW = NOT_VERIFIED
PHASE_1B_2 = NOT_AUTHORIZED
```

Conversation, message, task, and audit tables remain empty schema only. There
is no public repository/service CRUD for them.

## 3. Regression test for old behavior

- Phase 1A inference Rust tests: **67 passed / 0 failed**.
- Full Rust suite: **151 passed / 0 failed** after adding 35 storage tests.
- Existing inference TypeScript/Rust contract: **PASS**.
- TypeScript validation and production Vite build: **PASS**.
- Existing Dashboard composition, Inference, Messaging, Activation, Genesis,
  pairing, and worker modules compile under the full Rust/frontend gates.
- No inference, pairing, reset, worker, wallet, transport, Supabase, deployment,
  CI, or Tauri capability file changed.
- `package-lock.json` remains unchanged.

## 4. New feature test

The focused storage suite contains **35 passed / 0 failed** tests covering:

- fresh bootstrap and truthful `created_new` projection;
- exact five-table and fourteen-index inventory with no `sqlite_sequence`;
- required constraints, foreign keys, UUID bounds, task/audit allowlists;
- immutable migration ID/name/checksum/timestamp and exact-once replay;
- clean restart with the same checksum and `applied_at_ms`;
- checksum/name mismatch, newer schema, structural tamper, and interrupted
  transaction refusal;
- `foreign_keys=ON`, WAL, `synchronous=FULL`, `secure_delete=ON`,
  `trusted_schema=OFF`, `temp_store=MEMORY`, and bounded busy timeout;
- `quick_check`, `foreign_key_check`, and empty startup content tables;
- one named worker, one connection, capacity-128 queue, concurrent initialize,
  lock deadline, status deadline, and post-shutdown refusal;
- clean checkpoint/close/reopen and propagation of a busy checkpoint failure;
- traversal, symlink, non-regular file, database replacement, and runtime
  directory replacement with a same-inode hard-link refusal;
- Unix private modes, corruption preservation, size hard limit, read-only
  status behavior, post-start resource-limit re-evaluation, and redacted public
  errors.

The deterministic frontend contract validator confirms the command name,
Rust/TypeScript fields, all required UI states, typed-client-only invocation,
Dashboard mount, local-only copy, no prohibited projection fields, and absence
of public content CRUD/generic SQL commands.

## 5. Live smoke checklist

```text
DESKTOP_SMOKE = BLOCKED_BY_SAFE_PROFILE_REQUIREMENT
```

The existing application gates the Dashboard behind pairing/onboarding state.
Reaching that Dashboard in a fresh disposable desktop profile would require
creating broader pairing/enrollment state outside this storage-only slice; the
real user profile was explicitly prohibited. Therefore no Tauri desktop smoke
was run or claimed.

- Launch isolated desktop profile: **NOT RUN — blocked as above**.
- Observe first-open `Healthy`, schema 1, and “New local store initialized”:
  **NOT CLAIMED**.
- Exit/relaunch same desktop profile and observe “Existing local store
  reopened”: **NOT CLAIMED**.
- Navigate Inference, Messaging, and Activation in that desktop run:
  **NOT CLAIMED**.
- Confirm no path/raw error in a real desktop window: **NOT CLAIMED**.

The equivalent storage lifecycle is verified at Rust integration level with a
generated disposable root. That evidence does not replace the missing real
desktop smoke.

## Implementation evidence

### Changed files

Application and contract changes are limited to these exact paths:

- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/migrations/runtime_state/0001_runtime_state_initial.sql`
- `src-tauri/src/runtime_store/commands.rs`
- `src-tauri/src/runtime_store/config.rs`
- `src-tauri/src/runtime_store/connection.rs`
- `src-tauri/src/runtime_store/error.rs`
- `src-tauri/src/runtime_store/migrations.rs`
- `src-tauri/src/runtime_store/mod.rs`
- `src-tauri/src/runtime_store/path_policy.rs`
- `src-tauri/src/runtime_store/tests.rs`
- `src-tauri/src/runtime_store/types.rs`
- `src-tauri/src/runtime_store/worker.rs`
- `src-tauri/src/lib.rs`
- `src/lib/storageRuntimeClient.ts`
- `src/components/StorageRuntimeCard.tsx`
- `src/App.tsx`
- `scripts/validate-storage-runtime-contract.mjs`
- `package.json`

Directly affected documentation is limited to:

- `README.md`
- `docs/adr/0002-local-runtime-state-and-sqlite-foundation.md`
- `docs/architecture/CAPABILITY_STATUS_MATRIX.md`
- `docs/planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md`
- `docs/planning/phases/phase-01b-durable-runtime-state-plan.md`
- `docs/planning/phases/phase-01b1-storage-runtime-vertical-slice-plan.md`
- this completion report
- `docs/security/SECURITY_GATES.md`

No capability, CI, deployment, reset, inference, pairing, worker, wallet,
Supabase, or transport file changed.

### Dependency and lockfile resolution

Exactly one direct application dependency was added:

```toml
rusqlite = { version = "=0.40.1", default-features = false, features = ["bundled", "limits", "backup"] }
```

New lockfile packages:

| Package | Version | License |
| --- | --- | --- |
| `rusqlite` | 0.40.1 | MIT |
| `libsqlite3-sys` | 0.38.1 | MIT |
| `fallible-iterator` | 0.3.0 | MIT/Apache-2.0 |
| `fallible-streaming-iterator` | 0.1.9 | MIT/Apache-2.0 |
| `vcpkg` | 0.2.15 | MIT/Apache-2.0 |

The bundled source resolves SQLite **3.53.2**. `cargo tree` shows one native
SQLite chain only:

```text
libsqlite3-sys 0.38.1
└── rusqlite 0.40.1
    └── daarion-edge-client
```

No SQLx, tokio-rusqlite, Tauri SQL plugin, SQLCipher, migration framework,
channel crate, frontend package, or test framework was added.

`cargo audit` reports **0 advisories in the added SQLite chain**. The complete
repository lockfile still reports the same inherited baseline on both starting
main and this candidate: 13 vulnerability entries across `aws-lc-sys` 0.37.1,
`quick-xml` 0.38.4, `quinn-proto` 0.11.13, and `rustls-webpki` 0.103.9, plus 17
unmaintained and 5 unsound informational warnings. This slice neither adds nor
remediates those unrelated dependencies; a separate dependency-remediation
task remains required.

### Migration and schema

Migration identity:

```text
migration_id = 1
name = runtime_state_initial
checksum_sha256 = 843fa8f5d27d691359a4f3e167c4a454d3ae28e021563a429951c2430afcc4d6
schema_fingerprint = 0252c83b7989e552ac183a6a886b8eac9148de7a6d406709d6e366a41a05a5a0
```

Exact application table inventory:

1. `schema_migrations`
2. `conversations`
3. `messages`
4. `tasks`
5. `audit_events`

There is no `AUTOINCREMENT`, `sqlite_sequence`, custom sequence, sixth table,
seeded content row, or startup audit event. The only startup row is migration 1
inside `schema_migrations`.

Exact index inventory:

- `audit_events_created_idx`
- `audit_events_subject_idx`
- `audit_events_type_created_idx`
- `conversations_updated_idx`
- `messages_conversation_sequence_idx`
- `tasks_conversation_idx`
- `tasks_state_updated_idx`
- `sqlite_autoindex_audit_events_1`
- `sqlite_autoindex_conversations_1`
- `sqlite_autoindex_messages_1`
- `sqlite_autoindex_messages_2`
- `sqlite_autoindex_schema_migrations_1`
- `sqlite_autoindex_tasks_1`
- `sqlite_autoindex_tasks_2`

Migration SQL and history insertion execute in one `BEGIN IMMEDIATE`
transaction. Exact name/checksum/history, newer versions, structural schema,
`quick_check`, and `foreign_key_check` are all fail-closed gates. Invalid or
corrupt stores are preserved; they are not renamed, deleted, recreated, or
downgraded.

### Actor and lifecycle

- `RuntimeStoreManager` owns a bounded `std::sync::mpsc::sync_channel(128)`.
- Exactly one thread named `daarion-runtime-store` owns the connection.
- Production requests are limited to initialize, read status, and shutdown.
- Tauri setup only submits initialization; SQLite work runs on the blocking
  owner thread.
- The public async command uses `spawn_blocking` for status access.
- Read status revalidates trusted directory/file identities and storage size;
  it does not reopen, migrate, retry failed initialization, or write content.
- Shutdown stops new intake, drains FIFO accepted work, executes and verifies
  `wal_checkpoint(TRUNCATE)`, closes the connection, and joins within the
  bounded lifecycle. A busy checkpoint is returned as a controlled failure.
- Drop uses a bounded two-second shutdown attempt and never blocks process exit
  indefinitely.

### Trusted path policy

- Production accepts only Tauri `app.path().app_local_data_dir()`.
- No frontend or command argument can provide a database path.
- Layout is confined to `runtime-state/runtime-state-v1.sqlite3`, with reserved
  `tmp`, `backups`, and `exports` directories but no operation using them.
- Absolute normal paths, every existing root component, runtime directory,
  database target, and sidecars are checked for traversal, symlinks, and file
  type.
- App-root, runtime-root, and database identities are captured and revalidated
  after canonicalization/open and on status/shutdown operations. A replacement
  runtime directory containing a hard-link to the original database fails
  closed.
- SQLite opens with `SQLITE_OPEN_NOFOLLOW`, read-write/create, full mutex, and
  extended result codes.
- Unix directories are forced to `0700`; DB/WAL/SHM files are forced to `0600`.
- Windows does not inherit a Unix-mode claim; non-Unix metadata identity checks
  are repository-compiled but real Windows ACL/replacement behavior is not
  claimed without a Windows target run.
- Automated tests use generated temporary roots only.

### Tauri and TypeScript status contract

Exactly one no-argument command is registered:

```text
get_storage_runtime_status()
```

The Rust/TypeScript DTO contains only:

```text
state
initialized
schema_version
database_health
database_size_bytes
storage_backend = sqlite
sqlite_version
persistence_state
last_start_time_ms
error_code
```

States:

```text
initializing
healthy
warning
unavailable
migration_failed
integrity_failed
locked
permission_denied
resource_limited
```

Controlled error codes:

```text
path_invalid
permission_denied
locked
busy_timeout
migration_mismatch
newer_schema
migration_failed
integrity_failed
resource_limit
internal
```

The DTO has no database path/filename, SQL, pragma payload, migration content,
user content, environment value, stack trace, or raw SQLite error.

### Dashboard behavior

`StorageRuntimeCard` is mounted in the existing Dashboard. It visibly renders:

- Status
- Initialized
- Schema Version
- SQLite Version
- Database Health
- Database Size
- Persistence

It handles every required state and maps allowlisted error codes to controlled
copy. Persistence copy is exact:

```text
created_new -> New local store initialized
reopened_existing -> Existing local store reopened
unknown -> Unavailable
```

Refresh performs one read-only status request. There is no poll loop, cache,
local persistence, raw `invoke`, retry/reopen/migration behavior, or generic SQL
authority in the component.

## Verification evidence

All Rust commands were executed with the exact Rust 1.95.0 toolchain directory
first in `PATH`. The host also has an unrelated Homebrew Rust 1.94.1 binary;
unqualified child-process resolution was therefore not accepted as evidence.

| Gate | Result |
| --- | --- |
| Active rustup toolchain | `1.95.0-aarch64-apple-darwin` |
| `rustc --version --verbose` through exact toolchain | `1.95.0` |
| `cargo --version --verbose` through exact toolchain | `1.95.0` |
| Focused `runtime_store` tests | 35 passed / 0 failed |
| Phase 1A `inference::` tests | 67 passed / 0 failed |
| Full Rust tests | 151 passed / 0 failed |
| `cargo check --all-targets --locked` | PASS |
| `cargo clippy --all-targets --locked` | PASS; no `runtime_store` warning |
| Storage frontend/Rust contract | PASS |
| Inference frontend/Rust contract | PASS |
| TypeScript + production Vite build | PASS; 1,763 modules |
| `npm audit --omit=dev` | PASS; 0 vulnerabilities |
| Secret scan | PASS |
| Touched Rust warning gate | PASS |
| Scoped rustfmt | PASS |
| `git diff --check` | PASS |
| Package lock unchanged | PASS |
| Added SQLite-chain advisories | PASS; 0 |
| Complete RustSec baseline | FAIL; 13 inherited entries, unchanged from starting main |
| Repository-wide rustfmt | 94 legacy files; no candidate runtime/composition file in debt |
| Real desktop smoke | `BLOCKED_BY_SAFE_PROFILE_REQUIREMENT` |

Existing repository warning debt remains visible: regular Rust compilation
reports 312 legacy warnings and clippy reports 325/327 warnings. The scoped
warning gate reports no warning in the protected/new hardening modules. This
task does not perform an unrelated repository cleanup.

## Security review

### Critical

- **0 scoped open findings.**

### High

- **0 scoped open findings.**
- The complete repository RustSec baseline contains 13 inherited advisory
  entries, including high-severity advisories, identical on starting main and
  the candidate. The added SQLite chain has zero advisories. Remediation is
  outside this bounded dependency contract and remains a separate repository
  gate.

### Medium

- **Resolved:** shutdown originally executed `wal_checkpoint(TRUNCATE)` without
  validating its returned busy result. The candidate now checks the result,
  propagates failure, joins the worker, and has a deterministic busy-checkpoint
  test.
- **Resolved:** DB-inode checking alone could miss replacement of the parent
  runtime directory with a same-inode hard-link. The candidate now tracks and
  revalidates app/runtime directory identities and has a negative replacement
  test.
- **Resolved:** status refresh updated the byte count without re-evaluating the
  configured warning/hard limits. The connection now owns those limits and
  every status read fails closed at the hard threshold; post-start growth is
  covered by a deterministic test.
- **0 scoped open material findings** in migration integrity, corruption,
  path handling, actor bounds, shutdown, projection redaction, or UI authority.

### Low / informational

- Standard SQLite is plaintext by the accepted ADR. SQLCipher/key lifecycle is
  a separate pre-production decision.
- macOS Apple Silicon repository execution does not prove Windows, Linux,
  macOS-x64, Android, or iOS runtime behavior.
- Real desktop restart UI proof remains blocked and is not claimed.
- Repository-wide Rust warnings, rustfmt debt, and RustSec baseline remain
  separate inherited work.

Security checks found no raw SQL outside `runtime_store`/its embedded migration,
no generic frontend database authority, no new shell authority, no Tauri
capability change, no raw error/path disclosure, no automatic downgrade or
corruption recreation, no unbounded queue, and no Phase 1B.2 behavior.

## Protected old worktree integrity

The evidence-only worktree remains dirty and untouched:

```text
branch = phase-01b1/storage-bootstrap-migrations
HEAD = 0e6ff6ada0dd967b6543f3a534f756787c916c42
tracked_diff_sha256 = 8e069857177bf7174981d48e3b053d1ab8f0e7020b88ac938e3fec738e41fbea
```

Recorded file fingerprints remain:

```text
c54b5a430d701de3d4a232f5d03a48c260da89a2bf81a2744b88903a1b99fe03  phase-01b1-storage-bootstrap-plan.md
ac34f1b8f44eae75caf78fea6fa7a7ecdaf04c7090aada8b2af7e1e519a9f2cc  actor.rs
991959739229787f87ef4130893eed4f6b38722cc216df45b27a91a5d3f69589  connection.rs
46e2381033670d3b0444c831fdf169ff929a907e14760eecafe76510f3b44dea  error.rs
76b553befd5b0ce9012c188a56ed3ad147f73ebf9385ae41de455c585f3baab1  migrations.rs
2fc90882f66d9cf5d66181ff5e23bcf506dc7ea063e2e0d5aa95ab03bcf61667  old migration SQL
3c452f62508cf3311787a342efe35136fb3ddc54e028d9f0ef8df708b067381a  mod.rs
5f3cc6cc9fd694fa4ef823d305c68159da0bd2e9ccfd2ad76840a9acd75afc75  path_policy.rs
```

No old commit was cherry-picked and no old worktree file was modified.

## Rollback

Before merge, revert the bounded candidate commit or delete the candidate
branch. Preserve this report and test evidence. Do not modify the old evidence
worktree.

After any future merge/open, disable the Dashboard consumer and stop new
storage consumers if rollback is required, but preserve every valid database
and migration history. Never downgrade, silently delete, rename, recreate, or
raw-copy the database. Correct schema defects only through a separately
authorized forward migration.

## Operations and release status

```text
REMOTE_PRODUCTION_WRITES = 0
REAL_USER_PROFILE_WRITES = 0
DEPLOYMENTS = 0
OLLAMA_DOWNLOADS_OR_REAL_PROMPTS = 0
LOCAL_TEST_WRITES = GENERATED TEMP ROOTS / CARGO TARGET / NODE BUILD ONLY
```

Local test databases were created only under generated temporary directories
and removed by the test fixtures. No real DAARION app-local-data path was used.

Overall local release classification:

```text
SCOPED_PHASE_1B_1_GATE = PASS
REPOSITORY_BASELINE_GATE = CONDITIONAL_PASS
EXTERNAL_EXACT_HEAD_REVIEW = PENDING
PR_READY_OR_MERGE = NOT_AUTHORIZED
PHASE_1B_2 = NOT_AUTHORIZED
```

The conditional repository classification reflects the unchanged RustSec,
warning/formatting, cross-platform, and desktop-smoke baselines. It is not a
scoped implementation failure and does not convert them into verified claims.

## Final result

```text
PHASE_1B_1 =
IMPLEMENTED / TESTED / SECURITY-REVIEWED /
DRAFT_PR_GATE / EXTERNAL_REVIEW_PENDING

PRODUCT_SLICE =
VISIBLE IN REPOSITORY / REAL DESKTOP SMOKE NOT CLAIMED

STORAGE_RUNTIME =
RUST -> TAURI -> TYPESCRIPT -> REACT COMPLETE

SCHEMA =
EXACT FIVE TABLES / VERSION 1

PUBLIC_STORAGE_API =
READ_ONLY STATUS ONLY

PHASE_1B_2 =
NOT_AUTHORIZED

PROTECTED_OLD_WORKTREE =
UNCHANGED / EVIDENCE ONLY

REMOTE_PRODUCTION_WRITES =
0

REAL_USER_PROFILE_WRITES =
0

DEPLOYMENTS =
0
```
