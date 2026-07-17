# Sovereign Agent Capability Status Matrix

Status: **PHASE 1A MERGED / PHASE 1B.1 REPOSITORY CANDIDATE — 2026-07-17**

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
| Local model discovery | Edge | `IMPLEMENTED_BUT_UNVERIFIED` | Canonical summaries are marked installed only after daemon cloud-disabled proof and one exact stable `/api/tags` → `/api/show` → `/api/tags` local-evidence chain; deterministic fixtures pass, but no real installation was called | Controlled real-Ollama smoke |
| Ollama detection | Edge | `IMPLEMENTED_BUT_UNVERIFIED` | Loopback health plus required `/api/status` `cloud.disabled=true` policy proof exist; unsupported/malformed states fail closed; desktop live/platform behavior is unverified | Controlled real-Ollama/platform smoke |
| Model download | Edge | `PARTIALLY_IMPLEMENTED` | Request-scoped pull retains deadline/cancellation/bounded progress and now requires daemon plus complete local-model postflight before success; real pull, daemon-side stop and cryptographic artifact trust remain unverified | Verified manifest/artifact gate |
| Model verification | Edge | `PARTIALLY_IMPLEMENTED` | Official Ollama metadata proof rejects remote markers, aliases, duplicates, invalid size/digest/details and unstable evidence before chat/preparation success; it does not hash the file, verify a signature, or attest a malicious daemon | Artifact security phase |
| Model loading/unloading | Edge | `MOCK_OR_PLACEHOLDER` | Legacy simulated loader remains in source for unrelated dormant modules but is no longer registered or used by the approved inference surface | Later truthful provider lifecycle phase |
| Local-only execution policy | Edge | `IMPLEMENTED_AND_VERIFIED` | `InferencePolicy::LocalOnly`, loopback/redirect/proxy controls, fail-closed daemon cloud-disabled proof, stable per-model evidence, immediate pre-chat revalidation and zero-chat sentinel tests pass in repository fixtures; reviewed head `9e8c5d9…` is merged and fresh-main verified | Controlled real-Ollama smoke |
| Local inference | Edge | `IMPLEMENTED_BUT_UNVERIFIED` | Provider-neutral service and loopback Ollama adapter exist with fake/fixture tests; no real installed Ollama/model was called | Controlled real-Ollama smoke |
| Token streaming | Edge | `IMPLEMENTED_AND_VERIFIED` | Bounded byte-buffered NDJSON decoder and terminal event gate cover split UTF-8, multiple/final/malformed/oversized records and late-event suppression | Phase 1A review |
| Timeout | Edge | `IMPLEMENTED_AND_VERIFIED` | Service-owned probe, chat and preparation deadlines cover their queue/provider boundaries; cleanup and mutually exclusive terminal behavior are tested | Phase 1A review |
| Cancellation | Edge | `IMPLEMENTED_AND_VERIFIED` | One kind-aware registry owns chat/preparation UUIDs; dedicated cancellation, duplicate rejection, cleanup, isolation, stalled-socket teardown and no-late-success behavior are tested | Phase 1A review |
| Structured model output | Edge | `MISSING` | No validated structured decision schema found | Later Supervisor phase |
| Edge embeddings | Edge | `MISSING` | No local embedding provider/store found | Later memory phase |
| Web cloud chat/embeddings | Web cloud boundary | `IMPLEMENTED_BUT_UNVERIFIED` | `ai-agent-chat` calls cloud gateway after auth checks; live provider not called | Separate cloud feature verification |
| Durable runtime state | Edge | `PARTIALLY_IMPLEMENTED` | The Phase 1B.1 candidate adds a Rust-owned SQLite 3.53.2 connection on one bounded worker, immutable version-1 migration with exactly five empty tables, fail-closed path/migration/integrity controls, one safe read-only Tauri DTO, typed client, and mounted Dashboard card. Thirty-five focused repository tests prove bootstrap, path replacement refusal, resource-limit re-evaluation, controlled shutdown, and clean reopen; there is still no public content CRUD, deletion/export/backup, full desktop/platform smoke, or Phase 1B.2 behavior | Exact-head draft review, merge/fresh-main verification, then separately authorize Phase 1B.2 only if accepted |
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
- `IMPLEMENTED_AND_VERIFIED` is limited to the cited Phase 1A repository checks and fresh-main readback. It does not imply a live Ollama run, packaging proof, deployment truth or production readiness.
- A live/deployed result must be recorded separately from repository evidence.
- Status changes require evidence, date, command/result or deployed proof, and documentation update.
- Phase 1B.1 repository evidence does not claim that conversation, message, task,
  or audit persistence APIs exist. The empty schema and safe readiness projection
  are not complete memory or a completed Phase 1B release.
