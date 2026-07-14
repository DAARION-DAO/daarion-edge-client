# Sovereign Agent Capability Status Matrix

Status: **PHASE 1A CANDIDATE SNAPSHOT — 2026-07-14 / HUMAN ACCEPTANCE PENDING**

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
| Local model discovery | Edge | `IMPLEMENTED_BUT_UNVERIFIED` | Canonical bundled-registry summaries are matched to loopback Ollama `/api/tags`; unavailable provider behavior is tested, but no real installation was called | Controlled real-Ollama smoke |
| Ollama detection | Edge | `IMPLEMENTED_BUT_UNVERIFIED` | HTTP-only loopback health exists; CLI probing and webview shell authority were removed; desktop live/platform behavior is unverified | Controlled real-Ollama/platform smoke |
| Model download | Edge | `PARTIALLY_IMPLEMENTED` | Preparation accepts a canonical ID and maps it to local Ollama pull; artifact digest/signature verification and a real pull were not verified | Verified manifest/artifact gate |
| Model verification | Edge | `MOCK_OR_PLACEHOLDER` | `verifier.rs` does not compare a hash | Artifact security phase |
| Model loading/unloading | Edge | `MOCK_OR_PLACEHOLDER` | Legacy simulated loader remains in source for unrelated dormant modules but is no longer registered or used by the approved inference surface | Later truthful provider lifecycle phase |
| Local-only execution policy | Edge | `IMPLEMENTED_AND_VERIFIED` | `InferencePolicy::LocalOnly`, loopback endpoint validation, production-only Ollama composition and remote-provider rejection tests; human acceptance pending | Phase 1A review |
| Local inference | Edge | `IMPLEMENTED_BUT_UNVERIFIED` | Provider-neutral service and loopback Ollama adapter exist with fake/fixture tests; no real installed Ollama/model was called | Controlled real-Ollama smoke |
| Token streaming | Edge | `IMPLEMENTED_AND_VERIFIED` | Bounded byte-buffered NDJSON decoder and terminal event gate cover split UTF-8, multiple/final/malformed/oversized records and late-event suppression | Phase 1A review |
| Timeout | Edge | `IMPLEMENTED_AND_VERIFIED` | One service-owned absolute deadline covers queue and provider execution; cleanup/terminal behavior is tested | Phase 1A review |
| Cancellation | Edge | `IMPLEMENTED_AND_VERIFIED` | Active request IDs, cancellation ownership, duplicate rejection, cleanup and no-late-event behavior are tested | Phase 1A review |
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
- `IMPLEMENTED_AND_VERIFIED` is limited to the cited Phase 1A repository checks. It does not imply a live Ollama run, packaging proof, deployment truth or production readiness.
- A live/deployed result must be recorded separately from repository evidence.
- Status changes require evidence, date, command/result or deployed proof, and documentation update.
