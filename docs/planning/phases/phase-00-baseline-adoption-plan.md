# Phase 00: Sovereign Agent Baseline Adoption Plan

## Objective

Adopt the human-approved 2026-07-04 sovereign-agent audit decision as a documentation baseline for `daarion-edge-client` and its contract with `loval-echoes`. Establish canonical ownership, claim-status language, security gates, ADRs, and a phased roadmap without changing application behavior.

## Current State

- The repository is on `docs/repository-role-card` at commit `e25c41e73298ece63261302812c1a4b800bdad38`, authored 2026-07-04.
- Tracked files are clean. Existing untracked repository instructions, DAARION skills, roadmap, and phase reports are preserved as prior documentation work.
- `docs/REPOSITORY_ROLE.md` identifies this repository as the local Tauri/Rust device runtime.
- Source evidence includes partial local identity and Ollama integration, plus simulated or placeholder model lifecycle, remote inference, messaging, planning, worker, and wallet paths. No durable SQLite runtime-state foundation or Reticulum/LXMF integration was found.
- The complete original audit report is not present in the checked repositories or supplied attachments. The supplied human approval and current source evidence can be adopted, but missing audit prose must not be reconstructed or presented as recovered evidence.

## Scope

- Create the canonical 2026-07-04 baseline audit adoption record.
- Document target architecture, system context, ownership, public/private boundaries, capability status, claim status, threat model, security gates, roadmap, and open questions.
- Record ADRs for local-first inference, SQLite runtime-state foundation, and the Reticulum/LXMF integration boundary.
- Split the first implementation milestone into Phase 1A, 1B, and 1C.
- Define deterministic, bounded loop runtime as a later foundation before transport and multi-agent loops.
- Cross-link relevant existing repository-role and release-truth documents.

## Explicit Non-Goals

- No Rust, TypeScript, React, Tauri configuration, dependency, CI, deployment, or repository-setting changes.
- No inference, SQLite, Agent Supervisor, loop runtime, tools, identity, wallet, worker, Reticulum/LXMF, pairing, or readiness implementation.
- No remote service, deployed backend, native package, or production-readiness verification.
- No claim that the unavailable full audit artifact was recovered.

## Repository Ownership

`daarion-edge-client` owns the local Tauri/Rust runtime, device and agent identities, secure local key storage, pairing consumption, capability detection, local model lifecycle, local inference, Agent Supervisor, durable local state and memory, permission-controlled tools, safe projections, transport abstraction, and security-gated worker execution.

`loval-echoes` owns the authenticated product/control experience and must not receive raw local memory, private keys, unrestricted tool access, wallet signing authority, or transport-runtime internals. Cross-repository contracts remain explicit and versioned.

## Files and Modules Expected to Change

Documentation only:

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
- `docs/planning/phases/phase-00-baseline-adoption-completion.md`

No application module is expected to change.

## Contracts Affected

No executable contract changes. Documentation will define future requirements for an `ExecutionPolicy`, provider abstraction, durable runtime-state store, bounded Supervisor/loop state, versioned pairing and readiness schemas, and authenticated transport IPC. Existing behavior remains classified from source evidence rather than upgraded by documentation.

## Security Considerations

- Default Edge inference policy must be local-only; remote inference cannot be silent.
- Private keys, mnemonic material, raw memory, private prompts, unrestricted tool access, and wallet signing must remain outside LLM context and web projections.
- Document gates for pairing, tools, transport IPC, wallet signing, worker leases, model artifacts, and autonomous loops.
- Treat every external input as untrusted data and distinguish confirmed evidence from hypotheses.
- Preserve explicit stop conditions, budgets, checkpoints, idempotency, and approval states for future loops.

## Migration and Compatibility Considerations

This phase contains no migrations and makes no compatibility change. Future SQLite or contract/schema work requires versioned migrations, recovery and rollback design, cross-repository fixtures, and separate human review.

## Implementation Steps

1. Capture repository snapshot and evidence limitations.
2. Create common status vocabularies and apply them consistently.
3. Document repository ownership and trust boundaries.
4. Create the capability matrix and claim-drift table from verified source paths.
5. Record the target architecture and phased roadmap, including 1A/1B/1C and bounded loop runtime.
6. Record threat model, security gates, and ADR decisions.
7. Cross-link existing role/release documents without promoting placeholders to verified capabilities.
8. Validate paths, links, status vocabulary, scope, sensitive-data exclusions, and diff boundaries.
9. Produce the completion report and release-gate result.

## Tests

- Verify every required document exists.
- Verify required capability and claim-status tokens are used consistently.
- Verify ADR numbers are unique and all ADR links resolve.
- Search changed documentation for secrets, private-key material, credentials, machine-specific paths, and private infrastructure details.
- Confirm `git diff --name-only` contains documentation/instruction paths only and no application, dependency, CI, or deployment file.
- Run repository-provided documentation-safe or security-safe checks only where they do not install dependencies or modify application state.

## Acceptance Criteria

- All required baseline documents exist and agree on ownership, statuses, and phase ordering.
- Findings cite exact repository paths and symbols where available.
- Mocks, placeholders, documentation claims, and executable behavior remain distinct.
- The missing original audit artifact is disclosed as a provenance limitation.
- Phase 1A, 1B, and 1C are separate and no later phase is authorized.
- Application implementation remains `NO_GO`; only the narrowly defined Phase 1A may proceed after separate plan and human review.
- Documentation release gate returns `PASS` or a fully explained `CONDITIONAL_PASS`; required-check failure cannot be reported as `PASS`.

## Rollback Strategy

Remove only the new Phase 00 documentation files after review. No source, schema, dependency, runtime state, or deployment rollback is required.

## Documentation Updates

This phase is documentation-only. The documents listed above become the reviewable baseline; existing `README` claims are inventoried for later correction rather than modified unless a contradiction would otherwise make the new baseline unsafe.

## Open Questions

- Where is the complete approved audit report, if it must be preserved verbatim as a separate immutable artifact?
- What is the canonical versioned device-pairing schema shared by both repositories?
- What exact safe readiness projection fields may leave the device?
- Which future ADR will separately decide pairing and readiness projection contracts?
- Which platforms and secure-storage guarantees are mandatory for the first Edge release?

## GO / CONDITIONAL_GO / NO_GO

`CONDITIONAL_GO` for this documentation-only baseline adoption. Conditions: preserve the missing-audit-artifact limitation, make no application changes, and do not reinterpret documentation as production verification. Application implementation remains `NO_GO`; a separate human-reviewed Phase 1A plan is the only eligible next runtime milestone.
