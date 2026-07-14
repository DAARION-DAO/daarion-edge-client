# Phase 1A Model Preparation Cancellation Plan

Status: **GO**

## Objective

Close review thread `PRRT_kwDOR7OvXc6Qz0F9` with one bounded correction that
makes local Ollama model preparation request-scoped, deadline-bounded,
cancellable, kind-isolated, and truthful in the mounted UI. Preserve the
accepted Phase 1A local-only inference architecture and stop before merge.

## Confirmed Defect

At PR head `b4c25533a5ae6147e4c380d36baa5809611901ed`,
`prepare_local_model` accepts only a canonical model ID and calls
`InferenceService::prepare_model`. The service applies a one-hour timeout but
does not register the preparation in `active_requests`. The provider performs a
non-streaming `/api/pull`, and the mounted UI exposes a disabled spinner with no
cancel action. `cancel_local_inference` can therefore cancel chat but cannot
stop the DAARION-side preparation future before completion, timeout, or process
exit.

This is a confirmed bounded-execution and user-control defect. It is not
evidence that remote inference occurred, and the correction must not be
represented as an artifact-trust or production-readiness solution.

## Current State

- Repository: `DAARION-DAO/daarion-edge-client`.
- Existing PR: `#24`, branch `phase-01a/local-only-inference`.
- Pre-correction head: `b4c25533a5ae6147e4c380d36baa5809611901ed`.
- PR: open, ready, mergeable/clean, and unmerged according to GitHub.
- Review thread: unresolved, not outdated, and created by the fresh Codex review
  of the exact pre-correction head.
- Worktree: clean dedicated checkout of the existing PR branch.
- Baseline inference tests: 43 passed, 0 failed.
- Baseline full Rust tests: 92 passed, 0 failed.
- Baseline Rust warning inventory: 312 repository warnings and 0 in
  `src/inference/**`.
- Baseline repository-wide formatting debt: 94 legacy files; every changed
  Rust file must pass scoped rustfmt.
- Live Ollama/model smoke: not performed and not authorized for this correction.

## Current Call Path

```text
LocalInferencePanel::handlePrepare
  -> prepareLocalModel(canonicalModelId)
     -> prepare_local_model Tauri command
        -> InferenceService::prepare_model(canonicalModelId)
           -> ModelResolver::resolve(canonicalModelId)
           -> timeout(1 hour, InferenceProvider::prepare_model(providerTag))
              -> OllamaProvider POST /api/pull { stream: false }
```

The active chat path separately creates a UUID, registers a watch sender in
`active_requests`, and exposes `cancel_local_inference`. Preparation bypasses
that ownership and cancellation path.

## Scope

- Add typed preparation request/response contracts with a UUID request ID and
  canonical DAARION model ID.
- Safely generalize the existing active-request registry into one operation
  registry with explicit `Chat` and `ModelPreparation` kinds.
- Register preparation before concurrency admission or provider execution.
- Add a dedicated typed `cancel_local_model_preparation` command.
- Preserve a service-owned one-hour production deadline with an injectable test
  value.
- Extend the provider preparation boundary with the same watch-based
  cancellation primitive used by the service.
- Stream and bound Ollama pull progress with the existing NDJSON decoder.
- Add truthful mounted UI states and an enabled preparation Cancel action.
- Add deterministic service, loopback-provider, UI-contract, isolation, race,
  cleanup, and regression tests.
- Update only directly affected Phase 1A evidence and PR metadata after checks.

## Cancellation Ownership

`InferenceService` remains the canonical owner of operation registration,
deadline, kind validation, cancellation dispatch, and cleanup. The frontend
supplies only a UUID and canonical model ID; it cannot alter cancellation policy
or timeout. The provider receives a cancellation control so its request-send
and body-stream waits can terminate promptly, but it does not own the policy.

One operation registry will store request ID, operation kind, and watch sender.
The same duplicate-ID and RAII cleanup rules apply to chat and preparation.
Kind-aware cancellation prevents a preparation command from cancelling chat or
vice versa. No second unrelated registry or cancellation vocabulary is added.

## Request-ID Contract

```text
PrepareLocalModelRequest {
  request_id: UUID,
  canonical_model_id: canonical DAARION registry ID
}
```

The service validates the UUID and resolves the canonical ID before provider
execution. The provider-only Ollama tag remains private to `ModelResolver` and
cannot enter through IPC. Duplicate IDs are rejected across all active
operation kinds. Unknown, wrong-kind, or completed request cancellation returns
`false` without affecting another operation.

Successful completion returns a typed response containing controlled request,
canonical model, and provider identifiers. Cancellation, timeout, and failure
use the existing stable public error codes and are mutually exclusive terminal
command outcomes.

## Provider Cancellation Boundary

`InferenceProvider::prepare_model` will accept a reusable watch-based operation
control. `OllamaProvider` will:

1. check cancellation before request construction;
2. race cancellation against request send and response headers;
3. request streamed pull progress;
4. race cancellation against every response-body chunk;
5. drop the HTTP future/stream on cancellation or service timeout;
6. require exactly one final `status: success` record;
7. reject malformed, oversized, provider-error, incomplete, and post-terminal
   records with controlled errors.

Preparation progress is validation data only in this correction. It will not be
mixed with chat token events or exposed as an unbounded UI/log stream.

## Deadline Interaction

- Production preparation timeout remains one hour and moves into
  `ServiceLimits` as the canonical injectable value.
- One absolute deadline begins after registration and covers concurrency wait,
  provider request creation, connection, headers, streamed body, decoding, and
  completion.
- `tokio::select!` is biased to cancellation, then timeout, then provider work.
- `cancelled`, `timed_out`, `failed`, and successful completion remain distinct.
- Dropping a provider future after cancellation/timeout prevents a later
  DAARION success result.
- No global reqwest timeout is added; chat and probe deadlines remain unchanged.

## UI State Contract

The mounted panel will explicitly represent:

- `preparing`;
- `cancelling`;
- `cancelled`;
- `timed_out`;
- `failed`;
- `completed_locally`.

The preparation request ID is generated with `crypto.randomUUID()` and retained
only for the active UI operation. While preparation is active, the spinner is
paired with an enabled Cancel action. Cancellation invokes only
`cancel_local_model_preparation`; it stops the spinner after the command future
terminates, suppresses late success, and allows a new preparation after cleanup.
The UI must say only that the local DAARION operation was cancelled, not that
all upstream download or disk activity stopped.

## Upstream Ollama Limitation Analysis

The official Ollama pull contract documents streamed NDJSON status records and
a final `status: success`. The upstream repository documentation also says that
cancelled pulls may be resumed and multiple calls can share download progress:

- <https://docs.ollama.com/api/pull>
- <https://github.com/ollama/ollama/blob/main/docs/api.md#pull-a-model>

Neither source defines a dedicated daemon-wide cancellation endpoint or
guarantees that dropping one client request immediately stops all server-side
network or disk work. The correction can truthfully guarantee only that the
DAARION request future and response stream terminate promptly, the client
connection is dropped where observable, UI leaves the active state, the local
registry is cleaned, and no DAARION success follows cancellation. It must not
claim that the Ollama daemon stopped all work.

This is a useful fail-closed UI boundary because the user regains DAARION-side
control and the surface communicates the upstream limitation. Model artifact
trust and daemon lifecycle remain separately security-gated.

## Repository Ownership

The correction belongs only to `daarion-edge-client`, which owns local model
lifecycle, Tauri IPC, service cancellation, provider adaptation, and mounted
local-runtime UI. It changes no `loval-echoes`, Supabase, MicroDAO, pairing,
readiness, transport, wallet, or worker contract.

## Files and Modules Expected to Change

Application and deterministic contract files:

- `src-tauri/src/inference/types.rs`;
- `src-tauri/src/inference/provider.rs`;
- `src-tauri/src/inference/service.rs`;
- `src-tauri/src/inference/ollama_provider.rs`;
- `src-tauri/src/inference/commands.rs`;
- `src-tauri/src/lib.rs`;
- `src/lib/inferenceClient.ts`;
- `src/components/LocalInferencePanel.tsx`;
- `scripts/validate-inference-contract.mjs`.

Evidence files:

- this plan and matching completion report;
- Phase 1A primary completion report;
- ADR 0001 implementation evidence;
- capability status matrix;
- security gate;
- master roadmap;
- PR #24 body after final checks.

No manifest, lockfile, registry data, capability permission, CI, deployment, or
other repository file is expected to change. Stop for review if an unrelated
module becomes necessary.

## Contracts Affected

- Tauri preparation input changes from an unscoped canonical string to a typed
  request containing `request_id` and `canonical_model_id`.
- Tauri preparation returns a typed controlled success response.
- One new command, `cancel_local_model_preparation`, targets preparation only.
- The provider preparation method receives cancellation control.
- The mounted TypeScript adapter and deterministic validation script are
  updated in the same commit.

Chat event schemas, registry payloads, model mappings, LocalOnly policy,
cross-repository contracts, and public network destinations do not change.

## Explicit Non-Goals

- No SQLite, durable state, memory, Agent Supervisor, Loop Runtime, scheduler,
  tools, Reticulum/LXMF, messaging, pairing, Supabase, readiness projection,
  wallet, worker, or Phase 1B work.
- No remote/cloud inference, fallback, public-network test, real model pull,
  deployment, production write, or production-readiness claim.
- No model manifest/artifact trust, digest policy, disk cleanup, or daemon
  lifecycle manager.
- No dependency, lockfile, registry-data, CI, capability, shell permission, or
  packaging change.
- No progress-event protocol or reuse of chat token events for preparation.
- No claim that Ollama server-side download or disk writes definitely stop.
- No web repository or PR #29 change.

## Security Considerations

| Risk | Severity | Planned control |
| --- | --- | --- |
| Non-cancellable local pull consumes network/disk | MEDIUM | Service registration, deadline, dedicated cancel, provider future/stream drop, truthful UI |
| Cancellation only changes UI state | MEDIUM | Public service-path tests using stalled loopback sockets and registry cleanup assertions |
| Wrong-kind cancellation affects chat | HIGH hypothesis | One kind-aware registry and explicit isolation tests; unresolved failure blocks release |
| Cancel/timeout races become late success | MEDIUM | Biased select, dropped provider future, terminal result tests |
| Arbitrary provider tag crosses IPC | HIGH hypothesis | Typed canonical request plus resolver-before-provider tests |
| Unbounded/malformed pull progress | MEDIUM | Existing bounded NDJSON decoder and fail-closed terminal validation |
| Provider body or endpoint leaks | MEDIUM | Stable public errors, no raw response/log output, sentinel tests and secret/log review |
| False claim that daemon work stopped | MEDIUM | Explicit UI/docs limitation and no server-stop assertion |
| LocalOnly, proxy, redirects, chat cancellation regress | HIGH hypothesis | Full inference regression suite plus source/capability review |

No confirmed Critical or High blocker was found in the pre-implementation
review. Any new unresolved Critical/High finding stops this correction.

## Migration and Compatibility Considerations

There is no persisted data, schema, dependency, deployment, or cross-repository
migration. The mounted UI and Rust command boundary change atomically in the
same PR. This PR has not been merged, so no production compatibility promise is
being broken. The provider trait is internal to the repository; all
implementations and test doubles must be updated together.

Rollback is code-only. Reverting the correction must also remove or disable the
mounted preparation button; a known non-cancellable public preparation surface
cannot be restored while retaining Phase 1A PASS.

## Implementation Steps

1. Add typed preparation request/response types and injectable preparation
   timeout.
2. Generalize active-request storage to a kind-aware operation registry with
   shared RAII cleanup and kind-specific cancellation.
3. Route chat through the same registry without changing its event semantics.
4. Make preparation validate, register, wait for bounded concurrency, race
   cancel/timeout/provider execution, and return one terminal result.
5. Extend provider control and implement bounded streamed Ollama pull parsing.
6. Add the dedicated Tauri command and register it in the composition root.
7. Update the TypeScript adapter, mounted UI states, request ownership, and
   enabled preparation Cancel action.
8. Extend deterministic service/provider/contract tests, including real
   loopback stalls and cross-kind isolation.
9. Run focused tests, inspect the complete diff/security boundary, then run the
   complete Phase 1A gate.
10. Update completion and architecture evidence only from actual command
    results; commit/push/review handling occurs only after PASS.

## Test Matrix

| Behavior | Evidence |
| --- | --- |
| Valid unique preparation UUID is registered | service fake-provider test |
| Invalid UUID/provider tag rejected before provider | service validation/resolver tests |
| Duplicate active ID rejected across preparation/chat | kind-aware registry tests |
| Cancel before headers | actual `127.0.0.1:0` stalled `TcpListener` fixture |
| Cancel during streamed body | partial bounded NDJSON plus open socket fixture |
| Cancel during concurrency wait | held semaphore permit; provider not entered |
| Cancellation promptly drops DAARION future/connection | 500 ms outer guard and fixture disconnect observation where supported |
| Cancelled result cannot become success/timeout | delayed provider race tests |
| Timeout cannot become success/cancel | short injected deadline race test |
| Preparation A isolated from B | two active UUIDs and targeted cancellation |
| Preparation cancellation isolated from chat | simultaneous fake preparation/chat operations |
| Unknown/finished/cancelled IDs fail safely | `false` plus cleanup assertions |
| Fast streamed success | loopback final `status: success` fixture |
| Provider error/malformed/post-terminal/incomplete/oversized progress | bounded parser table tests |
| Public error privacy | sensitive sentinel absent from public error/log surface |
| Typed IPC/UI contract | deterministic Node source validator |
| Mounted Cancel action/state behavior | TypeScript build plus validator source assertions |
| Phase 1A invariants | all inference/full Rust tests, contract, build, security scans |

All intentionally stalled tests use short injectable deadlines, a separate
outer guard, explicit task abort/await cleanup, no public network, no Ollama
installation, and no model download.

## Tests

Focused and full required commands include:

```text
rustfmt --edition 2021 --check <each changed Rust file>
cargo test --manifest-path src-tauri/Cargo.toml preparation_ --lib
cargo test --manifest-path src-tauri/Cargo.toml inference:: --lib
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
npm ci
npm run test:inference-contract
npm run build
npm audit --omit=dev --json
bash scripts/check-no-secrets.sh
bash scripts/check-rust-touched-warnings.sh
git diff --check
```

Repository-wide `cargo fmt --check` will be classified against the recorded
94-file legacy debt; scoped rustfmt is mandatory for every changed Rust file.
Warnings must remain at 312 repository records or be explained as a reduction,
with zero warning in changed inference code.

## Acceptance Criteria

1. Preparation requires a validated UUID and canonical model ID.
2. It is registered before concurrency/provider work and cleaned on every exit.
3. Duplicate IDs are rejected across active operation kinds.
4. A dedicated cancel command targets preparation and cannot cancel chat.
5. Provider send/header/body waits observe cancellation; service cancellation
   drops the full provider future promptly.
6. Preparation timeout is service-owned, injectable, and distinct from cancel.
7. Cancel/timeout/error/success are mutually exclusive terminal results.
8. No cancelled or timed-out operation can later report DAARION success.
9. Streamed pull records are bounded, privacy-safe, and require explicit final
   success.
10. Stalled-header/body fixtures terminate under outer guards and cleanly stop.
11. Fast preparation and controlled provider failure remain compatible.
12. Mounted UI exposes preparing/cancelling/cancelled/timed-out/failed/completed
    states and an enabled Cancel action during preparation.
13. UI never claims daemon-side download or disk writes definitely stopped.
14. Canonical mapping, LocalOnly, loopback validation, redirect/proxy denial,
    chat deadlines/cancellation, NDJSON safety, sole terminal chat events, and
    shell-authority removal remain intact.
15. All focused/full checks pass with zero new warning in changed code and no
    dependency, lockfile, capability, registry, cross-repo, deployment, or
    later-phase change.
16. Completion evidence records the upstream limitation, live-smoke status,
    formatting debt, exact counts, and rollback.
17. Review thread is answered/resolved only after a passing push; PR remains
    open and unmerged; a fresh Codex review is triggered.

## Rollback Strategy

Before merge, revert the single correction commit on the existing PR branch.
Because the previous surface is known non-cancellable, rollback must also hide
or disable public model preparation and set the Phase 1A gate back to
`CORRECTION_REQUIRED`. No data, dependency, lockfile, or deployment rollback is
needed.

## Documentation Updates

After executable validation, create
`phase-01a-model-preparation-cancellation-completion.md` and narrowly update the
main Phase 1A completion report, ADR 0001, capability matrix, security gate,
master roadmap, and PR body. Keep live Ollama verification as not performed,
production readiness as not claimed, Phase 1B as `NO_GO`, and merge as a
separate controlled action.

## Open Questions

None block implementation. Upstream daemon-side cancellation is deliberately
classified as unproven and outside the DAARION guarantee; the approved task
authorizes a useful client-side cancellation boundary with explicit wording.

## GO / CONDITIONAL_GO / NO_GO

**GO.** The defect is confirmed, repository ownership is clear, the solution
fits Phase 1A without dependencies or adjacent architecture, the prior audit
and accepted Phase 1A plan cover this boundary, and no unresolved Critical/High
security blocker was found. Implementation may proceed only within this plan.
