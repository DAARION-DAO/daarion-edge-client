# Phase 1A Local Model Verification Correction Plan

Status: **GO**

## Objective

Close P1 review thread `PRRT_kwDOR7OvXc6Q1LOX` in existing PR #24 with one
bounded Phase 1A security correction. The correction must make `LocalOnly`
eligibility depend on explicit Ollama daemon cloud-policy evidence and
per-model local-artifact evidence, revalidate both immediately before prompt
transmission, verify preparation results before success, preserve all existing
deadlines and cancellation guarantees, and stop before merge.

## Current State

- Repository: `DAARION-DAO/daarion-edge-client` only.
- Existing PR: #24, branch `phase-01a/local-only-inference`.
- Pre-correction head: `1a8e21fbaa94d629db68e7698d28ff1d97ba3fed`.
- PR: open, ready, mergeable/clean, and unmerged at preflight.
- Blocking P1 thread: unresolved, current, and anchored to the expected head.
- Dedicated worktree: clean; no unrelated changes.
- Baseline: 55 inference tests and 104 full Rust tests passed.
- Baseline warning inventory: 312 repository warnings; no accepted new warning
  is allowed in changed inference code.
- Baseline repository-wide formatting debt: 94 legacy Rust files; every changed
  Rust file must pass scoped rustfmt.
- Live Ollama/model smoke: not performed and not authorized.

## Confirmed P1 Threat

The current provider reduces `/api/tags` entries to `name`, and the service
interprets a matching name as installed. `status()` proves only that a loopback
HTTP origin responds. `run()` then constructs and sends a prompt-bearing
`/api/chat` request without proving the daemon has cloud disabled or the model
has local artifact evidence. A standard Ollama cloud model or copied alias can
therefore match an expected DAARION provider tag and be displayed or executed
under a nominal `LocalOnly` policy.

This is a confirmed authorization defect. It is not evidence that any real
prompt left this device, and it does not justify a claim of malicious-daemon or
artifact-supply-chain protection.

## Trust Boundary

`InferenceService` owns the authorization decision. `OllamaProvider` may
collect and validate Ollama-specific status, tags, and show evidence, but a
public Tauri command must not reach model preparation or chat without the
service requiring provider-neutral verified evidence. The frontend is a
projection only and cannot authorize a later request from cached state.

The external Ollama daemon is outside the DAARION process boundary. DAARION
does not assume its own environment, configuration, process name, or loopback
address describes that daemon's cloud behavior.

## Localhost Is Not Local Execution

Ollama's official cloud documentation shows cloud models invoked through the
same `http://localhost:11434/api/chat` API. Loopback validation, disabled
redirects, and disabled system proxies remain necessary transport controls,
but they are insufficient execution evidence. The correction therefore keeps
those controls and adds two mandatory evidence gates.

## Ollama Cloud Policy Contract

`OllamaProvider` will query `GET /api/status` and accept only a well-formed
response containing exactly the required truth value:

```json
{ "cloud": { "disabled": true } }
```

`false`, missing fields, malformed JSON, non-success/unsupported responses,
timeouts, cancellation, and inability to prove the state fail closed. No
compatibility fallback or environment inference is allowed. The status API is
a required Ollama capability for Phase 1A.

The implementation contract is based on Ollama's official cloud/FAQ
documentation and the current official Ollama source API types/routes, where
`/api/status` returns `cloud.disabled` and tag/show responses carry remote
metadata. Runtime implementations that cannot provide this capability must be
upgraded or configured before DAARION enables `LocalOnly` inference.

## Model-Level Verification Contract

For one adapter-private provider model ID resolved from a canonical DAARION
model ID, `OllamaProvider::verify_local_model` will:

1. read `/api/tags` with full bounded metadata;
2. require exactly one exact matching tag;
3. require empty `remote_model` and `remote_host` markers;
4. require positive size, a normalized SHA-256 digest, and coherent model
   details;
5. call `/api/show` for only that exact provider ID;
6. require the same local markers and coherent model details in the show
   response;
7. re-read `/api/tags`;
8. require exactly one identical local entry with the same positive size,
   digest, details, and local markers;
9. return only an internal provider-neutral `VerifiedLocalModel` value.

Missing models remain distinguishable from rejected/unverifiable models.
Duplicate exact tags, contradictory identity/details, missing fields, changed
digest, disappeared entries, or changed remote markers are controlled failures.
No remote marker value, raw response, provider body, or provider-private tag is
exposed through public errors or frontend inputs.

## Alias and Copy Threat

Name suffixes, absence of `-cloud`, allowlists, positive size, digest, format,
or one successful endpoint are never sufficient alone. A copied cloud alias
using the normal expected tag is rejected by remote markers or by inconsistency
between the two tags reads and show evidence. The final decision always
combines daemon cloud-disabled proof with complete model-level proof.

## Prompt-Egress Prevention

The service will perform daemon policy verification, re-resolve and compare the
canonical mapping, and perform full model verification inside the existing
request deadline immediately before constructing and passing the prompt-bearing
request to `InferenceProvider::chat`. Verification methods receive no prompt.
If any verification step fails or cancellation/deadline wins, `/api/chat` is
never called.

No prompt content may be logged, placed in status/tags/show requests, or
included in public error text. A scripted loopback fixture will record request
order, paths, counts, and sanitized bodies and will assert that a sentinel
prompt is absent on every rejected path.

## TOCTOU Analysis

- UI inventory is display-only and cannot authorize chat.
- Each chat rechecks daemon policy and model evidence after concurrency
  admission and inside its absolute deadline.
- The canonical mapping is resolved once for validation and again immediately
  before chat; both resolutions must be identical.
- Model verification uses tags/show/tags and requires stable digest and local
  markers across both inventory reads.
- Preparation checks daemon policy before `/api/pull` and performs the complete
  daemon-plus-model verification after terminal pull success before returning
  success.
- These checks narrow normal Ollama races but cannot attest a malicious daemon
  that lies or local process tampering after the final check.

## Compatibility Policy

Fail closed on an unsupported or incomplete `/api/status`; no legacy permissive
mode is added. Fail closed on Ollama responses that omit the metadata needed for
the model proof. The UI may instruct the user to disable Ollama cloud or upgrade
Ollama, but this phase will not edit configuration, environment, launch agents,
systemd, Windows settings, or restart the daemon.

## Error Semantics

Use the smallest stable controlled vocabulary consistent with existing errors:

- `local_only_not_enforced`: cloud is explicitly enabled;
- `provider_capability_unsupported`: required policy/evidence capability is
  unavailable or incomplete;
- `model_not_local`: explicit remote metadata or contradictory remote state;
- `local_model_unverified`: ambiguous, malformed, unstable, or incomplete
  local artifact evidence.

Provider unavailability, cancellation, and timeout retain existing codes.
Public messages give remediation direction without remote names/hosts, response
bodies, raw errors, local paths, or prompt content.

## Scope

- Add provider-neutral daemon-policy and verified-local-model boundaries.
- Parse and validate Ollama status, tag, and show evidence fail closed.
- Enforce daemon/model verification for readiness, model inventory, chat, and
  preparation completion.
- Revalidate immediately before prompt transmission.
- Keep all calls within existing probe/request/preparation deadlines and
  cancellation ownership.
- Project truthful policy/model state in the mounted UI and disable unsafe
  Run/Prepare actions.
- Add one deterministic scripted loopback fixture and focused service/provider
  tests covering the approved matrix.
- Update only directly affected Phase 1A evidence after executable checks.

## Repository Ownership

The correction belongs only to `daarion-edge-client`, which owns local
inference policy, provider adaptation, model lifecycle, Tauri IPC, and the
mounted local-runtime UI. No `loval-echoes`, Supabase, MicroDAO, pairing,
readiness projection, transport, wallet, or worker contract changes.

## Files and Modules Expected to Change

Application and deterministic test/contract files:

- `src-tauri/src/inference/provider.rs`;
- `src-tauri/src/inference/ollama_provider.rs`;
- `src-tauri/src/inference/service.rs`;
- `src-tauri/src/inference/model_resolver.rs`;
- `src-tauri/src/inference/types.rs`;
- `src/lib/inferenceClient.ts` and
  `src/components/LocalInferencePanel.tsx` if required for truthful projection;
- `scripts/validate-inference-contract.mjs` if the public status contract
  changes.

Evidence files:

- this plan and its matching completion report;
- the primary Phase 1A completion report;
- ADR 0001;
- capability status matrix;
- Phase 1A security gate;
- master roadmap;
- PR #24 body after final checks.

No manifest, lockfile, capability permission, CI, deployment, registry-data, or
unrelated module change is expected. Stop for review if one becomes necessary.

## Contracts Affected

- Internal `InferenceProvider` gains provider-neutral local-execution and
  local-model verification methods.
- Internal successful verification returns a controlled provider model ID and
  digest only.
- `InferenceStatus` gains explicit local-policy evidence so UI reachability and
  verified LocalOnly readiness are not conflated.
- Existing canonical IDs remain the only frontend model inputs; Ollama tags and
  remote metadata remain adapter-private.
- Existing command names, chat events, preparation request/response shapes,
  cancellation commands, registry schema, and cross-repository contracts do not
  change.

## Security Considerations

| Risk | Severity | Planned control |
| --- | --- | --- |
| Cloud execution behind localhost | CRITICAL impact if prompt is private | Mandatory `/api/status` `cloud.disabled=true` before eligibility and chat |
| Copied/normal-looking cloud alias | HIGH | Exact tags/show/tags verification with remote markers, artifact evidence, and stable digest |
| Prompt sent before verification | HIGH | Service-owned pre-chat verification; provider evidence methods receive no prompt; fixture proof |
| Cached UI status becomes stale | HIGH | No cache authorizes chat; immediate service revalidation |
| Pull reports success for remote model | HIGH | Mandatory post-pull complete verification before success |
| Verification stalls | MEDIUM | Existing absolute probe/request/preparation deadlines and cancellation |
| Provider metadata leaks | MEDIUM | Stable controlled errors and sanitized fixture/log review |
| Malicious daemon lies | Residual HIGH outside scope | Explicit limitation; no attestation claim |

No unresolved implementation blocker remains before coding. Any new Critical
or High defect found in the correction blocks PASS.

## Migration and Compatibility Considerations

There is no persisted-data, dependency, deployment, or cross-repository
migration. The internal provider trait and all test doubles change atomically.
The frontend and Rust public status shape change together in this unmerged PR.
Older Ollama runtimes without the required API capability become intentionally
ineligible rather than receiving an insecure fallback.

## Implementation Steps

1. Add stable local-policy/model verification errors and an explicit status
   projection.
2. Add provider-neutral evidence contracts and a reusable uncancelled probe
   control bounded externally by the service.
3. Implement strict Ollama `/api/status`, `/api/tags`, `/api/show`, and
   tags-recheck parsing and validation.
4. Expose canonical provider candidates from the resolver without leaking them
   over IPC.
5. Enforce daemon policy and verified inventory in `status()` and `models()`.
6. Put daemon/model/mapping revalidation before prompt request construction and
   inside the existing request cancellation/deadline boundary.
7. Put daemon preflight and complete postflight verification around
   preparation inside its existing cancellation/deadline boundary.
8. Update mounted UI and typed contract to project verified versus unverified
   LocalOnly state and disable unsafe actions.
9. Add scripted loopback and service tests, then run focused checks.
10. Review security and complete diff, run the full release gate, update
    evidence only from results, then commit/push/reply/resolve/request one fresh
    review. Stop before merge.

## Test Matrix

The deterministic suite will cover all approved cases, grouped into focused
tests where coherent: valid daemon/local model; cloud enabled; missing,
malformed, unsupported, and stalled status; remote markers in either tags or
show; copied alias; zero size; missing/malformed digest; malformed or
contradictory details; duplicate tags; disappearance/digest/marker change across
reads; stale listing followed by daemon or model change before chat; zero chat
calls and no sentinel prompt on rejection; exactly one chat on valid evidence;
remote post-pull failure; no completion after failed postflight; valid local
preparation; cancellation, timeout, probe, privacy, and terminal-event
regressions.

One scripted `127.0.0.1:0` fixture will serve `/api/status`, `/api/tags`,
`/api/show`, `/api/pull`, and `/api/chat`; record sanitized request metadata;
use no public network, installed Ollama, model download, flaky sleep, or real
prompt; and terminate fixture tasks with outer timeout guards.

## Tests

Required focused and full verification:

```text
rustfmt --edition 2021 --check <every changed Rust file>
cargo test local_model --lib
cargo test prompt --lib
cargo test inference:: --lib
cargo check
cargo test
cargo clippy --all-targets
npm ci
npm run test:inference-contract
npm run build
npm audit --omit=dev --json
bash scripts/check-no-secrets.sh
bash scripts/check-rust-touched-warnings.sh
git diff --check
```

Repository-wide `cargo fmt --check` will be classified against the recorded
94-file legacy debt. P1-specific source inspection will confirm no permissive
fallback, prompt-bearing verification request, public remote metadata, shell
authority, remote provider, dependency, capability, or later-phase change.

## Acceptance Criteria

1. `LocalOnly` eligibility requires explicit daemon `cloud.disabled=true`.
2. Unsupported, malformed, missing, false, timed-out, or unprovable cloud state
   fails closed.
3. Only one exact, stable tags/show/tags model with empty remote markers,
   positive size, valid digest, and coherent details becomes verified.
4. Cloud aliases, duplicates, remote models, ambiguous entries, and unstable
   evidence cannot be displayed as installed or authorized.
5. `status`, `models`, chat, and preparation completion all require service-
   owned evidence.
6. Every chat revalidates daemon, canonical mapping, and model before prompt
   request construction/transmission.
7. Rejected chat produces zero `/api/chat` calls and no sentinel prompt in any
   verification request, error, or log.
8. Preparation verifies daemon before pull and complete evidence after pull;
   failed postflight cannot become completed locally.
9. Verification remains inside existing deadlines and cancellation; no global
   reqwest timeout changes streaming behavior.
10. UI distinguishes reachable, policy verified/unverified, model verified/
    rejected and disables unsafe Run/Prepare actions without claiming config
    mutation.
11. All focused, inference, full Rust, check, clippy, typed contract, build,
    production audit, secret, warning, formatting, capability, security, and
    diff checks pass with no new changed-code warning.
12. No dependency/lockfile, capability, deployment, production write, model
    download, real prompt, Phase 1B, or unrelated repository change occurs.
13. P1 is resolved only after pushed executable evidence; PR remains open and
    unmerged; exactly one fresh review is requested.

## Rollback Strategy

Before merge, revert the single correction commit on the existing PR branch.
There is no data, dependency, or deployment rollback. Reverting reopens the P1
and returns Phase 1A to `FAIL`; loopback-only inference must not retain PASS.

## Documentation Updates

After validation, create the matching correction completion report and update
only directly affected evidence in the primary Phase 1A completion report, ADR
0001, capability matrix, security gate, roadmap, and PR #24 body. Keep live
Ollama smoke `NOT PERFORMED`, production readiness unclaimed, malicious-daemon
attestation explicitly out of scope, and Phase 1B implementation `NO_GO`.

## Explicit Non-Goals

- No model signatures, trust roots, artifact hash allowlist, cryptographic
  attestation, malicious-daemon defense, or supply-chain proof.
- No SQLite, memory, Agent Supervisor, Loop Runtime, Tool Runtime,
  Reticulum/LXMF, pairing, Supabase, readiness projection, wallet, worker,
  remote inference, or cloud fallback.
- No dependency, lockfile, CI, deployment, capability, shell, configuration,
  environment, daemon restart, model download, real prompt, production write,
  or PR #29/loval-echoes change.

## Open Questions

None. The human-approved correction selects fail-closed `/api/status`, strict
tags/show/tags evidence, service ownership, no compatibility fallback, and the
required deterministic validation matrix.

## Rollback

Rollback is the same single-commit code/documentation revert described above;
it necessarily restores the P1 and invalidates Phase 1A repository PASS.

## GO / CONDITIONAL_GO / NO_GO

**GO**. The defect is source-confirmed; official Ollama API evidence supplies a
bounded implementation contract; the change can be implemented entirely in
the existing inference boundary without a dependency, lockfile, capability,
deployment, persistence, or later-phase expansion. No unresolved security
blocker prevents this correction.
