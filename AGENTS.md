# Repository Instructions: daarion-edge-client

## Scope and source of truth

These instructions apply to this repository unless a more specific nested `AGENTS.md` overrides them.

- Identify this repository and read its `README`, relevant `docs/`, manifests, configuration, and source before proposing or making changes.
- Treat repository evidence as authoritative. Keep verified facts, inferences, and recommendations distinct.
- Do not import architecture, terminology, schemas, endpoints, or business rules from another project without an explicit, versioned contract in this repository.
- Prefer bounded, reversible vertical slices. Preserve existing conventions and working behavior.
- Never expose credentials, keys, tokens, private endpoints, infrastructure topology, or operational evidence in code, documentation, logs, diffs, or reports.

## Canonical sovereign-agent baseline

- Read `docs/audits/SOVEREIGN_AGENT_BASELINE_AUDIT_2026-07-04.md`,
  `docs/architecture/SYSTEM_CONTEXT_AND_OWNERSHIP.md`, and
  `docs/planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md` before sovereign-agent work.
- The baseline is a read-only source snapshot, not production verification.
- Phase 1A, Phase 1B.1, and Phase 1B.2 are merged and verified on fresh
  `main`; Phase 1B.2 canonical merge is
  `ec99bf70d6ada94bc1caae9886cca25ad42852f9`. Phase 1B remains incomplete.
- Phase 1B.2 implements only five private Rust conversation/message operations
  with no public content Tauri command or frontend content authority.
  Phase 1B.3 has a source-grounded audit and bounded planning package with
  `PLAN_RESULT = CONDITIONAL_GO`. Its final planning review is
  `PASS_WITH_NONBLOCKING_FINDINGS`: 0 Critical, 0 High, 1 Medium, 2 Low, and
  2 Info. The Medium stringly-audit-boundary residual is accepted only for
  planning canonicalization and requires typed-boundary implementation and
  executable proof before Phase 1B.3 can complete. Its implementation is
  `NOT AUTHORIZED`; a separate human implementation authorization must name
  the exact reviewed planning merge SHA. Phase 1B.4 and Phase 1C remain
  `NOT AUTHORIZED`.
- Pairing and readiness projections require separate future threat-driven ADRs;
  current parsing, SQL, types, and documentation do not close their security
  gates.

## Repository boundary

`daarion-edge-client` is the local sovereign-agent runtime. It may own:

- the Tauri/Rust runtime, device and agent identity, secure local key storage, and pairing consumption;
- device capability detection, local model lifecycle, local inference, and Agent Supervisor;
- bounded Loop Runtime workflows, durable checkpoints, stop control, and private audit state;
- durable local memory, permission-controlled tools, and safe readiness projections;
- a transport abstraction and authenticated local IPC for a future Reticulum/LXMF sidecar or daemon;
- an optional, explicitly security-gated worker runtime.

It must not own the web product experience, MicroDAO membership authority, browser authentication policy, or live private infrastructure truth. Reticulum/LXMF stays behind a transport boundary; Octelium may provide external secure access but must not replace decentralized transport.

For DAARION.city/DAGI work, preserve the City of Agents/AgentOS model and the current core 6 + 1 map: SOFIIA, DAARWIZZ, AISTALK, DAIS, SENTINEL, MELISSA, plus KILLER. SOFIIA is the architecture/CTO-intelligence/build-evolution layer; DAARWIZZ is the principal orchestration/routing/mayor-grade coordination layer. Orchestration is not ownership. Avoid competing writers for canonical architecture, routing, identity, evidence, value, or sanction truth. Treat current node placement as transitional evidence, not doctrine, and preserve autonomy, sovereignty, auditability, shared state where required, and the Gödel-Darwin architecture frame.

## Local-first and cryptographic policy

- Local inference is the default. Remote inference must never happen silently or through implicit fallback.
- Every inference path must obey an explicit execution policy; private prompts may leave the device only through a separately approved policy and visible user consent.
- Private keys must never enter LLM context, logs, frontend state, or unrestricted tool arguments.
- Separate device-root, agent, pairing, transport, wallet, and session security domains. Do not reuse a key across incompatible domains.
- The LLM cannot directly execute privileged tools or sign financial transactions. Policy evaluation, confirmation, execution, and audit remain outside the model.
- No unbounded autonomous loop is allowed. The deterministic runtime owns limits, retries, cancellation, approval waits, terminal outcomes, checkpointing, and restart recovery.
- Do not publish live IP addresses, DNS, firewall, Octelium, NODA, or private operational evidence.

## Mandatory phase workflow

For every implementation phase:

1. Read all active `AGENTS.md` files and relevant skills.
2. Invoke `$daarion-phase-planner` and create `docs/planning/phases/<phase-id>-plan.md` before application-code changes.
3. Inspect the real implementation and git state; list assumptions, risks, blockers, ownership, contracts, acceptance criteria, tests, and non-goals. Confirm the applicable baseline audit has been completed and human-reviewed; otherwise stop at audit/planning with `NO_GO` for application implementation.
4. Continue only when the written plan is `GO`, or `CONDITIONAL_GO` with no unresolved security blocker.
5. Select only the domain skills relevant to the phase. Do not require unrelated skills.
6. Implement the complete bounded vertical slice and no broader architecture.
7. Run narrow tests first, then repository-required lint, typecheck, build, contract, and security checks as applicable.
8. Review the complete diff, update architecture/ADR/roadmap/status documentation, and write `docs/planning/phases/<phase-id>-completion.md`.
9. Invoke `$daarion-release-gate`. Never claim completion if required checks failed or evidence is missing.

Always use `$daarion-testing`, `$daarion-security-review`, `$daarion-documentation`, and `$daarion-release-gate` for implementation phases. Select these only when the phase touches their domain:

- `$daarion-rust-tauri` for Rust, Tauri commands, managed state, IPC, keyring, native filesystem, or process boundaries;
- `$daarion-local-llm` for model discovery, lifecycle, inference, streaming, structured output, or execution policy;
- `$daarion-agent-runtime` for Supervisor, planner, executor, verifier, context building, or task state;
- `$daarion-loop-runtime` for triggers, schedules, workflow definitions, repeated steps, checkpoints, loop limits, stop control, cancellation, or restart/resume;
- `$daarion-memory` for any memory model, SQLite schema, migration, retention, deletion, export, or retrieval;
- `$daarion-tool-security` for tools, shell/process/filesystem/network access, confirmations, or workers;
- `$daarion-identity-crypto` for keys, signing, identity, rotation, revocation, recovery, pairing proofs, or wallet boundaries;
- `$daarion-reticulum-lxmf` only for transport, sidecar/daemon, IPC envelope, mailbox, or Reticulum/LXMF phases;
- `$daarion-supabase-contracts` only when the edge phase directly changes a Supabase Auth, schema, RLS, RPC, pairing, membership, revocation, or projection contract;
- `$daarion-cross-repo-contracts` for contracts shared with `loval-echoes`;
- `$daarion-repository-auditor` for baseline, readiness, mock, TODO, or documentation-consistency audits.

For a Loop Runtime foundation phase, explicitly use this complete bundle:
`$daarion-phase-planner`, `$daarion-repository-auditor`,
`$daarion-rust-tauri`, `$daarion-agent-runtime`,
`$daarion-loop-runtime`, `$daarion-memory`, `$daarion-local-llm`,
`$daarion-tool-security`, `$daarion-testing`,
`$daarion-security-review`, `$daarion-documentation`, and
`$daarion-release-gate`. This bundle defines boundaries and checks; it does not
authorize model inference, tool execution, network access, dependency changes, or
application implementation outside the approved phase plan.

## Change and release gates

- Do not add a production dependency without explicit justification in the phase plan, architecture impact review, and lockfile review.
- Do not perform destructive migrations. Local schema changes require reversible migrations and compatibility/rollback tests.
- Do not deploy, publish, change repository settings, or open a pull request automatically.
- Validate all frontend-to-Rust and IPC inputs. Protect filesystem, shell, process, network, microphone, model-download, worker, and signing surfaces with explicit policy.
- Avoid oversized composition modules, unrestricted shell execution, unbounded retries or recursion, `unwrap()`/panic on untrusted paths, and optimistic readiness claims.
- Do not claim a feature is implemented because documentation, naming, mocks, or simulated states describe it.
- If a required check cannot run, record the exact blocker and safest verification command; the release gate cannot be `PASS`.
