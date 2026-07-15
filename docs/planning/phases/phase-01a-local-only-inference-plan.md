# Phase 1A — Local-Only Inference Policy and InferenceProvider Foundation

Status: **PLAN ONLY — HUMAN REVIEW REQUIRED**

Plan verdict: **CONDITIONAL_GO**

Application implementation performed by this task: **none**

## Objective

Create one bounded Edge-runtime slice in which every reachable inference request is resolved from a canonical model ID, admitted by an explicit `LocalOnly` policy, executed only by an Ollama adapter over a validated loopback endpoint, and terminated truthfully by completion, failure, timeout, or cancellation.

The phase establishes a provider-neutral Rust boundary without adding a remote provider. It removes the current simulated remote-success path from reachable execution, makes frontend/Tauri command names consistent, and supplies executable proof for policy, model mapping, streaming, deadline, cancellation, and privacy-safe logging.

## Current State

The following statements are source findings at the baseline HEAD, not production or deployed verification.

| Area | Evidence | Current classification and finding |
| --- | --- | --- |
| Reachable inference UI | `src/App.tsx:7,777`; `src/components/LocalInferencePanel.tsx::handleSend` | `PARTIALLY_IMPLEMENTED`. The panel is mounted and can list raw Ollama tags, but invokes `run_chat`; Rust registers `run_local_inference`. The request therefore has no aligned command path. |
| Dormant model UI | `src/components/LocalModelsPanel.tsx::init` | `MOCK_OR_PLACEHOLDER`. The component is not imported by the current app and invokes unregistered `get_supported_models` and `get_local_models`. It also mixes model controls with simulated placement/governance data. |
| Tauri registration | `src-tauri/src/lib.rs::run`; `src-tauri/src/models/mod.rs::run_local_inference` | `PARTIALLY_IMPLEMENTED`. `run_local_inference`, Ollama status/list/pull/smoke, generic download, and simulated load/unload commands are registered from the composition root. There is no cancellation command or managed inference state. |
| Local orchestration | `src-tauri/src/models/local_inference.rs::LocalInference::run_chat` | `PARTIALLY_IMPLEMENTED`. It validates total message length and looks up a canonical registry ID, then routes using simulated residency inputs. The client-supplied `max_tokens`, `temperature`, and `stream` values are not consistently validated or applied. |
| Remote fallback | `src-tauri/src/models/inference_arbitrator.rs::InferenceArbitrator::decide`; `local_inference.rs::handle_remote_fallback` | `MOCK_OR_PLACEHOLDER`, security severity **HIGH**. Cold/pressured states can select `RemoteExecution`; the handler sleeps and returns a fixed remote success. No prompt is shown leaving the device in this mock, but the reachable control path violates ADR 0001 and can normalize silent egress. |
| Ollama adapter behavior | `src-tauri/src/models/ollama.rs` | `PARTIALLY_IMPLEMENTED`. Detection, `/api/tags`, `/api/pull`, and `/api/generate` use loopback HTTP; chat is separately implemented in `local_inference.rs`. Provider logic and endpoint literals are duplicated. Pull and smoke commands accept a raw upstream tag from the frontend. |
| Endpoint boundary | `ollama.rs`; `local_inference.rs::execute_local` | `PARTIALLY_IMPLEMENTED`. URLs are hardcoded to `http://localhost:11434`, but there is no reusable endpoint validator, explicit redirect denial, proxy bypass, or test proving that a non-loopback endpoint cannot be constructed. The local chat path uses `reqwest::Client::new()` without a request deadline. |
| Canonical/provider model mapping | `public/fallback_registry.json`; `models/registry.rs`; `LocalInferencePanel.tsx` | `PARTIALLY_IMPLEMENTED`. The registry distinguishes canonical `id` from Ollama `upstream_tag`, but the UI sends a raw installed tag as `model_id`; Rust looks for that value as a canonical ID and later sends the canonical ID to Ollama. Unknown or mismatched IDs therefore fail or address the wrong provider identifier. |
| Registry access during inference | `models/registry.rs::ModelRegistry::fetch_registry` | `PARTIALLY_IMPLEMENTED`. A chat request can trigger a paired-backend registry GET before cache/bundled fallback. No prompt is included, but an inference request should not silently introduce external network dependency. Phase 1A needs a deterministic local registry read for model resolution. |
| Runtime loader | `models/runtime_loader.rs::RuntimeLoader` | `MOCK_OR_PLACEHOLDER`. Load sleeps for two seconds and emits `Warm`; unload only emits `Unloaded`. The chat path calls this fake loader before real Ollama HTTP. |
| Streaming | `local_inference.rs::execute_local`; `ollama.rs::pull_model` | `PARTIALLY_IMPLEMENTED`. Each transport chunk is parsed independently with `text.lines()`. Split NDJSON records are lost, multiple records are only conditionally handled, malformed records are silently skipped, HTTP error/done semantics are incomplete, and no buffer limit or terminal protocol error exists. |
| State/events | `inference_session.rs`; `LocalInferencePanel.tsx` | `PARTIALLY_IMPLEMENTED`. Rust and TypeScript duplicate loosely typed states. Several event emissions use `unwrap()`. Backend error paths do not consistently emit a terminal failure, and the frontend accepts unfiltered events without binding all transitions to the active request. |
| Limits | `models/inference_limits.rs`; `models/inference_session.rs` | `PARTIALLY_IMPLEMENTED`. Two different `InferenceLimits` types exist. Prompt length is checked, but the token clamp and timeout values are not applied to chat. Concurrency and duplicate request IDs are not controlled. |
| Cancellation | repository search over `src/` and `src-tauri/src/models/` | `MISSING`. There is no cancellation command, request registry, cancel signal, or proof that tokens/completion stop after cancellation. |
| Runtime truth | `local_inference.rs::execute_local`; `LocalInferencePanel.tsx` | `CLAIM_FALSE_OR_STALE`. A real Ollama `/api/chat` response is labeled `llama.cpp (Local)`, while UI text promises zero transmission and a fully active local tier despite the reachable remote mock and broken command name. |
| Logging/privacy | inference/model files and frontend panels | No direct prompt or completed response logging was found. Request IDs, model IDs, registry source, fixed remote lane text, and raw errors can be logged. Phase 1A must keep prompt, token, response, and endpoint values out of logs and sanitize provider errors. |
| Tauri capability | `src-tauri/capabilities/default.json` | `SECURITY_GATED`. The main webview receives `shell:default` plus `shell:allow-execute` for `ollama` with unrestricted arguments, although frontend source does not use the shell plugin. This exceeds the minimum required webview authority. |
| Existing inference tests | repository search for Rust/TypeScript tests | `MISSING`. No inference provider, policy, NDJSON, cancellation, timeout, model mapping, or command-contract tests were found. The repository has Rust tests in other domains but no frontend test runner dependency. |
| Baseline compilation | `cargo test --manifest-path src-tauri/Cargo.toml --no-run` | Passed at planning time. The baseline emits approximately 310 existing Rust warnings; this is not Phase 1A verification and must not be represented as a clean clippy gate. |

## Baseline HEAD

`a62626cab1fa1ede5a4990ef09fde940f8634c67` — merge commit for baseline PR #23 on `origin/main`.

Preflight evidence:

- `HEAD == origin/main` after fetching `origin main`;
- baseline commit `7d150d1fc8245f99e628264a062d6e69767d4c71` is an ancestor of `origin/main`;
- `AGENTS.md`, ADR 0001, the capability matrix, the baseline audit, master roadmap, and all 16 repository-scoped skills are present;
- the worktree was clean before this plan was created;
- no Ollama call, remote inference, deployment, production smoke, or application implementation was performed.

## Scope

1. Add a cohesive Rust inference module with provider-neutral request/result/error types, an `InferenceProvider` trait, `InferencePolicy::LocalOnly`, deterministic model resolution, a bounded service, and thin Tauri commands.
2. Implement Ollama as the only production provider for status, installed-model discovery, canonical-ID preparation/pull, smoke inference, and chat.
3. Consolidate the Ollama endpoint in one validated loopback-only type and one bounded HTTP client.
4. Resolve a canonical model ID to exactly one Ollama upstream tag from a local registry snapshot before provider invocation.
5. Replace chunk-local parsing with a bounded incremental NDJSON decoder.
6. Enforce server-owned request validation, deadline, one active request per request ID, cancellation by request ID, concurrency limit, and one terminal outcome.
7. Remove the simulated remote-success route and fake runtime-loader step from every reachable inference path.
8. Align mounted inference UI, Tauri commands, typed events, provider/runtime labels, and cancellation behavior.
9. Narrow the main webview's shell authority because browser code does not need arbitrary Ollama CLI arguments.
10. Add unit, service, command-contract, and loopback-only integration tests using controlled fakes/fixtures.
11. Update only the Edge documentation whose inference claims or status change in this phase.

## Explicit Non-Goals

This phase must not add or implement:

- SQLite, durable memory, or any of the six memory levels;
- Agent Supervisor, planner, executor, verifier, task runtime, or Loop Runtime;
- Tool Runtime, general shell access, filesystem tools, or workers;
- Reticulum, LXMF, sidecar/daemon, messaging, or multi-agent delegation;
- pairing, Supabase, MicroDAO membership, readiness projections, or `loval-echoes` changes;
- wallet, signing, identity rotation, or economic behavior;
- remote providers, cloud inference, automatic provider selection, remote fallback, or remote consent UI;
- generic GGUF artifact download, hash/signature verification, embedded `llama.cpp`, or repair of the simulated generic artifact/runtime-loader subsystem;
- CI workflow creation, deployment, packaging, release signing, or production verification;
- broad cleanup of `lib.rs`, existing warnings, agent-shaped modules, model-market mocks, or unrelated CSP/network permissions.

## Repository Ownership

- `daarion-edge-client` owns all Phase 1A code, local policy, local provider execution, Tauri contract, and local inference UI.
- `loval-echoes` owns no part of this phase and must not receive code or contract changes.
- Ollama is a local external dependency reached only through the Edge Rust adapter. It is not a DAARION remote provider.
- The bundled model registry supplies canonical-to-provider mapping for this phase. A future signed registry/artifact phase owns remote registry trust and managed model artifacts.

## Architecture Changes

The target request path is:

```text
LocalInferencePanel
  -> typed frontend inference client
  -> thin Tauri command
  -> InferenceService
  -> InferencePolicy::LocalOnly
  -> local ModelResolver
  -> OllamaProvider
  -> validated loopback HTTP client
  -> typed event sink
```

The service, not the LLM or UI, owns admission, request bounds, provider selection, deadline, cancellation, concurrency, event ordering, and terminal state. Production construction registers exactly one provider: Ollama. No production type or factory can construct a remote provider in Phase 1A.

The existing `models` namespace continues to own the model registry and unrelated lifecycle placeholders. Inference orchestration moves behind a dedicated `inference` module so `src-tauri/src/lib.rs` remains a composition root rather than absorbing policy and provider logic.

## Proposed InferenceProvider Interface

The exact Rust syntax may be adjusted during implementation review, but the semantic boundary must remain:

```rust
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    fn descriptor(&self) -> ProviderDescriptor;
    async fn health(&self) -> Result<ProviderHealth, InferenceError>;
    async fn list_installed_models(&self) -> Result<Vec<ProviderModelId>, InferenceError>;
    async fn prepare_model(
        &self,
        model: &ResolvedModel,
        control: RequestControl,
        events: &dyn InferenceEventSink,
    ) -> Result<(), InferenceError>;
    async fn chat(
        &self,
        request: ProviderChatRequest,
        control: RequestControl,
        events: &dyn InferenceEventSink,
    ) -> Result<ProviderChatResult, InferenceError>;
}
```

Required associated types and invariants:

- `ProviderDescriptor` declares stable provider kind and endpoint scope; production kind is only `Ollama`, scope only `Loopback`.
- `CanonicalModelId` and `ProviderModelId` are distinct validated newtypes.
- `ProviderChatRequest` contains a provider model ID only after policy and model resolution; a Tauri caller cannot submit it directly.
- `RequestControl` carries a server-owned absolute deadline and cancellation receiver.
- `InferenceEventSink` receives typed metadata/state/token events and is replaceable by an in-memory test sink.
- `InferenceError` is typed and privacy-safe: invalid request, policy denied, provider unavailable, model unknown/not installed, protocol error, timeout, cancelled, busy, and internal emission failure.
- The fake provider exists only under test configuration and is never registered by production composition.

## Proposed OllamaProvider Boundary

`OllamaProvider` owns all Ollama-specific HTTP routes and response schemas. It must:

- accept a prevalidated `OllamaEndpoint`, not a raw string per request;
- build one reusable `reqwest::Client` with redirects disabled, ambient proxy use disabled for loopback, bounded connect/read behavior, and no cookies;
- implement status, tags, pull, generate/smoke, and chat without exposing raw URLs or upstream tags to the frontend;
- map non-success HTTP status, Ollama error records, premature EOF, malformed NDJSON, timeout, and cancellation into typed failures;
- avoid the simulated `RuntimeLoader`; Ollama chat/pull responses are the provider truth;
- report provider/runtime as `Ollama`, never `llama.cpp`;
- use platform gates: desktop may probe the CLI with fixed arguments; unsupported mobile platforms return an explicit unavailable state and never simulate success.

All inference, including smoke inference, passes through `InferenceService` and `LocalOnly`. A smoke command may supply a fixed local prompt, but it cannot bypass endpoint, policy, model mapping, timeout, cancellation, or logging rules.

## InferencePolicy Enforcement

Phase 1A defines and enables exactly one policy:

```rust
pub enum InferencePolicy {
    LocalOnly,
}
```

Admission order:

1. validate request ID, roles, message count/size, temperature, token limit, and supported streaming mode;
2. reject duplicate active request IDs and enforce a small server-owned concurrency limit;
3. resolve the canonical model from the local bundled registry;
4. select the only production provider registered for that model;
5. require `ProviderDescriptor.endpoint_scope == Loopback` under `LocalOnly`;
6. create server-owned deadline/cancellation control;
7. invoke the provider and emit exactly one terminal result.

Unknown policy, provider, model, endpoint, or terminal state fails closed. There is no `Remote`, `Automatic`, `BestAvailable`, arbitration, or fallback policy variant in production Phase 1A code. A local failure returns a local failure and cannot become a fake or remote success.

## Endpoint Validation

Create a single `OllamaEndpoint` value object with these rules:

- scheme is `http` for Phase 1A;
- host is an IP loopback address, or literal `localhost` normalized to `127.0.0.1` without DNS-based provider selection;
- no username, password, query, fragment, or non-root base path;
- default production port is `11434`; supporting another loopback port requires an explicit human decision from the open questions below;
- redirects are disabled so a loopback response cannot redirect the client to a remote origin;
- proxy environment variables cannot route provider traffic through a proxy;
- endpoint values never arrive from the LLM, prompt, registry payload, or ordinary frontend request;
- non-loopback IPv4, IPv6, hostname, or redirect target is rejected before any request is sent.

Tests must prove rejection, not merely test a helper while production bypasses it.

## Model Identifier Mapping

The Tauri/UI contract uses only a canonical model ID such as the registry `id`. The backend resolver:

1. loads and validates the bundled local registry without a network fetch during an inference request;
2. finds exactly one canonical entry;
3. finds exactly one `install_sources` entry with runtime `ollama`;
4. validates a non-empty upstream tag and produces `ResolvedModel { canonical_id, provider, provider_model_id }`;
5. compares installed Ollama tags with `provider_model_id` while returning canonical IDs to the UI;
6. rejects unknown, ambiguous, missing-source, or malformed mappings.

`pull`/prepare and smoke/chat commands accept a canonical ID only. A frontend cannot choose an arbitrary upstream tag. The existing remote registry sync remains separate and cannot silently change the mapping of an in-flight Phase 1A request.

## Streaming Design

Add a bounded incremental NDJSON decoder shared by Ollama pull and chat:

- preserve incomplete bytes between transport chunks;
- accept CRLF/LF delimiters and multiple records per chunk;
- decode UTF-8 and JSON only after a complete record is assembled;
- enforce maximum buffered-record and total-response limits;
- treat malformed JSON, an Ollama `error`, missing required fields, overflow, and EOF without a valid terminal record as typed failure;
- never silently discard malformed non-empty records;
- emit monotonically sequenced events bound to one request ID;
- check cancellation/deadline before parsing, before emitting a token, and before emitting completion;
- emit exactly one of `Completed`, `Failed`, `Cancelled`, or `TimedOut` and no token/completion after a terminal event.

The event payload contains typed state and content fields; it does not embed arbitrary JSON strings. Provider errors are sanitized before crossing Tauri IPC.

## Timeout Design

- The backend owns a fixed default and bounded maximum request deadline; the frontend cannot disable it.
- One absolute deadline covers provider admission, HTTP connection, response headers, and full stream consumption.
- `tokio::select!`/deadline control maps expiry to `TimedOut`, cancels stream processing, removes request state, and emits one terminal event.
- Ollama status/list/pull may have operation-specific bounds, but none may be unbounded. Pull has a larger explicit maximum and user cancellation.
- Client connect limits supplement but do not replace the service deadline.
- The existing duplicate/unused limit definitions are consolidated into one validated request-limits model.

## Cancellation Design

- Add managed `InferenceRuntimeState` containing the service, an active-request registry, and the concurrency permit pool.
- `run_local_inference` registers the validated request ID before provider work.
- `cancel_local_inference(request_id)` signals only a currently active matching request and returns a typed `Cancelled`, `AlreadyTerminal`, or `NotFound` result.
- Cancellation is idempotent for an already-cancelled active request and does not cancel another request.
- The service checks the signal at every await/emit boundary, aborts provider stream consumption, suppresses later tokens/completion, emits one `Cancelled`, and removes the registry entry in a guaranteed cleanup path.
- Window/component cleanup requests cancellation for the active request but the Rust service remains authoritative.

No persistence or restart recovery is added; those belong to later phases.

## Tauri Command Contract

Proposed public commands:

| Command | Input | Output/purpose |
| --- | --- | --- |
| `get_local_inference_status` | none | Provider kind, local-only policy, availability, and privacy-safe reason. |
| `list_inference_models` | none | Canonical ID, display metadata, provider kind, and installed/prepared state. No raw endpoint. |
| `prepare_local_model` | canonical model ID | Bounded Ollama pull/preparation events; no arbitrary upstream tag. |
| `run_local_inference` | validated `LocalInferenceRequest` | Final typed result while typed stream/state events carry progress. |
| `cancel_local_inference` | request ID | Typed cancellation acknowledgement. |
| `run_local_inference_smoke` | canonical model ID | Same policy/provider path with a fixed local test prompt. |

Legacy direct commands that accept upstream tags or bypass policy are removed from registration or converted to private adapter methods. `src-tauri/src/lib.rs` only registers thin command adapters and managed state.

A deterministic contract check must fail if mounted frontend code invokes an inference command absent from the approved manifest/registration. It specifically prevents recurrence of `run_chat` versus `run_local_inference` and records the dormant `LocalModelsPanel` mismatches.

## UI State Contract

The mounted panel uses a typed frontend adapter and one active request ID. Proposed states are:

```text
Idle -> Starting -> Streaming -> Completed
                    |          -> Failed
                    |          -> Cancelled
                    |          -> TimedOut
```

Rules:

- filter every event by the active request ID and monotonic sequence;
- show a cancel control only while active;
- never append a completed assistant message after cancellation/failure/timeout;
- show provider `Ollama`, policy `LocalOnly`, and installed/unavailable truth;
- replace “zero latency”, “100% local” before policy proof, `llama.cpp`, network-routing, and fake warm-state language with evidence-based labels;
- disable send when Ollama or the canonical model is unavailable;
- surface bounded, sanitized errors without logging prompts/responses;
- repair only the inference/model-discovery contract in dormant `LocalModelsPanel`; simulated placement/governance content remains explicitly out of scope and must not be presented as Phase 1A proof.

## Logging and Privacy

Allowed structured metadata:

- request ID;
- canonical model ID;
- provider kind;
- state/outcome;
- bounded latency and error category;
- token count/sequence count without token text.

Forbidden in logs:

- prompts, system messages, retrieved context, token text, completed output;
- raw provider response bodies;
- full endpoint URLs, proxy values, environment variables, credentials, or local filesystem paths;
- private infrastructure details or future remote-routing identifiers.

Errors crossing Tauri IPC use stable public codes plus safe messages. Debug formatting of raw `reqwest`/JSON errors is not sent directly to the UI. Tests install a recording sink/logger and assert sensitive sentinels never appear.

## Files Expected to Change

The implementation phase should remain within this list unless a human approves a revised plan.

| Path | Expected action |
| --- | --- |
| `src-tauri/src/inference/mod.rs` | New cohesive inference module and exports. |
| `src-tauri/src/inference/types.rs` | New validated IDs, requests, results, events, limits, errors, and provider descriptors. |
| `src-tauri/src/inference/policy.rs` | New fail-closed `LocalOnly` admission logic. |
| `src-tauri/src/inference/provider.rs` | New `InferenceProvider` and event-sink boundaries; test fake under `cfg(test)`. |
| `src-tauri/src/inference/model_resolver.rs` | New local canonical-ID to Ollama-tag resolver. |
| `src-tauri/src/inference/ndjson.rs` | New bounded incremental decoder. |
| `src-tauri/src/inference/ollama_provider.rs` | Consolidated loopback client and status/list/pull/smoke/chat adapter. |
| `src-tauri/src/inference/service.rs` | Policy enforcement, concurrency, deadline, cancellation, cleanup, and terminal-event ordering. |
| `src-tauri/src/inference/commands.rs` | Thin Tauri commands and privacy-safe event adapter. |
| `src-tauri/src/lib.rs` | Register the module, managed state, and aligned commands; remove direct unsafe inference registrations. |
| `src-tauri/src/models/registry.rs` | Expose a deterministic validated local-registry read for inference mapping; keep network sync separate. |
| `src-tauri/src/models/mod.rs` | Remove old inference command exports/module wiring after migration. |
| `src-tauri/src/models/local_inference.rs` | Remove after all behavior moves behind the service; the remote mock must not remain reachable. |
| `src-tauri/src/models/inference_arbitrator.rs` | Remove from Phase 1A execution/module wiring; remote arbitration is not an MVP policy. |
| `src-tauri/src/models/inference_session.rs` | Replace/remove duplicated request/state/limits after canonical types move. |
| `src-tauri/src/models/inference_limits.rs` | Replace/remove duplicate unenforced limits. |
| `src-tauri/src/models/ollama.rs` | Replace/remove direct commands after adapter consolidation. |
| `src-tauri/capabilities/default.json` | Remove unnecessary webview shell default/arbitrary Ollama-argument authority; do not expand capabilities. |
| `src/lib/inferenceClient.ts` | New typed command/event adapter and exact command constants. |
| `src/components/LocalInferencePanel.tsx` | Use aligned contract, active-request filtering, truthful labels, cancel/error/timeout states. |
| `src/components/LocalModelsPanel.tsx` | Repair only canonical discovery/preparation command calls and remove fake loader claims from the inference contract. |
| `scripts/verify-inference-contract.mjs` | New deterministic, no-network contract validator if the implementation confirms no existing harness can cover the cross-language command list. |
| `package.json` | Add only a local contract-check script if the no-dependency validator is used; no dependency changes. |
| `README.md` | Correct Phase 1A-relevant GGUF/`llama.cpp`/local-inference claims. |
| `docs/adr/0001-local-first-inference-and-remote-consent.md` | Record implementation evidence/status without changing the accepted decision. |
| `docs/architecture/CAPABILITY_STATUS_MATRIX.md` | Update only capabilities proven by Phase 1A checks. |
| `docs/security/SECURITY_GATES.md` | Record whether the local-only gate closes and cite executable evidence. |
| `docs/planning/SOVEREIGN_AGENT_MASTER_ROADMAP.md` | Record Phase 1A outcome without authorizing Phase 1B. |
| `docs/planning/phases/phase-01a-local-only-inference-completion.md` | Required evidence-based completion report after implementation. |

Expected inspect-only/no-change files include `public/fallback_registry.json`, `src-tauri/src/models/runtime_loader.rs`, `src-tauri/tauri.conf.json`, and all `loval-echoes` files. If implementation discovers that a registry mapping itself is invalid, stop and revise this plan rather than silently editing product registry data.

## Dependencies

- No new production or development dependency is planned.
- Existing `async-trait`, `reqwest` with JSON/stream, `tokio` with sync/time, `url`, `serde`, `serde_json`, and `uuid` are sufficient.
- Cancellation should use existing Tokio synchronization primitives rather than adding a cancellation crate.
- Loopback HTTP fixtures should use `tokio::net::TcpListener` rather than adding a test-server dependency.
- `Cargo.toml`, `Cargo.lock`, `package-lock.json`, and dependency versions should remain unchanged. Any discovered need to alter them requires human review of a revised plan.

## Compatibility and Migration

- No database, filesystem, persisted-state, pairing, or data migration is allowed.
- Frontend and Rust command changes land atomically in one phase; old unsafe command aliases are not retained as silent compatibility fallbacks.
- Canonical model IDs remain the public application identifiers; raw Ollama tags become adapter-private.
- Existing remote registry sync remains available outside an inference request but does not determine an in-flight provider target.
- Desktop hosts without Ollama receive a truthful unavailable result. Mobile receives an explicit unsupported/unavailable result.
- Existing generic GGUF/artifact and fake runtime-loader paths remain security-gated and are not upgraded by this phase.
- Rollback is code-only because Phase 1A writes no durable schema or format.

## Security Findings

| Severity | Finding | Phase disposition |
| --- | --- | --- |
| HIGH | Reachable simulated remote execution can return fake success under local arbitration. | Blocking acceptance; remove production path and prove no remote provider is constructible/reachable. |
| HIGH | No technical `LocalOnly` invariant across every inference/smoke entry point. | Blocking acceptance; central policy/service gate plus negative fake-provider tests. |
| MEDIUM | Loopback literals lack reusable validation, redirect denial, and proxy bypass. | Add `OllamaEndpoint` and production-path tests. |
| MEDIUM | Pull/smoke accept arbitrary upstream tags from frontend input. | Accept canonical IDs only and resolve tags locally. |
| MEDIUM | Main webview can execute Ollama with unrestricted arguments despite no frontend use. | Remove webview shell permission or constrain it to the proven minimum; no permission expansion. |
| MEDIUM | Stream parser silently loses/skips split or malformed records and has no buffer bound. | Bounded incremental decoder and adversarial tests. |
| MEDIUM | Missing deadline/cancellation can retain resources and emit late completion. | Service-owned deadline/cancellation and terminal-order tests. |
| MEDIUM | Fake loader and misleading runtime/privacy labels can produce false security assurance. | Remove fake loader from inference and correct UI/runtime/docs. |
| LOW | Several event emissions can panic and errors can expose raw provider detail. | Typed emission errors and sanitization; no prompt/response logging. |

No confirmed current prompt exfiltration was found in the inspected mock. That does not reduce the HIGH severity of the reachable remote control path or satisfy the local-only gate.

## Implementation Steps

1. Re-run preflight on current `origin/main`; stop on overlapping dirty changes or baseline drift and update this plan if source evidence changed.
2. Add canonical inference types, validated request limits, provider descriptor/error/event types, and one `LocalOnly` policy.
3. Add pure endpoint and model-resolver components with rejection/mapping unit tests before network code.
4. Add the bounded NDJSON decoder with split/multiple/malformed/overflow/EOF tests.
5. Implement `OllamaProvider` with one loopback client, no redirects/proxy, typed schemas, HTTP status validation, and no raw logging.
6. Implement `InferenceService`, active-request registry, concurrency permits, deadline, cancellation, cleanup guard, and fake-provider tests.
7. Add thin Tauri commands/state/event sink; route status/list/pull/smoke/chat through the new boundary.
8. Remove old direct registrations and make remote arbitration/fallback and fake loader unreachable, then remove obsolete inference modules/types.
9. Add the typed frontend client, align mounted UI command/event names, add cancellation and truthful states/labels, and narrowly repair dormant model UI contract calls without touching its unrelated mocks.
10. Narrow the webview shell capability and confirm no frontend shell call is required.
11. Run narrow unit/contract tests, then full local Rust/frontend/build/security checks. Do not call a real Ollama instance in automated tests.
12. Review the complete changed-path list and diff for scope escape, egress, secrets, prompt/response logs, panic paths, and late terminal events.
13. Update ADR/status/security/README/roadmap documents strictly from executable results and write the completion report.
14. Apply the release gate. Do not claim Phase 1A complete or authorize Phase 1B unless all acceptance evidence is present and a human accepts the phase.

## Test Matrix

| Requirement | Test level | Required proof |
| --- | --- | --- |
| Non-loopback rejection | Rust unit + service test | IPv4/IPv6 remote hosts, remote hostname, credentials, query/fragment, and redirect attempt fail before provider request. Loopback fixture is accepted. |
| Ollama unavailable | Provider/service test | Connection refusal produces `ProviderUnavailable`; no completion or fallback is emitted. |
| Unknown canonical model | Resolver/service test | Unknown ID fails before provider construction/call. |
| Canonical ID to Ollama tag | Resolver test | Every selected fixture maps to the exact registry `ollama` source; missing/duplicate/malformed source fails. |
| Fake provider success/refusal | Service unit test | Test-only local fake can complete; remote-scoped fake is denied by `LocalOnly` and records zero chat calls. |
| Split NDJSON record | Decoder unit test | One record divided across arbitrary chunks yields one valid event. |
| Multiple records per chunk | Decoder unit test | Multiple records preserve order and sequence. |
| Malformed stream | Decoder/provider test | Malformed/oversized/error/premature-EOF input yields one failure, never partial success. |
| Timeout | Service test with paused/controlled time | Absolute deadline aborts work, emits only `TimedOut`, releases permit, and clears request registry. |
| Cancellation by request ID | Service/command test | Correct active request cancels; unknown ID is typed; another request is unaffected. |
| No completion after cancellation | Concurrency/event-sink test | Provider attempts late token/completion; sink records `Cancelled` as the sole terminal outcome and no later content. |
| Duplicate request ID | Service test | Second active request is rejected without replacing cancellation ownership. |
| Command contract alignment | No-network deterministic contract test + `npm run build` | Mounted UI command constants equal registered Rust names; old `run_chat`, `get_supported_models`, and `get_local_models` are absent from the approved inference surface. |
| No reachable remote fallback | Source/constructor test + fake call counter | Production provider set contains only loopback Ollama; `RemoteExecution`/remote handler/fixed offload success are absent from production module graph. |
| Request bounds | Rust unit/service tests | Invalid UUID, role, size, token count, temperature, and unsupported stream mode fail before provider call. |
| HTTP behavior | Loopback-only integration fixture | Non-success status, Ollama error, disconnect, done record, and response limits map to exact outcomes. No external network is used. |
| Privacy-safe logs/errors | Recording logger/sink test + diff scan | Sentinels placed in prompt/token/response/endpoint never appear in logs or public error strings. |
| UI truth and cancellation | Type/build plus adapter tests/controlled event fixture where practical | Provider/policy/status labels are accurate; events for another request are ignored; cancelled request is not appended as completed. |
| Capability minimization | Static check | Main webview cannot invoke arbitrary Ollama arguments and no new shell/network capability is granted. |

Narrow-first verification commands planned for implementation:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml inference::
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
npm ci
npm run build
npm run test:inference-contract
bash scripts/check-no-secrets.sh
git diff --check
git status --short
```

Because the baseline currently emits hundreds of unrelated Rust warnings, Phase 1A may not claim a repository-wide `-D warnings` result unless a separate approved cleanup makes that gate truthful. Changed inference code must add no new warnings, and every clippy warning touching changed code must be resolved.

## Acceptance Criteria

1. All reachable inference and smoke paths pass through `InferenceService` and `InferencePolicy::LocalOnly`.
2. Production composition registers exactly one inference provider, Ollama, with a validated loopback endpoint.
3. Non-loopback, redirected, proxied, unknown, or malformed provider targets fail before prompt-bearing traffic.
4. Simulated remote arbitration/fallback and fixed remote success are absent from the reachable production module graph.
5. A local/Ollama failure never returns a fake, remote, or successful completion.
6. Tauri callers supply canonical model IDs only; backend mapping to Ollama tags is deterministic and tested.
7. Mounted UI and Tauri registration use the same typed command/event contract; known broken names are removed from the approved inference surface.
8. Incremental NDJSON parsing passes split, multi-record, malformed, error, overflow, and premature-EOF tests.
9. Each request has a bounded deadline, server-controlled concurrency, cancellation by request ID, guaranteed cleanup, and exactly one terminal outcome.
10. No token or completion event appears after `Cancelled`, `TimedOut`, or `Failed`.
11. UI and response metadata report `Ollama` and `LocalOnly` truthfully and do not claim embedded `llama.cpp`, zero latency, fake warm state, or remote success.
12. Prompts, tokens, completed responses, raw endpoints, proxy/environment values, and raw provider bodies are absent from logs and public errors.
13. Main webview shell authority is reduced; the phase adds no capability, generic shell, filesystem, remote provider, or cloud egress.
14. No production/development dependency or lockfile changes occur unless this plan is revised and reapproved.
15. Narrow and full required checks pass, or the release result is not `PASS`.
16. Completion docs cite commands/results and update statuses only for behavior actually proven.
17. The final diff contains no SQLite, memory, Supervisor, loops, tools, pairing, Supabase, readiness, Reticulum/LXMF, wallet, worker, CI, deployment, or `loval-echoes` changes.

## Rollback Strategy

- Implement on a dedicated branch from the recorded baseline and keep commits separated into Rust boundary/tests, frontend contract, and documentation where practical.
- Before merge, rollback is branch deletion or commit reversion; do not preserve unsafe compatibility aliases.
- After merge, revert the Phase 1A commits as one bounded change if regression is found.
- No database/schema/state rollback is required because the phase adds no durable format.
- Rollback must not re-enable a remote fallback. If full rollback would restore the unsafe branch, ship a minimal fail-closed disablement first and keep inference unavailable until correction.

## Documentation Updates

After implementation and executable verification:

- correct README claims about native GGUF, embedded `llama.cpp`, and local inference maturity;
- update ADR 0001 implementation status without changing its decision or implying cloud-wide prohibition;
- update only local discovery/inference/stream/timeout/cancellation rows in the capability matrix;
- record the local-only security-gate result with exact tests;
- mark Phase 1A outcome in the master roadmap while leaving Phase 1B `NO_GO` until separately planned/reviewed;
- create `phase-01a-local-only-inference-completion.md` with changed files, checks, failures/limitations, security review, diff review, and release verdict.

No documentation may claim a real Ollama run, packaging, mobile support, production readiness, or remote-provider safety without corresponding evidence.

## Known Limitations

- Automated tests use fakes and a loopback fixture; they do not prove a real user-installed Ollama/model works on every platform.
- No model artifact signature/hash verification is added; Ollama-managed pull trust remains a later explicit gate.
- No generic GGUF or embedded `llama.cpp` runtime is added.
- No conversation persistence, memory, crash recovery, or cross-session cancellation exists.
- The bundled registry is local/deterministic but not a completed signed-registry supply-chain design.
- Desktop packaging and mobile behavior remain unverified; unsupported mobile returns explicit unavailable state.
- Repository-wide warning cleanup and CI automation are separate tasks.
- Dormant model-market/governance mock UI remains outside this phase and cannot be cited as implemented behavior.

## Open Questions

These questions are not resolved by current repository evidence and require human decisions before or during implementation:

1. Must Phase 1A support a user-configurable loopback Ollama port, or lock production to `127.0.0.1:11434`? The safer default is the fixed port; any configurable value must still pass the same validator.
2. Should `localhost` be accepted as input and normalized to `127.0.0.1`, or should only literal loopback IPs be accepted? The recommendation is normalization without DNS-based provider selection.
3. Is CLI version detection a required product feature, or may HTTP health be the sole availability signal? Removing CLI probing would allow removal of more shell authority.
4. Should the dormant `LocalModelsPanel` be retained after its narrow contract repair, or quarantined from the build until a later truthful model-management phase? This phase must not activate its simulated governance content.
5. What product-level maximum chat deadline and concurrent request count should be exposed? Until decided, use conservative server-owned constants and do not make them frontend-configurable.

## GO / CONDITIONAL_GO / NO_GO

**CONDITIONAL_GO for a separate Phase 1A implementation task; NO_GO for implementation in this plan-only task.**

Conditions that must be satisfied before code changes:

1. A human reviews and accepts this plan, including the command surface, endpoint rules, file list, and answers/defaults for the open questions.
2. Implementation begins from current clean `origin/main` (or an explicitly rebased dedicated branch) with no overlapping inference changes.
3. Scope remains Edge-only and excludes every explicit non-goal.
4. No dependency/lockfile, CI, capability expansion, registry-data, or private-infrastructure change occurs without a revised human-reviewed plan.
5. Local checks are mandatory because no GitHub CI status gate currently proves Phase 1A; skipped or failed checks prevent `PASS`.

The current HIGH findings are inside the defined remediation scope. They block Phase 1A acceptance, but they do not block starting the separately authorized implementation once this plan is approved. Any discovery that a non-loopback path remains reachable, canonical mapping cannot be made deterministic, or cancellation cannot prevent late completion changes the verdict to `NO_GO` until the plan is revised.
