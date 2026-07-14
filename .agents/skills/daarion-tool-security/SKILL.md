---
name: daarion-tool-security
description: "Use for DAARION tools, shell/process/filesystem/network access, model downloads, microphone, worker execution, permission gates, or user confirmations."
---

# DAARION Tool Security

Follow the mandatory lifecycle in the active `AGENTS.md`: require a reviewed baseline audit and written phase plan with an allowed verdict before implementation, then require tests, security review, full diff review, documentation, a completion report, and the release gate. Never claim completion when a required check failed.

Every tool must declare one primary risk class:

- `READ_ONLY`
- `LOCAL_MUTATION`
- `EXTERNAL_COMMUNICATION`
- `FINANCIAL`
- `PRIVILEGED_SYSTEM`

Register tools with typed input/output schemas, explicit purpose, allowlisted arguments, path/network scope, timeout, cancellation, and audit policy. Normalize and authorize paths after canonicalization; avoid time-of-check/time-of-use assumptions. Scrub environment variables and child-process inheritance.

Prohibit unrestricted shell access, raw command strings from the model, arbitrary executable selection, uncontrolled network egress, and implicit privilege escalation. Sensitive classes require visible user confirmation with the exact effect; financial and privileged actions must fail closed. Revalidate permissions at execution time, not only during planning.

Audit request, policy decision, approval, bounded result metadata, and error without logging secrets. Validate tool output before returning it to the model. Test injection, traversal, symlink, environment leakage, egress denial, timeout, cancellation, approval denial, replay, and partial failure.
