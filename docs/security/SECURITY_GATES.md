# Sovereign Agent Security Gates

Status: **MANDATORY / FAIL CLOSED**

No gate is satisfied by documentation or module names alone. “Open” means implementation or proof is missing. A gate closes only with code, tests, security review, documentation and human acceptance in the owning phase.

| Gate | Current state | Required evidence to close | Owner/phase |
| --- | --- | --- | --- |
| Local-only inference | REPOSITORY PASS — MERGE PENDING | Phase 1A has only `LocalOnly`, validated/revalidated loopback Ollama composition, redirect/proxy denial, unique canonical mapping, service-owned probe/chat/preparation deadlines, one kind-aware chat/preparation cancellation registry, bounded chat and pull streaming, deterministic stalled-socket/race/isolation tests, truthful preparation Cancel UI, sole-terminal chat enforcement and no main-webview shell authority. All repository checks pass under the documented changed-scope formatting amendment. No approved mapped model was available for a real Ollama smoke, so live provider and daemon-side cancellation behavior remain unverified. | Edge / 1A |
| Durable runtime state | OPEN | SQLite migrations, transactions, restart recovery, deletion/export, permissions and corruption/migration tests | Edge / 1B |
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

This evidence does not close the model-download trust gate, packaging/mobile gate or production-release gate. It does not authorize merge or Phase 1B. The exact commands, formatting amendment and known baseline limitations are recorded in the Phase 1A completion report.
