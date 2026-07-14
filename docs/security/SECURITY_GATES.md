# Sovereign Agent Security Gates

Status: **MANDATORY / FAIL CLOSED**

No gate is satisfied by documentation or module names alone. “Open” means implementation or proof is missing. A gate closes only with code, tests, security review, documentation and human acceptance in the owning phase.

| Gate | Current state | Required evidence to close | Owner/phase |
| --- | --- | --- | --- |
| Local-only inference | OPEN | Only enabled policy is `LocalOnly`; remote branch unreachable; provider abstraction; timeout/cancel; no-egress and fake-provider tests; truthful UI | Edge / 1A |
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
