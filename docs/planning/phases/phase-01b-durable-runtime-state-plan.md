# Phase 1B — Durable Runtime State Plan

Status: **CONDITIONAL_GO / PLANNING ONLY**

Starting `main`: `62a1d514b93925e8b7098c6db19f8751a70a7bf8`

This document proposes a bounded implementation. It does not add SQLite, a
migration, a runtime API, or application behavior. Each implementation slice
requires separate human authorization.

## 1. Objective

Define the smallest Rust-owned durable local runtime-state layer needed before
an inert Agent Supervisor can be designed. The layer must persist structured
conversations, messages, inert task records, and privacy-safe audit events;
survive restart; migrate deterministically; and support bounded deletion,
export, and fail-closed recovery.

## 2. Phase boundary

Phase 1B includes only:

- SQLite bootstrap and deterministic embedded migrations;
- `conversations`, `messages`, inert `tasks`, and structured `audit_events`;
- explicit transaction boundaries;
- restart readback and WAL recovery;
- user-directed deletion and deterministic versioned export;
- corruption, lock, integrity, and migration-failure handling.

It explicitly excludes:

- six-level memory, memory extraction, embeddings, semantic search, vectors,
  entities, and graph relations;
- Agent Supervisor behavior, task orchestration, scheduling, loops, tools,
  Reticulum/LXMF, or worker execution;
- model-controlled transitions or autonomous actions;
- Supabase synchronization, readiness projection, wallet/signing, or a cloud
  database fallback;
- import, legacy JSON migration, and a generic frontend database interface.

Conversation history in this phase is runtime state, not semantic or long-term
memory.

## 3. Current repository inventory

The inventory is based on executable source at the starting commit.

| Component | Classification | Evidence and boundary |
| --- | --- | --- |
| Rust/Tauri composition | `IMPLEMENTED` | `src-tauri/src/lib.rs` declares services and registers commands; it is already a composition hotspot and must not acquire SQL logic. |
| SQLite dependency/schema | `ABSENT` | `src-tauri/Cargo.toml`, `Cargo.lock`, and source contain no runtime SQLite driver, database bootstrap, schema, or migration runner. |
| Device identity metadata | `PARTIAL` | `src-tauri/src/identity.rs` stores `identity.json` under the Tauri app-data directory and keeps the Ed25519 secret in the OS keyring. Rotation/recovery remain outside this phase. |
| Enrollment state | `PARTIAL` | `src-tauri/src/enrollment.rs` stores `enrollment.json`; node-token material uses keyring. Some JSON read failures fall back to a default state. |
| Pairing state | `PARTIAL` | `src-tauri/src/pairing.rs` validates then writes `pairing.json` directly. Production trust, atomic replacement, revocation, and replay are separate gates. |
| Worker state | `OUT_OF_SCOPE` | `src-tauri/src/worker/mod.rs` writes `worker_mode.json`; the cryptographic gate remains fail-safe disabled. |
| Genesis media | `OUT_OF_SCOPE` | `src-tauri/src/genesis.rs` writes a local WAV and includes wallet-adjacent behavior governed by another security gate. |
| Factory reset | `PARTIAL` | `src-tauri/src/reset.rs` deletes current JSON/WAV/keyring material but knows nothing about a future runtime database, WAL, backups, or exports. |
| Boot logging | `PARTIAL` | `src-tauri/src/lib.rs` appends an unstructured boot log under a platform-dependent directory. It is not a structured audit store. |
| Messaging state | `MOCK` | `src-tauri/src/messaging.rs` keeps `Mutex<Vec<Message>>`; sessions, polling, and local echo are stubs and disappear at restart. |
| Local chat UI state | `PLACEHOLDER` | `src/components/LocalInferencePanel.tsx` keeps prompts and messages in React state; no durable conversation contract exists. |
| General UI state | `PLACEHOLDER` | `src/App.tsx` owns transient React state that is recreated after restart. |
| Messaging UI | `MOCK` | `src/components/MessagingPanel.tsx` mirrors the Rust messaging stub in component state. It is not a persistence consumer for Phase 1B. |
| Frontend registry cache | `OUT_OF_SCOPE` | `src/lib/preflightApi.ts` uses IndexedDB `DaarionEdgeDB/registry_cache`; this is a disposable frontend cache, not authoritative runtime state. |
| Genesis browser state | `OUT_OF_SCOPE` | `src/components/GenesisWizard.tsx` stores creator and wallet-adjacent values in `localStorage`; its privacy/removal work is a separate scope. |
| Worker queue | `OUT_OF_SCOPE` | `src-tauri/src/worker/queue.rs` is an in-memory `VecDeque` behind the worker security gate. |
| Agent intent/execution/result types | `PLACEHOLDER` | `src-tauri/src/agents/agent_intent.rs`, `agent_execution.rs`, and `agent_result_binding.rs` do not form a durable Supervisor contract. |
| Backend task payloads | `PLACEHOLDER` | `src-tauri/src/registry_client.rs` represents tasks as `Vec<serde_json::Value>` and does not execute them. No backend schema is adopted here. |
| Conversation domain model | `ABSENT` | No Rust-owned durable `Conversation` model or repository was found. |
| Audit event domain model | `ABSENT` | No structured append-only local audit-event model was found. |
| Tests | `PARTIAL` | Rust unit tests are mostly inline. There are no persistence migration/restart/corruption fixtures. Phase 1A currently has 67 inference tests and 116 full Rust tests. |
| Backup/export/recovery | `ABSENT` | No consistent runtime-state snapshot, deterministic export, corruption workflow, or database recovery path exists. |

The repository targets desktop macOS, Windows, and Linux. Release configuration
also includes Android, while iOS is future/commented. No persistence packaging
claim may be raised until the selected dependency compiles and the database
path/permissions/recovery contract is tested on each claimed target.

Existing ADRs are `0001`, `0002`, and `0003`. ADR 0002 already accepts SQLite
as the initial local-state foundation and defines the five-table boundary, so
this planning task does not create a duplicate ADR.

## 4. Storage technology decision

### Options evaluated

| Option | Benefits | Costs and boundary risk | Decision |
| --- | --- | --- | --- |
| `rusqlite` | Small, explicit synchronous API; precise transaction/pragma/open-flag control; embedded migration SQL is straightforward; MIT; bundled SQLite supports consistent packaging | Must be isolated from Tokio; native C compilation; the connection must not be shared as an unrestricted global | **Selected** |
| SQLx SQLite | Async API, compile-time query facilities, migration support | Larger multi-database surface and dependency graph; pool/macro behavior is unnecessary for one local database; raises toolchain and packaging surface | Rejected for Phase 1B |
| `tokio-rusqlite` | Provides an async handle around a dedicated connection thread | Adds an extra wrapper and channel abstraction; current release trails the selected `rusqlite` line, complicating version/security maintenance | Rejected; implement a narrow project-owned actor |
| Tauri SQL plugin | Convenient JavaScript access and all-platform packaging | Its generic frontend SQL model violates the Rust-owned domain boundary and expands IPC/database authority | Rejected |

Recommendation: add exactly one direct database dependency during authorized
slice 1B.1: `rusqlite` pinned to a reviewed compatible version, initially
`0.40.1`, with `default-features = false` and only `bundled`, `limits`, and
`backup`. The last feature is required for consistent pre-migration snapshots;
it does not authorize a generic backup command.
The implementation slice must verify the exact version against the repository
Rust toolchain and macOS/Windows/Linux/Android packaging before accepting the
pin. Version drift is a stop condition, not an implicit update.

Runtime ownership remains in Rust. One service-owned SQLite connection runs on
a dedicated blocking worker and accepts bounded typed requests from Tokio. No
SQL, path, table name, or connection handle crosses frontend IPC.

The `bundled` feature makes the application compile its selected SQLite instead
of relying on varied system copies. It increases native build time and package
review surface, but improves version consistency. `load_extension` and SQLCipher
features are not enabled. Migration control remains project-owned and fully
transactional. The test harness uses temporary app-local roots and the same
bootstrap path as production.

Dependency facts must be rechecked during slice 1B.1 against the official
[`rusqlite` documentation](https://docs.rs/crate/rusqlite/latest), including
its MIT license and open flags. SQLx and the
[`Tauri SQL plugin`](https://v2.tauri.app/plugin/sql/) remain documented
alternatives, not installed components.

## 5. At-rest encryption decision

Decision proposal:

`RECOMMEND_STANDARD_SQLITE_WITH_EXPLICIT_RISK_ACCEPTANCE`

### Threat model and trade-off

Standard SQLite plus an application-local directory, per-user permissions,
redaction, and OS full-disk encryption protects against other unprivileged
accounts in normal operation. It does **not** protect a copied database from an
administrator, malware running as the user, or offline disk access when
full-disk encryption is absent, bypassed, or already unlocked. Backups and JSON
exports create equivalent additional disclosure surfaces.

SQLCipher would add database-file confidentiality for a stolen copy, but it
requires a new domain-separated database key, OS-keyring lifecycle, recovery
semantics, lost-key behavior, encrypted backup/export decisions, native crypto
packaging on every target, license/attribution review, and migration tests. The
existing device-identity key must never be reused as a database key. SQLCipher
community licensing and platform packaging must be reviewed from its official
[`license`](https://www.zetetic.net/sqlcipher/license/) before adoption.

For the bounded foundation, standard SQLite is recommended only if a named
human explicitly accepts the residual stolen-disk risk and confirms that the
deployment profile requires OS full-disk encryption. If that risk is rejected,
or database-level encryption is a requirement, implementation must stop and
this plan must be amended to `RECOMMEND_SQLCIPHER` with a separate key/recovery
design. No documentation or UI may call the standard database encrypted.

Backups/exports must inherit restrictive permissions, disclose their plaintext
status, and never include cryptographic secrets. Key loss is not applicable to
the standard-SQLite choice; SQLCipher adoption would make key loss and recovery
a blocking product decision.

## 6. Database location and permissions

- Resolve the root exclusively through Tauri
  `app.path().app_local_data_dir()`; the frontend cannot supply or override it.
- Use `<app-local-data>/runtime-state/runtime-state-v1.sqlite3`.
- Reserve sibling directories `tmp/`, `backups/`, and `exports/` beneath the
  trusted `runtime-state/` root. Exports may later be copied through a separate
  user-approved save flow, never through a caller-supplied database path.
- Create parent directories before opening the database. On Unix, require mode
  `0700` for directories and `0600` for the database, `-wal`, `-shm`, backup,
  temporary, and export files. Recheck after creation/open.
- On Windows, use current-user `LocalAppData`; verify inherited ACL behavior in
  a Windows integration test and fail closed if a supported ACL check proves
  broader write access. Do not claim Unix modes apply on Windows.
- Canonicalize and compare every derived path to the trusted root. Reject
  symlinks, parent traversal, non-regular database files, and replacements.
  Open with `SQLITE_OPEN_NOFOLLOW` where supported and revalidate metadata.
- Put temporary files on the same filesystem/root and finalize with atomic
  rename only after sync. Use unguessable runtime-created filenames.
- Keep WAL and shared-memory sidecars beside the database with the same
  protection. Treat them as database content in reset, backup, disclosure, and
  recovery logic.
- Never raw-copy a live database. Use SQLite's consistent
  [backup API](https://www.sqlite.org/backup.html) after a bounded checkpoint.

Mobile path and permission behavior must be verified on Android before that
target can pass this phase. iOS remains unclaimed.

## 7. Connection and concurrency model

`RuntimeStore` owns one `rusqlite::Connection` on a dedicated named blocking
thread/actor. Tokio callers submit typed operations through a bounded channel
(proposed capacity: 128) and await a one-shot result. The Tauri async runtime,
UI thread, and command handlers never execute synchronous SQLite work.

Initial behavior is one serialized writer and serialized reads on the same
connection. No pool is needed. Configure and verify on every open:

- `foreign_keys = ON`;
- `journal_mode = WAL`;
- `synchronous = FULL`;
- `secure_delete = ON`;
- `trusted_schema = OFF`;
- `temp_store = MEMORY`;
- five-second busy timeout;
- conservative SQLite runtime limits through the selected feature.

Use parameterized statements only. Ordinary operations have a ten-second
service deadline; migrations and streamed exports have separate bounded
deadlines proposed at 120 seconds. Timeout values remain runtime-owned, not
model-controlled.

Startup order is: resolve/validate path and permissions; open with no-follow
flags; apply and read back pragmas; run integrity/foreign-key checks as required;
acquire the migration transaction; apply/verify migrations; then publish the
managed service. No frontend or Supervisor consumer starts earlier.

Clean shutdown stops intake, drains already accepted operations within a
deadline, runs `wal_checkpoint(TRUNCATE)`, and closes the connection. On caller
cancellation before dequeue, remove/reject the request. Once a transaction has
begun, it must complete or roll back; cancellation suppresses delivery of the
result but cannot abandon a partial commit. SQLite interruption must not be
used across a commit boundary.

## 8. Migration contract

Migrations are append-only Rust-embedded SQL resources with a compiled manifest.
Each row contains:

- sequential integer `migration_id`, starting at `1`;
- immutable `name`, such as `runtime_state_initial`;
- SHA-256 checksum of the exact embedded bytes;
- UTC application timestamp in Unix epoch milliseconds.

The initial bootstrap begins `BEGIN IMMEDIATE`, creates
`schema_migrations` and the four state tables, records migration `1`, and
commits all changes together. Later migrations apply in ascending order, once,
with their schema mutation and migration record in one transaction.

On startup the runner compares the database history with the compiled manifest.
A gap, changed checksum/name, duplicate ID, unknown higher ID, or partially
matched manifest fails closed. Downgrade is refused. An interrupted transaction
is allowed to roll back through SQLite recovery and is retried only after the
manifest/integrity checks pass. Concurrent startup is serialized by
`BEGIN IMMEDIATE` plus the configured busy deadline.

No mutable “latest schema” shortcut is authoritative. Before a future migration
explicitly marked destructive, create and verify a consistent SQLite backup;
the first migration on a fresh database needs no backup. A failed migration
keeps the prior schema, preserves the database, and returns a controlled recovery
error without paths or raw SQL.

## 9. Initial schema

Identifiers are lowercase hyphenated UUID v4 text generated in Rust. Timestamps
are signed `INTEGER` Unix epoch milliseconds UTC. The service validates byte
limits before binding; database `CHECK` constraints use
`length(CAST(value AS BLOB))` as a second UTF-8 byte boundary.

### `schema_migrations`

| Field | Contract |
| --- | --- |
| `migration_id` | `INTEGER PRIMARY KEY`, positive sequential ID |
| `name` | non-empty `TEXT NOT NULL UNIQUE`, maximum 128 bytes |
| `checksum_sha256` | 64-character lowercase hexadecimal `TEXT NOT NULL` |
| `applied_at_ms` | `INTEGER NOT NULL`, UTC epoch milliseconds |

Privacy: operational metadata, no user content. No delete through the public
service.

### `conversations`

| Field | Contract |
| --- | --- |
| `id` | UUID `TEXT PRIMARY KEY` |
| `title` | nullable text, maximum 512 UTF-8 bytes |
| `status` | `TEXT NOT NULL`, allowlist `active`, `archived` |
| `created_at_ms`, `updated_at_ms` | UTC `INTEGER NOT NULL`, monotonic per record |
| `next_message_sequence` | positive `INTEGER NOT NULL DEFAULT 1` |
| `revision` | non-negative `INTEGER NOT NULL DEFAULT 0` for optimistic updates |

Index `(updated_at_ms, id)`. Privacy: sensitive local user metadata. Deleting a
conversation cascades to its messages and linked tasks.

### `messages`

| Field | Contract |
| --- | --- |
| `id` | UUID `TEXT PRIMARY KEY` |
| `conversation_id` | `TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE` |
| `sequence_no` | positive `INTEGER NOT NULL` allocated from the conversation row |
| `role` | `TEXT NOT NULL`, allowlist `system`, `user`, `assistant` |
| `content` | non-empty `TEXT NOT NULL`, maximum 65,536 UTF-8 bytes |
| `created_at_ms` | UTC `INTEGER NOT NULL` |

Unique `(conversation_id, sequence_no)` and index
`(conversation_id, sequence_no, id)`. Privacy: highly sensitive local content.
No audit event may duplicate `content`.

### `tasks`

| Field | Contract |
| --- | --- |
| `id` | UUID `TEXT PRIMARY KEY` |
| `conversation_id` | nullable foreign key with `ON DELETE CASCADE` |
| `task_kind` | non-empty `TEXT NOT NULL`, maximum 64 bytes |
| `state` | `TEXT NOT NULL DEFAULT 'created'` with a Phase 1B database allowlist containing only `created`; Phase 1C must extend it through a forward migration |
| `idempotency_key` | nullable `TEXT UNIQUE`, maximum 128 bytes |
| `created_at_ms`, `updated_at_ms` | UTC `INTEGER NOT NULL` |
| `revision` | non-negative `INTEGER NOT NULL DEFAULT 0` |

Indexes `(state, updated_at_ms, id)` and `(conversation_id, id)`. Privacy:
confidential runtime metadata. There is no prompt, payload, plan, result, tool,
model, endpoint, wallet, or remote identity column.

### `audit_events`

| Field | Contract |
| --- | --- |
| `sequence_no` | `INTEGER PRIMARY KEY AUTOINCREMENT`, local total ordering |
| `event_id` | UUID `TEXT NOT NULL UNIQUE` |
| `event_type` | allowlisted `TEXT NOT NULL`, maximum 64 bytes |
| `actor_type` | allowlisted `TEXT NOT NULL`, maximum 32 bytes |
| `subject_type` | allowlisted `TEXT NOT NULL`, maximum 32 bytes |
| `subject_id` | nullable UUID `TEXT`, never a path or content value |
| `outcome` | allowlisted `TEXT NOT NULL`, maximum 16 bytes |
| `reason_code` | nullable allowlisted `TEXT`, maximum 128 bytes |
| `correlation_id` | nullable UUID `TEXT` |
| `created_at_ms` | UTC `INTEGER NOT NULL` |

Indexes `(created_at_ms, sequence_no)`, `(event_type, created_at_ms)`, and
`(subject_type, subject_id, sequence_no)`. Privacy: privacy-safe operational
metadata, still local-sensitive. Public-service deletion is forbidden except
through the approved retention process.

No other table is permitted in migration 1.

## 10. Domain boundary

Expose Rust types and traits/services such as `ConversationStore`,
`MessageStore`, `TaskStore`, `AuditStore`, and a transaction-scoped
`RuntimeUnitOfWork`. A private `SqliteRuntimeStore` implements them. Callers use
validated provider-neutral inputs and controlled errors.

No frontend API receives or submits raw SQL, a database path, arbitrary table
name, pragma, transaction handle, SQLite row/value, or generic query. Tauri
commands are not required for bootstrap. Any later export/delete command must
be purpose-specific, validate all inputs, use the managed service, and receive
separate review.

## 11. Conversation and message invariants

- Create a conversation with a runtime-generated ID, canonical timestamps,
  `active` status, sequence `1`, and its audit event in one transaction.
- Allocate a message sequence while holding the conversation write transaction;
  insert the message, increment `next_message_sequence`, update timestamp and
  revision, and append the audit event atomically.
- Accept only the three defined roles and valid UTF-8 within byte limits.
- Foreign keys prevent orphan messages; ordering is always
  `(sequence_no, id)`, never timestamp alone.
- Restart readback produces the same IDs, roles, content, and ordering.
- Conversation deletion cascades to messages and linked inert tasks, verifies
  affected rows, and adds a privacy-safe tombstone event without content.
- Export follows the same stable ordering. Conversation history remains local
  runtime state and is not promoted to memory.

## 12. Task boundary

Phase 1B stores only identity, optional conversation ownership, kind,
`created` state, idempotency key, timestamps, and revision. Repository-level
update mechanics may be tested using explicitly supplied expected revisions,
but application callers cannot transition beyond `created` in this phase.

Task-state semantics, allowed transitions, cancellation states, recovery,
deterministic task IDs, and idempotency ownership are deferred to the Phase 1C
Supervisor ADR/plan. Phase 1B does not authorize autonomous execution, model-
controlled transitions, tools, network, scheduling, retry, or delegation.

## 13. Audit event contract

Audit events are structured, ordered, timestamped, and append-only through the
public service. State-changing operations write the event in the same database
transaction. Initial allowlists are:

- event types: `conversation.created`, `conversation.deleted`,
  `message.appended`, `task.created`, `task.recorded`, `task.deleted`,
  `runtime.content_deleted`, `export.completed`, and
  `storage.recovery_required`;
- actors: `user`, `local_runtime`;
- subjects: `conversation`, `message`, `task`, `runtime`, `export`, `storage`;
- outcomes: `success`, `denied`, `failed`;
- reason codes: a versioned Rust enum of short machine codes, never free-form
  error text.

The strict metadata set is the table fields above. Prompts, message content,
secrets, raw provider responses, tokens, private keys, filesystem paths, raw
SQL, environment values, and full error dumps are forbidden.

## 14. Transaction boundaries

The following operations are atomic:

1. Schema mutation plus its `schema_migrations` row.
2. Conversation creation plus `conversation.created`.
3. Message insertion plus sequence allocation, conversation timestamp/revision
   update, and `message.appended`.
4. Task create/update/delete plus the corresponding privacy-safe audit event.
5. Conversation deletion/cascade plus a tombstone event.
6. Delete-all user content plus one aggregate tombstone event.

Export spans database and filesystem boundaries and cannot be one SQLite
transaction. It reads a stable snapshot, streams to a protected temporary file,
syncs and atomically renames it, then records `export.completed`. If the file
step fails, no success event is recorded; if event recording fails after rename,
the export is returned as incomplete and reconciled explicitly.

## 15. Restart recovery

- Clean shutdown drains bounded work, checkpoints WAL, and closes.
- After a process crash, SQLite replays/recovers WAL, then startup verifies
  pragmas, migration manifest, `quick_check`, and `foreign_key_check` before
  exposing the store.
- Interrupted transactions roll back; no application-level partial state is
  accepted.
- Incomplete `.tmp` exports are never reported as complete. Startup may remove
  only files matching the service-owned name pattern and age policy.
- Migration failure preserves the prior database and returns a typed
  `migration_failed`/`recovery_required` error.
- Unreadable, corrupt, replacement, locked-after-deadline, or newer-schema
  databases fail closed. The runtime never silently deletes, overwrites,
  recreates, downgrades, or imports them.
- Controlled user instructions identify the recovery class and safe next step
  without exposing paths, SQL, content, or internal error dumps.

## 16. Deletion and retention

- Single-conversation deletion is explicit and transactional; messages and
  linked tasks cascade, row counts are verified, and a privacy-safe event is
  retained.
- Delete-all removes conversations/messages/tasks, retains only audit metadata
  allowed by the reviewed retention policy, and verifies no user-content rows
  remain.
- Task deletion is physical in Phase 1B with an audit tombstone; Phase 1C may
  propose state-based retention separately.
- Enable `secure_delete = ON`. An explicit post-delete `VACUUM` may be offered
  as bounded maintenance only after free-space and interruption analysis.
- Factory reset must first close the service, then remove the database, WAL,
  shared-memory, service-created backups, temporary files, and local exports
  under its trusted root; this integration belongs to a reviewed slice.
- Backups or exports copied elsewhere are independent copies and are not
  silently removed.

SQLite secure-delete and `VACUUM` can reduce recoverable logical remnants, but
cannot guarantee physical erasure from SSD wear leveling, filesystem snapshots,
cloud backup, or copied exports. The UI/documentation must say so. See SQLite's
[`secure_delete`](https://www.sqlite.org/pragma.html#pragma_secure_delete) and
[`VACUUM`](https://www.sqlite.org/lang_vacuum.html) limits.

Retention proposal requiring human approval: conversations remain until user
deletion; audit metadata is bounded to 100,000 rows and 180 days, except records
explicitly required for an unresolved local recovery event. No silent content
summarization or memory extraction is permitted.

## 17. Export contract

Export is UTF-8 JSON with `export_version: 1`, `schema_version`,
`generated_at_ms`, `database_integrity` metadata limited to approved status and
manifest checksum, then arrays ordered as follows:

- conversations by `(created_at_ms, id)`;
- messages by `(conversation_id, sequence_no, id)`;
- tasks by `(created_at_ms, id)`;
- audit events by `sequence_no`.

Keys use a fixed documented order and timestamps use epoch milliseconds.
The generator streams to avoid materializing the database in RAM and produces a
SHA-256 digest of the finalized JSON bytes as separate integrity metadata. The
generation timestamp necessarily differs between exports; deterministic means
identical state and supplied timestamp produce identical bytes.

Exports exclude secrets, keyring values, paths, raw errors/SQL, provider bodies,
environment data, caches, localStorage, identity keys, wallet material, and all
out-of-scope JSON files. Export is plaintext under the standard-SQLite decision
and requires explicit user disclosure/approval. Import is out of scope.

## 18. Corruption and integrity policy

- Run `foreign_key_check` and `quick_check` after open/recovery and migrations;
  use full `integrity_check` for an explicit diagnostic or after a quick-check
  failure, under a separate deadline.
- Verify every applied migration checksum/name/ID before accepting the store.
- Rely on transaction rollback for interrupted writes and migrations.
- Use the SQLite backup API before a future destructive migration; verify the
  snapshot before proceeding.
- Classify errors as path/permission, locked, busy timeout, migration mismatch,
  newer schema, integrity failure, resource limit, invalid input, or internal.
- Public errors expose only class, stable code, retryability, and safe action.

No corruption path automatically deletes, replaces, renames, or rebuilds the
database. Recovery is user-directed from a verified backup or export under a
future, separately authorized restore/import design.

## 19. Resource limits

Proposed bounded defaults, subject to human review:

- message content: 65,536 UTF-8 bytes;
- conversation title: 512 bytes;
- task kind: 64 bytes; idempotency key: 128 bytes;
- audit string fields: per-field limits above and at most 2 KiB total encoded
  audit row data;
- database soft warning: 512 MiB; hard write gate: 1 GiB, while read, export,
  and delete remain available;
- export hard limit: 1 GiB, checked during streaming;
- queued operations: 128; one active database operation;
- normal transaction deadline: 10 seconds; migration/export: 120 seconds;
- audit retention proposal: 100,000 rows and 180 days.

Limits are enforced before allocation/binding and rechecked within the service.
The LLM, frontend, and future agent cannot raise them. Human decisions remain
open for the database cap, audit retention, and mobile-specific storage budget.

## 20. Security model

| Threat | Required control |
| --- | --- |
| Local privilege/stolen disk | App-local per-user permissions, OS full-disk-encryption requirement, explicit residual-risk acceptance; no claim of DB encryption |
| Malicious frontend input | Typed purpose-specific methods, strict enums/byte limits, no generic SQL/path/table interface |
| SQL injection | Parameters only; static embedded SQL; no identifier interpolation |
| Path traversal/symlink replacement | Runtime-owned root, canonical containment, no-follow open, metadata revalidation, same-root temp files |
| Database replacement | File-type/owner/permission checks where supported, migration checksum and integrity validation, fail closed |
| Migration tampering/downgrade | Embedded immutable manifest, SHA-256, sequential IDs, unknown-newer and mismatch refusal |
| Sensitive logging | Stable codes only; never content, paths, SQL, secrets, provider bodies, or environment values |
| Denial of service | Size, queue, time, busy, database, export, and audit-retention limits |
| Oversized content | Validate UTF-8 byte length before allocation/bind plus database constraints |
| Backup leakage | Protected directory/modes, plaintext disclosure, no raw live copy, no automatic cloud location |
| Hostile imports | Import remains absent; future import treats every file as untrusted and needs a separate parser/security gate |

The database service is a local integrity and availability boundary, not a
sandbox against code already executing as the same OS user. Critical/High
findings in the selected dependency, path controls, migration runner, logging,
or deletion/export flow block the relevant slice.

## 21. Test matrix

The implementation must provide deterministic tests for all of the following:

1. Fresh database bootstrap creates exactly the five approved tables and indexes.
2. Sequential embedded migration execution records name, checksum, and timestamp.
3. Migration replay executes every migration exactly once.
4. Migration checksum or name mismatch fails closed without mutation.
5. Unknown newer schema/migration fails closed.
6. Interrupted migration rolls back schema and migration record together.
7. Foreign keys are enabled and read back on every connection/open.
8. WAL/restart recovery restores committed work and rejects partial work.
9. Transaction failure rolls back state and audit together.
10. Concurrent writes serialize without duplicate sequence or partial state.
11. Conversations persist with identical IDs and fields across restart.
12. Message ordering remains stable across restart.
13. Foreign keys and service APIs prevent orphan messages.
14. Inert task records persist across restart without acquiring behavior.
15. Audit `sequence_no` ordering is total and stable.
16. State plus required audit write is atomic in success and failure cases.
17. Single-conversation deletion cascades only its owned rows and verifies counts.
18. Delete-all removes approved user content and applies reviewed audit retention.
19. Export bytes are deterministic for identical state and supplied timestamp.
20. Export excludes secrets, paths, raw errors/SQL, caches, and out-of-scope files.
21. A generated corrupt-database fixture fails closed and is preserved.
22. A locked database returns a controlled error within the busy deadline.
23. Oversized titles/messages/tasks/audit values are rejected before write.
24. Traversal, symlink, non-regular file, and replacement attempts fail closed.
25. Directory/file/WAL/export permission checks pass where the platform supports them.
26. Public errors contain no raw SQL, database details, paths, content, or secrets.
27. Existing 67 Phase 1A inference tests and the full Rust suite remain unchanged and passing.
28. Clean shutdown checkpoints/closes and the store reopens cleanly.
29. Rollback restores compatible code/database state without a downgrade attempt.
30. Static and integration contracts prove no frontend generic SQL execution path exists.

Add negative tests for cancellation before dequeue, cancellation during a
transaction, queue saturation, database hard cap, export failure before/after
rename, and reset cleanup. Corrupt/locked fixtures are generated at test time;
do not commit user-like database content.

## 22. Implementation slices

Every slice requires a fresh exact-main plan readback, explicit human approval,
its own tests/security review, and a stop decision. Approval of this document
does not authorize all slices.

### 1B.1 — Storage bootstrap, path policy, and migration runner

- Goal: add the selected dependency, private storage actor, path/permission
  policy, connection pragmas, migration manifest, and empty fresh schema.
- Expected files: `src-tauri/Cargo.toml`, `Cargo.lock`, new
  `src-tauri/src/runtime_store/**`, one embedded initial SQL resource, and a
  composition-only edit to `src-tauri/src/lib.rs`.
- Tests: matrix 1–10, 21–26, 28–30 where applicable, plus target compile checks.
- Entry gate: human accepts dependency/version and encryption residual risk.
- Stop: packaging failure, path/permission ambiguity, migration mismatch,
  unresolved Critical/High finding, or unrelated scope change.
- Rollback: remove the unopened feature/module/dependency; never downgrade an
  opened higher schema. Preserve/backup created test data only.
- Non-goals: repositories, frontend APIs, state migration, Supervisor behavior.

### 1B.2 — Conversations and messages

- Goal: add typed models/repositories and atomic ordered persistence.
- Expected files: runtime-store models plus conversation/message repositories
  and tests; no generic IPC.
- Tests: matrix 9–13, 16, 17, 23, 26–28.
- Entry gate: 1B.1 fresh-main verified; content/retention limits accepted.
- Stop: orphan/ordering/privacy failure or any need to call a model/tool/network.
- Rollback: disable consumers and revert code only while preserving the schema;
  use a forward migration for any shipped schema correction.
- Non-goals: chat UI integration, semantic memory, summaries, embeddings.

### 1B.3 — Inert tasks and audit persistence

- Goal: add inert task records and privacy-safe transactional audit events.
- Expected files: task/audit repositories, allowlisted enums/errors, tests.
- Tests: matrix 9, 14–16, 23, 26–27 plus metadata redaction cases.
- Entry gate: 1B.2 verified and Phase 1C state semantics remain explicitly deferred.
- Stop: executable task semantics, free-form audit metadata, secret/content logging.
- Rollback: remove callers while retaining compatible rows; schema fixes are forward-only.
- Non-goals: transitions, planner/executor/verifier, retries, tools, scheduling.

### 1B.4 — Deletion, export, and recovery verification

- Goal: implement explicit deletion, deterministic plaintext export, restart,
  corruption, reset integration, and recovery instructions.
- Expected files: runtime-store export/recovery modules, narrowly reviewed
  `src-tauri/src/reset.rs` integration, purpose-specific command only if a
  reviewed UI flow requires it, and tests/docs.
- Tests: matrix 8, 17–26, 28–29 plus partial-export and SSD-limit disclosure.
- Entry gate: human accepts deletion/retention/export semantics and destinations.
- Stop: raw live copy, silent overwrite/recreate, path disclosure, or unbounded export.
- Rollback: disable export/delete UI entrypoints, preserve the database, retain
  verified backups; do not undo a committed schema by downgrade.
- Non-goals: import, cloud backup/sync, encrypted export, memory export.

### 1B.5 — Final integration and security gate

- Goal: run the full test/platform/license/security matrix, review the complete
  diff, update truth documents, and close only the implemented Phase 1B gate.
- Expected files: tests and canonical architecture/security/planning docs;
  application changes only for defects inside the approved prior slices.
- Tests: all 30 cases, full Rust/inference/frontend contract/build checks,
  dependency/license/secret/platform checks.
- Entry gate: 1B.1–1B.4 individually merged and fresh-main verified.
- Stop: any failed required check, unresolved unaccepted Critical/High finding,
  false capability claim, or unsupported claimed platform.
- Rollback: feature-disable state consumers, preserve/backup data, forward-fix
  schema; never destructively downgrade.
- Non-goals: Phase 1C Supervisor or later phases.

## 23. Proposed implementation file map

No file below is created by this planning task.

```text
src-tauri/src/
├── runtime_store/
│   ├── mod.rs                 # private public boundary and managed service
│   ├── config.rs              # bounded settings, not frontend-controlled
│   ├── path_policy.rs         # app-local root, permissions, no-follow checks
│   ├── error.rs               # controlled internal/public classifications
│   ├── worker.rs              # dedicated connection actor and shutdown
│   ├── connection.rs          # open flags and verified pragmas
│   ├── migrations.rs          # immutable manifest/checksum runner
│   ├── models.rs              # provider-neutral records and validated inputs
│   ├── repositories/
│   │   ├── mod.rs
│   │   ├── conversations.rs
│   │   ├── messages.rs
│   │   ├── tasks.rs
│   │   └── audit.rs
│   ├── export.rs
│   └── recovery.rs
├── lib.rs                     # module + managed-state composition only
└── reset.rs                   # later bounded reset integration

src-tauri/migrations/runtime_state/
└── 0001_runtime_state_initial.sql

src-tauri/tests/
├── runtime_store_migrations.rs
├── runtime_store_repositories.rs
├── runtime_store_recovery.rs
└── runtime_store_security.rs
```

If repository conventions favor embedded constants or colocated tests, the
implementation plan may adjust locations while preserving the boundary. The
initial migration path does not exist until authorized 1B.1. Tauri commands are
omitted by default; any purpose-specific export/delete command needs an explicit
consumer and security review.

Expected future documentation updates include this plan's completion report,
ADR 0002 status/readback if a material decision changes, capability/security
matrices, roadmap, and a Phase 1B implementation completion report.

## 24. Dependency changes

Only slice 1B.1 proposes manifest/lock changes:

| Dependency | Purpose | Version/features | Risk/license/platform | Rollback |
| --- | --- | --- | --- | --- |
| `rusqlite` | Rust-owned SQLite connection, transactions, backup and limits | Exact reviewed compatible release, initially `0.40.1`; `default-features = false`; `bundled`, `limits`, `backup` only | MIT; bundled SQLite is public-domain upstream; native C compile/package and security-version review required on every target | Remove before persisted use; after a database is opened, preserve data and remove only after compatibility/readback plan |

`Cargo.lock` would change transitively, including the selected SQLite FFI
package. The implementation PR must inventory every added/changed transitive,
license, advisory, duplicate native library, and mobile packaging effect. It
must not add SQLx, `tokio-rusqlite`, Tauri SQL, SQLCipher, loadable extensions,
or a second database abstraction without an amended plan/ADR.

If `0.40.1` does not pass the repository toolchain and target matrix, stop and
propose one explicit alternative with evidence. The removal path is a narrow
dependency/module revert only before production data exists; after persisted
use, schema/data compatibility controls rollback.

## 25. Acceptance criteria

Completed Phase 1B can pass only when:

- exactly the five approved tables and specified indexes/constraints exist;
- migration history is immutable, checksummed, transactional, exact-once, and
  fails closed for mismatch/newer schema/interruption;
- Rust exclusively owns the database path, SQL, connection, transactions, and
  typed repository APIs;
- no generic frontend SQL or database-path API exists;
- the blocking actor, queue, time, size, busy, and database limits are enforced;
- conversations/messages/tasks/audit survive restart with stable ordering and
  defined atomicity;
- task persistence remains inert and cannot execute/transition autonomously;
- deletion/export/recovery meet the documented privacy and fail-closed rules;
- at-rest residual risk, retention, resource limits, export behavior, and
  platform gate have explicit human decisions;
- all 30 required tests plus negative security cases pass;
- all existing 67 inference tests and the full Rust suite pass unchanged;
- macOS, Windows, Linux, and any still-claimed Android build/package checks pass;
- dependency/license/advisory/secret reviews pass with no unaccepted
  Critical/High finding;
- canonical documentation describes only verified behavior;
- the complete diff is reviewed and the implementation PR is separately
  approved, merged, and verified from fresh `main`.

Live deployment, Phase 1C, memory, and production readiness remain separate.

## 26. Rollback

- Code deployment: disable consumers behind a compile/runtime feature owned by
  the application, then revert service wiring without deleting data.
- Failed migration: rely on transaction rollback; preserve the database and
  verified pre-migration backup; correct with a new forward migration.
- Database backup: use SQLite backup API, restrictive permissions, checksum,
  explicit destination, and verification before a destructive migration.
- Schema incompatibility: refuse downgrade/newer schema, run compatible code,
  or ship a reviewed forward migration. Never edit migration history.
- Feature disablement: stop new writes, keep read/export/delete/recovery paths
  available where safe, checkpoint/close cleanly, and preserve the database.

Rollback never silently recreates the database, imports a JSON export, restores
an unverified backup, or deletes user content.

## 27. Final decision

`CONDITIONAL_GO`

This classification applies to the Phase 1B **plan**, not implementation.
Implementation remains `NO_GO` until explicit human review closes every blocking
decision below and authorizes only slice 1B.1.

Unresolved human decisions:

1. Accept standard SQLite's residual stolen-disk exposure and require OS
   full-disk encryption, or amend the plan to SQLCipher.
2. Accept `rusqlite` as the sole integration and the exact dependency/version
   selection process.
3. Accept WAL + `synchronous=FULL`, the five-second busy timeout, ten-second
   operation deadline, and 120-second migration/export deadline.
4. Accept plaintext export behavior and decide the approved user-visible save
   destination/consent flow.
5. Accept conversation retention until deletion and the proposed 180-day /
   100,000-row audit retention.
6. Accept the 512 MiB warning, 1 GiB database/export hard gates, and message/
   queue limits.
7. Decide whether Android packaging is a mandatory Phase 1B merge gate or a
   separately documented blocked target; iOS remains unclaimed.
8. Approve the exact 1B.1 slice after a fresh-main readback; no later slice is
   implicitly authorized.

```text
PHASE_1A =
MERGED / FRESH-MAIN VERIFIED / PASS

PHASE_1B_PLAN =
CONDITIONAL_GO

PHASE_1B_IMPLEMENTATION =
NO_GO PENDING EXPLICIT HUMAN REVIEW
```
