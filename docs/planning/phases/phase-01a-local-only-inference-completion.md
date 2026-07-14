# Phase 1A — Local-Only Inference Completion Report

Final release result: **CONDITIONAL_PASS**

Merge authorization: **NO**. Human diff/security review and resolution or explicit plan amendment for the pre-existing repository-wide formatting gate are required. Phase 1B and every later runtime phase remain **NO_GO**.

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
- provider construction and service admission both validate a plain HTTP loopback origin;
- the fixed production endpoint is normalized to IPv4 loopback; redirects and system proxies are disabled;
- canonical DAARION model IDs resolve from the bundled registry to adapter-private Ollama tags;
- one service owns request validation, server-side limits, concurrency, absolute deadline, request-ID cancellation, cleanup and terminal ordering;
- a bounded byte-buffered NDJSON decoder preserves split UTF-8 and final buffered records and rejects malformed, oversized or incomplete streams;
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
- `run_local_inference`
- `cancel_local_inference`
- `run_local_inference_smoke`

Canonical event: `local-inference-event`.

Terminal events are `completed`, `failed`, `cancelled` and `timed_out`. The Rust terminal gate accepts exactly one terminal event and suppresses every later token or completion. The UI presents successful termination as `completed_locally`.

## Model mapping

Tauri callers provide `canonical_model_id`. `ModelResolver` reads only the bundled registry during inference and requires exactly one non-empty, whitespace-free Ollama mapping. Tests prove `qwen35-2b-stable` maps to `qwen3.5:2b`; unknown, missing, duplicate and malformed mappings fail before provider execution.

No registry payload, model artifact, lockfile or remote registry contract changed. Ollama pull remains only a canonical-ID compatibility path; artifact signature/digest verification remains a separate open security gate.

## Verification evidence

| Check | Result |
| --- | --- |
| `rustfmt --edition 2021 --check src-tauri/src/inference/*.rs` | PASS |
| `cargo test --manifest-path src-tauri/Cargo.toml inference::` | PASS, 26 tests |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS, 75 tests after final run |
| `cargo check --manifest-path src-tauri/Cargo.toml` | PASS |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets` | PASS command exit; no findings reference `src/inference/**` |
| `npm ci` | PASS; lockfile unchanged |
| `npm run test:inference-contract` | PASS |
| `npm run build` | PASS (`tsc` and Vite production build) |
| `npm audit --omit=dev --json` | PASS, 0 production findings |
| `bash scripts/check-no-secrets.sh` | PASS |
| `git diff --check` | PASS |
| capability/remote/log/static contract review | PASS |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | FAIL on pre-existing repository formatting debt: baseline 101 files, candidate 97 files, zero hits under new `src/inference/**` |

There is no configured frontend lint or frontend unit-test command. TypeScript checking is part of `npm run build`; the deterministic cross-language contract check was added without dependencies.

## Warning comparison

- Baseline full tests: 310 warnings.
- Candidate full tests/check: 312 warnings.
- Net change: +2 repository warnings, with no warning located in new inference code.
- The count changed because removing the legacy inference caller exposed existing authority/trust/runtime-loader placeholder types as dead code while obsolete inference warnings and unused imports were removed.
- Clippy reports no `src/inference/**` finding.
- Repository-wide warning cleanup is not part of Phase 1A and no warning suppression was added.

`npm ci` reports 11 findings in development dependencies (2 low, 4 moderate, 5 high), already present under the unchanged lockfile. `npm audit --omit=dev` reports zero production findings. No automated dependency update was performed.

## Security review

| Threat/finding | Result and evidence |
| --- | --- |
| Silent remote inference / SSRF | Closed for the Phase 1A inference surface: only validated loopback HTTP is accepted; remote provider construction is rejected before use |
| Redirect or proxy escape | Redirect policy is `none`; system proxy use is disabled; loopback fixture proves redirect rejection |
| Arbitrary provider tag | Closed: commands accept canonical IDs and the local resolver owns provider tags |
| Malformed/provider-controlled stream | Closed within phase: split, multi-record, blank, malformed, error, disconnect, final-buffer, premature-EOF and size-limit cases are tested |
| Cancellation/timeout race | Closed within phase: request-scoped cancellation, cleanup, sole terminal outcome and late-event suppression are tested |
| Prompt/response leakage | No inference logging calls exist; sentinel test proves prompt text is absent from response metadata and emitted events; raw provider error bodies are not exposed |
| Main-webview shell escalation | Closed for this surface: shell capability and plugin initialization were removed; no general command executor was added |
| Model artifact supply chain | OPEN and explicitly out of scope; Ollama-managed pull is not artifact verification |
| Real platform/provider behavior | UNVERIFIED; automated tests use deterministic loopback fixtures and fakes, not a user-installed Ollama/model |

No new Critical or High Phase 1A finding remains. Existing repository-wide warnings, dev-dependency audit findings and unrelated placeholder modules are recorded rather than represented as repaired.

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
| No late token/completion after a terminal event | PASS |
| Truthful provider/policy/runtime/UI state | PASS |
| Privacy-safe logs, errors and metadata | PASS |
| Main webview shell authority reduced; no new capability | PASS |
| No dependency or lockfile change | PASS |
| Full Rust/frontend/security checks | CONDITIONAL: all functional checks pass; repository-wide rustfmt remains a reproduced baseline failure |
| Documentation reflects only proven behavior | PASS |
| Phase scope remains Edge-only and excludes later phases | PASS |

## Approved-plan deviations and limitations

- The dormant `LocalModelsPanel` was quarantined rather than rewritten because it is not imported by the application and contains broader out-of-scope governance mocks. Its unmounted status is checked deterministically.
- CLI version detection was removed; HTTP health is the sole availability signal.
- The production port is fixed at the conservative default and is not frontend-configurable.
- No real Ollama/model, packaging, notarization or mobile execution smoke was performed or claimed.
- There is no persistence or crash recovery for active requests.
- Model preparation is bounded but not request-ID cancellable; chat/smoke cancellation is the Phase 1A cancellation contract.
- The repository-wide formatting gate cannot pass without a broad 97-file formatting change, which would violate this phase's bounded scope. Human review must choose a separate formatting change or amend the phase gate to accept the passing changed-scope rustfmt proof.

## Rollback

Before merge, delete the branch or revert its single Phase 1A commit. After merge, revert the Phase 1A commit as one bounded change. A rollback must not silently restore simulated remote fallback; if a full revert would do so, keep inference fail-closed/unavailable until the correction is ready. No data/schema rollback is required.

## Next gate

1. Review the complete draft PR diff and this security assessment.
2. Resolve or explicitly amend the pre-existing repository-wide rustfmt gate.
3. Re-run the full check set and require **PASS** before merge.
4. After merge, verify from fresh `main`.
5. Only then create a separate Phase 1B planning-only task for SQLite durable runtime state.

No Phase 1B implementation, Agent Supervisor or Loop Runtime work is authorized by this report.
