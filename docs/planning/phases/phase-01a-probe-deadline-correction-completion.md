# Phase 1A Probe Deadline Correction Completion

Final repository release result: **PASS**

Merge action: **NOT PERFORMED**. This correction updates the existing PR #24
only. Phase 1B remains **NO_GO** until Phase 1A is merged separately and verified
from fresh `main`.

## Defect and Root Cause

The previous controlled-finalize attempt correctly stopped with `FAIL` on
review thread `PRRT_kwDOR7OvXc6Qyp5v`. At head
`a505bc07bff1b1db67c3f2d35c4745f882761292`, the reqwest client had only a TCP
connect timeout. `InferenceService::status` and `InferenceService::models`
awaited provider health and inventory futures without an overall deadline, so a
loopback process could accept TCP and stall before headers or while streaming the
model-list body.

## Correction

- `InferenceService` now owns one `probe_timeout` in `ServiceLimits`.
- The production default is 5 seconds and is not exposed to frontend or model
  input.
- `run_probe` uses an absolute Tokio deadline around the complete provider
  future.
- Both `health` and `list_installed_models` are covered, including connection,
  response headers, body reads and JSON decoding performed by the provider.
- Elapsed probes return the existing `InferenceError::TimedOut`; model-list
  timeout is not converted into an empty successful inventory.
- No global reqwest client timeout was added. Streaming chat, model preparation,
  request cancellation and chat deadline semantics are unchanged.
- The mounted panel already catches command rejection and transitions from
  `checking` to `failed`, so no frontend change was required.

## Scope and Changed Files

- `src-tauri/src/inference/service.rs`
- `docs/planning/phases/phase-01a-probe-deadline-correction-plan.md`
- `docs/planning/phases/phase-01a-probe-deadline-correction-completion.md`
- `docs/planning/phases/phase-01a-local-only-inference-completion.md`
- `docs/adr/0001-local-first-inference-and-remote-consent.md`
- `docs/security/SECURITY_GATES.md`
- `docs/planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md`

No dependency, lockfile, registry, capability, frontend, web, database,
Supervisor, memory, loop, tool, transport, pairing, readiness, wallet, worker,
CI or deployment file changed.

## Deterministic Test Evidence

The correction adds six service-level tests. Five use an actual loopback
`TcpListener`; stalled fixtures accept the socket and keep it open. Probe test
deadlines are 20 milliseconds, fast-response deadlines are 200 milliseconds,
and every probe call has a separate 500-millisecond outer test guard. Stalled
fixture tasks are aborted and awaited.

| Test behavior | Result |
| --- | --- |
| health accepts TCP and stalls before headers | PASS / `TimedOut` |
| model inventory accepts TCP and stalls before headers | PASS / `TimedOut` |
| model inventory sends valid headers then stalls its body | PASS / `TimedOut` |
| health timeout cannot become success | PASS |
| model timeout cannot become empty successful inventory | PASS |
| public timeout error omits endpoint and fixture body data | PASS |
| fast health response | PASS |
| fast mapped model inventory | PASS |
| chat exceeds a 1-millisecond probe budget but remains governed by its 500-millisecond request deadline | PASS |
| existing cancellation, timeout, late-event and NDJSON tests | PASS |

## Validation Results

| Command | Result |
| --- | --- |
| pre-correction `cargo test --manifest-path src-tauri/Cargo.toml inference:: --lib` | PASS, 37 tests |
| pre-correction `cargo test --manifest-path src-tauri/Cargo.toml` | PASS, 86 tests |
| `rustfmt --edition 2021 --config skip_children=true --check src-tauri/src/inference/service.rs` | PASS |
| `cargo test --manifest-path src-tauri/Cargo.toml probe_ --lib` | PASS, 6 tests |
| `cargo test --manifest-path src-tauri/Cargo.toml inference:: --lib` | PASS, 43 tests |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS, 92 tests |
| `cargo check --manifest-path src-tauri/Cargo.toml` | PASS |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets` | PASS command exit; no correction/inference finding |
| `npm ci` | PASS; lockfile unchanged |
| `npm run test:inference-contract` | PASS |
| `npm run build` | PASS (`tsc` and production Vite build) |
| `npm audit --omit=dev --json` | PASS, 0 production findings |
| `bash scripts/check-no-secrets.sh` | PASS |
| `bash scripts/check-rust-touched-warnings.sh` | PASS |
| JSON `cargo check` warning inventory | 312 repository records; 0 in `src/inference/**` |
| `git diff --check` | PASS |
| capability, provider, remote-fallback, shell and log review | PASS |
| repository-wide `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | expected pre-existing failure across 94 legacy files |

`npm ci` still reports the unchanged baseline of 11 development-dependency
findings (2 low, 4 moderate, 5 high). The production-only audit reports zero.
No dependency update was authorized or performed.

## Warning and Formatting Comparison

- Pre-correction full Rust warnings: 312.
- Post-correction full Rust warnings: 312.
- New warnings in `src/inference/**`: 0.
- Changed Rust file scoped rustfmt: PASS.
- Repository-wide formatting debt: unchanged at 94 legacy files, tracked in
  `docs/planning/RUST_FORMATTING_DEBT.md`.

## Security Review

| Finding | Severity | Result |
| --- | --- | --- |
| Loopback provider stalls after TCP accept | MEDIUM | Closed for status/model probes by service-owned deadline and real-socket fixtures |
| False successful empty inventory on timeout | MEDIUM | Closed; elapsed deadline returns `TimedOut` before existing unavailable mapping |
| Provider body/error disclosure | MEDIUM | Closed within correction; public timeout is controlled and fixture sentinel is absent |
| Global HTTP timeout breaks streaming chat/pull | MEDIUM | Avoided; reqwest client semantics were not changed and delayed-chat test passes |
| Remote fallback or egress expansion | HIGH hypothesis | Not introduced; only loopback Ollama remains composed and LocalOnly remains enforced |
| Shell/capability expansion | HIGH hypothesis | Not introduced; capability and composition diff unchanged |

No unresolved Critical or High correction finding remains.

## Acceptance Matrix

- health and model probes bounded by service: **PASS**
- one production default and injectable test deadline: **PASS**
- stalled headers and body terminate deterministically: **PASS**
- stable `timed_out`, no false success, no raw provider data: **PASS**
- fast probes remain compatible: **PASS**
- frontend cannot remain permanently in `checking`: **PASS** through bounded
  command rejection and existing `catch -> failed` transition
- chat/LocalOnly/cancellation/NDJSON/provider composition unchanged: **PASS**
- dependency, lockfile, capability and scope boundaries: **PASS**
- complete Phase 1A gate: **PASS** under the existing focused-formatting
  amendment
- live Ollama/model smoke: **NOT PERFORMED / NOT CLAIMED**
- production readiness: **NOT CLAIMED**
- production writes/deployments: **0**

## Remaining Limitations

- The tests use controlled loopback fixtures and a fake delayed chat provider;
  they do not verify a user-installed Ollama/model.
- Model downloads and artifact verification remain separately security-gated.
- Repository-wide Rust formatting debt remains open across 94 files.
- Phase 1B, Agent Supervisor, memory and Loop Runtime remain out of scope and
  `NO_GO`.

## Release Gate and Next Action

**PASS.** After this evidence is committed and pushed to the existing PR branch,
reply to review comment `3580091038`, re-fetch the review thread, and resolve it
only if GitHub shows the correction at the new head. PR #24 must remain open and
unmerged. The next authorized task after review closure is the previously
defined controlled merge/readback, not Phase 1B implementation.
