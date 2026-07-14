# DAARION Sovereign Agent Master Roadmap

Status: **ACCEPTED BASELINE / PHASE-GATED**

This roadmap records dependency order, not calendar estimates or implementation claims. Every phase follows:

```text
audit -> written plan -> human review -> bounded vertical slice
      -> tests -> security review -> documentation -> diff review
      -> release gate -> roadmap update
```

Only one phase is authorized at a time. A `GO` or `CONDITIONAL_GO` plan does not bypass explicit human review.

## Phase map

| Phase | Goal | Primary repository | Scope and non-goals | Acceptance evidence | Security gate | Complexity |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | Baseline audit, instructions and skills | Both | Docs/instructions only; no runtime changes | Adopted audit, ownership, matrices, ADRs, roadmap, threat model, validation report | No sensitive data; no false implementation claims | M |
| 1A | Local-only inference foundation | Edge | `InferenceProvider`, Ollama adapter, `LocalOnly`, model/command mapping, timeout, cancellation, truthful UI, fake-provider tests. No SQLite/Supervisor/tools/network fallback | Policy/provider unit tests, loopback-only integration tests, cancellation/deadline tests, UI truth tests, diff/security review | Remote branch unreachable; no silent egress | L |
| 1B | Durable runtime state | Edge | SQLite bootstrap/migrations, conversations, messages, tasks, audit events, restart recovery, deletion. Not six-level memory | Migration/replay/transaction/restart/deletion/export tests | Local DB permissions, no secret logging, recovery integrity | L |
| 1C | Inert Agent Supervisor | Edge | Deterministic task IDs, explicit state machine, bounded transitions, recovery integration. No tools/network/autonomous scheduling | State-machine, idempotency, cancellation and crash-recovery tests | Model cannot bypass state/policy; no execution capability | L |
| 2 | Six-level memory evolution | Edge | Repository interfaces then working/conversation/episodic/semantic/procedural/graph memory; embeddings only after ADR | Migration, provenance, retention, deletion/export, poisoning/dedup tests | Extracted memory untrusted; raw memory local | XL |
| 3 | Bounded Loop Runtime | Edge | Versioned definitions, triggers, limits, checkpoints, policy hooks, outcomes; Local Agent Health reference loop only | Completion/no-op/cancel/timeout/limits/restart/duplicate/failed-checkpoint tests | No unbounded loop; no network/tools/signing in reference loop | XL |
| 4 | Read-only Tool Runtime | Edge | Typed registry and READ_ONLY tools only; no mutation/financial/privileged tools | Allowlist, argument/path, denial, confirmation, audit, injection/traversal tests | Permission broker required; fail closed | L |
| 5 | Signed readiness projection | Both | Separate projection ADR, safe schema, producer/consumer, freshness/revocation; no raw memory | Cross-repo fixtures, signature, replay, expiry, privacy and downgrade tests | Minimal allowlist and provenance | L |
| 6 | Reticulum/LXMF transport | Edge + selected mesh component | Transport interface, authenticated IPC, signed envelope, mailbox/offline queue; no agent policy in daemon | IPC authentication, replay/expiry/idempotency, queue recovery and packaging tests | Daemon replacement/spoofing and envelope validation closed | XL |
| 7 | Multi-agent loops | Edge + mesh | Bounded message processing, capability discovery and delegation; no financial actions | Signed capability, duplicate/replay, policy, cancellation and recovery tests | Remote agents are untrusted; bounded delegation | XL |
| 8 | Wallet and economic loops | Edge signer boundary | Separate identities, proposal flow, isolated signing, explicit approval; worker only as separately gated subphase | Derivation/recovery, transaction display, approval, replay and signer-isolation tests | Financial action always approved outside LLM | XL |
| 9 | Hardening and production gates | Both + release components | Threat closure, platform packaging, update/supply-chain, recovery drills, observability | Release matrices, platform evidence, incident/recovery drills, dependency/license review | No unresolved Critical/High finding | XL |

## Phase 1A: the only eligible next runtime milestone

Required scope:

- define `InferenceProvider`;
- implement an Ollama provider without coupling the domain API to Ollama;
- make `InferencePolicy::LocalOnly` the only enabled MVP policy;
- remove or make simulated remote fallback unreachable;
- align command names, registry model IDs and Ollama upstream tags;
- enforce request timeout and cancellation;
- make provider/runtime/status UI truthful;
- use fake-provider tests for success, refusal, timeout, cancellation and attempted remote invocation.

Explicit non-goals:

- SQLite or memory;
- Agent Supervisor or loops;
- tools, shell expansion or worker;
- pairing/readiness changes;
- Reticulum/LXMF;
- wallet/signing;
- remote provider implementation.

Entry gate:

1. Phase 00 completion report reviewed.
2. ADR 0001 accepted by a human.
3. A separate `phase-01a-local-only-inference-plan.md` exists with current source inventory and exact files.
4. No unresolved High/Critical Phase 1A blocker.
5. No unrelated dirty changes overlap the target files.

## Phase 1B boundary

The first SQLite schema contains only:

- `schema_migrations`;
- `conversations`;
- `messages`;
- `tasks`;
- `audit_events`.

It must not be described as complete memory. Memory types, embeddings, entities/relations and external vector systems are deferred.

## Phase 1C boundary

The first Supervisor is inert. It can create, transition, cancel, persist and recover tasks, but it cannot call tools, open network connections, schedule itself, sign data, mutate user files or recursively delegate.

## Loop Runtime dependency

Loop Runtime follows local inference, durable state and an inert Supervisor. The runtime, not the LLM, owns budgets, state transitions, approvals and stop conditions. The first health loop consumes injected/local status and has no network, shell, file mutation, remote inference, signing or worker access.

## Future ADR queue

- 0004 — signed device pairing envelope;
- 0005 — signed device readiness projection;
- identity domain separation and recovery;
- tool permission and egress policy;
- loop definition/versioning and recovery;
- wallet signer isolation;
- model manifest/artifact trust;
- Android transport/background execution.

Numbers are reservations for planning, not accepted decisions.

## Current gate

- Baseline documentation: pending Phase 00 release-gate completion.
- Phase 1A: `CONDITIONAL_GO` to a separate plan and human review only.
- Phase 1B and later: `NO_GO`.
- Production readiness: `NO_GO`.
