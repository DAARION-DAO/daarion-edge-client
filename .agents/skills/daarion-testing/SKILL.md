---
name: daarion-testing
description: "Use in every DAARION implementation phase to derive unit, integration, contract, and end-to-end verification from acceptance criteria and record evidence."
---

# DAARION Testing

Translate each acceptance criterion into executable evidence or an explicit manual check with an owner. Select unit, integration, contract, end-to-end, migration, native, and security tests according to the changed boundary.

- Cover success plus denial, malformed input, timeout, cancellation, retry, partial failure, restart, and recovery paths.
- Require explicit tests that local-only policy rejects remote inference and prevents prompt egress.
- Maintain sanitized versioned fixtures for cross-repository contracts and test both producer and consumer.
- Distinguish real adapters, controlled fakes, fixtures, and mocks. A test that only asserts mocked success cannot verify the claimed capability.
- Run the narrowest relevant checks first, then broader repository lint/typecheck/build/test checks.
- Do not repair unrelated failures silently; identify whether they are pre-existing and how they affect the gate.

Record every command, environment assumption, exit result, skipped check, and failure in `docs/planning/phases/<phase-id>-completion.md`. Never state that tests passed when they did not run.
