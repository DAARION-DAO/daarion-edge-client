---
name: daarion-phase-planner
description: "Use before any DAARION implementation phase to inspect repository truth and write the required bounded phase plan and GO gate before code changes."
---

# DAARION Phase Planner

Use this as the mandatory entry point for every implementation phase.

1. Read every active `AGENTS.md`, the repository `README`, relevant docs/manifests/configuration, and all domain skills needed by the phase.
2. Inspect source, tests, git status, existing contracts, mocks, TODOs, and claimed behavior. Separate verified facts, inferences, and recommendations.
3. Confirm the applicable baseline audit exists, covers the current repository state, and has completed human review. If it does not, plan the missing audit and issue `NO_GO` for application implementation.
4. Choose a stable `<phase-id>` and create `docs/planning/phases/<phase-id>-plan.md` before changing application code.
5. Bound one end-to-end vertical slice. Record adjacent work as non-goals rather than silently expanding scope.

The plan must contain these sections:

- Objective
- Current State
- Scope
- Explicit Non-Goals
- Repository Ownership
- Files and Modules Expected to Change
- Contracts Affected
- Security Considerations
- Migration and Compatibility Considerations
- Implementation Steps
- Tests
- Acceptance Criteria
- Rollback Strategy
- Documentation Updates
- Open Questions
- GO / CONDITIONAL_GO / NO_GO

List assumptions, risks, blockers, dependencies, security gates, exact acceptance evidence, and the narrowest relevant commands. A `CONDITIONAL_GO` must name every condition and may not hide an unresolved security blocker.

Do not modify application code until the plan exists. Stop at the plan when the verdict is `NO_GO`, when repository ownership is unresolved, or when required authority would expand the task. After implementation, require a matching `docs/planning/phases/<phase-id>-completion.md` and the release gate.
