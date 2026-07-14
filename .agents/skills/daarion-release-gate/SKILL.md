---
name: daarion-release-gate
description: "Use at the end of every DAARION implementation phase to review git state, diff, checks, security, documentation, and acceptance evidence and issue PASS, CONDITIONAL_PASS, or FAIL."
---

# DAARION Release Gate

Run only after implementation, testing, security review, and documentation updates.

1. Confirm the repository/worktree identity, branch, base, and clean or fully understood pre-existing git state.
2. Review the complete diff for scope, regressions, accidental generated files, dependency/lockfile changes, destructive migration, and repository-boundary violations.
3. Run repository-required formatting, lint, typecheck, unit/integration/contract/security tests, and builds relevant to the phase.
4. Verify no credentials, keys, tokens, private endpoints, infrastructure details, raw private memory, or unsafe logs were added.
5. Map every acceptance criterion to evidence and confirm architecture/ADR/roadmap/README and completion-report updates.
6. Verify rollback/compatibility notes and list every skipped, blocked, or failed check.

Write the evidence and verdict to `docs/planning/phases/<phase-id>-completion.md`:

- `PASS`: all required criteria and checks passed, with evidence.
- `CONDITIONAL_PASS`: no unresolved critical/high security blocker, but named non-critical conditions or unavailable non-required evidence remain.
- `FAIL`: any required check failed, required evidence is absent, scope/boundary is violated, or a critical/high security blocker remains.

Never issue `PASS` when a required check failed, did not run, or was replaced by a mock. Do not deploy, publish, merge, commit, push, or open a pull request unless separately authorized.
