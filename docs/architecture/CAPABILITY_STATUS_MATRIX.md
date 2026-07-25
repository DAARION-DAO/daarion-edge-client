# Sovereign Agent Capability Status Matrix

Status: **PHASE 1A MERGED / PHASE 1B.1 MERGED / PHASE 1B.2 LOCAL GATE PASS AND INDEPENDENT REVIEW PENDING — 2026-07-25**

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
| Storage bootstrap | Edge | `IMPLEMENTED_AND_VERIFIED` | Reviewed head `5d894f42a967c9360d86382c1aab9e603472e0c8` merged as `cd903fb18d1618bbe0787d2397948622849ef9d4` at `2026-07-24T11:44:00Z` and passed fresh-main verification: one Rust-owned SQLite 3.53.2 connection on a bounded worker, immutable version-1 migration, exactly five empty application tables, seven explicit indexes, seven SQLite autoindexes, no `sqlite_sequence`, and no migration 2. Migration SHA and structural fingerprint remain exact | Later Phase 1B content services require separate authorization |
| Storage runtime projection | Edge | `IMPLEMENTED_AND_VERIFIED` | One no-user-argument read-only Rust status command, typed frontend adapter, private command constant, no raw Tauri export, and mounted Dashboard card passed 29/29 primary boundary fixtures, 13/13 defense-in-depth fixtures and 46 structural checks on fresh main. Remote CI was not present; repository verification is local exact-head evidence | Real desktop restart and cross-platform runtime remain unverified |
| Durable runtime state | Edge | `PARTIALLY_IMPLEMENTED` | Phase 1B.1 is merged and fresh-main verified. Phase 1B.2 now has a private Rust-only five-operation conversations/messages implementation in a draft PR with local gate pass and independent exact-head review pending: 36 repository, 100 runtime-store, 67 inference and 216 full Rust tests pass; 20 create plus 20 append growth proofs stay within the immutable 16-MiB reserve and 2-/4-MiB WAL bounds. It adds no Tauri/frontend content API, migration, dependency or production write. This is not merged evidence. Tasks, retention, deletion/export/backup, full recovery/privacy closure, six-level memory and Phase 1B.3 remain absent/unauthorized. `PHASE_1B = NOT COMPLETE` | `PHASE_1B_2 / INDEPENDENT_EXACT_HEAD_REVIEW` |
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
- `IMPLEMENTED_AND_VERIFIED` is limited to the cited merged Phase 1A and Phase
  1B.1 repository checks and fresh-main readbacks. Phase 1B.2 remains
  `PARTIALLY_IMPLEMENTED` until exact-head review, separate ready/merge
  authorization and fresh-main verification. None of these states imply a live Ollama
  run, real desktop storage restart, cross-platform execution, packaging proof,
  deployment truth or production readiness.
- A live/deployed result must be recorded separately from repository evidence.
- Status changes require evidence, date, command/result or deployed proof, and documentation update.
- Phase 1B.2 candidate evidence claims only five private Rust
  conversation/message operations. It does not claim public content CRUD,
  task services, memory, retention, deletion/export/backup or a completed
  Phase 1B release.

Current Phase 1B.1 evidence boundary:

```text
PHASE_1B_1 = MERGED / FRESH_MAIN_VERIFIED
MERGED_REVIEWED_HEAD = 5d894f42a967c9360d86382c1aab9e603472e0c8
MERGE_COMMIT = cd903fb18d1618bbe0787d2397948622849ef9d4
MERGED_AT = 2026-07-24T11:44:00Z
STORAGE_BOOTSTRAP = IMPLEMENTED_AND_VERIFIED
STORAGE_RUNTIME_PROJECTION = IMPLEMENTED_AND_VERIFIED_IN_REPOSITORY
DURABLE_RUNTIME_STATE = PARTIALLY_IMPLEMENTED
PHASE_1B = NOT COMPLETE
PHASE_1B_2_AT_PHASE_1B1_MERGE = NOT AUTHORIZED
REAL_DESKTOP_RESTART_FLOW = NOT VERIFIED
CROSS_PLATFORM_RUNTIME = NOT VERIFIED
REMOTE_CI = NOT PRESENT / NOT CLAIMED
REMOTE_PRODUCTION_WRITES = 0
REAL_USER_PROFILE_WRITES = 0
DEPLOYMENTS = 0
```

Current Phase 1B.2 candidate boundary:

```text
PHASE_1B_2 = IMPLEMENTED_IN_DRAFT_PR / LOCAL_GATE_PASS
INDEPENDENT_EXACT_HEAD_REVIEW = PENDING
MERGED = NO
CONTENT_OPERATIONS = 5 PRIVATE RUST ONLY
CONTENT_MUTATIONS = 2
CONTENT_READS = 3
PUBLIC_CONTENT_TAURI_COMMANDS = 0
SCHEMA_CHANGE = NONE
DEPENDENCY_CHANGE = NONE
HARD_RESERVE = 16 MiB IMMUTABLE
GROWTH_PROOF = 20 CREATE + 20 APPEND / 0 FAILURES
RUNTIME_STORE_TESTS = 100/100 PASS
INFERENCE_TESTS = 67/67 PASS
FULL_RUST_TESTS = 216/216 PASS
PHASE_1B = NOT COMPLETE
PHASE_1B_3 = NOT AUTHORIZED
REMOTE_PRODUCTION_WRITES = 0
REAL_USER_PROFILE_WRITES = 0
DEPLOYMENTS = 0
```

Fresh-main verification preserved these exact evidence and schema boundaries:

```text
RUST_TOOLCHAIN = 1.95.0 PINNED
STORAGE_TESTS = 64/64 PASS
INFERENCE_TESTS = 67/67 PASS
FULL_RUST_TESTS = 180/180 PASS
CARGO_CHECK = PASS
CARGO_CLIPPY = PASS
RUNTIME_STORE_WARNING_LOCATIONS = 0
PRIMARY_BOUNDARY_FIXTURES = 29/29 PASS
DEFENSE_IN_DEPTH_FIXTURES = 13/13 PASS
STRUCTURAL_CHECKS = 46 PASS
PRODUCTION_BUILD = PASS / 1,763 MODULES
PRODUCTION_NPM_AUDIT = 0 VULNERABILITIES
NPM_DEV_INCLUSIVE_ADVISORIES = 11 INHERITED / OUTSIDE PRODUCTION DEPENDENCY SET
INHERITED_RUSTSEC_WARNING_RUSTFMT_DEBT = UNCHANGED
MIGRATION_SHA = 62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d
STRUCTURAL_FINGERPRINT = 37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77
TABLES = 5
EXPLICIT_INDEXES = 7
SQLITE_AUTOINDEXES = 7
SQLITE_SEQUENCE = 0
MIGRATION_2 = ABSENT
```
