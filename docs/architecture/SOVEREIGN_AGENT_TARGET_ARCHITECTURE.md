# DAARION Sovereign Agent Target Architecture

Status: **ACCEPTED TARGET / INCREMENTAL / NOT IMPLEMENTATION EVIDENCE**

This document defines the target boundaries adopted from the 2026-07-04 baseline. Current implementation status is tracked in [CAPABILITY_STATUS_MATRIX.md](CAPABILITY_STATUS_MATRIX.md); target names do not prove that modules exist.

## Design invariants

1. The sovereign Edge runtime is local-first.
2. `InferencePolicy::LocalOnly` is the only enabled inference policy for the MVP.
3. No Edge prompt leaves the device without a separately enabled future policy and explicit user consent.
4. Web cloud-assisted AI is a separate trust domain and is never represented as Edge local inference.
5. Deterministic runtime code, not the LLM, owns limits, retries, cancellation, approval, stop conditions, idempotency, and persistence.
6. Private keys, raw memory, unrestricted tools, and signing authority never enter LLM context or the web projection.
7. External inputs are untrusted data, not instructions.
8. Repository contracts are versioned; browser state, URLs, and display claims are not authorization evidence.
9. Transport is replaceable and cannot own agent policy, memory, or wallet signing.
10. Public repositories contain sanitized architecture and contracts, never private operational truth.

## System flow

```text
User
-> loval-echoes authenticated MicroDAO context
-> single-purpose device pairing invitation
-> daarion-edge-client pairing verification
-> local identity and runtime initialization
-> explicit local model/provider selection
-> deterministic Agent Supervisor
-> bounded Loop Runtime
-> relevant local state/memory recall
-> local LLM structured decision
-> policy evaluation
-> optional permission-controlled action
-> verification and checkpoint
-> safe signed readiness projection
-> loval-echoes Dashboard
```

Future mesh flow:

```text
Agent Supervisor
-> Transport interface
-> authenticated local IPC
-> daarion-meshd (preferred desktop direction)
-> Reticulum/LXMF
-> decentralized agent mesh
```

No mesh implementation is authorized before Phase 6.

## Trust domains

| Domain | Owner | Trusted for | Not trusted for |
| --- | --- | --- | --- |
| Browser/product | `loval-echoes` | User interaction, authenticated product context, consent UI | Local keys, raw memory, runtime enforcement |
| Supabase/web backend | Web product contract | Authenticated membership, invitation/readiness read models under RLS and grants | Device-local truth without signed fresh evidence |
| Edge deterministic core | `daarion-edge-client` | Local policy, limits, state transitions, tool/signing gates | Treating model output as authority |
| Local model provider | Provider adapter inside Edge | Generating bounded structured proposals | Policy, permissions, secrets, stop control |
| OS secure storage | Edge/platform | Private key storage where supported | General application state or LLM context |
| Mesh daemon | Future separate component | Transport, mailbox, routing | Agent policy, memory semantics, wallet signing |
| Remote/cloud AI | Separately enabled future Edge policy or labeled web feature | Only explicitly disclosed requests | Implicit fallback or local capability claims |

## Proposed Edge module map

The names below are target boundaries, not proof of current files:

| Module | Responsibility |
| --- | --- |
| `app_state` | Composition and typed managed-state wiring only |
| `inference` | `InferenceProvider`, Ollama adapter, execution policy, timeout/cancellation, streaming |
| `agent_runtime` | Inert Supervisor, task state, planner/executor/verifier boundaries |
| `loop_runtime` | Versioned definitions, triggers, limits, checkpoints, outcomes, recovery |
| `memory` | Repository interfaces; later working/conversation/episodic/semantic/procedural/graph memory |
| `runtime_store` | SQLite bootstrap, migrations, conversations, messages, tasks, audit events |
| `tools` | Typed registry, risk classification, argument/path/network gates, approvals |
| `identity` | Domain-separated device, agent, pairing, transport and session identities |
| `wallet` | Isolated proposal/signing boundary with explicit approval |
| `transports` | Generic transport interface and authenticated daemon IPC |
| `projections` | Minimal versioned readiness/status documents with provenance/freshness |
| `policy` | Inference, tool, autonomy, egress and approval decisions |
| `audit` | Append-only security/runtime events and export/redaction rules |
| `worker` | Optional signed-lease execution, disabled until its security gate passes |

`src-tauri/src/lib.rs` should remain composition-focused; service logic should move behind the boundaries above incrementally, not through a total rewrite.

## Proposed web module map

| Product area | Responsibility |
| --- | --- |
| Product onboarding | Account and MicroDAO create/join journeys |
| MicroDAO context | Membership and active-community context |
| Device connection | Invitation UI, handoff, status display and revocation UX |
| Agent controls | User-facing policy/consent controls, never local execution |
| Readiness projections | Display only safe, versioned, fresh summaries |
| Privacy controls | Clear local/cloud disclosure, deletion/export requests and consent |
| Cloud-assisted features | Separately labeled provider/trust domain and server-side authorization |

## Durable state and memory layers

Phase 1B creates only:

- schema migrations;
- conversations;
- messages;
- tasks;
- audit events.

It must include retention hooks, deletion, export, crash recovery, transactions, and migration tests. It is not “six-level memory.”

Later memory repositories may add:

- working state;
- conversation memory;
- episodic records;
- semantic facts;
- procedural knowledge;
- graph entities/relations;
- embeddings.

Model-extracted memory remains untrusted until validated and keeps source/evidence references.

## Bounded Loop Runtime

A loop is a versioned workflow, not a separate agent. Every run has:

- maximum iterations, duration, tool calls, retries, token budget, and cost units;
- explicit states and terminal outcomes;
- a durable checkpoint after every accepted transition;
- deterministic task/run IDs and duplicate-trigger handling;
- policy evaluation before every action;
- bounded backoff, cancellation, and restart-safe resume;
- an audit trail;
- explicit `AWAITING_APPROVAL` behavior for sensitive actions.

Terminal outcomes are `DONE`, `NO_OP`, `PARTIAL`, `BLOCKED`, `FAILED`, `CANCELLED`, `BUDGET_EXCEEDED`, and `AWAITING_APPROVAL`.

The first reference loop is Local Agent Health. It reads injected/local status only and cannot access network, shell, user files, remote inference, signing, messaging, or worker activation.

## Cross-repository projections

A readiness projection is a bounded read model, not remote control. A future signed projection may contain only approved fields such as:

- schema version;
- pseudonymous device identifier;
- capability/status enum;
- health category;
- policy mode;
- generated-at and expires-at times;
- source identity and signature;
- non-sensitive reason code.

Raw prompts, messages, memories, filenames, keys, wallet addresses, tool arguments/results, network topology, logs, and model outputs are excluded by default.

## Reticulum/LXMF direction

The sequence is:

1. Define a transport interface.
2. Define signed, expiring, idempotent message envelopes.
3. Define authenticated local IPC and daemon identity.
4. Prototype packaging only after the ADR gate.
5. Prefer a separate `daarion-meshd` desktop daemon for background operation.
6. Make an independent Android decision.

A bundled Python sidecar is prototype-only. Direct Rust porting is premature. Containers are not the default desktop/mobile integration boundary.

## Related decisions

- [ADR 0001](../adr/0001-local-first-inference-and-remote-consent.md)
- [ADR 0002](../adr/0002-local-runtime-state-and-sqlite-foundation.md)
- [ADR 0003](../adr/0003-reticulum-lxmf-integration-boundary.md)
- [Security gates](../security/SECURITY_GATES.md)
- [Master roadmap](../planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md)
