# Sovereign Agent Security Gates

Status: **MANDATORY / FAIL CLOSED**

No gate is satisfied by documentation or module names alone. “Open” means implementation or proof is missing. A gate closes only with code, tests, security review, documentation and human acceptance in the owning phase.

| Gate | Current state | Required evidence to close | Owner/phase |
| --- | --- | --- | --- |
| Local-only inference | REPOSITORY PASS — MERGED / FRESH-MAIN VERIFIED | Phase 1A has only `LocalOnly`; loopback/redirect/proxy controls; mandatory `/api/status` cloud-disabled proof; exact stable `/api/tags` → `/api/show` → `/api/tags` local-model evidence; immediate pre-chat revalidation; post-preparation verification; service-owned probe/chat/preparation deadlines and cancellation; bounded streaming; zero-chat sentinel tests; truthful UI; and no main-webview shell authority. Reviewed head `9e8c5d9…` was merged as `62a1d514…` and verified from fresh main. No real Ollama/model smoke or cryptographic daemon/artifact attestation is claimed. | Edge / 1A |
| Durable runtime state | OPEN — PHASE 1B.1 MERGED / FRESH-MAIN VERIFIED | Reviewed head `5d894f42a967c9360d86382c1aab9e603472e0c8` passed independent R6 with accepted nonblocking findings, merged as `cd903fb18d1618bbe0787d2397948622849ef9d4` at `2026-07-24T11:44:00Z`, and passed fresh-main verification. The verified slice includes the five-table bootstrap and one safe storage-status projection, not content persistence services. Fresh main passed 64/64 storage, 67/67 inference and 180/180 full Rust tests, Cargo check/Clippy, 29/29 primary fixtures, 13/13 defense fixtures, 46 structural checks, production build over 1,763 modules and zero production npm vulnerabilities. Remote CI was absent and is not claimed. The broader gate remains open for public content services, retention/deletion/export/backup, desktop/platform evidence, full recovery/privacy closure and the pre-production SQLCipher decision. Phase 1B.2 is not authorized | Edge / 1B |
| Inert Supervisor | OPEN | Deterministic IDs, explicit bounded state machine, idempotency, cancellation and crash recovery; no tools/network/scheduling | Edge / 1C |
| Production pairing | OPEN | Signed purpose-bound invitation, trusted issuer, membership/device binding, expiry, nonce/replay, single use, revocation and downgrade tests | Both / ADR 0004 |
| Readiness projection | OPEN | Separate signed schema, minimal allowlist, producer identity, expiry/freshness, replay/revocation, cross-repo fixtures and privacy tests | Both / ADR 0005 + Phase 5 |
| Six-level memory | OPEN | Provenance/trust labels, migrations, retention, deletion/export, poisoning/dedup/contradiction tests; raw data local | Edge / 2 |
| Autonomous loops | OPEN | Versioned bounded definitions, durable checkpoints, explicit outcomes, budgets, cancellation, backoff, idempotency and resume tests | Edge / 3 |
| Tool Runtime | OPEN | Typed registry, risk classes, argument/path/egress allowlists, confirmation, audit, injection/traversal tests; no unrestricted shell | Edge / 4 |
| Reticulum/LXMF | OPEN | Transport interface, authenticated IPC, verified daemon package, signed/expiring/idempotent envelope, replay cache, offline queue and platform tests | Edge + mesh / 6 |
| Wallet | OPEN | Domain-separated identity, isolated signer, derivation/rotation/recovery, transaction display, explicit approval, replay/limit tests; no keys in frontend/model | Edge / 8 |
| Worker | OPEN | Hard-disabled default, signed targeted leases, hardened sandbox, resource/time limits, secret isolation, cancellation/kill and escape/adversarial tests | Edge / 8 or later |
| Model downloads | OPEN | Approved versioned manifest, digest/size/signature verification, safe paths, atomic install/quarantine, rollback and negative tests | Edge / later model phase |
| Production release | OPEN | All scoped gates closed, no unresolved unaccepted Critical/High finding, platform packaging/signing/update evidence and recovery drills | Both / 9 |

## Tool risk classes

Future tools are classified as:

- `READ_ONLY`
- `LOCAL_MUTATION`
- `EXTERNAL_COMMUNICATION`
- `FINANCIAL`
- `PRIVILEGED_SYSTEM`

The LLM may propose an action but cannot grant permission. Sensitive classes require policy and user approval; financial actions always require explicit approval outside model context.

## Loop limits

Every future run enforces at least:

- maximum iterations;
- maximum duration;
- maximum tool calls;
- maximum retries with bounded backoff;
- maximum token budget;
- maximum cost units;
- cancellation and explicit terminal outcome;
- checkpoint after every accepted transition;
- duplicate-trigger idempotency.

No unbounded autonomous loop may pass a release gate.

## Release result

- `PASS`: every required check passed and evidence exists.
- `CONDITIONAL_PASS`: non-blocking limitation is explicit, no scoped Critical/High blocker remains, and required checks did not fail.
- `FAIL`: a required check failed, required evidence is absent, scope escaped, sensitive data leaked, or a scoped Critical/High blocker remains.

A skipped required check cannot produce `PASS`.

## Phase 1A local-only evidence

The candidate implementation is intentionally narrower than production readiness:

- production construction accepts only an HTTP loopback origin and disables redirects and system proxies;
- a remote-scoped provider is rejected before any provider method can run;
- the loopback daemon must explicitly report `cloud.disabled=true`; false,
  missing, malformed, unsupported, timed-out or otherwise unverifiable policy
  evidence fails closed;
- an installed/ready model requires exactly one canonical private tag with empty
  remote markers, positive size, valid digest syntax, coherent details,
  matching show evidence, and an identical second tags read;
- each chat repeats daemon policy, canonical mapping and model evidence checks
  inside its request deadline before the prompt-bearing request is constructed;
- deterministic rejected-path fixtures make zero `/api/chat` calls and observe
  no sentinel prompt in status, tags or show requests;
- prompts, tokens, completed output, raw provider bodies and environment/proxy values are not logged by the inference module;
- provider errors cross IPC as controlled messages rather than raw response bodies;
- cancellation, timeout or failure closes the event gate, suppressing late tokens and completion;
- model preparation and inference accept canonical IDs only; upstream Ollama tags remain adapter-private;
- main-window shell permissions and shell plugin initialization were removed;
- duplicate canonical/model mappings, malformed adapter tags and endpoint changes after service construction fail closed;
- queue-wait and streaming timeout/cancellation races, final-token-before-cancel and late-provider-error behavior produce exactly one terminal event;
- NDJSON record and aggregate-buffer limits are enforced before buffer growth, and post-terminal or malformed provider records fail closed.
- health and installed-model probes use one service-owned deadline; actual loopback fixtures prove that accepted sockets stalled before headers or during the inventory body return controlled `timed_out` rather than hanging or producing false success;
- the probe deadline is not a global reqwest timeout and a delayed-chat test proves streaming chat retains its separate request deadline.
- model preparation requires a UUID and canonical model ID, registers before concurrency/provider work, and has a dedicated kind-isolated cancel command;
- streamed pull progress reuses bounded NDJSON framing and rejects malformed, oversized, error, incomplete and post-terminal records;
- stalled-header/body fixtures prove cancellation drops the DAARION future/stream, cleans the registry and leaves chat/other preparation operations active;
- mounted UI cancellation says only that the local request stopped; no claim is made that the Ollama daemon immediately stopped network or disk work.
- a preparation result is successful only after the daemon and complete local
  model evidence pass again; failed postflight cannot become
  `completed_locally` in the mounted UI.

This evidence does not close the model-download trust gate, malicious-daemon
attestation, packaging/mobile gate or production-release gate. Official Ollama
metadata is treated as evidence, not cryptographic truth about the daemon or
artifact. Phase 1A was merged from its exact reviewed head, but that does not
authorize production operations or Phase 1B implementation. The exact commands,
formatting amendment and known baseline limitations are recorded in the Phase
1A completion report.

## Phase 1B implementation status

The docs-only Phase 1B plan now records human acceptance of a Rust-owned
`rusqlite` service (`bundled`, `limits`, `backup`) and standard plaintext SQLite
for the foundation. Supported production devices require full-disk encryption;
the database must not contain private keys, wallet seeds, access tokens,
credentials, or model secrets. Backups and JSON exports are also plaintext
unless separately encrypted and may never be described as encrypted.

Accepted operations are `foreign_keys=ON`, WAL, `synchronous=FULL`,
`secure_delete=ON`, `trusted_schema=OFF`, `temp_store=MEMORY`, five-second busy
timeout, `wal_checkpoint(TRUNCATE)` at clean shutdown, and SQLite backup API
only—raw live-database copy is forbidden. Retention, bounded size/deadline/
queue/backup limits, explicit plaintext export, and desktop macOS/Windows/Linux
validation are fixed in the plan. Android remains a separately authorized gate;
iOS is unsupported and unclaimed.

The merged and fresh-main-verified Phase 1B.1 slice adds only `rusqlite` 0.40.1,
bundled SQLite 3.53.2, the empty five-table migration, private Rust service, one
no-argument status command, typed client, and Dashboard card. It has no content
CRUD, backup/export, remote sync, Supervisor, memory extraction, or generic SQL
authority. Automated tests use generated temporary roots; no real user profile
or production system is written. R1 and R2 blocked earlier heads. Independent
local R3 passed exact head `86ef384a…` with three nonblocking Low findings. R4
and R5 then exposed assignment- and object-literal alias false-negatives in the
overclaimed validator model. Accepted ADR 0006 makes the command-scoped
module/import/re-export graph the primary control and limited AST checks
defense in depth. Arbitrary TypeScript data-flow proof is not claimed.
Independent exact-head R6 passed with accepted nonblocking findings, followed
by controlled merge and fresh-main verification.

SQLCipher remains a separate pre-production decision. The durable-state gate
stays open through later separately authorized Phase 1B slices, desktop target
evidence, deletion/export, and full recovery/privacy closure. The merged
Phase 1B.1 repository slice is not production readiness.

```text
PHASE_1B_1 = MERGED / FRESH_MAIN_VERIFIED
MERGED_REVIEWED_HEAD = 5d894f42a967c9360d86382c1aab9e603472e0c8
MERGE_COMMIT = cd903fb18d1618bbe0787d2397948622849ef9d4
MERGED_AT = 2026-07-24T11:44:00Z
STORAGE_BOOTSTRAP = IMPLEMENTED_AND_VERIFIED
STORAGE_RUNTIME_PROJECTION = IMPLEMENTED_AND_VERIFIED_IN_REPOSITORY
DURABLE_RUNTIME_STATE = PARTIALLY_IMPLEMENTED
PHASE_1B = NOT COMPLETE
PHASE_1B_2 = NOT AUTHORIZED
REAL_DESKTOP_RESTART_FLOW = NOT VERIFIED
CROSS_PLATFORM_RUNTIME = NOT VERIFIED
REMOTE_CI = NOT PRESENT / NOT CLAIMED
REMOTE_PRODUCTION_WRITES = 0
REAL_USER_PROFILE_WRITES = 0
DEPLOYMENTS = 0
```

The frozen nine-path Tauri-core importer baseline is grandfathered technical
debt, not global adapter approval. `src/lib/storageRuntimeClient.ts` remains
the sole executable frontend owner of `get_storage_runtime_status`; its command
constant is private and it exports no raw Tauri binding. Rust exposes one
read-only status command with no user-deserialized arguments. No Phase 1B.2
CRUD/API authority exists.

Fresh-main evidence and schema invariants:

```text
RUST_TOOLCHAIN = 1.95.0 PINNED
STORAGE_TESTS = 64/64 PASS
INFERENCE_TESTS = 67/67 PASS
FULL_RUST_TESTS = 180/180 PASS
CARGO_CHECK = PASS
CARGO_CLIPPY = PASS
RUNTIME_STORE_WARNING_LOCATIONS = 0
PRIMARY_BOUNDARY_FIXTURES = 29/29 PASS
DEFENSE_IN_DEPTH_FIXTURES = 13/13 PASS
STRUCTURAL_CHECKS = 46 PASS
PRODUCTION_BUILD = PASS / 1,763 MODULES
PRODUCTION_NPM_AUDIT = 0 VULNERABILITIES
NPM_DEV_INCLUSIVE_ADVISORIES = 11 INHERITED / OUTSIDE PRODUCTION DEPENDENCY SET
INHERITED_RUSTSEC_WARNING_RUSTFMT_DEBT = UNCHANGED
MIGRATION_SHA = 62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d
STRUCTURAL_FINGERPRINT = 37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77
TABLES = 5
EXPLICIT_INDEXES = 7
SQLITE_AUTOINDEXES = 7
SQLITE_SEQUENCE = 0
MIGRATION_2 = ABSENT
```
