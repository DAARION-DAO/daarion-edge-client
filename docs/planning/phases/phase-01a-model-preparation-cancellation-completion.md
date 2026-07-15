# Phase 1A Model Preparation Cancellation Completion

Final repository release result: **PASS**

Merge action: **NOT PERFORMED**. This correction belongs to existing PR #24
only. Phase 1B remains **NO_GO** until Phase 1A is separately merged and
verified from fresh `main`.

## Defect and Root Cause

Fresh Codex review of head
`b4c25533a5ae6147e4c380d36baa5809611901ed` opened substantive thread
`PRRT_kwDOR7OvXc6Qz0F9`. Model preparation had a one-hour service timeout but no
request ID, no active-operation registration, no cancellation token at the
provider boundary, and no mounted UI Cancel action. A large or stalled
loopback Ollama pull could therefore keep the DAARION operation active until
success, timeout, or application exit.

## Correction Architecture

- `PrepareLocalModelRequest` requires a UUID `request_id` and canonical
  `canonical_model_id`.
- `ModelPreparationResponse` returns only controlled request, canonical model,
  and provider identifiers after explicit success.
- The previous chat-only registry is generalized into one service-owned
  operation registry with `Chat` and `ModelPreparation` kinds.
- Duplicate IDs are rejected across operation kinds. Kind-specific cancellation
  prevents preparation from cancelling chat or another request.
- RAII cleanup removes every completed, cancelled, timed-out, rejected, or
  failed operation.
- The existing one-hour production preparation deadline is now an injectable
  `ServiceLimits` value and starts after registration, before concurrency wait.
- Cancellation, timeout, provider failure, and success are mutually exclusive
  command results; cancellation and timeout drop the provider future.
- `InferenceProvider::prepare_model` accepts a shared watch-based
  `OperationControl`.
- Ollama pull uses streamed progress, races control against request send and
  every response-body chunk, and reuses the bounded NDJSON decoder.
- Progress must contain one final `status: success`; malformed, oversized,
  error, incomplete, and post-terminal records fail closed.
- No preparation progress is logged or mixed with chat token events.

## Tauri and Mounted UI Contract

The public Tauri surface now accepts only:

```text
prepare_local_model({ request_id: UUID, canonical_model_id })
cancel_local_model_preparation(request_id)
```

Provider-specific Ollama tags remain private to `ModelResolver`. The mounted
panel generates the UUID, retains separate active chat/preparation ownership,
and exposes these states:

- `preparing`;
- `cancelling`;
- `cancelled`;
- `timed_out`;
- `failed`;
- `completed_locally`.

While preparation is active, the panel exposes an enabled **Cancel local model
preparation** action. After cancellation, the command future returns controlled
`cancelled`, the spinner stops, the active ID is cleared, and the user can start
a new preparation. The UI says that the local request was cancelled and that
Ollama may retain resumable progress; it does not claim that all daemon-side
download or disk work stopped.

## Upstream Ollama Limitation

Official Ollama documentation describes streamed `/api/pull` progress and
states that cancelled pulls may resume and concurrent callers can share
progress:

- <https://docs.ollama.com/api/pull>
- <https://github.com/ollama/ollama/blob/main/docs/api.md#pull-a-model>

No reviewed upstream contract provides a daemon-wide cancel endpoint or
guarantees immediate termination of all server-side network/disk work when one
client disconnects. Repository evidence therefore proves only:

- prompt termination of the DAARION future;
- response/request-stream drop and loopback disconnect in controlled fixtures;
- deterministic local registry and UI cleanup;
- no later DAARION completion after cancellation or timeout.

Server-side Ollama termination, artifact trust, digest/signature verification,
partial-file cleanup, and real download behavior remain unverified and
security-gated.

## Changed Files

Application and contract:

- `src-tauri/src/inference/types.rs`;
- `src-tauri/src/inference/provider.rs`;
- `src-tauri/src/inference/service.rs`;
- `src-tauri/src/inference/ollama_provider.rs`;
- `src-tauri/src/inference/commands.rs`;
- `src-tauri/src/lib.rs`;
- `src/lib/inferenceClient.ts`;
- `src/components/LocalInferencePanel.tsx`;
- `scripts/validate-inference-contract.mjs`.

Evidence:

- `docs/planning/phases/phase-01a-model-preparation-cancellation-plan.md`;
- this completion report;
- primary Phase 1A completion report;
- ADR 0001;
- capability status matrix;
- security gate;
- master roadmap.

No manifest, dependency, lockfile, bundled model registry, capability,
database, memory, Supervisor, loop, tool, transport, pairing, readiness,
wallet, worker, web, CI, deployment, or production file changed.

## Deterministic Test Evidence

The focused preparation filter contains 13 tests: the existing provider
preparation test plus 12 new correction tests. No test contacts public network,
requires Ollama, or downloads a model.

| Behavior | Result |
| --- | --- |
| UUID preparation request, typed success, finished cleanup | PASS |
| invalid UUID and arbitrary Ollama tag rejected before provider | PASS |
| duplicate active preparation ID rejected | PASS |
| cross-kind request-ID alias rejected | PASS |
| cancellation during concurrency wait; provider never entered | PASS |
| accepted loopback socket stalled before headers | PASS / prompt `Cancelled` |
| streamed body stalled after controlled progress | PASS / prompt `Cancelled` |
| fixture observes client connection/stream drop after cancellation | PASS |
| cancelled request cannot later time out or complete | PASS |
| timed-out delayed success cannot later complete | PASS |
| preparation A cancellation leaves preparation B active | PASS |
| preparation cancellation leaves active chat untouched | PASS |
| unknown, finished and cancelled IDs fail safely after cleanup | PASS |
| fast preparation success remains compatible | PASS |
| provider failure remains controlled and cleans registry | PASS |
| malformed, error, incomplete, post-terminal and oversized progress | PASS |
| sensitive fixture body absent from public error | PASS |
| existing probe, chat timeout/cancel, NDJSON and terminal-event tests | PASS |

Stalled fixtures bind actual `127.0.0.1:0` listeners, accept the request, keep
the socket open before headers or during an incomplete body, cancel through the
public service control path, enforce 500-millisecond outer guards, observe
connection termination, and await fixture tasks. Test preparation deadlines are
20/200 milliseconds or two seconds according to the tested path; the production
default remains one hour.

## Validation Results

| Command | Result |
| --- | --- |
| pre-correction `cargo test --manifest-path src-tauri/Cargo.toml inference:: --lib` | PASS, 43 tests |
| pre-correction `cargo test --manifest-path src-tauri/Cargo.toml` | PASS, 92 tests |
| `cargo test --manifest-path src-tauri/Cargo.toml preparation_ --lib` | PASS, 13 tests |
| `cargo test --manifest-path src-tauri/Cargo.toml inference:: --lib` | PASS, 55 tests |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS, 104 tests |
| `cargo check --manifest-path src-tauri/Cargo.toml` | PASS |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets` | PASS command exit; 0 findings in `src/inference/**` |
| scoped rustfmt for 5 changed inference files | PASS |
| `rustfmt --config skip_children=true --check src-tauri/src/lib.rs` | PASS |
| `npm ci` | PASS; lockfile unchanged |
| `npm run test:inference-contract` | PASS, 7 typed commands and mounted Cancel assertion |
| `npm run build` | PASS (`tsc` and Vite production build) |
| `npm audit --omit=dev --json` | PASS, 0 production findings |
| `bash scripts/check-no-secrets.sh` | PASS |
| `bash scripts/check-rust-touched-warnings.sh` | PASS |
| `git diff --check` | PASS |
| LocalOnly, redirect, proxy, capability, shell, log and scope review | PASS |
| repository-wide `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | expected pre-existing failure across 94 legacy files |

No frontend lint or frontend unit-test command is configured. TypeScript
validation is part of `npm run build`; the deterministic adapter/Rust/UI source
contract supplies the scoped frontend assertion without adding a dependency.

`npm ci` still reports the unchanged baseline of 11 development-dependency
findings (2 low, 4 moderate, 5 high). The production-only audit reports zero.
No dependency update was authorized or performed.

## Warning and Formatting Comparison

- Pre-correction `cargo check` warnings: 312.
- Post-correction `cargo check` warnings: 312.
- New warnings in `src/inference/**`: 0.
- Clippy findings in `src/inference/**`: 0.
- Changed Rust file scoped rustfmt: PASS, 6/6.
- Repository-wide formatting debt: unchanged at 94 legacy files and tracked in
  `docs/planning/RUST_FORMATTING_DEBT.md`.

## Security Review

| Finding | Severity | Result |
| --- | --- | --- |
| Non-cancellable DAARION pull future | MEDIUM | Closed by request-scoped service registration, cancel, deadline and provider control |
| Cosmetic UI-only cancellation | MEDIUM | Closed by public service-path loopback tests and observed stream/connection drop |
| Preparation cancel affects chat or another preparation | HIGH hypothesis | Not observed; kind-aware registry and isolation tests pass |
| Cancellation/timeout becomes late success | MEDIUM | Closed by biased select, dropped future, cleanup and delayed-result tests |
| Arbitrary provider tag crosses IPC | HIGH hypothesis | Not introduced; canonical request and resolver rejection test pass |
| Unbounded/malformed progress | MEDIUM | Closed within correction by bounded NDJSON and terminal validation |
| Provider response/endpoint disclosure | MEDIUM | Closed within correction; no raw body/log and sentinel test passes |
| False daemon-stop claim | MEDIUM | Avoided by explicit UI/docs limitation |
| Remote fallback, proxy, redirect, shell or capability expansion | HIGH hypothesis | Not introduced; static/source/diff checks pass |

No unresolved Critical or High correction finding remains. Model artifact
supply chain and upstream daemon-side behavior remain open later-phase gates,
not silently accepted risks.

## Acceptance Matrix

- typed UUID and canonical-only preparation request: **PASS**
- registration before queue/provider execution: **PASS**
- duplicate and cross-kind ID rejection: **PASS**
- dedicated, kind-isolated preparation cancel command: **PASS**
- cancellation during send/header/body/queue wait: **PASS**
- deterministic cleanup on every tested terminal path: **PASS**
- cancellation and timeout distinct; no late DAARION success: **PASS**
- bounded streamed progress with explicit terminal success: **PASS**
- mounted enabled Cancel action and truthful typed states: **PASS**
- upstream server-side limitation disclosed: **PASS**
- LocalOnly, probes, chat cancellation/deadline, NDJSON and sole chat terminal
  events preserved: **PASS**
- dependencies, lockfiles, capabilities and phase boundaries: **PASS**
- complete Phase 1A release gate: **PASS** under the existing focused-format
  amendment
- live Ollama/model smoke: **NOT PERFORMED / NOT CLAIMED**
- production readiness: **NOT CLAIMED**
- production writes/deployments: **0**

## Remaining Limitations

- Controlled fixtures prove DAARION-side cancellation, not undocumented Ollama
  daemon behavior.
- No real model was pulled and no installed mapped model was executed.
- Artifact digest/signature verification and partial-file policy remain open.
- Active operations are process-local and do not recover after restart; durable
  runtime state remains Phase 1B.
- Repository-wide Rust formatting debt remains open across 94 files.
- Phase 1B, Supervisor, memory and Loop Runtime remain out of scope and `NO_GO`.

## Rollback

Before merge, revert the correction commit on the existing PR branch. A
rollback must also remove or disable mounted model preparation; it cannot
restore the known non-cancellable button while retaining repository PASS. No
data, dependency, lockfile, or deployment rollback exists.

## Release Gate and Next Action

**PASS.** After this evidence is committed and pushed to the existing PR branch,
reply to top-level review comment `3580514538`, re-fetch thread
`PRRT_kwDOR7OvXc6Qz0F9`, and resolve it only if GitHub shows the correction at
the new head. Update PR #24 evidence, trigger `@codex review`, keep the PR open
and unmerged, and classify the external gate as pending until the fresh review
finishes. The next possible implementation action is another narrow correction
only if that review finds a substantive blocker.
