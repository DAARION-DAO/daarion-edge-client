# Phase 00: Sovereign Agent Baseline Adoption Completion

Final status: **CONDITIONAL_PASS**

## Human-review status

- The baseline audit was reviewed by the human project owner.
- The capability classifications were accepted as the implementation baseline.
- The audit content supplied in the review conversation and its canonical
  repository transcription are accepted as the effective baseline artifact; a
  separate original file is not a blocker for Phase 1A planning.
- `CONDITIONAL_GO` applies only to planning Phase 1A. Application code still
  requires a separate phase plan and human review before implementation.
- Remote branches beyond the dated snapshot, live Supabase schema, deployed PWA,
  Edge Backend, and other deployed environments remain unverified by the audit.
- No production-readiness claim is made.

## Outcome

The human-approved 2026-07-04 sovereign-agent audit decision is now represented as a mirrored repository baseline in `loval-echoes` and `daarion-edge-client`. The phase changed documentation and repository instructions only. No runtime, application, schema, dependency, CI, deployment, or repository-setting implementation occurred.

Human review closes the previously recorded audit-artifact availability concern
as a process blocker. `CONDITIONAL_PASS` remains because the adopted audit is a
dated source snapshot and remote/deployed systems were not verified. These are
explicit evidence limitations, not hidden validation failures, and they do not
authorize a production-readiness claim.

## Files created in both repositories

- `docs/audits/SOVEREIGN_AGENT_BASELINE_AUDIT_2026-07-04.md`
- `docs/architecture/SOVEREIGN_AGENT_TARGET_ARCHITECTURE.md`
- `docs/architecture/SYSTEM_CONTEXT_AND_OWNERSHIP.md`
- `docs/architecture/CAPABILITY_STATUS_MATRIX.md`
- `docs/architecture/PUBLIC_PRIVATE_BOUNDARIES.md`
- `docs/planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md`
- `docs/planning/OPEN_QUESTIONS.md`
- `docs/security/THREAT_MODEL.md`
- `docs/security/SECURITY_GATES.md`
- `docs/adr/0001-local-first-inference-and-remote-consent.md`
- `docs/adr/0002-local-runtime-state-and-sqlite-foundation.md`
- `docs/adr/0003-reticulum-lxmf-integration-boundary.md`
- `docs/planning/phases/phase-00-baseline-adoption-plan.md`
- `docs/planning/phases/phase-00-baseline-adoption-completion.md`

## Files updated in both repositories

- `AGENTS.md`: added the canonical baseline links, current runtime gate, and separate pairing/readiness ADR requirement.
- `docs/planning/SOVEREIGN_AGENT_ROADMAP.md`: marked the older view as superseded by the master roadmap and corrected the current gate/provenance statement.

The existing 16 repository-scoped DAARION skills were present and valid for this task. No skill file was added or modified.

## Adopted audit decisions

- Web product/control ownership stays in `loval-echoes`.
- Local sovereign enforcement/runtime ownership stays in `daarion-edge-client`.
- Web cloud-assisted AI and Edge local inference are separate trust domains.
- `InferencePolicy::LocalOnly` is the only enabled Edge MVP policy.
- Provider abstraction precedes additional inference backends.
- SQLite is the first local runtime-state foundation, not complete six-level memory.
- An inert Supervisor follows durable state and precedes tools/network/autonomous scheduling.
- Loops are bounded, deterministic, checkpointed, idempotent and policy-controlled.
- Transport abstraction precedes Reticulum/LXMF.
- A separate authenticated `daarion-meshd` is the preferred long-term desktop direction; bundled sidecar is prototype-only.
- Pairing and readiness projections require separate future threat-driven ADRs.
- Wallet, worker, tools, model artifacts, mesh and autonomous loops remain security-gated.

## Roadmap corrections

The former combined first milestone was split into:

- Phase 1A — local-only inference foundation;
- Phase 1B — durable runtime state;
- Phase 1C — inert Agent Supervisor.

Phase 1B includes only migrations, conversations, messages, tasks and audit events. It is not described as six-level memory.

The next dependency order is:

```text
baseline
-> Phase 1A inference
-> review
-> Phase 1B state
-> review
-> Phase 1C inert Supervisor
-> review
-> six-level memory
-> bounded Loop Runtime
-> tools
-> readiness
-> Reticulum/LXMF
-> multi-agent
-> wallet/economy
-> hardening
```

## Claim corrections recorded

The baseline claim-drift table records, without broadly rewriting README/UI files:

- native GGUF execution claim is false/stale relative to simulated/unimplemented paths;
- `llama.cpp ... via /api/chat` is a false/stale runtime label for the Ollama HTTP path;
- complete local-inference privacy is partial until `LocalOnly`, timeout and cancellation are enforced;
- identity is partial rather than lifecycle-complete;
- wallet is security-gated and not production-ready;
- messaging is stubbed/local-only rather than Matrix/NATS;
- worker sandbox readiness is unverified/security-gated;
- older “device pairing/status missing” documentation is stale because later UI/RPC/migration source exists, but end-to-end security remains partial;
- web cloud AI is not Edge local inference;
- Reticulum/LXMF and bounded autonomous loops are not implemented.

## Security review result

The threat model and gates explicitly block:

- production pairing until signature, binding, expiry, replay, single use and revocation are verified;
- tool execution until a typed permission broker exists;
- Reticulum/LXMF until authenticated IPC and signed envelopes exist;
- wallet until signer isolation, derivation/rotation/recovery and approval exist;
- worker until signed leases and sandbox hardening exist;
- managed model downloads until manifest and artifact verification exist;
- autonomous loops until bounded runtime and persistent checkpoints exist.

High/Critical findings are not reclassified as remediated. They are out of implementation scope for this documentation phase and remain entry gates for their owning phases.

## Validation commands and results

| Validation | Result |
| --- | --- |
| Required-file existence checks | PASS in both repositories |
| Local Markdown-link resolver | PASS in both repositories |
| ADR number uniqueness | PASS in both repositories |
| Capability-status vocabulary allowlist | PASS in both repositories |
| Claim-status vocabulary allowlist | PASS in both repositories |
| `AGENTS.md` skill-reference resolution | PASS in both repositories |
| Repo-prefixed evidence-path existence check | PASS in both repositories |
| Sensitive value and machine-specific path scan | PASS in both repositories |
| Trailing-whitespace scan | PASS in both repositories |
| `git diff --check` | PASS in both repositories |
| Changed-path allowlist review | PASS; only `AGENTS.md`, `.agents/**` prior files, and `docs/**` are present |
| Mirrored canonical-document `cmp` checks | PASS for all 12 shared baseline documents |
| `npm run security:check` in `loval-echoes` | PASS |
| `bash scripts/check-no-secrets.sh` in `daarion-edge-client` | PASS |
| Complete `git status --short -uall` scope review | PASS; no application/source/schema/dependency/CI/deployment paths changed |

Application lint, build, Rust tests, frontend tests, migrations, deployments, remote fetches, live service checks, Ollama calls and package installation were intentionally not run because the approved task required documentation-safe checks only and changed no application code.

## Unresolved questions

The canonical list is `docs/planning/OPEN_QUESTIONS.md`. Immediate blockers for later work include:

- exact provider-neutral command/model contract for Phase 1A;
- pairing trust anchors and offline/online verification semantics;
- readiness projection allowlist, signer and freshness model;
- platform SQLite protection requirements;
- identity-domain rotation/recovery;
- desktop IPC and Android background-execution decisions.

## Runtime-change confirmation

Confirmed absent from this phase:

- Rust, TypeScript, React, SQL migration and Edge Function changes;
- production dependency or lockfile changes;
- CI/deployment/repository-setting changes;
- inference, SQLite, Supervisor, memory, loop, tools, pairing/readiness, Reticulum/LXMF, wallet or worker implementation;
- network/deployment operations;
- secrets or private infrastructure truth.

## Recommendation for Phase 1A

Create a separate `docs/planning/phases/phase-01a-local-only-inference-plan.md` after human review. Limit it to:

- provider-neutral `InferenceProvider`;
- Ollama adapter;
- only-enabled `LocalOnly` policy;
- unreachable/removed simulated remote fallback;
- command/model-ID mapping;
- timeout and cancellation;
- truthful UI;
- fake-provider and no-egress tests.

Do not include SQLite, Supervisor, loops, tools, pairing/readiness, transport, wallet or worker changes in Phase 1A.

## Release gate

**CONDITIONAL_PASS**

Conditions carried forward:

1. Preserve the dated-snapshot and remote/deployed verification limitations.
2. Do not treat repository evidence as deployed or production proof.
3. Obtain human review before executing the Phase 1A implementation plan.
4. Keep Phase 1B and later work at `NO_GO` until separately planned and authorized.
