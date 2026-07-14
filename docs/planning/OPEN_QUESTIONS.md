# Sovereign Agent Open Questions

Status: **UNRESOLVED ITEMS NOT PROVABLE FROM THE AUDITED SNAPSHOTS**

These questions require a human product/security decision, deployed-system evidence, or a later bounded phase. They are not permission to infer an answer.

## Audit provenance

1. Where should the complete original approved audit report be preserved, if it exists outside the supplied artifacts?
2. Must the baseline snapshots be refreshed against remote branches before Phase 1A, and which commits are canonical?

## Phase 1A

3. Which frontend command and model registry fields are the canonical provider-neutral API?
4. Which Ollama versions/platforms are supported for MVP?
5. What cancellation UX is required when a model ignores or delays cancellation?
6. Is loopback traffic always considered local, and what validation prevents proxy/environment redirection?
7. What exact user-facing disclosure is required for web cloud-AI features versus Edge local inference?

## Pairing — future ADR 0004

8. What backend/device keys are trust anchors for the pairing envelope?
9. Which fields are cryptographically bound: user, MicroDAO, role, purpose, backend profile, device nonce and platform?
10. Is consumption online-only, or can Edge verify an offline invitation?
11. How are single use, replay cache, expiry, revocation and device transfer reconciled?
12. Which schema downgrade behavior is safe?

## Readiness — future ADR 0005

13. Which fields are allowed in a public/product readiness projection?
14. Which Edge identity signs it, and how does the backend validate rotation/revocation?
15. What are freshness, expiry and offline semantics?
16. Can a user revoke all projections and delete historical summaries?
17. Which projection fields are product truth versus diagnostic hints?

## Durable state and memory

18. Which SQLite encryption/OS-at-rest guarantees are required per platform?
19. What are default retention periods for conversations, task state and audit events?
20. What export format and secure deletion semantics are required?
21. Which memories require human confirmation before consolidation?
22. Which local embedding model and dimensionality, if any, will be selected later?

## Identity and wallet

23. How are device-root, agent, pairing, transport, session and wallet keys derived or generated independently?
24. What rotation, recovery and revocation user journey is required?
25. Which chains/wallet standards are in product scope, if any?
26. Is hardware-backed signing mandatory on supported platforms?

## Loop Runtime and tools

27. What default budgets apply per loop class?
28. Which terminal states are user-visible and which are internal?
29. What approval channel remains safe after restart?
30. Which first read-only tools are in scope, and what path/network allowlists do they use?
31. How is audit-log tamper evidence implemented without exposing private content?

## Reticulum/LXMF

32. Which Reticulum/LXMF versions and licenses are acceptable?
33. Is `daarion-meshd` a new repository or a packaged component owned by Edge?
34. What authenticated local IPC mechanism is required on Windows, macOS and Linux?
35. What background-execution model is viable on Android?
36. What mailbox retention, encryption and backup semantics are required?

## Operations and release

37. Which public/backend environments are authoritative for later controlled verification?
38. Which platforms form the MVP release matrix?
39. What supply-chain/signing/auto-update trust roots are approved?
40. Who can accept a residual High finding, and how is that acceptance recorded?
