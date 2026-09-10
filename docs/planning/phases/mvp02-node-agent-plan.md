# MVP02 managed Node Agent plan

## Objective
Reuse device identity and hardware probes for one approved outbound signed observation.
## Current State
Canonical main eafb30d; adopted human-reviewed baseline and explicit MVP02 operator authorization apply. Installed GUI heartbeat startup crashes; headless adapter avoids activating it.
## Scope
One-shot macOS CLI; existing identity service and capability scanner. City registry verifies observations; no membership or worker authority.
## Explicit Non-Goals
Inference, worker jobs, GUI repair, pairing, wallet, mesh, identity replacement.
## Repository Ownership
Edge owns local device key and probes. City/node domain owns operator-approved logical-node binding. DAIS legitimacy unchanged.
## Files and Modules Expected to Change
identity.rs, headless binary, macOS keyring feature, plan and completion.
## Contracts Affected
Domain daarion.node.observation.v1. Canonical sorted JSON payload binds logical node, device UUID, observation UUID, timestamp, audience, facts digest. Ed25519 hex signature. HTTPS only, redirects disabled, explicit local CA trust.
## Security Considerations
Local private key never leaves Keychain. No arbitrary signing input. CLI gathers fixed hardware and loopback Ollama facts only. Server approval is out of band; unknown/revoked keys fail. Replay and stale timestamps fail. No inference. macOS only.
## Migration and Compatibility Considerations
Existing metadata/key format retained. Enable existing keyring apple-native backend because mock storage is unsuitable for persistent identity. No new crypto algorithm or crate version. Review lock delta.
## Implementation Steps
Expose bounded headless identity wrapper, reuse scanner, sign fixed observation and send once.
## Tests
Existing identity negative tests; binary tests; cargo fmt/clippy/build; cross-repo signed observation and invalid signature/replay tests; candidate then production Office.
## Acceptance Criteria
Restart-stable identity, approved binding, TLS verified delivery, fresh attributable Observer evidence.
## Rollback Strategy
Stop scheduled sender; preserve identity and previous production binaries. No deletion of keys or records.
## Documentation Updates
This versioned contract and matching completion; no private deployment values.
## Open Questions
None requiring operator decision. Explicit task authorizes initial identity, City registry and exact reviewed delivery.
## GO
GO under explicit MVP02 FAST PATH authorization; no unreviewed release.
