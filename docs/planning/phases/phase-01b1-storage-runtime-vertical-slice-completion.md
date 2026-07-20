# Phase 1B.1 — Storage Runtime Vertical Slice Completion

Status: **R4 REVIEW BLOCKED / ASSIGNMENT-ALIAS CORRECTION LOCAL GATE PASS / R5 REVIEW PENDING**

Starting `main`: `eb0d7def94675e5668f8a061ecc9e74b493c48c3`

Independent R1 reviewed head: `ffcc83d031b4506aebc4e9fb68e6db11590cecde`
Independent R2 reviewed head: `d1a2617455a844a61652ced82bff7ad5f78ba95d`
Independent R3 reviewed head: `86ef384a8820989096c5859e7ea481078a55cb97`
Independent R4 reviewed head: `7465673d0128e850d30f1b8f00c7c102d69b983a`

Branch: `phase-01b1/storage-runtime-vertical-slice`

Date: 2026-07-17

R1 correction date: 2026-07-18

R2 correction date: 2026-07-19

Final closeout date: 2026-07-19

R4 assignment-alias correction date: 2026-07-20

This completion report covers only the separately authorized Phase 1B.1
vertical slice. It does not authorize or claim Phase 1B.2, public content CRUD,
full memory, production readiness, deployment, or live-user-profile execution.

## R1/R2/R3/R4 review and closeout ledger

```text
INDEPENDENT_REVIEW_R1 =
REVIEW_BLOCKED_BY_FINDINGS

R1_FINDINGS =
CRITICAL 0 / HIGH 0 / MEDIUM 5 / LOW 3 / INFO 4

INDEPENDENT_REVIEW_R2 =
REVIEW_BLOCKED_BY_FINDINGS

R2_FINDINGS =
CRITICAL 0 / HIGH 0 / MEDIUM 1 / LOW 2 / INFO 4

INDEPENDENT_R3_REVIEW =
R3_REVIEW_PASS_WITH_NONBLOCKING_FINDINGS

R3_FINDINGS =
CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 3 / INFO 4

R2_M_01 = CLOSED
R2_M_03 = CLOSED
R2_L_03 = CLOSED

R3_L_01 =
FOURTH_COMMIT_CORRECTED_NAMESPACE_CALL / R4_FOUND_ASSIGNMENT_ALIAS_FALSE_NEGATIVE

R3_L_02 =
LOCALLY_CORRECTED / R4_RETAINED / RUNTIME_STORE_CLIPPY_MAP_IDENTITY

R3_L_03 =
LOCALLY_CORRECTED / R4_RETAINED / STALE_CANONICAL_EVIDENCE

INDEPENDENT_REVIEW_R4 =
R4_REVIEW_BLOCKED_BY_FINDINGS

R4_BLOCKER =
ASSIGNMENT_ALIAS_FALSE_NEGATIVE

R4_REVIEWED_HEAD =
7465673d0128e850d30f1b8f00c7c102d69b983a

R4_ASSIGNMENT_ALIAS_CORRECTION =
IMPLEMENTED / LOCAL_TESTED / R5_REVIEW_PENDING

R5_REVIEW =
REQUIRED / NOT PERFORMED

FINAL_CLOSEOUT_COMMIT =
7465673d0128e850d30f1b8f00c7c102d69b983a

ASSIGNMENT_ALIAS_CORRECTION_COMMIT =
RECORDED IN PR BODY AND FINAL REPORT AFTER COMMIT
```

The independent local R3 review closed the R2 lifecycle, deadline-boundary and
comment-spoof findings without regressing the earlier R1 corrections. It also
reported three new Low findings. This final closeout is intentionally limited
to those findings: TypeScript AST coverage for named/renamed/namespace/property
invoke paths, removal of one identity `map_err`, and canonical evidence refresh.
R4 reviewed the exact fourth commit and found one residual validator
false-negative: a compiling `rawInvoke = tauriCore.invoke` assignment followed
by `rawInvoke(...)` passed because alias discovery visited declaration
initializers but not assignment expressions. The fifth bounded correction uses
TypeChecker symbol identity, monotonic fixed-point propagation across variable
declarations and `=` assignments, and conservative namespace/property flow.
It changes no runtime or product capability. R5 must review the exact fifth
commit before any separate ready/merge authorization.

| R3 finding | Local closeout | Evidence | Residual boundary |
| --- | --- | --- | --- |
| R3-L-01 | Fourth commit added namespace/property coverage; R4 then found that separate assignment expressions were not propagated | R4 compiling reproduction passed the 42/24 validator at reviewed head `7465673d…` | Fifth correction locally rejects the exact bypass; R5 exact-head review required |
| R3-L-02 | Removed identity `.map_err(|error| error)` without changing the surrounding `Result` type or lifecycle control | 64/64 storage tests; full Clippy PASS; zero `runtime_store` Clippy locations | Retained by R4; runtime source is byte-identical in the fifth correction |
| R3-L-03 | Roadmap, capability matrix, security gates, ADR 0002 and this report preserve R1/R2/R3 history and actual evidence | R4 documentation readback; completion report now also records R4 truth | Phase 1B.1 remains unmerged; desktop/cross-platform evidence remains absent |

The assignment correction constructs an in-memory TypeScript `Program` for
each analyzed source and uses `TypeChecker` symbols rather than identifier text.
Imported invoke/namespace symbols seed finite sets. Declaration initializers,
direct and chained assignments, namespace aliases, static property/element
targets, and namespace destructuring assignments propagate until no set grows.
Sets are monotonic, so a later reassignment cannot erase unsafe reachability and
the loop terminates after at most the finite number of source symbols is added.
Shadowed identifiers have distinct symbols and remain safe when unrelated to a
Tauri import. The fifth correction changes exactly
`scripts/validate-storage-runtime-contract.mjs` and this completion report;
runtime source, product source, schema, manifests, lockfiles and dependencies
remain byte-identical to R4-reviewed head `7465673d…`.

| Finding | State before correction | Affected paths and symbols | Root cause | Bounded correction design | Required evidence | Residual boundary |
| --- | --- | --- | --- | --- | --- | --- |
| M-01 | `OPEN` | `runtime_store/worker.rs::RuntimeStoreManager::shutdown`, `RuntimeStoreManagerInner::drop`; `runtime_store/connection.rs::RuntimeStoreConnection::close`; Tauri composition | Production lifecycle had no explicit shutdown call, `Drop` discarded the inner close result, and `join` was not guarded by worker-exit proof | One public-in-crate idempotent shutdown primitive with an absolute deadline, capacity-one exit notification, checkpoint deadline/interrupt handling, safe failure projection, and a Tauri `ExitRequested`/`Exit` lifecycle adapter | Production-helper, checkpoint, deadline, exit-before-join, missing-exit, idempotency, post-shutdown, redaction, and composition tests | Fresh independent exact-head review remains required; real desktop restart remains unverified |
| M-02 | `OPEN` | `runtime_store/worker.rs::run_worker`, `RuntimeStoreManager::read_status` | Worker termination did not always disable intake and request failures could return the previously cached healthy status | Panic-boundary supervision with unconditional finalization, explicit exit classification, fresh unavailable/internal status on abnormal termination or communication failure, and separate clean-shutdown classification | Panic-after-healthy, disconnect-after-healthy, stale-success denial, exit observation, bounded post-panic shutdown, and clean-exit tests | No public crash command; fresh independent exact-head review remains required |
| M-03 | `OPEN` | `runtime_store/worker.rs::initialize_connection`; `runtime_store/connection.rs`; `runtime_store/migrations.rs` | Initialization elapsed time was checked only after all SQLite work returned | One absolute deadline propagated through path/open/configuration/migration/integrity/post-open phases plus one scoped rusqlite interrupt watchdog that is disarmed and joined | Real SQLite interruption, transaction rollback/history absence, bounded completion, no late healthy status, bounded shutdown, and watchdog cleanup tests | No new public DTO; cross-platform runtime evidence remains limited to executed targets |
| M-04 | `OPEN` | `runtime_store/path_policy.rs::{FileIdentity,existing_regular_file_identity,revalidate_database,enforce_sidecar_permissions}` | Unix identity omitted link count and non-Unix identity used mutable size/timestamps | Separate directory/file identities; Unix `dev`/`ino`/`nlink == 1`; Windows standard-library `MetadataExt` volume/file-index/link-count identity; fail closed elsewhere; validate before permission mutation | Initial/post-open DB hard-link, WAL/SHM hard-link, identity continuity, permission preservation, and Windows-gated source/tests | Windows runtime PASS must not be claimed on the macOS host |
| M-05 | `OPEN` | `migrations/runtime_state/0001_runtime_state_initial.sql`; migration checksum/fingerprint constants and schema tests | UUID checks allowed `-` at non-separator positions | Replace every UUID-bearing-column check with exact lowercase UUID-v4 segment constraints in migration 1, then recalculate checksum and structural fingerprint | Per-column malformed/uppercase/version/variant/length/hyphen negatives, NULL positives, FK positives, exact inventory, checksum, and reopen evidence | No migration 2 and no public CRUD |
| L-01 | `OPEN` | `runtime_store/connection.rs::configure_limits` | Successful setters were treated as effective-limit proof | Read back every configured SQLite limit and compare with the required value | Table-driven configured-limit test | None beyond fresh exact-head review |
| L-02 | `OPEN` | `runtime_store/worker.rs`; `runtime_store/tests.rs::worker_contract_has_one_named_owner_and_bounded_queue` | Queue evidence was source-text/constant-only | Test-only worker hold plus the real capacity-128 sync channel to prove exact saturation and controlled overflow | Behavioral saturation and bounded cleanup test | Test-only authority must remain private to `runtime_store` |
| L-03 | `OPEN` | `scripts/validate-storage-runtime-contract.mjs` | Validator did not independently compare Rust/TypeScript enums or enforce the no-user-argument command contract | Parse exact enum/command/registration contracts and run deterministic mutation self-tests | Validator PASS plus mutation detection | No frontend testing framework |

R1 and R2 remain historical failed reviews and are not rewritten. R3 is the
first independent local exact-head review to close their remaining scoped
findings. R4 is preserved as blocked by the assignment-alias false-negative;
the local fifth correction does not claim independent closure. A Git commit
cannot truthfully embed its own SHA, so the exact fifth SHA is recorded after
commit in the PR body, remote readback and final task report rather than as a
self-reference in the commit it identifies.

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

Capability classification after local correction verification:

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
- Full Rust suite: **180 passed / 0 failed** after adding 64 storage tests.
- Existing inference TypeScript/Rust contract: **PASS**.
- TypeScript validation and production Vite build: **PASS**.
- Existing Dashboard composition, Inference, Messaging, Activation, Genesis,
  pairing, and worker modules compile under the full Rust/frontend gates.
- No inference, pairing, reset, worker, wallet, transport, Supabase, deployment,
  CI, or Tauri capability file changed.
- `package-lock.json` remains unchanged.

## 4. New feature test

The focused storage suite contains **64 passed / 0 failed** tests covering:

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
- production `ExitRequested`/`Exit` lifecycle composition, idempotent shutdown,
  bounded checkpoint/worker-exit/join handling, and missing-exit fail-closed
  behavior;
- worker panic/disconnect finalization with intake disabled and no stale healthy
  projection;
- one absolute initialization deadline, real SQLite interruption, transactional
  rollback, no late healthy publication, and joined interrupt watchdogs;
- shutdown cancellation before/after interrupt registration, during a real
  long SQLite statement, migration transaction, and integrity query;
- out-of-band shutdown priority over status/FIFO work, deterministic rejection
  after shutdown, response disconnect, delayed exit proof, and panic cleanup;
- explicit `MANAGER -> REAPER -> COMPLETED` join ownership, one per-manager
  worker, eventual reaper join after a blocked pre-open stage, and zero active
  initialization watchdogs/workers after completion;
- traversal, symlink, non-regular file, database replacement, and runtime
  directory replacement with a same-inode hard-link refusal;
- initial/post-open database hard-link refusal, WAL/SHM hard-link refusal,
  permission preservation, Unix stable identity, and Windows stable-identity
  source/test gates;
- exact lowercase UUID-v4 lexical constraints for all eight UUID-bearing
  columns, exercised by twelve unique malformed values per column plus valid,
  nullable, and foreign-key-positive cases;
- exact readback of all eleven configured SQLite limits and behavioral
  saturation of the real capacity-128 queue;
- Unix private modes, corruption preservation, size hard limit, read-only
  status behavior, post-start resource-limit re-evaluation, and redacted public
  errors.

The deterministic frontend contract validator uses the installed TypeScript
compiler API for executable named, renamed and namespace imports; property,
element and locally aliased invoke calls; constants; exported function
arguments; and JSX mounting. Its in-memory `Program` and `TypeChecker` assign
symbol identity to imports, declarations, assignments, property/element
targets and shadowed identifiers. Monotonic fixed-point propagation covers
direct, chained, multi-step, parenthesized, asserted, non-null, nested-function,
dynamic-property and namespace-destructuring flows without allowing a later
reassignment to erase unsafe reachability. A comment-aware Rust lexer excludes
line and nested block comments, normal/raw/byte strings, and character literals
before checking the command attribute, signature, injected state arguments,
exact `generate_handler!` registration, and absence of CRUD/generic SQL
authority. It passes 42 positive assertions, accepts 7/7 explicit safe fixtures
and rejects 43/43 mutation fixtures, including the R2 comment-spoof, R3
namespace bypass and exact R4 assignment-alias reproduction.

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
- `src-tauri/src/runtime_store/control.rs`
- `src-tauri/src/runtime_store/deadline.rs`
- `src-tauri/src/runtime_store/error.rs`
- `src-tauri/src/runtime_store/lifecycle.rs`
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

The R1 correction adds **no** dependency and changes no Rust or frontend
manifest/lockfile. Rust 1.95.0 provides the required Windows stable file
identity through `std::os::windows::fs::MetadataExt`, so the conditionally
authorized `windows-sys` dependency was not needed.

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
checksum_sha256 = 62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d
schema_fingerprint = 37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77
```

Historical replacement inventory:

```text
R1_HEAD_CHECKSUM_SUPERSEDED = 843fa8f5d27d691359a4f3e167c4a454d3ae28e021563a429951c2430afcc4d6
CORRECTED_CURRENT_CHECKSUM = 62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d
```

The superseded value above is historical review evidence only and is not
accepted as the current embedded migration identity anywhere in code or tests.

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
- Tauri `ExitRequested` invokes one idempotent production shutdown primitive;
  `Exit` is a fallback only when that primitive was not already invoked.
- Production shutdown owns one absolute five-second deadline, atomically stops
  new intake, publishes cancellation, and uses a separate capacity-one control
  channel so shutdown does not wait behind accepted ordinary FIFO work.
- A private generation-scoped registration holds only a safe rusqlite
  `InterruptHandle`. Shutdown pulses that handle without holding lifecycle locks
  until the RAII registration clears or the same absolute deadline expires;
  an old generation cannot interrupt a later attempt.
- `wal_checkpoint(TRUNCATE)` remains protected by a bounded busy timeout and
  SQLite interrupt. Checkpoint/close failures are propagated, and clean close
  removes stale healthy/warning status before acknowledgement.
- Join ownership is explicit: `MANAGER -> REAPER -> COMPLETED`. A timeout
  atomically transfers the sole worker `JoinHandle` and exit receiver to one
  prestarted reaper; no second reaper or worker can be created. The reaper waits
  for exit evidence, joins after a blocked stage returns, and records completion.
- Missing or delayed exit proof therefore returns a controlled failure without
  dropping the active worker handle. Drop reuses the same idempotent state
  machine with a bounded two-second caller budget.
- The worker has a panic boundary and unconditional finalization. Panic,
  unexpected exit, request/reply disconnect, and post-health communication
  failure disable intake and replace any cached success with a fresh
  unavailable/internal projection.
- Initialization propagates one absolute 120-second production deadline through
  path preparation, open/configuration, migration, schema/integrity validation,
  and post-open checks. A scoped rusqlite interrupt watchdog is always disarmed
  and joined before the attempt returns.
- Filesystem/path preparation and individual OS open calls use cooperative
  before/after deadline checks; in-process Rust does not claim to forcibly
  cancel every blocking syscall. Once the SQLite handle exists, SQLite work is
  actively interruptible and shutdown retains join ownership until exit/join
  accounting completes.

R2 lifecycle evidence classification:

```text
INITIALIZATION_DEADLINE = ONE ABSOLUTE DEADLINE

FILESYSTEM_AND_OS_OPEN_CALLS =
COOPERATIVE BOUNDARY CHECKS /
INDIVIDUAL BLOCKING SYSCALL NOT FORCE-INTERRUPTIBLE IN PROCESS

SQLITE_OPERATIONS =
ACTIVELY INTERRUPTIBLE THROUGH RUSQLITE INTERRUPT HANDLE

SHUTDOWN_OWNERSHIP =
PRESERVED UNTIL WORKER EXIT/JOIN ACCOUNTING

ACTIVE_SQLITE_SHUTDOWN_HARNESS =
100 MS BUDGET / COMPLETED OWNERSHIP / ZERO ACTIVE WORKERS /
ZERO ACTIVE INITIALIZATION WATCHDOGS / NO LATE HEALTHY STATUS /
OBSERVED 1 MS ELAPSED / 98 MS INTEGER MARGIN

PREOPEN_BLOCKED_HARNESS =
CONTROLLED TIMEOUT / REAPER OWNERSHIP / ONE ACCOUNTED WORKER /
RELEASE -> EXIT PROOF -> EVENTUAL JOIN -> COMPLETED
```

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
- Unix regular-file identity is `dev`/`ino`/`nlink`, requires `nlink == 1`
  before permission mutation, and is rechecked after open, on status, before
  checkpoint/close, and for every existing WAL/SHM sidecar.
- Windows does not inherit a Unix-mode claim. Target-gated code uses Rust 1.95
  standard-library `volume_serial_number`, `file_index`, and `number_of_links`;
  its source contract and Windows-gated tests exist, but neither target
  compilation nor real Windows runtime behavior is claimed from the macOS host.
- Platforms that are neither Unix nor Windows fail closed rather than using a
  timestamp/length identity fallback.
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
| Historical R1 post-resume gate | `git diff --check` PASS; `cargo check --all-targets --locked` PASS; 53/53 tests before the R2/R3 lifecycle additions |
| Focused `runtime_store` tests | 64 passed / 0 failed |
| Phase 1A `inference::` tests | 67 passed / 0 failed |
| Full Rust tests | 180 passed / 0 failed |
| `cargo check --all-targets --locked` | PASS |
| `cargo clippy --all-targets --locked` | PASS; no `runtime_store` warning |
| Storage frontend/Rust contract | PASS; 42 structural checks, 7/7 safe fixtures and 43/43 negative fixtures, including the exact R4 assignment-alias bypass |
| Disposable compiling R4 assignment reproduction | PASS; production build succeeds and the validator rejects it with `Dashboard card must contain no raw invoke` |
| Inference frontend/Rust contract | PASS |
| TypeScript + production Vite build | PASS; 1,763 modules |
| `npm audit --omit=dev` | PASS; 0 vulnerabilities |
| Secret scan | PASS |
| Touched Rust warning gate | PASS |
| Scoped rustfmt | PASS |
| `git diff --check` | PASS |
| Package lock unchanged | PASS |
| R1 correction manifest/lockfiles versus `ffcc83d…` | PASS; no dependency change |
| Added SQLite-chain advisories | PASS; 0 |
| Complete RustSec baseline | FAIL; 13 inherited entries, unchanged from starting main |
| Repository-wide rustfmt | 94 legacy files; no candidate runtime/composition file in debt |
| Repeated R2 lifecycle/cancellation gate | PASS; 20 runs x 11 tests = 220 executions; 0 failures; slowest wall run 0.414 s |
| Final closeout lifecycle regression gate | PASS; 10 runs x 11 tests = 110 executions; 0 failures; slowest wall run 0.278 s |
| Real desktop smoke | `BLOCKED_BY_SAFE_PROFILE_REQUIREMENT` |

Existing repository warning debt remains visible: regular Rust compilation
reports 312 legacy warnings and Clippy reports 325/327 warnings. The scoped
warning gate reports no warning in the protected/new hardening modules. This
task does not perform an unrelated repository cleanup.

## Security review

```text
R4_ASSIGNMENT_ALIAS_CORRECTION_SELF_REVIEW =
CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 0 / INFO 4
```

The informational boundaries are unchanged inherited RustSec/rustfmt/warning
debt, missing real desktop restart proof, incomplete cross-platform execution,
and accepted plaintext-SQLite risk. This local self-review records the R4
assignment-alias correction but does not replace independent exact-head R5.

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

- Independent review R1 reported five Medium findings: production shutdown and
  checkpoint propagation, abnormal worker termination/stale success,
  enforceable initialization deadline, stable hard-link-aware file identity,
  and strict UUID-v4 lexical constraints.
- R2 and R3 independently retained the bounded R1 corrections. R3 found no
  regression in lifecycle, deadline, path-identity, schema or false-status
  behavior.
- Independent review R2 reported one Medium lifecycle/ownership finding and two
  Low documentation/validator findings. R3 independently closed them at exact
  head `86ef384a…` and reported three new nonblocking Low closeout findings.
- R4 reviewed exact head `7465673d…` and blocked it because assignment aliases
  were absent from the validator flow analysis even though direct namespace
  calls were rejected. The fifth correction is locally verified against direct,
  renamed, chained, multi-step, nested, asserted, property/element, dynamic and
  shadowing cases; independent R5 closure remains mandatory.
- A pre-final repeated gate exposed an intermittent M-01 result-delivery race:
  the former 100 ms worker-exit reserve could collapse a busy-checkpoint result
  into `Unavailable`. The correction now reserves up to 500 ms inside the same
  absolute shutdown deadline; the subsequent ten full focused runs passed.
- The first R2 20-run gate exposed a second real race on run 17: a one-shot
  shutdown interrupt could land after handle registration but before the next
  SQLite statement became active, leaving the migration watchdog to consume its
  five-second test deadline. Shutdown now pulses only the registered generation
  until RAII cleanup or the same caller deadline. The fresh 20-run/220-execution
  gate passed with no five-second tail or ownership leak.

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
and removed by the test fixtures. One additional disposable temporary database
was created for the SQLite table/index inventory. A disposable source copy was
used to compile and independently reject the exact R4 assignment mutation. No
real DAARION app-local-data path was used.

Overall local release classification:

```text
CORRECTION_LOCAL_GATE = PASS
SCOPED_PHASE_1B_1_GATE = PASS / R5_REVIEW_PENDING
REPOSITORY_BASELINE_GATE = CONDITIONAL_PASS
INDEPENDENT_R3_REVIEW = R3_REVIEW_PASS_WITH_NONBLOCKING_FINDINGS
R3_FINDINGS = CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 3 / INFO 4
INDEPENDENT_R4_REVIEW = R4_REVIEW_BLOCKED_BY_FINDINGS
R4_BLOCKER = ASSIGNMENT_ALIAS_FALSE_NEGATIVE
R4_ASSIGNMENT_ALIAS_CORRECTION = IMPLEMENTED / LOCAL_TESTED / R5_REVIEW_PENDING
R5_REVIEW = REQUIRED / NOT_PERFORMED
PR_READY_OR_MERGE = NOT_AUTHORIZED
PHASE_1B_2 = NOT_AUTHORIZED
```

The conditional repository classification reflects the unchanged RustSec,
warning/formatting, cross-platform, and desktop-smoke baselines. It is not a
scoped implementation failure and does not convert them into verified claims.

## Final result

```text
PHASE_1B_1 =
COMPLETE_IN_PR / NOT_MERGED / R4_BLOCKED_ASSIGNMENT_ALIAS /
FIFTH_CORRECTION_LOCAL_GATE_PASS / R5_REVIEW_PENDING

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

REAL_DESKTOP_RESTART_FLOW =
NOT_VERIFIED

READY =
NOT_PERFORMED

MERGE =
NOT_PERFORMED

REMOTE_PRODUCTION_WRITES =
0

REAL_USER_PROFILE_WRITES =
0

DEPLOYMENTS =
0
```
