# DAARION Sovereign Agent Roadmap

Status: **LEGACY HIGH-LEVEL VIEW / SUPERSEDED BY MASTER ROADMAP**

This document preserves the earlier high-level phase order. The canonical,
reviewable phase boundaries now live in
`docs/planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md`, which splits the former first
runtime milestone into Phase 1A, 1B, and 1C. Neither roadmap is evidence that a
capability is implemented or production-ready.

## Operating Model

Every phase follows:

```text
audit -> written plan -> human review -> bounded vertical slice
      -> tests -> security review -> documentation -> release gate
```

A phase may begin application implementation only with a written `GO`, or a
`CONDITIONAL_GO` that has no unresolved security blocker. Each phase must end
with a completion report and `PASS`, `CONDITIONAL_PASS`, or `FAIL`.

## Repository Boundaries

- `loval-echoes`: authenticated product UX, MicroDAO context, Connect Device,
  user controls, pairing invitation UX, privacy controls, and safe projections.
- `daarion-edge-client`: local identity, inference, memory, Agent Supervisor,
  bounded Loop Runtime, permission-controlled tools, private checkpoints, safe
  projection generation, and transport abstraction.
- Future Reticulum/LXMF sidecar or daemon: mesh transport, mailbox, and background
  routing behind authenticated local IPC after an ADR.
- Public repositories contain sanitized contracts and summaries, never live
  infrastructure truth, private keys, raw local memory, or private operational
  evidence.

## Phase Order

| Phase | Goal | Primary repository | Entry gate | Required exit evidence |
|---|---|---|---|---|
| 0 | Audit and repository-scoped skills | Both | Read-only access to both current checkouts | Human-reviewed baseline audit, contract matrix, accepted roadmap, validated skills |
| 1 | Local-only inference foundation | Edge | Phase 0 accepted | Fail-closed execution policy, provider boundary, real-vs-mock inventory, policy and egress tests |
| 2 | Durable memory foundation | Edge | Phase 1 stable; memory ADR accepted | SQLite foundation, migrations, ownership/retention/deletion/export rules, restart tests |
| 3 | Loop Runtime and Agent Supervisor | Edge | Phases 1-2 accepted | Versioned definitions, bounded state machine, durable checkpoints, policy hooks, health reference loop, recovery tests |
| 4 | Read-only Tool Runtime | Edge | Phase 3 policy/action boundary proven | Typed registry, read-only allowlists, denial/approval/audit paths, injection and traversal tests |
| 5 | Cross-repository readiness projection | Both | Stable local state and contract ADR | Versioned safe-summary schema, producer/consumer fixtures, freshness, revocation, privacy and contract tests |
| 6 | Reticulum/LXMF transport | Edge plus selected sidecar/daemon | Transport ADR, authenticated IPC contract, Phase 3 recovery model | Signed envelope, replay/expiry protection, offline queue, packaging and IPC tests |
| 7 | Multi-agent loops | Edge plus transport component | Phases 3, 4, and 6 verified | Bounded delegation, capability verification, idempotent message processing, policy and recovery evidence |
| 8 | Wallet and economic loops | Edge | Identity/wallet ADRs, isolated signing boundary, explicit human approval | Domain-separated identity, transaction proposal/signing isolation, replay and approval tests |
| 9 | Hardening and production gates | Both plus release components | Prior phase-specific gates passed | Threat-model closure, platform release evidence, recovery drills, dependency and update review |

Phase numbers express dependency order, not calendar estimates.

## Loop Runtime Position

Loop Runtime precedes tools, transport, multi-agent behavior, and wallet actions.
It is the deterministic operating layer used by the Agent Supervisor; a loop is a
versioned workflow, not a separate agent or authority.

Supported workflow classes are planned as:

- event-driven;
- scheduled;
- goal-driven;
- maintenance.

Every workflow must have bounded iterations, duration, tool calls, retries, token
and cost budgets; explicit terminal outcomes; durable checkpoints; idempotency;
policy evaluation; cancellation; restart recovery; and a sanitized audit trail.
The LLM cannot control these limits or approval rules.

The first Phase 3 reference workflow is the **Local Agent Health Loop**. It may
inspect only already-local/injected model, memory-store, and queue status. It may
not open network or loopback sockets, call a backend health endpoint, invoke
shell/process execution, modify user files, send messages, perform remote
inference, sign data, or activate a worker.

After the foundation is verified, local chat-task and memory-consolidation
workflows may be proposed as separate bounded Phase 3 sub-milestones.

## Phase 3 Skill Set

Use at minimum:

- `$daarion-phase-planner`
- `$daarion-repository-auditor`
- `$daarion-rust-tauri`
- `$daarion-agent-runtime`
- `$daarion-loop-runtime`
- `$daarion-memory`
- `$daarion-local-llm`
- `$daarion-tool-security`
- `$daarion-testing`
- `$daarion-security-review`
- `$daarion-documentation`
- `$daarion-release-gate`

The repository audit determines whether any additional skill is relevant. Do not
activate Reticulum/LXMF, Supabase, cross-repository, identity/wallet, or other
domain skills unless the bounded Phase 3 plan actually crosses those boundaries.

## Current Gate

The human-approved audit decision is adopted in
`docs/audits/SOVEREIGN_AGENT_BASELINE_AUDIT_2026-07-04.md` with an explicit
provenance limitation: the complete original report was not supplied. Only Phase
1A is eligible for a separate plan and human review. Phase 1B and later remain
`NO_GO`, and production readiness remains `NO_GO`.
