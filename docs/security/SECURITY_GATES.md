# Sovereign Agent Security Gates

Status: **MANDATORY / FAIL CLOSED**

No gate is satisfied by documentation or module names alone. “Open” means implementation or proof is missing. A gate closes only with code, tests, security review, documentation and human acceptance in the owning phase.

| Gate | Current state | Required evidence to close | Owner/phase |
| --- | --- | --- | --- |
| Local-only inference | REPOSITORY PASS — MERGED / FRESH-MAIN VERIFIED | Phase 1A has only `LocalOnly`; loopback/redirect/proxy controls; mandatory `/api/status` cloud-disabled proof; exact stable `/api/tags` → `/api/show` → `/api/tags` local-model evidence; immediate pre-chat revalidation; post-preparation verification; service-owned probe/chat/preparation deadlines and cancellation; bounded streaming; zero-chat sentinel tests; truthful UI; and no main-webview shell authority. Reviewed head `9e8c5d9…` was merged as `62a1d514…` and verified from fresh main. No real Ollama/model smoke or cryptographic daemon/artifact attestation is claimed. | Edge / 1A |
| Durable runtime state | OPEN — PHASE 1B.1 ARCHITECTURAL CORRECTION / R6 REVIEW PENDING | R1 and R2 blocked earlier PR #27 heads; R3 passed with nonblocking findings; R4 blocked an assignment-alias false-negative; R5 blocked an object-literal false-negative at exact head `fdbb9c88…`. Accepted ADR 0006 rejects a custom complete data-flow analyzer and replaces the overclaim with a command-scoped module/import/re-export gate plus limited AST defense in depth. The local correction passes 29/29 primary fixtures, 13/13 secondary fixtures, 46 structural checks, 64 storage, 67 inference and 180 full Rust tests. Phase 1B.1 remains draft and unmerged. Closure still requires acceptable exact-head R6, separate ready/merge authorization and fresh-main proof; the broader durable-state gate additionally requires later Phase 1B content services, deletion/export/backup, desktop/platform evidence, and the pre-production SQLCipher decision | Edge / 1B |
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

The separately authorized Phase 1B.1 candidate adds only `rusqlite` 0.40.1,
bundled SQLite 3.53.2, the empty five-table migration, private Rust service, one
no-argument status command, typed client, and Dashboard card. It has no content
CRUD, backup/export, remote sync, Supervisor, memory extraction, or generic SQL
authority. Automated tests use generated temporary roots; no real user profile
or production system is written. R1 and R2 blocked earlier heads. Independent
local R3 passed exact head `86ef384a…` with three nonblocking Low findings. R4
and R5 then exposed assignment- and object-literal alias false-negatives in the
overclaimed validator model. Accepted ADR 0006 makes the command-scoped
module/import/re-export graph the primary control and limited AST checks
defense in depth. Arbitrary TypeScript data-flow proof is not claimed. The
architecture correction is locally green and awaits independent exact-head R6.

SQLCipher remains a separate pre-production decision. The durable-state gate
stays open through acceptable R6 review, separate merge/fresh-main
verification, later
separately authorized Phase 1B slices, desktop target evidence, deletion/export,
and full recovery/privacy closure. A repository candidate is not production
readiness.
