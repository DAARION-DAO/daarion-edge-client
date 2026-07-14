---
name: daarion-agent-runtime
description: "Use for DAARION Agent Supervisor, planner, executor, verifier, context builder, task/session state, retries, cancellation, or audit-event work."
---

# DAARION Agent Runtime

Follow the mandatory lifecycle in the active `AGENTS.md`: require a reviewed baseline audit and written phase plan with an allowed verdict before implementation, then require tests, security review, full diff review, documentation, a completion report, and the release gate. Never claim completion when a required check failed.

Loops are versioned workflows managed by the Agent Supervisor, not additional
agent identities. When work includes triggers, schedules, repeated steps,
checkpoints, stop control, restart/resume, or maintenance workflows, also use
`$daarion-loop-runtime`.

- Model tasks and sessions with explicit persisted state machines and legal transitions.
- Separate Supervisor, planner, executor, verifier, context builder, policy evaluation, and audit emission. Named modules do not count as functional until their execution paths are traced.
- Generate deterministic task IDs where possible and require idempotency for retries, recovery, tool effects, and message handling.
- Bound recursion, steps, retries, context, cost, and elapsed work. Support cancellation, deadlines, crash recovery, and terminal failure states.
- Require schema-validated structured decisions with provenance. Treat model output and retrieved memory as untrusted inputs.
- Route all tool requests through the permission runtime and all signing through an isolated signing boundary. The LLM never receives private keys or direct privileged execution authority.
- Emit append-oriented audit events for policy, decisions, approvals, effects, verification, and recovery without logging sensitive payloads.

Test state transitions, idempotent retry, cancellation, verifier rejection, malformed decisions, policy denial, restart recovery, and bounded-loop behavior. Do not label a prompt wrapper as an autonomous Agent Supervisor.
