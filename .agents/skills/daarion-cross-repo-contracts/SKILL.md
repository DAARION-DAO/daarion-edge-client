---
name: daarion-cross-repo-contracts
description: "Use when work crosses loval-echoes and daarion-edge-client through pairing, deep links, shared schemas, enums, revocation, or readiness projections."
---

# DAARION Cross-Repository Contracts

Follow the mandatory lifecycle in the active `AGENTS.md`: require a reviewed baseline audit and written phase plan with an allowed verdict before implementation, then require tests, security review, full diff review, documentation, a completion report, and the release gate. Never claim completion when a required check failed.

Use only when a phase affects a contract between the product web layer and the local edge runtime.

1. Identify producer, consumer, authority, transport/handoff, schema owner, and current implementation on both sides.
2. Build a source-to-destination matrix with field meanings, version, validation, authentication, expiry, revocation, compatibility, and evidence paths.
3. Detect duplicated types, enum/name drift, optionality differences, stale fixtures, incompatible versions, and unsafe trust assumptions.
4. Define a versioned schema and sanitized shared fixtures without sharing secrets, private keys, raw local memory, or infrastructure configuration.

A community or membership invitation must never be reused as device-pairing authorization. Privileged payloads must be single-purpose and either cryptographically signed and verified or verified by an authoritative backend. Pairing requires short lifetime, single-use behavior, replay resistance, revocation, intended-user/MicroDAO binding, and explicit failure states.

The web repository owns invitation UX and safe projections; the edge repository owns local consumption, validation, identity initialization, and private state. Require contract tests on both sides and document rollout/backward compatibility. Never invent an absent backend contract.
