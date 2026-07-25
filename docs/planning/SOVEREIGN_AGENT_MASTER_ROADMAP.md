# DAARION Sovereign Agent Master Roadmap

Status: **ACCEPTED BASELINE / PHASE-GATED**

This roadmap records dependency order, not calendar estimates or implementation claims. Every phase follows:

```text
audit -> written plan -> human review -> bounded vertical slice
      -> tests -> security review -> documentation -> diff review
      -> release gate -> roadmap update
```

Only one phase is authorized at a time. A `GO` or `CONDITIONAL_GO` plan does not bypass explicit human review.

## Phase map

| Phase | Goal | Primary repository | Scope and non-goals | Acceptance evidence | Security gate | Complexity |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | Baseline audit, instructions and skills | Both | Docs/instructions only; no runtime changes | Adopted audit, ownership, matrices, ADRs, roadmap, threat model, validation report | No sensitive data; no false implementation claims | M |
| 1A | Local-only inference foundation | Edge | `InferenceProvider`, Ollama adapter, `LocalOnly`, explicit daemon cloud-disabled proof, stable per-model local evidence, model/command mapping, timeout, cancellation, truthful UI, fake/scripted-provider tests. No SQLite/Supervisor/tools/network fallback | Policy/provider unit tests, loopback integration tests, remote-alias and zero-prompt-egress tests, cancellation/deadline tests, UI truth tests, diff/security review | Cloud-disabled plus local-model proof required before prompt; no silent egress | L |
| 1B | Durable runtime state | Edge | SQLite bootstrap/migrations, conversations, messages, tasks, audit events, restart recovery, deletion. Not six-level memory | Migration/replay/transaction/restart/deletion/export tests | Local DB permissions, no secret logging, recovery integrity | L |
| 1C | Inert Agent Supervisor | Edge | Deterministic task IDs, explicit state machine, bounded transitions, recovery integration. No tools/network/autonomous scheduling | State-machine, idempotency, cancellation and crash-recovery tests | Model cannot bypass state/policy; no execution capability | L |
| 2 | Six-level memory evolution | Edge | Repository interfaces then working/conversation/episodic/semantic/procedural/graph memory; embeddings only after ADR | Migration, provenance, retention, deletion/export, poisoning/dedup tests | Extracted memory untrusted; raw memory local | XL |
| 3 | Bounded Loop Runtime | Edge | Versioned definitions, triggers, limits, checkpoints, policy hooks, outcomes; Local Agent Health reference loop only | Completion/no-op/cancel/timeout/limits/restart/duplicate/failed-checkpoint tests | No unbounded loop; no network/tools/signing in reference loop | XL |
| 4 | Read-only Tool Runtime | Edge | Typed registry and READ_ONLY tools only; no mutation/financial/privileged tools | Allowlist, argument/path, denial, confirmation, audit, injection/traversal tests | Permission broker required; fail closed | L |
| 5 | Signed readiness projection | Both | Separate projection ADR, safe schema, producer/consumer, freshness/revocation; no raw memory | Cross-repo fixtures, signature, replay, expiry, privacy and downgrade tests | Minimal allowlist and provenance | L |
| 6 | Reticulum/LXMF transport | Edge + selected mesh component | Transport interface, authenticated IPC, signed envelope, mailbox/offline queue; no agent policy in daemon | IPC authentication, replay/expiry/idempotency, queue recovery and packaging tests | Daemon replacement/spoofing and envelope validation closed | XL |
| 7 | Multi-agent loops | Edge + mesh | Bounded message processing, capability discovery and delegation; no financial actions | Signed capability, duplicate/replay, policy, cancellation and recovery tests | Remote agents are untrusted; bounded delegation | XL |
| 8 | Wallet and economic loops | Edge signer boundary | Separate identities, proposal flow, isolated signing, explicit approval; worker only as separately gated subphase | Derivation/recovery, transaction display, approval, replay and signer-isolation tests | Financial action always approved outside LLM | XL |
| 9 | Hardening and production gates | Both + release components | Threat closure, platform packaging, update/supply-chain, recovery drills, observability | Release matrices, platform evidence, incident/recovery drills, dependency/license review | No unresolved Critical/High finding | XL |

## Phase 1A: completed repository milestone

Required scope:

- define `InferenceProvider`;
- implement an Ollama provider without coupling the domain API to Ollama;
- make `InferencePolicy::LocalOnly` the only enabled MVP policy;
- remove or make simulated remote fallback unreachable;
- require explicit Ollama daemon cloud-disabled evidence and fail closed when
  the capability is absent or incomplete;
- require stable tags/show/tags local-model evidence and revalidate it before
  every prompt-bearing request and after preparation;
- align command names, registry model IDs and Ollama upstream tags;
- enforce request timeout and cancellation;
- make provider/runtime/status UI truthful;
- use fake-provider tests for success, refusal, timeout, cancellation and attempted remote invocation.

Explicit non-goals:

- SQLite or memory;
- Agent Supervisor or loops;
- tools, shell expansion or worker;
- pairing/readiness changes;
- Reticulum/LXMF;
- wallet/signing;
- remote provider implementation.

Entry gate:

1. Phase 00 completion report reviewed.
2. ADR 0001 accepted by a human.
3. A separate `phase-01a-local-only-inference-plan.md` exists with current source inventory and exact files.
4. No unresolved High/Critical Phase 1A blocker.
5. No unrelated dirty changes overlap the target files.

## Phase 1B boundary

The first SQLite schema contains only:

- `schema_migrations`;
- `conversations`;
- `messages`;
- `tasks`;
- `audit_events`.

It must not be described as complete memory. Memory types, embeddings, entities/relations and external vector systems are deferred.

Human decisions HD-01 through HD-09 approve the Phase 1B plan: SQLite is the
authoritative local transactional store; `rusqlite` with `bundled`, `limits`,
and `backup` is owned by a Rust blocking actor; standard plaintext SQLite is
accepted for the foundation with full-disk encryption required on supported
production devices; retention, limits, pragmas, export, and desktop platform
gates are fixed in the phase plan. Semantic and graph stores remain rebuildable
projections, large bytes belong outside SQLite by default, and remote sync is
optional and non-authoritative. This approval does not authorize implementation
or slice 1B.1.

## Future Storage Evolution

The following sequence is a storage-evolution architecture stream, not a
renumbering of the accepted execution phase map above:

```text
Phase 1B — SQLite foundation
    ↓
Phase 2 — semantic index
    ↓
Phase 3 — graph memory
    ↓
Phase 4 — artifact storage
    ↓
Phase 5 — optional remote projections
```

These are architectural placeholders only. Phase 1B owns only authoritative
SQLite transactional state. Phase 2 semantic indexing may be proposed within
the accepted six-level memory phase. The storage-stream labels Phase 3–5 do not
replace or alter the current top-level Phase 3 Loop Runtime, Phase 4 Tool
Runtime, or Phase 5 readiness-projection scopes. Scheduling any graph, artifact,
or remote-storage implementation requires a separately reviewed roadmap change,
future ADR, bounded plan, tests, security gate, and explicit human authorization.

Future semantic and graph stores are rebuildable projections. Artifact bytes
may live outside SQLite, but artifact lifecycle metadata remains authoritative
in SQLite. Remote systems may receive projections, signed summaries, or optional
encrypted backups and never become the canonical runtime writer. No Qdrant,
LanceDB, graph database, object-store product, SQLCipher integration, replication
engine, or cloud backend is selected by this placeholder sequence.

## Phase 1C boundary

The first Supervisor is inert. It can create, transition, cancel, persist and recover tasks, but it cannot call tools, open network connections, schedule itself, sign data, mutate user files or recursively delegate.

## Loop Runtime dependency

Loop Runtime follows local inference, durable state and an inert Supervisor. The runtime, not the LLM, owns budgets, state transitions, approvals and stop conditions. The first health loop consumes injected/local status and has no network, shell, file mutation, remote inference, signing or worker access.

## Future ADR queue

- 0004 — signed device pairing envelope;
- 0005 — signed device readiness projection;
- identity domain separation and recovery;
- tool permission and egress policy;
- loop definition/versioning and recovery;
- wallet signer isolation;
- model manifest/artifact trust;
- Android transport/background execution.
- semantic retrieval engine, provenance and full-index rebuild;
- graph projection engine, reconciliation and rebuild;
- artifact byte storage, integrity and lifecycle metadata;
- pre-production SQLCipher/key-lifecycle decision for database-file encryption;
- optional remote projection, encrypted-backup or replication boundary.

Numbers are reservations for planning, not accepted decisions.

## Current gate

- Baseline documentation: adopted with the limitations recorded in the Phase 00 completion report.
- Phase 1A: `MERGED / FRESH-MAIN VERIFIED / PASS` after focused diff/security review plus
  probe-deadline, request-scoped model-preparation cancellation, and P1 local
  model verification corrections. The service now requires explicit daemon
  cloud-disabled proof and stable tags/show/tags evidence for readiness,
  inventory, immediate pre-chat authorization and post-preparation success.
  Remote/copy aliases, stale evidence and unsupported policy capability fail
  closed; rejected-path fixtures prove zero chat calls and zero sentinel prompt
  egress. Health/inventory probes, chat and preparation remain bounded and
  cancellable. 67 inference and 116 full Rust tests pass. Every changed Rust
  file, contract, build and scoped security check passes. Repository-wide
  formatting debt is pre-existing, reduced from 101 baseline files to 94 and
  tracked separately. Live Ollama/model behavior, malicious-daemon attestation,
  cryptographic artifact trust and daemon-side cancellation remain unverified.
  PR #24 was reviewed at `9e8c5d9c8adb4c02bfa9b11e970e33a0bbfd640f`
  and merged as `62a1d514b93925e8b7098c6db19f8751a70a7bf8`;
  fresh-main verification passed. No live Ollama smoke is claimed.
- Phase 1B planning: `APPROVED / HUMAN_DECISIONS_RECORDED`; the docs-only
  plan and Rust 1.95.0 toolchain unblock are merged.
- Phase 1B implementation: `PARTIALLY_IMPLEMENTED`; Phase 1B.1 is merged and
  fresh-main verified. The separately authorized Phase 1B.2 slice is also
  merged and fresh-main verified; the broader Phase 1B remains incomplete.
- Phase 1B.1: `MERGED / FRESH_MAIN_VERIFIED`. The final independently reviewed
  head `5d894f42a967c9360d86382c1aab9e603472e0c8` was merged as
  `cd903fb18d1618bbe0787d2397948622849ef9d4` at
  `2026-07-24T11:44:00Z`. The SQLite bootstrap and safe Storage Runtime
  projection are `IMPLEMENTED_AND_VERIFIED`; the broader durable runtime state
  remains `PARTIALLY_IMPLEMENTED`. Accepted ADR 0006 defines the command-scoped
  module boundary and import/re-export graph as the primary control, with
  limited AST checks as defense in depth. Arbitrary TypeScript data-flow proof
  is not claimed, a custom full data-flow analyzer is rejected, and global
  frontend adapter migration is deferred to a separate phase. The verified
  gate includes 29/29 primary fixtures, 13/13 secondary fixtures, 46 structural
  checks, 64 storage, 67 inference and 180 full Rust tests; `cargo check`,
  `cargo clippy --all-targets`, the production build (1,763 modules), and
  production npm audit (0 findings) passed under Rust 1.95.0. Runtime-store
  warning locations are 0. Dev-inclusive npm audit still reports 11 inherited
  findings; inherited RustSec, warning and rustfmt debt are unchanged.
- Phase 1B.1 limitations: the real desktop restart flow and cross-platform
  runtime are not verified. Remote CI is not present and is not claimed. Remote
  production writes, real user-profile writes and deployments were all 0.
- Phase 1B.2: `MERGED / FRESH_MAIN_VERIFIED`. Reviewed implementation head
  `c2fdcc5a234779c7ad886ee5aa0d0762c938a59d` was merged as
  `ec99bf70d6ada94bc1caae9886cca25ad42852f9` at
  `2026-07-25T14:27:32Z`. Exactly five private Rust operations cover
  create/get/list conversations and append/list messages. Two mutations bind
  subject, global operation-ID evidence and audit in one transaction. The
  16-MiB reserve, 8-/32-MiB aggregate envelopes and 2-/4-MiB WAL bounds passed
  20 create plus 20 append measurements. The local gate passed 36 repository,
  100 runtime-store, 67 inference and 216 full Rust tests, Cargo check/Clippy,
  frontend contracts, production build, production npm audit and secret scan.
  No schema, dependency, Tauri/frontend content API, production or real-profile
  write was added. Private conversation and message storage are
  `IMPLEMENTED_AND_VERIFIED`; durable runtime state remains
  `PARTIALLY_IMPLEMENTED`.
- Phase 1B.3: `NOT AUTHORIZED`; task services and later durable-state behavior
  remain out of scope.
- Phase 1C and later: `NO_GO`.
- Production readiness: `NO_GO`.

```text
PHASE_1A =
MERGED / FRESH-MAIN VERIFIED / PASS

PHASE_1B_PLANNING =
APPROVED / MERGED

PHASE_1B_IMPLEMENTATION =
PARTIALLY_IMPLEMENTED

PHASE_1B_1 =
MERGED / FRESH_MAIN_VERIFIED

MERGED_REVIEWED_HEAD =
5d894f42a967c9360d86382c1aab9e603472e0c8

MERGE_COMMIT =
cd903fb18d1618bbe0787d2397948622849ef9d4

MERGED_AT =
2026-07-24T11:44:00Z

STORAGE_BOOTSTRAP =
IMPLEMENTED_AND_VERIFIED

STORAGE_RUNTIME_PROJECTION =
IMPLEMENTED_AND_VERIFIED_IN_REPOSITORY

DURABLE_RUNTIME_STATE =
PARTIALLY_IMPLEMENTED

PHASE_1B =
NOT COMPLETE

PHASE_1B2 =
MERGED / FRESH_MAIN_VERIFIED

MERGED_IMPLEMENTATION_HEAD =
c2fdcc5a234779c7ad886ee5aa0d0762c938a59d

PHASE_1B2_MERGE_COMMIT =
ec99bf70d6ada94bc1caae9886cca25ad42852f9

PHASE_1B2_MERGED_AT =
2026-07-25T14:27:32Z

PRIVATE_CONVERSATION_STORAGE =
IMPLEMENTED_AND_VERIFIED

PRIVATE_MESSAGE_STORAGE =
IMPLEMENTED_AND_VERIFIED

PUBLIC_CONTENT_TAURI_COMMANDS =
0

STORAGE_STATUS_TAURI_COMMANDS =
1

FRONTEND_CONTENT_AUTHORITY =
0

PHASE_1B3 =
NOT AUTHORIZED

NEXT_AUTHORIZED_ACTION =
SEPARATE PHASE 1B.3 AUDIT / PLAN / HUMAN AUTHORIZATION

REAL_DESKTOP_RESTART =
NOT VERIFIED

CROSS_PLATFORM_RUNTIME =
NOT VERIFIED

REMOTE_CI =
NOT PRESENT / NOT CLAIMED

REMOTE_PRODUCTION_WRITES =
0

REAL_USER_PROFILE_WRITES =
0

DEPLOYMENTS =
0
```

Phase 1B.2 merged and fresh-main-verified evidence:

```text
CONTENT_OPERATIONS = 5 PRIVATE RUST ONLY
CONTENT_MUTATIONS = 2
CONTENT_READS = 3
PUBLIC_CONTENT_TAURI_COMMANDS = 0
SCHEMA_CHANGE = NONE
DEPENDENCY_CHANGE = NONE
HARD_RESERVE = 16 MiB IMMUTABLE
CREATE_MAX_AGGREGATE_GROWTH = 32,960 bytes
CREATE_MAX_WAL_GROWTH = 32,960 bytes
APPEND_MAX_AGGREGATE_GROWTH = 313,120 bytes
APPEND_MAX_WAL_GROWTH = 313,120 bytes
HARD_DATABASE_LIMIT = 4294967296 bytes
IMMUTABLE_RESERVE = 16777216 bytes
NORMAL_MUTATION_USABLE_LIMIT = 4278190080 bytes
CREATE_OPERATION_ENVELOPE = 8388608 bytes
APPEND_OPERATION_ENVELOPE = 33554432 bytes
WAL_AUTOCHECKPOINT = 128 pages
PHYSICAL_WAL_CEILING = 10485760 bytes
CREATE_WAL_BOUND = 2097152 bytes
APPEND_WAL_BOUND = 4194304 bytes
EXECUTABLE_GROWTH_PROOF = 40/40 PASS
REPOSITORY_TESTS = 36/36 PASS
RUNTIME_STORE_TESTS = 100/100 PASS
INFERENCE_TESTS = 67/67 PASS
FULL_RUST_TESTS = 216/216 PASS
RUNTIME_STORE_WARNING_LOCATIONS = 0
```

Phase 1B.2 fresh-main evidence and schema invariants:

```text
RUST_TOOLCHAIN = 1.95.0 PINNED
REPOSITORY_TESTS = 36/36 PASS
RUNTIME_STORE_TESTS = 100/100 PASS
INFERENCE_TESTS = 67/67 PASS
FULL_RUST_TESTS = 216/216 PASS
CARGO_CHECK = PASS
CARGO_CLIPPY = PASS
RUNTIME_STORE_WARNING_LOCATIONS = 0
PRIMARY_BOUNDARY_FIXTURES = 29/29 PASS
DEFENSE_IN_DEPTH_FIXTURES = 13/13 PASS
STRUCTURAL_CHECKS = 46 PASS
PRODUCTION_BUILD = PASS / 1,763 MODULES
PRODUCTION_NPM_AUDIT = 0 VULNERABILITIES
NPM_DEV_INCLUSIVE_ADVISORIES = 11 INHERITED / OUTSIDE PRODUCTION DEPENDENCY SET
INHERITED_RUSTSEC = UNCHANGED
INHERITED_RUSTFMT_DEBT = 94 FILES / UNCHANGED
MIGRATION_SHA = 62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d
STRUCTURAL_FINGERPRINT = 37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77
TABLES = 5
EXPLICIT_INDEXES = 7
SQLITE_AUTOINDEXES = 7
SQLITE_SEQUENCE = 0
MIGRATION_2 = ABSENT
```
