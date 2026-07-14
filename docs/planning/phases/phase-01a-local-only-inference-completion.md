# Phase 1A — Local-Only Inference Completion Report

Final repository release result: **PASS**

Merge action: **NOT PERFORMED**. The focused diff/security review is complete and the PR may be marked ready for human merge review, but this task does not authorize or perform merge. Phase 1B and every later runtime phase remain **NO_GO** until Phase 1A is merged and verified from fresh `main`.

The first controlled-finalize attempt correctly stopped with `FAIL` on an
unresolved review finding: status and installed-model probes had no overall
deadline after TCP connect. The follow-up correction adds a service-owned
5-second production probe deadline plus deterministic stalled-header/body
fixtures. See `phase-01a-probe-deadline-correction-completion.md`. A later fresh
review correctly identified that model preparation remained outside the
request-scoped cancellation registry. The follow-up correction adds a typed
preparation UUID, kind-aware shared registry, dedicated cancel command, bounded
streamed pull parsing, and mounted UI cancellation. See
`phase-01a-model-preparation-cancellation-completion.md`. The final candidate
has 55 inference tests and 104 full Rust tests; merge remains a separate action.

## Baseline and scope

- Repository: `DAARION-DAO/daarion-edge-client`
- Baseline HEAD: `a62626cab1fa1ede5a4990ef09fde940f8634c67`
- Branch: `phase-01a/local-only-inference`
- Approved plan SHA-256: `0f40e90a8293fc7d76d0394e40269f54ccbfddce6971d607abeaa79e167d567f`
- Scope owner: Edge only
- Dependencies/lockfiles: unchanged
- Registry data: unchanged
- Database, memory, Supervisor, loops, tools, pairing, readiness, transport, wallet, worker, CI, deployment and web application code: unchanged

Baseline verification before implementation:

- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 49 tests, 310 reported Rust warnings;
- `npm ci`: PASS using `package-lock.json`;
- `npm run build`: PASS;
- repository-wide `cargo fmt --check`: independently reproduced as a pre-existing failure across 101 Rust files.

## Architecture delivered

The candidate adds one cohesive `inference` module:

- `InferencePolicy::LocalOnly` is the only policy variant;
- `InferenceProvider` is the provider-neutral boundary;
- production composition constructs exactly one provider, Ollama;
- provider construction, service admission and every provider-facing service boundary validate a plain HTTP loopback origin;
- the fixed production endpoint is normalized to IPv4 loopback; redirects and system proxies are disabled;
- canonical DAARION model IDs resolve only when the bundled registry contains exactly one canonical entry and one adapter-private Ollama mapping with a bounded valid tag;
- one service owns request validation, server-side limits, concurrency, absolute deadlines, kind-aware chat/preparation request-ID cancellation, cleanup and terminal ordering;
- a bounded incremental NDJSON decoder preserves split UTF-8 and final buffered records, rejects malformed/post-terminal/oversized/incomplete streams, and checks aggregate buffer growth before appending input;
- stable public error codes and controlled messages cross Tauri IPC;
- the mounted UI uses one typed adapter, filters events by active request ID and exposes only truthful local states;
- mobile builds return explicit local-provider unavailability;
- the main webview no longer initializes or receives shell execution authority.

## Removed mocks and false paths

Removed from production module wiring and source:

- simulated remote arbitration and offload success;
- the direct `run_local_inference` mock path;
- direct Ollama Tauri commands accepting arbitrary upstream tags;
- duplicated unenforced inference limits/session types;
- false `llama.cpp`, zero-latency, warm-loader and network-routing claims from the mounted inference UI.

The legacy simulated `RuntimeLoader` and broader model-market/governance mock UI remain outside Phase 1A, but their commands are not registered and `LocalModelsPanel` is not mounted. The deterministic contract check enforces that quarantine. They are not evidence of implemented model lifecycle or governance.

## Command and event contract

Registered commands:

- `get_local_inference_status`
- `list_inference_models`
- `prepare_local_model`
- `cancel_local_model_preparation`
- `run_local_inference`
- `cancel_local_inference`
- `run_local_inference_smoke`

Canonical event: `local-inference-event`.

Terminal events are `completed`, `failed`, `cancelled` and `timed_out`. The Rust terminal gate accepts exactly one terminal event and suppresses every later token or completion. The UI presents successful termination as `completed_locally`.

## Model mapping

Tauri callers provide `canonical_model_id`; preparation additionally requires a UUID `request_id`. `ModelResolver` reads only the bundled registry during inference and requires exactly one matching canonical entry and exactly one Ollama mapping. Provider tags are bounded to 256 ASCII bytes and a conservative name/tag grammar. Tests prove `qwen35-2b-stable` maps to `qwen3.5:2b`; unknown, duplicate canonical, missing, duplicate-source, empty, whitespace, URL-shaped and otherwise malformed mappings fail before provider execution.

No registry payload, model artifact, lockfile or remote registry contract changed. Ollama pull remains only a canonical-ID compatibility path; artifact signature/digest verification remains a separate open security gate.

## Verification evidence

| Check | Result |
| --- | --- |
| changed-scope command shown below for every added/modified Rust file | PASS, 12/12 files |
| probe-deadline correction fixtures | PASS, 6 tests using bounded loopback/fake-provider fixtures |
| model-preparation cancellation correction | PASS, 13 focused tests; 6/6 correction Rust files pass scoped rustfmt |
| `cargo test --manifest-path src-tauri/Cargo.toml inference:: --lib` | PASS, 55 tests |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS, 104 tests |
| `cargo check --manifest-path src-tauri/Cargo.toml` | PASS |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets` | PASS command exit; 0 findings reference `src/inference/**` |
| `npm ci` | PASS; lockfile unchanged |
| `npm run test:inference-contract` | PASS |
| `npm run build` | PASS (`tsc` and Vite production build) |
| `npm audit --omit=dev --json` | PASS, 0 production findings |
| `bash scripts/check-no-secrets.sh` | PASS |
| `bash scripts/check-rust-touched-warnings.sh` | PASS |
| `git diff --check` | PASS |
| capability/remote/log/static contract review | PASS |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | `PRE_EXISTING_DEBT / NON_BLOCKING_FOR_PHASE_1A`: baseline 101 files, initial candidate 97, focused-review candidate 94; tracked separately in `docs/planning/RUST_FORMATTING_DEBT.md` |

There is no configured frontend lint or frontend unit-test command. TypeScript checking is part of `npm run build`; the deterministic cross-language contract check was added without dependencies.

The exact changed-scope formatting command was:

```bash
git diff --name-only --diff-filter=AM origin/main -- '*.rs' \
  | while IFS= read -r file; do
      rustfmt --edition 2021 --config skip_children=true --check "$file" || exit 1
    done
```

Warning inventories for baseline and candidate were generated from clean
worktrees with:

```bash
cargo check --message-format=json \
  | jq -r 'select(.reason == "compiler-message" and .message.level == "warning") | [(.message.spans[0].file_name // "<none>"), (.message.code.code // "none"), .message.message] | @tsv' \
  | sort -u
```

The sorted inventories were compared with `comm -13` and `comm -23`; the exact
classification is recorded below.

## Warning comparison

- Baseline full tests: 310 warnings.
- Candidate full tests/check: 312 warnings.
- Net change: +2 repository warnings, with no warning located in new inference code.
- Set comparison shows 20 newly surfaced `dead_code` diagnostics in pre-existing authority, observability, trust and `RuntimeLoader` paths, while 18 legacy inference/model diagnostics were removed. The newly surfaced diagnostics result from removing the unsafe legacy inference caller; no changed inference implementation emits a warning.
- Clippy reports no `src/inference/**` finding.
- Repository-wide warning cleanup is not part of Phase 1A and no warning suppression was added.

`npm ci` reports 11 findings in development dependencies (2 low, 4 moderate, 5 high), already present under the unchanged lockfile. `npm audit --omit=dev` reports zero production findings. No automated dependency update was performed.

## Security review

| Threat/finding | Result and evidence |
| --- | --- |
| Silent remote inference / SSRF | Closed for the Phase 1A inference surface: only validated loopback HTTP is accepted; remote provider construction is rejected before use |
| Redirect or proxy escape | Redirect policy is `none`; system proxy use is disabled; loopback fixture proves redirect rejection |
| Arbitrary provider tag | Closed: commands accept canonical IDs and the local resolver owns provider tags |
| Malformed/provider-controlled stream | Closed within phase: split, multi-record, blank, malformed schema, provider error, post-terminal data, disconnect, final-buffer, premature-EOF, record-size, aggregate-buffer and output-size cases are tested |
| Cancellation/timeout race | Closed within phase: duplicate IDs, queue-wait cancellation/timeout, streaming cancellation/timeout, final-token-before-cancel, late provider error, cleanup, sole terminal outcome and late-event suppression are tested deterministically |
| Non-cancellable model preparation | Closed for the DAARION boundary: preparation has a UUID, shared kind-aware registry, dedicated cancel command, absolute deadline, bounded streamed progress, stalled-header/body fixtures, cross-operation isolation and mounted UI Cancel action. Ollama daemon-side termination is not claimed |
| Stalled local status/model probe | Closed within phase: `InferenceService` applies one 5-second production probe deadline; loopback fixtures prove stalled headers and model-list body return controlled `TimedOut` without false success or raw provider data |
| Prompt/response leakage | No inference logging calls exist; sentinel test proves prompt text is absent from response metadata and emitted events; raw provider error bodies are not exposed |
| Main-webview shell escalation | Closed for this surface: shell capability and plugin initialization were removed; no general command executor was added |
| Model artifact supply chain | OPEN and explicitly out of scope; Ollama-managed pull is not artifact verification |
| Real platform/provider behavior | UNVERIFIED; automated tests use deterministic loopback fixtures and fakes, not a user-installed Ollama/model |

No new Critical or High Phase 1A finding remains. Existing repository-wide warnings, dev-dependency audit findings, fixed-argument worker process commands and unrelated placeholder modules are recorded rather than represented as repaired. No Phase 1A inference path can invoke those worker commands.

## Focused review A–G

| Review area | Result |
| --- | --- |
| A. Local-only enforcement | PASS — production constructs only `OllamaProvider`; fixed endpoint is plain HTTP loopback; provider/service boundaries revalidate it; redirects and system proxies are disabled; no remote inference/fallback path is registered |
| B. Provider/service boundary | PASS — mounted UI uses the typed adapter; every inference command reaches `InferenceService`; legacy direct commands are unregistered/deleted; frontend provides canonical IDs only |
| C. Races | PASS — deterministic tests cover duplicates, queue wait, streaming, timeout/cancel ordering, late failures, cleanup and exactly one terminal event |
| D. NDJSON bounds | PASS — record, aggregate buffer and output limits are enforced; malformed, incomplete, error and post-terminal records fail closed |
| E. Model resolver | PASS — duplicate canonical entries and duplicate mappings fail; adapter tags are bounded/validated; bundled registry use is deterministic but not represented as cryptographic trust |
| F. Tauri authority | PASS for Phase 1A — shell permission and plugin initialization are removed; no generic executor was added. Pre-existing fixed-argument worker/process commands remain separately security-gated |
| G. Privacy | PASS for Phase 1A — no prompt/token/response logging; public errors are controlled; raw provider bodies and reqwest errors are not exposed; mounted inference frontend has no console logging |

## Formatting gate amendment

- repository-wide formatting debt: **PRE_EXISTING / separate remediation**;
- changed-scope formatting: **PASS**, every changed Rust file;
- no formatting regression: **PASS**, 101 baseline files versus 94 final candidate files;
- semantic review: **PASS**, changed-file formatting was isolated and the non-whitespace diff was reviewed; no unrelated file was retained in the formatting change;
- tracking record: `docs/planning/RUST_FORMATTING_DEBT.md`.

## Acceptance matrix

| Criterion | Result |
| --- | --- |
| All reachable inference/smoke paths use the service and LocalOnly policy | PASS |
| Exactly one production provider with validated loopback endpoint | PASS |
| Non-loopback, credentials, path, query, fragment, redirect and proxy paths fail closed | PASS |
| Simulated remote fallback/fake success removed | PASS |
| Local failure never becomes remote/fake success | PASS |
| Canonical model IDs map deterministically to private provider tags | PASS |
| Typed frontend/Tauri command and event contract | PASS |
| Robust bounded incremental NDJSON handling | PASS |
| Server-owned limits, concurrency, deadline, cancellation and cleanup | PASS |
| Model preparation is request-scoped, cancellable and kind-isolated | PASS for DAARION-side operation; upstream daemon-stop behavior remains unverified |
| Status and model probes remain bounded after TCP connect | PASS |
| No late token/completion after a terminal event | PASS |
| Truthful provider/policy/runtime/UI state | PASS |
| Privacy-safe logs, errors and metadata | PASS |
| Main webview shell authority reduced; no new capability | PASS |
| No dependency or lockfile change | PASS |
| Full Rust/frontend/security checks | PASS under the focused formatting amendment; repository-wide debt is pre-existing, reduced and separately tracked |
| Documentation reflects only proven behavior | PASS |
| Phase scope remains Edge-only and excludes later phases | PASS |

## Approved-plan deviations and limitations

- The dormant `LocalModelsPanel` was quarantined rather than rewritten because it is not imported by the application and contains broader out-of-scope governance mocks. Its unmounted status is checked deterministically.
- CLI version detection was removed; HTTP health is the sole availability signal.
- The production port is fixed at the conservative default and is not frontend-configurable.
- Ollama is installed on the review host, but no installed model exactly matches an approved bundled canonical mapping. No model was downloaded solely for review, so no real Ollama inference smoke, packaging, notarization or mobile execution proof is claimed.
- There is no persistence or crash recovery for active requests.
- Model preparation cancellation proves DAARION future/stream teardown and UI cleanup only. Official Ollama documentation does not provide a daemon-wide stop guarantee; resumable upstream progress may remain.
- Repository-wide formatting remains open across 94 legacy files and is tracked as a separate cleanup. It is non-blocking only under the explicit Phase 1A amendment above.

## Rollback

Before merge, delete the branch or revert its Phase 1A commits. After merge, revert the Phase 1A PR as one bounded change. A rollback must not silently restore simulated remote fallback; if a full revert would do so, keep inference fail-closed/unavailable until the correction is ready. No data/schema rollback is required.

## Next gate

1. Complete a fresh Codex review of the exact correction head; do not merge in this task.
2. If the review has no substantive blocker, run the separately controlled merge procedure.
3. After merge, verify the exact merge result from fresh `main`.
4. Only then create a separate Phase 1B planning-only task for SQLite durable runtime state.

No Phase 1B implementation, Agent Supervisor or Loop Runtime work is authorized by this report.
