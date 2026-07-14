# ADR 0003: Reticulum/LXMF Integration Boundary

- Status: Accepted architecture direction; implementation deferred to Phase 6
- Date: 2026-07-04
- Scope: Edge transport boundary and future mesh runtime

## Context

No Reticulum/LXMF implementation exists in the audited Edge snapshot. Directly embedding transport before the deterministic runtime, persistent state, policy and message-envelope contracts would entangle packaging, background execution and agent authority.

## Decision

1. A provider-neutral transport interface comes before Reticulum implementation.
2. Reticulum/LXMF are excluded from the first runtime milestones.
3. A separate authenticated `daarion-meshd` daemon is the preferred long-term desktop direction.
4. A bundled Python sidecar may be used only for a bounded prototype selected by a later plan.
5. The mesh component owns transport, routing, mailbox and offline delivery only.
6. It must not own Agent Supervisor policy, local memory semantics, tool permissions or wallet signing.
7. Edge↔daemon IPC requires mutual authentication, daemon/package identity, versioning, least privilege and replay-resistant requests.
8. Mesh envelopes require sender identity, message ID, schema version, signature, issued/expiry time, replay protection and idempotency.
9. Android background execution and packaging require a separate platform decision.

## Strategy assessment

| Strategy | Benefit | Drawback | Baseline decision |
| --- | --- | --- | --- |
| Bundled Python sidecar | Fastest access to upstream ecosystem | Runtime packaging, lifecycle and replacement risk | Prototype only |
| Separate `daarion-meshd` daemon | Background independence and clean ownership | Installer/service/IPC complexity | Preferred long-term desktop |
| Direct Rust port | Native packaging potential | Large protocol/maintenance/security burden | Not selected now |
| Container | Stronger separation in some desktop ops | Poor consumer/mobile fit and heavy dependency | Not default |
| Generic interface first | Preserves reversibility and testability | No immediate mesh capability | Required first step |

## Consequences

- Phase 6 can test transport without giving the daemon policy authority.
- Desktop and Android may use different process models behind the same contract.
- The public repositories contain no live mesh topology or private infrastructure truth.

## Verification gate

See [Reticulum/LXMF gate](../security/SECURITY_GATES.md). This ADR does not authorize transport implementation.
