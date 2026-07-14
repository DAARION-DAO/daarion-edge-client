# Sovereign Agent Baseline Audit — 2026-07-04

Status: **ADOPTED BASELINE / READ-ONLY EVIDENCE / NOT PRODUCTION VERIFICATION**

Decision: **CONDITIONAL_GO only for a separately planned Phase 1A**

## Provenance and limits

This document adopts the human-approved conclusions of the sovereign-agent audit as the canonical repository baseline. The complete original audit report was not present in either checked repository or in the supplied attachments. Therefore this file records the approved decision summary and rechecks its material claims against the 2026-07-04 repository snapshots; it does not claim to reproduce an unavailable artifact.

Snapshot evidence:

| Repository | Snapshot commit | Snapshot date | Remote/deployed verification |
| --- | --- | --- | --- |
| `loval-echoes` | `0f16a31841eebde3545eaf1cd58fb2ef40dd800a` | 2026-07-04 | Not performed |
| `daarion-edge-client` | `e25c41e73298ece63261302812c1a4b800bdad38` | 2026-07-04 | Not performed |

The audit and this adoption are read-only with respect to application code. No fetch, deployed Supabase inspection, backend health call, Ollama call, packaging test, or production smoke is evidence here. Repository paths and symbols are evidence of source state, not proof of deployed behavior.

## Status vocabularies

Capabilities use only:

- `IMPLEMENTED_AND_VERIFIED`
- `IMPLEMENTED_BUT_UNVERIFIED`
- `PARTIALLY_IMPLEMENTED`
- `MOCK_OR_PLACEHOLDER`
- `DOCUMENTED_ONLY`
- `MISSING`
- `BLOCKED_BY_EXTERNAL_DEPENDENCY`
- `SECURITY_GATED`

Documentation and product claims use only:

- `CLAIM_VERIFIED`
- `CLAIM_PARTIAL`
- `CLAIM_UNVERIFIED`
- `CLAIM_FALSE_OR_STALE`

## Executive summary

The repository pair already has useful foundations: a clear product/runtime split, a real Supabase-backed web product surface, distinct community and device invitation concepts, a Tauri/Rust application, Ed25519/keyring identity code, loopback Ollama discovery and inference paths, pairing persistence, health-state modeling, and substantial release/process documentation.

It does not yet contain a sovereign local agent runtime. The Edge model path mixes real loopback Ollama calls with simulated GGUF lifecycle, simulated remote offload, misleading `llama.cpp` labeling, and no fail-closed `LocalOnly` policy. Agent-named modules do not form a traced Supervisor with deterministic tasks, bounded state, durable checkpoints, recovery, and policy enforcement. Durable SQLite runtime state and six-level memory are absent. Messaging, remote execution, model verification, parts of worker execution, and several planning paths are mocks or placeholders. Reticulum and LXMF are missing. Wallet terminology is present, but signer isolation, domain separation, recovery, and safe frontend boundaries are not.

The web repository contains separate cloud-assisted AI functions. That is compatible with the target architecture only if the web cloud trust domain remains clearly labeled and isolated from Edge local-agent claims. “DAARION never uses cloud inference” would be false; the enforceable rule is that the sovereign Edge runtime never sends a prompt remotely unless a separately enabled future policy and explicit user consent allow it.

Implementation must not begin as a combined inference/storage/Supervisor change. The only eligible next runtime slice is Phase 1A: provider abstraction, Ollama adapter, `LocalOnly`, model/command alignment, timeout, cancellation, truthful UI, and fake-provider tests. Phase 1B (SQLite runtime state) and Phase 1C (inert Supervisor) require separate plans and reviews.

### Top five blockers

1. No fail-closed Edge execution policy; the inference path contains a simulated remote branch.
2. Pairing payload consumption does not verify a signature, expiry, replay state, binding, or revocation.
3. No durable SQLite runtime-state foundation or restart-safe task/audit model.
4. No real bounded Agent Supervisor/Loop Runtime with deterministic recovery.
5. Security-sensitive wallet, worker, tool, model-artifact, and transport surfaces lack their required gates.

### Top five strengths

1. Repository ownership is separable without a total rewrite.
2. Tauri/Rust provides an appropriate local enforcement boundary.
3. Real loopback Ollama discovery/pull/inference code provides an incremental starting point.
4. Ed25519 and OS keyring code, with unit tests, provides a partial identity foundation.
5. Web pairing/readiness schemas and UI exist as a contract starting point, while community invites remain a distinct concept.

## Repository architecture maps

### `loval-echoes`

| Layer | Source evidence | Baseline reading |
| --- | --- | --- |
| Product UI | `src/pages/MicroDAOOnboarding.tsx`, `src/components/device/DeviceConnectionCard.tsx`, `src/pages/Install.tsx` | MicroDAO onboarding, Connect Device, invitation handoff, and readiness presentation |
| Product contracts | `src/services/deviceConnection.ts` | Version-1 intent/invite/status types and Supabase RPC adapters |
| Auth/data client | `src/integrations/supabase/client.ts`, `src/integrations/supabase/types.ts` | Browser Supabase client and generated schema types |
| Database contract | `supabase/migrations/20260620070143_device_pairing_invites_status.sql` | Pairing/readiness tables, RLS, RPCs, expiry and status fields; deployed state unverified |
| Web cloud AI | `supabase/functions/ai-agent-chat/index.ts` | Authenticated cloud gateway chat/embedding path; separate from Edge inference |
| Community invitation | `src/services/communityMembers.ts` | Human MicroDAO membership invitation; must never authorize device pairing |
| Repository boundary | `docs/REPOSITORY_ROLE.md` | Web product/control layer; no local sovereign runtime ownership |

### `daarion-edge-client`

| Layer | Source evidence | Baseline reading |
| --- | --- | --- |
| Composition/IPC | `src-tauri/src/lib.rs` | Large Tauri command/state composition surface; a future modularization concern |
| Identity | `src-tauri/src/identity.rs` | Ed25519 generation/signing, public metadata file, private key in OS keyring, unit tests; lifecycle incomplete |
| Pairing | `src-tauri/src/pairing.rs` | Local `pairing.json`, URL/base64 JSON parsing, backend normalization, unit tests; no cryptographic envelope verification |
| Health/provisioning | `src-tauri/src/backend_health.rs`, `src-tauri/src/provisioning.rs` | Paired-backend health/provisioning paths; end-to-end service behavior unverified |
| Ollama | `src-tauri/src/models/ollama.rs` | Loopback detection, model listing, pull stream, and smoke inference |
| Inference | `src-tauri/src/models/local_inference.rs` | Loopback Ollama chat stream plus simulated network offload and inaccurate runtime label |
| Model lifecycle | `src-tauri/src/models/mod.rs`, `artifact_store.rs`, `verifier.rs`, `runtime_loader.rs` | Generic download unimplemented; artifact download, verification, load/unload are simulated |
| Agent-shaped modules | `src-tauri/src/agents/`, `coordination/`, `intelligence/` | Types and partial logic, not a traced autonomous Supervisor runtime |
| Messaging | `src-tauri/src/messaging.rs` | In-memory state, stub session/room, simulated polling and local echo |
| Wallet/Genesis | `src-tauri/src/genesis.rs` | Real mnemonic generation mixed with placeholder addresses; mnemonic returned across a Tauri command |
| Worker | `src-tauri/src/worker/worker_loop.rs` | Unbounded polling loop with mock lease and dummy key; not acceptable for enablement |
| Durable memory | `src-tauri/Cargo.toml`, `src-tauri/src/` | No SQLite runtime-state dependency/schema or six-level memory implementation found |
| Mesh transport | `src-tauri/Cargo.toml`, `src-tauri/src/` | No Reticulum or LXMF implementation found |

## Cross-repository contract matrix

| Contract | Producer | Consumer | Current schema/evidence | Status | Primary risk |
| --- | --- | --- | --- | --- | --- |
| Product authentication | Supabase Auth via `loval-echoes` | Web product/RPCs | `src/integrations/supabase/client.ts`; server RPCs use `auth.uid()` | `IMPLEMENTED_BUT_UNVERIFIED` | Deployed Auth/RLS state not inspected |
| MicroDAO membership | Supabase tables/RPCs | Web product and pairing-invite RPC | `community_members`; membership checks in pairing migration | `IMPLEMENTED_BUT_UNVERIFIED` | Deployment and policy tests absent here |
| Community invitation | `src/services/communityMembers.ts` | Web onboarding | `/onboarding?inviteCode=...` | `IMPLEMENTED_BUT_UNVERIFIED` | Must not be accepted as device authority |
| Device pairing invitation | `create_device_pairing_invitation` plus `DevicePairingInvitePayload` | Edge `pair_backend` / `pairing.rs` | `daarion-pair:<base64url-json>`, schema version 1 | `PARTIALLY_IMPLEMENTED` | Edge ignores invitation identity, membership binding, expiry, signature and revocation |
| Web handoff | `buildDeviceSetupPathFromPairingCode` | `src/pages/Install.tsx` and Edge user journey | `deviceInvite` query parameter | `PARTIALLY_IMPLEMENTED` | Browser parsing is not authorization evidence |
| Local pairing state | Edge `pair_backend` | Edge health/provisioning | Local `pairing.json` | `PARTIALLY_IMPLEMENTED` | Malicious or stale payload can persist an untrusted backend |
| Backend health | Paired backend contract | Edge and projected Dashboard state | `backend_health.rs`; web health/readiness enums | `BLOCKED_BY_EXTERNAL_DEPENDENCY` | Public service/deployed contract not verified |
| Readiness write | `record_device_readiness` RPC | Supabase read model | SQL migration | `PARTIALLY_IMPLEMENTED` | No verified authenticated Edge producer/signature/provenance |
| Dashboard readiness read | `get_device_connection_status` RPC | `DeviceConnectionCard` | `DeviceConnectionStatusView` | `PARTIALLY_IMPLEMENTED` | Projection can outlive or misrepresent local truth |
| Genesis/provisioning | Edge plus backend | Edge UI and future projection | `provisioning.rs`, `GenesisWizard.tsx`, `genesis.rs` | `PARTIALLY_IMPLEMENTED` | Wallet terminology and backend dependency exceed verified evidence |
| Revocation | Pairing database status | Web/Edge | SQL has pending/consumed/expired/revoked | `PARTIALLY_IMPLEMENTED` | Edge has no verified revocation synchronization |
| Agent readiness projection | Edge (target) | Web Dashboard (target) | No signed versioned producer/consumer contract | `DOCUMENTED_ONLY` | Forgery, replay, excessive private data, stale status |

## Severity-ordered findings

### HIGH — Pairing envelope is not authenticated by Edge

- Affected: both repositories.
- Evidence: `loval-echoes/src/services/deviceConnection.ts::DevicePairingInvitePayload`; pairing SQL RPC; `daarion-edge-client/src-tauri/src/pairing.rs::payload_from_json`.
- Observation: the web/SQL payload contains identifiers and expiry, but Edge extracts only backend URL, label, and environment. No signature, nonce/replay cache, membership binding, expiry check, or revocation check is enforced during consumption.
- Impact: a malicious or replayed payload could bind a device to an unintended backend if a user supplies it.
- Gate: production pairing remains blocked pending separate signed-envelope ADR and executable contract tests.

### HIGH — Edge local-only policy is not a technical invariant

- Affected: `daarion-edge-client`.
- Evidence: `src-tauri/src/models/local_inference.rs::run_chat` and `handle_remote_fallback`.
- Observation: real Ollama HTTP inference coexists with a simulated remote-offload branch. No explicit MVP `InferencePolicy::LocalOnly` boundary fails closed before routing.
- Impact: the current mock does not prove remote data leakage, but turning the route real could silently move private prompts off-device.
- Gate: Phase 1A must remove or make the branch unreachable under the only enabled policy and prove no remote provider invocation.

### HIGH — Wallet material crosses an unsafe command boundary

- Affected: `daarion-edge-client`.
- Evidence: `src-tauri/src/genesis.rs::WalletKeys` and `generate_wallet_keys`.
- Observation: a real BIP39 mnemonic is generated and returned in a serializable Tauri response; EVM addresses are labeled as mocked.
- Impact: frontend state/logging or compromised UI code could expose recovery material.
- Gate: wallet remains disabled until signer isolation, derivation, approval, recovery, and domain-separated identities are designed and tested.

### HIGH — Worker and autonomous execution are not bounded/security-ready

- Affected: `daarion-edge-client`.
- Evidence: `src-tauri/src/worker/worker_loop.rs::start_worker_loop`.
- Observation: the loop is unbounded and uses simulated NATS, a mock lease, dummy public key, and placeholder evidence.
- Impact: enablement would create uncontrolled repeated execution around untrusted work.
- Gate: worker remains hard-disabled; autonomous loops require the bounded Loop Runtime and workers additionally require signed leases and sandbox hardening.

### HIGH — Model artifacts are not verified

- Affected: `daarion-edge-client`.
- Evidence: `src-tauri/src/models/mod.rs::download_model`; `artifact_store.rs::download_model`; `verifier.rs::verify_hash`; `runtime_loader.rs`.
- Observation: generic download returns “Not implemented yet”; the artifact store writes dummy data; hash verification assumes an existing file passes; load/unload emit simulated states.
- Impact: a future unverified model artifact could execute malicious or corrupted content.
- Gate: managed model downloads remain blocked until signed/approved manifests, digest/size verification, safe paths, and failure tests exist.

### MEDIUM — Durable local state and crash recovery are absent

- Affected: `daarion-edge-client`.
- Evidence: `src-tauri/Cargo.toml`; in-memory `MessagingState.messages` in `messaging.rs`.
- Observation: no SQLite runtime schema for conversations, messages, tasks, audit events, or migrations was found.
- Impact: restart recovery, retention, deletion, auditability, and deterministic task resumption cannot be guaranteed.
- Remediation: Phase 1B only; do not label it six-level memory.

### MEDIUM — Agent-shaped modules overstate runtime maturity

- Affected: `daarion-edge-client`.
- Evidence: `src-tauri/src/agents/planning_agent.rs::generate_plan` contains placeholder telemetry; no `AgentSupervisor` or durable bounded run path was found.
- Impact: documentation/UI can mistake domain types and simulated decisions for an autonomous system.
- Remediation: Phase 1C inert Supervisor, then later bounded Loop Runtime.

### MEDIUM — Cloud web AI and local Edge AI can be conflated

- Affected: `loval-echoes`.
- Evidence: `supabase/functions/ai-agent-chat/index.ts::generateEmbedding` and `generateChatCompletion`.
- Observation: authenticated web requests send content to a cloud AI gateway. This is not Edge inference.
- Impact: privacy claims can mislead users if cloud-assisted product features are marketed as local.
- Remediation: label trust domain, provider, and consent separately; never reuse local-agent readiness to attest cloud privacy.

### LOW — Documentation drift already exists

- Affected: both repositories.
- Evidence: `loval-echoes/docs/product/CONNECT_DEVICE_CONTRACT.md` still calls device pairing/status missing although later source/migrations exist; Edge README/runtime labels overstate GGUF/`llama.cpp` behavior.
- Impact: planning and release decisions can be based on stale claims.
- Remediation: maintain the claim-drift table and correct claims in their owning phase.

## Mock and placeholder inventory

| Evidence | Classification | Notes |
| --- | --- | --- |
| `daarion-edge-client/src-tauri/src/models/local_inference.rs::handle_remote_fallback` | `MOCK_OR_PLACEHOLDER` | Simulated NATS delay and fixed remote result |
| `models/artifact_store.rs::download_model` | `MOCK_OR_PLACEHOLDER` | Simulated progress and dummy model data |
| `models/verifier.rs::verify_hash` | `MOCK_OR_PLACEHOLDER` | Expected hash unused; existing file passes |
| `models/runtime_loader.rs` | `MOCK_OR_PLACEHOLDER` | Load/unload are status events only |
| `models/mod.rs::download_model` | `MISSING` | Returns “Not implemented yet” |
| `messaging.rs::bootstrap_messaging` | `MOCK_OR_PLACEHOLDER` | Stub session and mock room |
| `messaging.rs::poll_endpoint` | `MOCK_OR_PLACEHOLDER` | Random local events/errors, no transport |
| `messaging.rs::send_node_message` | `MOCK_OR_PLACEHOLDER` | Local in-memory echo only |
| `agents/planning_agent.rs::generate_plan` | `MOCK_OR_PLACEHOLDER` | Fixed plan and telemetry claim |
| `worker/worker_loop.rs` | `SECURITY_GATED` | Mock lease/dummy key inside unbounded loop |
| `genesis.rs::generate_wallet_keys` | `SECURITY_GATED` | Real mnemonic mixed with placeholder addresses and unsafe return boundary |
| Reticulum/LXMF source | `MISSING` | No implementation found in Edge manifests/source |
| Durable memory/Loop Supervisor | `MISSING` | No SQLite runtime state or durable bounded workflow runtime found |

## Claim-drift table

| Claim | Source file | Classification | Evidence | Required correction | Target phase |
| --- | --- | --- | --- | --- | --- |
| Edge downloads/manages/executes GGUF models natively | `daarion-edge-client/README.md` | `CLAIM_FALSE_OR_STALE` | Generic download unimplemented; artifact/download/load/verify paths simulated | Describe Ollama path separately from planned managed artifacts | 1A plus later model-artifact phase |
| Runtime is “llama.cpp (Local) via /api/chat” | `src-tauri/src/models/local_inference.rs` | `CLAIM_FALSE_OR_STALE` | `/api/chat` is called on Ollama loopback; no embedded llama.cpp evidence | Report provider/runtime truthfully | 1A |
| Edge local inference works as a complete private policy boundary | README/UI/runtime claims | `CLAIM_PARTIAL` | Real loopback Ollama path exists; no explicit LocalOnly invariant/cancellation/model validation | Bind behavior to `ExecutionPolicy` and provider tests | 1A |
| Device identity is secure and complete | `daarion-edge-client/README.md` | `CLAIM_PARTIAL` | Ed25519/keyring/tests exist; rotation, recovery, revocation and identity-domain separation do not | Narrow claim to current device-key foundation | Identity phase |
| Genesis provides wallet management | `daarion-edge-client/README.md`, `genesis.rs` | `CLAIM_PARTIAL` | Mnemonic generation is real; EVM addresses are mock and mnemonic crosses frontend boundary | Do not call production wallet; isolate signer | Wallet phase |
| Messaging uses a real Matrix/NATS/control plane | README/module names | `CLAIM_FALSE_OR_STALE` | `messaging.rs` explicitly creates stub session/room and local echo | Label as placeholder until transport is real | 6 |
| Worker is hard-sandboxed and ready | `daarion-edge-client/README.md` | `CLAIM_UNVERIFIED` | Worker path uses mock lease/dummy key/unbounded loop; no release proof in this audit | Keep disabled and require worker security gate | 8/9 |
| Device pairing invitation and Dashboard status are missing | `loval-echoes/docs/product/CONNECT_DEVICE_CONTRACT.md` | `CLAIM_FALSE_OR_STALE` | Later migrations, RPC adapters and `DeviceConnectionCard` now exist | Say “partially implemented, end-to-end unverified” | Pairing/readiness ADRs |
| Web AI is local sovereign-agent inference | Product copy if stated | `CLAIM_FALSE_OR_STALE` | `ai-agent-chat` calls a cloud gateway | Label as separate cloud-assisted web feature | Documentation/product phase |
| Reticulum/LXMF are implemented | Any roadmap/module-name implication | `CLAIM_FALSE_OR_STALE` | No manifest/source implementation found | Keep as future transport behind interface | 6 |
| Bounded autonomous loops exist | Any agent/worker naming implication | `CLAIM_FALSE_OR_STALE` | No LoopDefinition/Supervisor/checkpoint store; worker loop is unbounded | Keep documented-only until Loop Runtime passes tests | 3 |

## Adopted architecture decisions

1. `loval-echoes` remains the authenticated web product/control layer.
2. `daarion-edge-client` remains the local sovereign enforcement/runtime layer.
3. Web cloud AI and Edge local inference are separate trust domains.
4. Edge MVP enables only local inference through an explicit provider boundary and fail-closed policy.
5. SQLite is the first durable runtime-state store; its foundation is not six-level memory.
6. The inert Supervisor follows durable state and precedes tools/network/autonomous scheduling.
7. Loops are declarative, bounded, checkpointed, idempotent, policy-controlled workflows.
8. Transport abstraction precedes Reticulum/LXMF; a separate authenticated daemon is the preferred long-term desktop direction.
9. Pairing and readiness require separate future threat-driven ADRs.
10. Wallet, worker, tools, model artifacts, and mesh remain security-gated.

## Recommendation

- Phase 00 documentation baseline: `CONDITIONAL_GO`, subject to documentation release-gate validation and human review.
- Phase 1A local-only inference foundation: `CONDITIONAL_GO` to planning only. Implementation requires its own written plan, acceptance criteria, security review, and explicit human approval.
- Phase 1B and later runtime phases: `NO_GO` until their prerequisites and reviews are complete.
- Production readiness: `NO_GO`.

The next implementation task, and only that task, is Phase 1A. It must not include SQLite, Supervisor, tools, pairing changes, Reticulum/LXMF, wallet, worker, or autonomous loops.
