# Sovereign Agent Capability Status Matrix

Status: **BASELINE SNAPSHOT — 2026-07-04**

The status describes executable evidence in the audited snapshots, not target architecture, live deployment, or product aspiration.

## Allowed classifications

`IMPLEMENTED_AND_VERIFIED`, `IMPLEMENTED_BUT_UNVERIFIED`, `PARTIALLY_IMPLEMENTED`, `MOCK_OR_PLACEHOLDER`, `DOCUMENTED_ONLY`, `MISSING`, `BLOCKED_BY_EXTERNAL_DEPENDENCY`, `SECURITY_GATED`.

## Matrix

| Capability | Owner | Status | Evidence and limit | Next gate |
| --- | --- | --- | --- | --- |
| Product Auth/MicroDAO context | `loval-echoes` | `IMPLEMENTED_BUT_UNVERIFIED` | Supabase client/types and product flows exist; deployed state not inspected | Auth/RLS integration verification |
| Community invitation | `loval-echoes` | `IMPLEMENTED_BUT_UNVERIFIED` | `src/services/communityMembers.ts`; distinct from device pairing | Contract tests |
| Device identity | Edge | `PARTIALLY_IMPLEMENTED` | `identity.rs`: Ed25519/keyring/tests; no rotation/recovery/revocation/domain separation | Identity ADR/security tests |
| Agent identity | Edge | `DOCUMENTED_ONLY` | No separate lifecycle from device root was proven | Identity-domain design |
| Pairing invitation creation | Web/backend | `PARTIALLY_IMPLEMENTED` | Version-1 SQL/RPC/types/UI exist; deployment not verified | Signed-envelope ADR/tests |
| Pairing consumption | Edge | `PARTIALLY_IMPLEMENTED` | `pairing.rs` parses/persists; does not verify signature/expiry/replay/revocation | Signed-envelope gate |
| Backend health | Edge + backend | `BLOCKED_BY_EXTERNAL_DEPENDENCY` | Edge client code and contract states exist; service not called | Controlled live contract test |
| Genesis/provisioning | Edge + backend | `PARTIALLY_IMPLEMENTED` | Provisioning/UI paths exist; wallet material unsafe/partial and E2E unverified | Separate provisioning/wallet gates |
| Local model discovery | Edge | `IMPLEMENTED_BUT_UNVERIFIED` | `ollama.rs::list_local_models` uses loopback `/api/tags` | Provider integration tests |
| Ollama detection | Edge | `PARTIALLY_IMPLEMENTED` | Loopback and shell checks exist; command exposure/platform behavior need review | Phase 1A |
| Model download | Edge | `PARTIALLY_IMPLEMENTED` | Ollama pull exists; generic managed download unimplemented; artifact store simulates | Verified manifest/artifact gate |
| Model verification | Edge | `MOCK_OR_PLACEHOLDER` | `verifier.rs` does not compare a hash | Artifact security phase |
| Model loading/unloading | Edge | `MOCK_OR_PLACEHOLDER` | `runtime_loader.rs` emits simulated state | Provider lifecycle implementation |
| Local inference | Edge | `PARTIALLY_IMPLEMENTED` | Real loopback Ollama chat stream; no LocalOnly invariant or full provider boundary | Phase 1A |
| Token streaming | Edge | `PARTIALLY_IMPLEMENTED` | Response chunks emit Tauri events; framing/cancel/error/backpressure not proven | Phase 1A tests |
| Timeout | Edge | `PARTIALLY_IMPLEMENTED` | Some Ollama calls set timeouts; local chat path uses default client | Phase 1A uniform deadline |
| Cancellation | Edge | `MISSING` | No request cancellation contract found | Phase 1A |
| Structured model output | Edge | `MISSING` | No validated structured decision schema found | Later Supervisor phase |
| Edge embeddings | Edge | `MISSING` | No local embedding provider/store found | Later memory phase |
| Web cloud chat/embeddings | Web cloud boundary | `IMPLEMENTED_BUT_UNVERIFIED` | `ai-agent-chat` calls cloud gateway after auth checks; live provider not called | Separate cloud feature verification |
| Durable runtime state | Edge | `MISSING` | No SQLite foundation/schema found | Phase 1B |
| Six-level memory | Edge | `MISSING` | No working/conversation/episodic/semantic/procedural/graph implementation | Phase 2 after foundation |
| Agent Supervisor | Edge | `MISSING` | Agent-shaped modules do not form a traced deterministic Supervisor | Phase 1C |
| Bounded Loop Runtime | Edge | `MISSING` | No versioned definition, durable run/checkpoint model, limits or resume | Phase 3 |
| Tool permission broker | Edge | `SECURITY_GATED` | Shell/network/worker surfaces exist without unified typed broker | Phase 4 |
| Messaging | Edge | `MOCK_OR_PLACEHOLDER` | `messaging.rs`: stub session, random polling, local echo, RAM storage | Phase 6 transport |
| Transport abstraction | Edge | `DOCUMENTED_ONLY` | Target/ADR only | Phase 6 entry contract |
| Reticulum | Mesh component | `MISSING` | No source/dependency found | Phase 6 |
| LXMF | Mesh component | `MISSING` | No source/dependency found | Phase 6 |
| Offline mailbox | Mesh component | `MISSING` | No durable mailbox found | Phase 6 |
| Wallet | Edge signer boundary | `SECURITY_GATED` | Real mnemonic generation mixed with mock addresses and unsafe serialized return | Wallet ADR/signer isolation |
| Worker | Edge | `SECURITY_GATED` | Mock lease/dummy key/unbounded loop; no safe enablement evidence | Signed leases/sandbox gate |
| Dashboard readiness display | Web | `PARTIALLY_IMPLEMENTED` | Status RPC adapter and `DeviceConnectionCard` exist | Signed projection contract |
| Readiness projection production | Edge | `DOCUMENTED_ONLY` | No signed versioned Edge producer found | Phase 5 |
| Production readiness | Both | `SECURITY_GATED` | Audit is read-only and several high gates remain open | Phase 9 evidence |

## Interpretation rules

- A module name, README statement, enum, UI state, or test fixture alone cannot raise a capability status.
- `IMPLEMENTED_AND_VERIFIED` requires relevant executable checks in the current evidence set; none of the system-level capabilities above meet that bar in this read-only baseline.
- A live/deployed result must be recorded separately from repository evidence.
- Status changes require evidence, date, command/result or deployed proof, and documentation update.
