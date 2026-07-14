# Phase 1A Probe Deadline Correction Plan

Status: **GO**

## Objective

Close review thread `PRRT_kwDOR7OvXc6Qyp5v` with one narrow correction that
makes local-provider health and installed-model probes deterministically bounded
by `InferenceService`, without changing chat, model preparation, provider
composition, permissions, dependencies, or any later-phase architecture.

## Confirmed Defect

At PR head `a505bc07bff1b1db67c3f2d35c4745f882761292`,
`OllamaProvider::new` configures only a TCP connect timeout. After a loopback
process accepts TCP, `InferenceService::status` awaits
`InferenceProvider::health` and `InferenceService::models` awaits
`InferenceProvider::list_installed_models` without an overall deadline. A
provider that stalls before headers or during the inventory body can therefore
leave the mounted inference panel in `checking` indefinitely.

The named review thread is unresolved and not outdated. This is a confirmed
availability and bounded-execution defect, not evidence of remote inference or
secret exposure.

## Current State

- Repository: `DAARION-DAO/daarion-edge-client`.
- Existing PR: `#24`, branch `phase-01a/local-only-inference`.
- Pre-correction head: `a505bc07bff1b1db67c3f2d35c4745f882761292`.
- PR state: open, ready for review, mergeable, and clean according to GitHub.
- Worktree: clean dedicated checkout of the existing PR branch.
- Existing Phase 1A diff: 30 files; no dependency or lockfile change.
- Pre-correction inference tests: 37 passed, 0 failed.
- Pre-correction full Rust tests: 86 passed, 0 failed.
- Pre-correction Rust output: 312 repository warnings; the Phase 1A completion
  report classifies zero warnings in `src/inference/**`.
- Frontend `LocalInferencePanel::refresh` awaits status and models through
  `Promise.all` and already transitions a rejected command to `failed`.

## Current Call Paths

```text
LocalInferencePanel::refresh
  -> get_local_inference_status Tauri command
     -> InferenceService::status
        -> InferenceProvider::health
           -> OllamaProvider GET /

LocalInferencePanel::refresh
  -> list_inference_models Tauri command
     -> InferenceService::models
        -> InferenceProvider::list_installed_models
           -> OllamaProvider GET /api/tags + response-body JSON decoding
```

The Tauri commands are thin adapters and already convert `InferenceError` into
the stable `InferencePublicError` contract.

## Scope

- Add one service-owned probe deadline to `ServiceLimits` with a bounded
  production default and injectable test value.
- Apply the deadline around the complete health and installed-model provider
  futures.
- Preserve the existing `InferenceError::TimedOut` public vocabulary.
- Add deterministic loopback `TcpListener` tests for stalled headers, stalled
  model-list body, fast success, controlled errors, and frontend-relevant
  service completion.
- Prove the probe deadline does not govern streaming chat.
- Update Phase 1A evidence, security gate, ADR, roadmap, and PR metadata only
  after executable checks pass.

## Deadline Ownership

`InferenceService` owns and enforces the absolute probe budget. This ensures
every current or future `InferenceProvider` is bounded even if its adapter does
not configure an HTTP request timeout. The timeout begins before invoking the
provider future and covers connection, response headers, body reads, decoding,
and any provider-side wait inside that future.

The production value has one source of truth in the service module. Tests inject
a short value through `ServiceLimits`. Frontend input and model-generated data
cannot alter it.

No global reqwest client timeout will be added because that could change the
semantics of streaming chat or model pull. The existing connect timeout,
loopback validation, redirect denial, and proxy denial remain unchanged.

## Error Semantics

- A service-owned probe deadline returns `InferenceError::TimedOut`.
- Tauri exposes the existing stable `timed_out` code and controlled message.
- A health timeout is not converted to available/success.
- A model-list timeout is not converted to an empty successful inventory.
- Raw reqwest errors, response bodies, endpoint internals, and remote/fallback
  claims are not exposed.
- Existing non-timeout `ProviderUnavailable` handling remains unchanged and is
  outside this correction.

## Explicit Non-Goals

- No SQLite, durable state, memory, Agent Supervisor, Loop Runtime, tools,
  Reticulum/LXMF, messaging, pairing, Supabase, readiness, wallet, or worker.
- No remote provider, fallback, public-network request, model download, live
  Ollama smoke, deployment, production write, or production-readiness claim.
- No dependency, lockfile, registry data, CI, capability, shell permission, or
  packaging change.
- No global reqwest timeout and no change to chat/request deadline,
  cancellation, NDJSON, model preparation, or terminal-event semantics.
- No web repository or PR #29 change.

## Repository Ownership

The correction belongs exclusively to `daarion-edge-client`, which owns the
local inference enforcement boundary. It changes no cross-repository, Supabase,
membership, pairing, or readiness contract.

## Files and Modules Expected to Change

- `src-tauri/src/inference/service.rs`: service limit, deadline enforcement, and
  deterministic service/loopback tests.
- `docs/planning/phases/phase-01a-probe-deadline-correction-plan.md`: this plan.
- `docs/planning/phases/phase-01a-probe-deadline-correction-completion.md`:
  completion evidence after validation.
- Existing Phase 1A completion, ADR 0001, security gate, and master roadmap:
  narrowly updated after checks pass.

`ollama_provider.rs`, frontend files, types, commands, manifests, and lockfiles
are inspect-only unless implementation proves a strictly required correction.

## Contracts Affected

The existing Tauri command and public error schemas do not change. The behavior
is tightened: status and model-list commands now terminate with the existing
`timed_out` error instead of being able to wait indefinitely. Chat events and
responses are unchanged.

## Security Considerations

- Availability: a malicious or wedged loopback process cannot hold status/model
  probes forever.
- Local-only: provider endpoint validation still executes before every probe.
- Privacy: timeout errors contain no provider response or request data.
- False success: timed-out inventory must not become an empty successful list.
- Authority: no new Tauri command, shell capability, network destination, or
  frontend-controlled limit is introduced.
- Regression risk: an incorrectly global timeout could break streaming chat;
  tests and diff review must prove the deadline is probe-only.
- Fixture safety: loopback tasks must be aborted/awaited and every deliberately
  stalled call must have an outer test guard.

No new Critical or High finding was identified in this bounded pre-
implementation review.

## Migration and Compatibility Considerations

There is no schema, data, IPC-shape, dependency, or deployment migration.
Clients already handle rejected Tauri commands and transition the mounted panel
to `failed`. The correction is rollback-safe by reverting the correction commit;
rollback would reopen the review defect and must not be used to claim bounded
probes.

## Implementation Steps

1. Add a `probe_timeout` field to `ServiceLimits` and a conservative production
   constant.
2. Add one service helper that applies `tokio::time::timeout` to an entire
   provider probe future and maps elapsed time to `InferenceError::TimedOut`.
3. Use the helper in `status` and `models`; preserve all other logic.
4. Add loopback fixtures that accept TCP and stall before headers or after valid
   headers, with short injected deadlines and outer guards.
5. Add fast health/model success tests, controlled public-error assertions, and
   a test proving chat is independent of the probe timeout.
6. Run focused checks, review the correction diff, then run the complete Phase
   1A gate.
7. Update evidence documents and PR body only from final command results.
8. Commit and push one correction commit, reply to the existing review, re-fetch
   it, and resolve it only when the code and checks prove closure. Do not merge.

## Test Design

- Actual `tokio::net::TcpListener` bound to `127.0.0.1:0`.
- Stalled-header fixture accepts and reads the request but sends no response.
- Stalled-body fixture sends valid JSON headers with a longer declared body and
  keeps the socket open.
- Short injectable probe deadline; separate outer timeout guard prevents the
  test suite from hanging if the service regression returns.
- Fixture tasks are explicitly aborted and awaited after each stalled case.
- Fast fixtures return normal health and model-list responses.
- Public error conversion is checked for stable `timed_out` without fixture
  content.
- Chat proof uses the same service with a probe timeout shorter than a
  deliberately delayed local fake chat, while the normal request timeout remains
  authoritative.

Required commands include:

```text
cargo test --manifest-path src-tauri/Cargo.toml probe_
cargo test --manifest-path src-tauri/Cargo.toml inference:: --lib
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
npm ci
npm run test:inference-contract
npm run build
npm audit --omit=dev --json
bash scripts/check-no-secrets.sh
bash scripts/check-rust-touched-warnings.sh
git diff --check
```

Repository-wide `cargo fmt --check` remains classified against the documented
94-file pre-existing debt. Every changed Rust file must pass scoped rustfmt.

## Acceptance Criteria

1. Health and installed-model futures are bounded by one service-owned probe
   deadline.
2. The default is fixed and safe; tests can inject a short value; frontend/model
   input cannot control it.
3. Stalled headers for health and models return within the outer guard as
   `TimedOut`.
4. A valid model-list header with a stalled body returns `TimedOut`.
5. Health timeout is not success; model timeout is not an empty success.
6. Public errors contain no raw fixture/provider content.
7. Fast health and model inventory still succeed.
8. Streaming chat is not governed by the probe deadline.
9. LocalOnly, endpoint validation, redirect/proxy denial, canonical model
   resolution, cancellation, NDJSON, and terminal-event behavior remain intact.
10. Scoped rustfmt, correction/inference/full Rust tests, check, clippy, typed
    contract, build, production audit, secret scan, warning check, scope review,
    and whitespace review pass.
11. No changed-code warning, dependency/lockfile change, capability expansion,
    remote fallback, production write, or later-phase file appears.
12. The review reply cites final evidence; the thread is resolved only after a
    passing push; PR #24 remains open and unmerged.

## Rollback Strategy

Before merge, revert the single correction commit on the existing PR branch.
No data or dependency rollback exists. A reverted branch must return to
`CORRECTION_REQUIRED`; it cannot retain repository-level PASS while probes are
unbounded.

## Documentation Updates

After executable validation, create the matching correction completion report
and update only the evidence/status language in the Phase 1A completion report,
ADR 0001, security gate, master roadmap, and PR body. Keep live Ollama smoke as
not performed, production readiness as not claimed, and Phase 1B as `NO_GO`.

## Open Questions

None. The human-approved correction prompt selects service ownership, existing
timeout semantics, no global reqwest timeout, and the required test behavior.

## GO / CONDITIONAL_GO / NO_GO

**GO.** Repository ownership, defect, scope, error contract, test design, and
rollback are explicit. The correction requires no new dependency or later-phase
work, and no unresolved Critical or High security blocker exists. Any need for a
global HTTP timeout, schema change, dependency, remote provider, or broader
runtime change changes this verdict to `NO_GO` pending a revised human review.
