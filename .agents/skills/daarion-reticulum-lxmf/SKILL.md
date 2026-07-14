---
name: daarion-reticulum-lxmf
description: "Use only for DAARION transport abstraction, Reticulum/LXMF, sidecar or daemon, authenticated IPC, message envelope, offline mailbox, or mesh packaging phases."
---

# DAARION Reticulum and LXMF

Follow the mandatory lifecycle in the active `AGENTS.md`: require a reviewed baseline audit and written phase plan with an allowed verdict before implementation, then require tests, security review, full diff review, documentation, a completion report, and the release gate. Never claim completion when a required check failed.

Do not activate this skill for phases unrelated to transport.

- Keep transport behind a generic interface so the Agent Supervisor does not depend on Reticulum implementation details.
- Compare a bundled Python sidecar with a separate `daarion-meshd` daemon for the concrete phase. Prefer the separate daemon as the long-term direction unless an accepted ADR selects another approach; do not treat that preference as implemented architecture.
- Require mutually authenticated local IPC, least-privilege endpoints, peer/process identity, secure lifecycle, version negotiation, and resistance to sidecar replacement or IPC spoofing.
- Define versioned message envelopes with message ID, sender identity, signature, audience, creation time, expiry, replay protection, idempotency, payload type, and bounded size.
- Separate routing, offline queue, mailbox persistence, capability announcements, and application authorization.
- Analyze packaging, updates, background execution, and permissions for Windows, macOS, Linux, and Android.
- Keep live IP, DNS, firewall, Octelium, NODA, and private topology out of public files. Octelium is an optional secure-access layer, not a Reticulum replacement.

Do not implement transport during an unrelated phase. Require an ADR, threat model, IPC contract tests, envelope test vectors, replay/expiry tests, and upgrade/rollback analysis before production integration.
