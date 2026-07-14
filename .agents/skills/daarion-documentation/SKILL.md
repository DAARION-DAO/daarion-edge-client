---
name: daarion-documentation
description: "Use in every DAARION phase to keep architecture, ADRs, status matrices, roadmap, README claims, phase plans, and completion evidence aligned with code."
---

# DAARION Documentation

- Update architecture, ADR, roadmap, capability/status, README, contract, and operational documentation affected by the phase.
- Keep implemented-and-verified, implemented-but-unverified, partial, mocked, documented-only, missing, external-blocked, and security-gated states distinct.
- Cite exact repository paths, symbols, tests, and commands for behavior claims. Documentation is not implementation evidence.
- Preserve product-web versus local-runtime ownership and public/private boundaries. Never add secrets, private endpoints, infrastructure topology, or sensitive operational evidence.
- Create/update `docs/planning/phases/<phase-id>-plan.md` before code and `docs/planning/phases/<phase-id>-completion.md` after verification.
- Use ADRs for durable decisions and alternatives; do not rewrite history or present a preferred future choice as accepted.
- Update README claims when behavior changes, and call out stale or contradictory claims that remain unresolved.

Do not claim production readiness, security, local execution, wallet support, transport, or verification without executable evidence. Record limitations and the next bounded milestone.
