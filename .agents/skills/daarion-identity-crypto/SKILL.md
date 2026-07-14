---
name: daarion-identity-crypto
description: "Use for DAARION keys, signatures, device/agent/pairing/session identity, secure storage, rotation, revocation, recovery, or wallet boundaries."
---

# DAARION Identity and Cryptography

Follow the mandatory lifecycle in the active `AGENTS.md`: require a reviewed baseline audit and written phase plan with an allowed verdict before implementation, then require tests, security review, full diff review, documentation, a completion report, and the release gate. Never claim completion when a required check failed.

Treat cryptographic work as security-sensitive and ADR-gated.

- Separate device-root, agent, Reticulum/transport, pairing, wallet, and session identities. Do not reuse a key across incompatible domains.
- Define algorithms, key purpose, domain-separated signing payloads, canonical serialization, verification authority, lifetime, storage, rotation, revocation, and recovery.
- Generate and use private keys inside the narrowest trusted boundary. Keep them out of LLM context, prompts, logs, frontend state, crash reports, fixtures, and unrestricted process environments.
- Use OS secure storage where supported and state platform limitations honestly. Never simulate secure storage as production-ready.
- Verify signatures, expiry, audience/purpose, nonce/message ID, and revocation before privileged effects.
- Isolate wallet signing behind an explicit user approval boundary. The LLM may propose a transaction but cannot access key material or invoke unrestricted signing.

Require review of dependencies and side channels, deterministic test vectors that contain no real secrets, negative verification tests, rotation/revocation/recovery tests, and compatibility analysis. Do not invent wallet support when only identity terminology exists.
