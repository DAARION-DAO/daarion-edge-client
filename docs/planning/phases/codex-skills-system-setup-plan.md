# Codex Skills System Setup Plan

Phase ID: `codex-skills-system-setup`

## Objective

Create a repository-scoped, reviewable Codex instruction and skill system for
bounded end-to-end DAARION phases without changing application behavior.

## Current State

- The repository has a sanitized role card but no root `AGENTS.md`.
- `.agents/skills/` and `docs/planning/phases/` do not exist.
- `daarion-edge-client` is the Tauri/Rust local sovereign runtime boundary.
- The referenced `Архітектура агента на Reticulum.txt` was not found in the
  accessible workspace and is therefore not used as evidence.

## Scope

- Add root repository instructions.
- Add the fifteen requested DAARION skills under `.agents/skills/`.
- Add this plan and a completion report.
- Validate skill metadata, references, boundaries, and sanitized content.

## Explicit Non-Goals

- No application source changes.
- No dependency, migration, deployment, database, or infrastructure changes.
- No live endpoint, credential, private operations, NODA, DNS, firewall, or
  Octelium truth.
- No implementation of inference, memory, tools, wallet, worker, or transport.

## Repository Ownership

This repository owns the native runtime, device and agent identity, secure local
storage, pairing consumption, capability detection, local inference, Agent
Supervisor, durable memory, gated tools, safe projections, transport abstraction,
and future security-gated worker behavior. Reticulum/LXMF implementation remains
behind a sidecar or daemon boundary selected by ADR.

## Files and Modules Expected to Change

- `AGENTS.md`
- `.agents/skills/*/SKILL.md`
- `docs/planning/phases/codex-skills-system-setup-plan.md`
- `docs/planning/phases/codex-skills-system-setup-completion.md`

## Contracts Affected

No runtime contract changes. The new instructions will require future phases to
version and test cross-repository schemas before implementation.

## Security Considerations

All content must be sanitized. Skills must prohibit secret disclosure, silent
remote inference, unsafe privileged payloads, unrestricted tools, direct LLM key
access, and unverified completion claims.

## Migration and Compatibility Considerations

No migrations. Skills use portable Markdown front matter and repository-relative
paths. The complete set is mirrored in both repositories so sibling checkouts have
the same names and workflow semantics.

## Implementation Steps

1. Add this plan before any other repository edits.
2. Add tailored root instructions.
3. Add all requested skill entrypoints with unique names and explicit triggers.
4. Validate front matter, references, boundary language, and leakage patterns.
5. Simulate skill selection for the local-only inference foundation.
6. Review the diff and write the completion report.

## Tests

- Parse every `SKILL.md` front matter block.
- Assert unique and expected skill names.
- Assert all `AGENTS.md` skill references resolve.
- Scan added files for prohibited secret/private-infrastructure patterns.
- Check whitespace and repository status.
- Read-only simulation of the requested local-only inference phase.

## Acceptance Criteria

- Fifteen valid skills exist in this repository and its sibling.
- Root instructions enforce architecture ownership and mandatory phase planning.
- No application, migration, dependency, or deployment file changes.
- Validation passes or limitations are recorded without a false PASS.

## Rollback Strategy

Revert only the files listed in this plan. No runtime or persistent data rollback
is required because the phase changes documentation and Codex metadata only.

## Documentation Updates

Add the setup completion report. Future application phases must create their own
plan and completion files in this directory.

## Open Questions

- A fresh Codex process is required to prove automatic discovery from disk.
- The missing Reticulum architecture text must be reviewed separately if it is
  intended to add or override public architectural decisions.

## GO / CONDITIONAL_GO / NO_GO

`GO` for repository-scoped instruction and skill files only.
