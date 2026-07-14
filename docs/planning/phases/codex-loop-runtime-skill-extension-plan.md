# Loop Runtime Skill Extension Plan

Phase ID: `codex-loop-runtime-skill-extension`

## Objective

Extend the repository-scoped Codex system with a bounded, durable,
policy-controlled Loop Runtime skill and an audit-gated sovereign-agent roadmap,
without implementing application runtime behavior.

## Current State

- The repository has the original mirrored set of 15 DAARION skills.
- Existing `heartbeat`, `messaging`, and worker background loops are specialized
  paths, not a verified general Agent Supervisor or durable Loop Runtime.
- No `LoopDefinition`, `LoopSupervisor`, durable checkpoint store, or SQLite
  persistence dependency was found in the inspected edge source/manifests.
- No completed, human-reviewed sovereign-agent baseline audit is proven in this
  checkout.

## Scope

- Add `.agents/skills/daarion-loop-runtime/SKILL.md` with explicit trigger rules.
- Keep the skill identical in both sibling repositories.
- Update repository instructions and the agent-runtime skill to preserve loop
  ownership and require the new skill when relevant.
- Add a sanitized, proposed, audit-gated roadmap with Loop Runtime before tools
  and transport.
- Add a completion report and mark the original 15-skill report as a historical
  setup snapshot.

## Explicit Non-Goals

- No Rust, TypeScript, Tauri, SQLite, scheduler, database, or UI implementation.
- No production dependency or migration changes.
- No implementation of Agent Supervisor, health loop, memory, tools,
  Reticulum/LXMF, wallet, or multi-agent behavior.
- No deployment, commit, push, pull request, or infrastructure change.

## Repository Ownership

`daarion-edge-client` owns the future local Loop Runtime, definitions, run state,
checkpoints, policy enforcement, private audit trail, and safe projection
generation. `loval-echoes` may consume only versioned safe summaries and expose
user controls.

## Files and Modules Expected to Change

- `AGENTS.md`
- `.agents/skills/daarion-loop-runtime/SKILL.md`
- `.agents/skills/daarion-agent-runtime/SKILL.md`
- `docs/planning/SOVEREIGN_AGENT_ROADMAP.md`
- `docs/planning/phases/codex-skills-system-setup-completion.md`
- `docs/planning/phases/codex-loop-runtime-skill-extension-plan.md`
- `docs/planning/phases/codex-loop-runtime-skill-extension-completion.md`

## Contracts Affected

No runtime contract changes. The roadmap reserves a future safe-summary contract
from edge to web; its schema must be versioned and audited in its own phase.

## Security Considerations

The new skill must prohibit unbounded autonomy, implicit instructions from
external input, silent failures, unrestricted effects, remote inference, and
destructive actions without approval. Every run must have limits, policy checks,
idempotency, durable checkpoints, explicit outcomes, and a sanitized audit trail.

## Migration and Compatibility Considerations

No migrations are performed. The skill may require SQLite and versioned migrations
for a future authorized phase, but it must not select a dependency or schema in
this metadata-only extension.

## Implementation Steps

1. Add this plan before other extension edits.
2. Add and mirror the Loop Runtime skill.
3. Update root ownership/selection rules and the Agent Runtime skill.
4. Add the proposed audit-gated roadmap.
5. Validate metadata, content requirements, cross-repository mirroring, scope,
   formatting, and sensitive-content rules.
6. Run a fresh read-only Codex discovery/selection simulation if practical.
7. Review the complete diff and write the completion report.

## Tests

- Parse all skill front matter and assert 16 expected unique names.
- Assert root skill references resolve and edge references the loop skill.
- Assert required bounded-loop, checkpoint, policy, idempotency, approval,
  outcome, recovery, and audit rules are present.
- Assert both skill trees are identical.
- Assert only approved documentation/metadata paths changed.
- Scan all added/updated content for credential-shaped or private infrastructure
  evidence without printing values.

## Acceptance Criteria

- `daarion-loop-runtime` exists with an explicit, narrow trigger in both repos.
- Edge owns loop execution; web owns only controls and safe projections.
- The roadmap orders memory before Loop Runtime and Loop Runtime before tools and
  Reticulum/LXMF.
- The future health-loop scope is strictly local and read-only.
- Application implementation remains blocked pending the reviewed baseline audit.
- All required metadata and security checks pass.

## Rollback Strategy

Revert only the files listed above. No runtime or data rollback is needed because
the phase makes no application or persistence changes.

## Documentation Updates

Add the proposed roadmap, extension completion report, and a historical-snapshot
note to the original setup completion report.

## Open Questions

- The canonical baseline audit and human-review evidence remain unresolved.
- SQLite crate, migration tooling, exact schema, scheduler integration, and
  platform lifecycle behavior must be selected during audited implementation
  planning, not in this skill extension.

## GO / CONDITIONAL_GO / NO_GO

`GO` for repository instruction, skill, roadmap, and completion documentation
only. `NO_GO` for Loop Runtime application implementation.
