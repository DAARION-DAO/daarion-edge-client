# Phase 1A Local Model Verification Correction Completion

Final repository release result: **PASS**

Merge action: **NOT PERFORMED**. This correction belongs only to existing PR
#24. Phase 1B remains **NO_GO** until Phase 1A is merged separately and verified
from fresh `main`.

## Defect and Baseline

- Pre-correction head: `1a8e21fbaa94d629db68e7698d28ff1d97ba3fed`.
- Blocking review thread: `PRRT_kwDOR7OvXc6Q1LOX`.
- Finding: P1, “Verify Ollama models are local before marking installed”.
- Baseline: 55 inference tests, 104 full Rust tests, 312 repository warnings,
  zero warnings in `src/inference/**`, and 94 legacy formatting-debt files.

The prior Phase 1A candidate proved that the HTTP transport terminated at a
loopback Ollama endpoint, but reduced `/api/tags` entries to model names.
Standard Ollama cloud models are also invoked through the local API. A matching
cloud model or copied alias could therefore be displayed and executed under a
nominal `LocalOnly` policy without daemon cloud-policy or local-artifact proof.

## Correction Architecture

`InferenceService` remains the only authorization owner. The provider-neutral
boundary now exposes two fail-closed evidence operations:

- `verify_local_execution(OperationControl)`;
- `verify_local_model(provider_model_id, OperationControl)` returning only an
  internal `VerifiedLocalModel { provider_model_id, digest }` after complete
  verification.

The trait defaults reject unsupported providers. No public command accepts an
Ollama tag, remote model name, remote host, digest, or provider evidence.

### Daemon cloud-policy contract

Ollama is eligible only when bounded `GET /api/status` decoding yields:

```json
{ "cloud": { "disabled": true } }
```

An explicit false value returns `local_only_not_enforced`. Missing or malformed
fields, malformed JSON, 404/non-success responses, unsupported capability, and
unverifiable evidence return `provider_capability_unsupported`. Timeout and
cancellation retain their existing controlled results. There is no environment,
process-name, localhost, model-name, or legacy compatibility fallback.

DAARION reads the daemon policy but does not edit Ollama configuration,
environment, launch agents, services, or user files and does not restart the
daemon.

### Local model evidence contract

For each canonical model, the resolver first produces exactly one private
Ollama tag. The Ollama adapter then:

1. reads a bounded `/api/tags` response retaining `name`, `model`,
   `remote_model`, `remote_host`, `size`, `digest`, and `details`;
2. requires exactly one entry whose `name` and `model` both equal the resolved
   provider ID;
3. requires empty remote markers, positive size, a normalized SHA-256 digest,
   and complete coherent details;
4. calls `/api/show` for the same provider ID with no prompt-bearing field;
5. requires empty show remote markers and details identical to the first tags
   read;
6. re-reads `/api/tags` and requires the complete evidence to remain identical.

Missing models are unavailable. Duplicate, remote, malformed, incomplete,
contradictory, disappeared, or unstable entries fail closed and cannot be
reported as installed. Digest syntax is evidence stability only; this phase
does not hash the artifact or provide cryptographic attestation.

### Prompt-egress prevention and TOCTOU

After request registration and concurrency admission, but before constructing
the prompt-bearing provider request, `InferenceService::run`:

1. revalidates the loopback provider boundary;
2. verifies daemon cloud is disabled;
3. re-resolves and compares the canonical mapping;
4. completes tags/show/tags verification;
5. only then constructs and calls `/api/chat`.

Status/model probes use the existing five-second service deadline. Chat
verification uses the existing request deadline and cancellation token.
Preparation performs the daemon preflight before pull and the full
daemon-plus-model postflight inside its existing deadline and cancellation
ownership. A failed postflight returns a controlled failure, so the command and
mounted UI cannot emit `completed_locally`.

No prompt is included in status, tags, or show requests, public errors, or logs.
The scripted fixture records only request path, model ID, JSON key names, and a
sentinel-presence boolean; it never retains raw prompt bodies.

## UI and Public Error Contract

The typed status projection adds `local_only_verified`. The mounted UI
distinguishes unreachable provider, unverified daemon policy, rejected model,
unavailable model, and verified-ready state. Run and Prepare remain disabled
until the daemon policy is verified; a model is labelled `locally verified`
only after complete evidence. The remediation text says Ollama cloud must be
disabled and explicitly says DAARION did not change Ollama settings.

Stable new public codes are:

- `local_only_not_enforced`;
- `provider_capability_unsupported`;
- `model_not_local`;
- `local_model_unverified`.

Messages contain no remote host, remote model name, raw body, endpoint detail,
local path, authentication detail, or sentinel prompt.

## Changed Files

Application and executable contract:

- `src-tauri/src/inference/model_resolver.rs`;
- `src-tauri/src/inference/ollama_provider.rs`;
- `src-tauri/src/inference/provider.rs`;
- `src-tauri/src/inference/service.rs`;
- `src-tauri/src/inference/types.rs`;
- `src/lib/inferenceClient.ts`;
- `src/components/LocalInferencePanel.tsx`;
- `scripts/validate-inference-contract.mjs`.

Evidence:

- `phase-01a-local-model-verification-correction-plan.md`;
- this completion report;
- primary Phase 1A completion report;
- ADR 0001;
- capability status matrix;
- security gate;
- master roadmap.

No manifest, dependency, lockfile, capability, database, memory, Supervisor,
loop, tool, transport, pairing, Supabase, readiness, wallet, worker, web-product,
CI, deployment, or production file changed.

## Deterministic Correction Evidence

Twelve new Rust test functions raise the inference suite from 55 to 67 and the
full Rust suite from 104 to 116. Table cases cover all required variants using
one scripted `127.0.0.1:0` fixture and no public network, installed Ollama,
model download, or real prompt.

| Behavior | Result |
| --- | --- |
| `cloud.disabled=true`; false; missing; malformed; unsupported | PASS / fail closed as specified |
| stalled `/api/status` under service probe deadline | PASS |
| remote marker in tags or show, including normal-looking copied alias | PASS / rejected |
| zero size, missing/malformed digest, malformed/contradictory details | PASS / rejected |
| duplicate exact tag | PASS / rejected |
| disappeared model, changed digest, changed remote marker | PASS / rejected |
| stale UI listing followed by changed daemon policy or model state | PASS / chat rejected |
| rejected chat request count | PASS / zero `/api/chat` calls |
| sentinel prompt on every rejected request | PASS / absent |
| verified-local chat request order | PASS / one final `/api/chat` call |
| cancellation during policy verification | PASS / no prompt transmission |
| pull success followed by remote evidence | PASS / preparation fails |
| valid local preparation postflight | PASS / succeeds |
| failed postflight and UI completion ordering | PASS / no `completed_locally` |
| prior preparation/chat cancellation, timeout, cleanup and probe tests | PASS |
| controlled errors and direct-log inspection | PASS / no sensitive metadata |

The provider evidence bodies are bounded to one MiB and decoded incrementally
with cancellation. No global HTTP timeout was added, preserving the separate
streaming deadlines owned by the service.

## Complete Phase 1A Validation

| Check | Result |
| --- | --- |
| pre-correction `cargo test ... inference::` | PASS, 55 tests |
| pre-correction `cargo test` | PASS, 104 tests |
| final `cargo test inference:: --quiet` | PASS, 67 tests |
| final `cargo test --quiet` | PASS, 116 tests |
| `cargo check --quiet` | PASS |
| JSON warning inventory | PASS, 312 warnings; 0 in `src/inference/**` |
| `cargo clippy --all-targets --quiet` | PASS command exit; 0 findings in changed inference code |
| scoped rustfmt for all five changed Rust files | PASS |
| repository-wide `cargo fmt -- --check` classification | expected pre-existing debt in 94 legacy files |
| `npm ci` | PASS; lockfile unchanged; baseline 11 dev-only findings |
| `npm run test:inference-contract` | PASS |
| `npm run build` | PASS (`tsc` and Vite production build) |
| `npm audit --omit=dev --json` | PASS, 0 production findings |
| `bash scripts/check-no-secrets.sh` | PASS |
| `bash scripts/check-rust-touched-warnings.sh` | PASS |
| `git diff --check` | PASS |
| P1-specific policy, prompt-egress, capability, shell, fallback, logging and path review | PASS |
| full changed diff and dependency/lockfile review | PASS |

No frontend lint or frontend unit-test command is configured. TypeScript is
validated by the production build; the deterministic cross-language contract
script verifies the mounted UI and public adapter without a new dependency.

## Warning and Formatting Comparison

- Repository warnings before: 312.
- Repository warnings after: 312.
- Warnings in changed `src/inference/**`: 0.
- Changed Rust files passing scoped rustfmt: 5/5.
- Repository-wide formatting debt before/after: 94/94 legacy files.
- No warning suppression or broad mechanical formatting was added.

## Security Review

| Finding | Severity | Result |
| --- | --- | --- |
| Loopback treated as proof of local execution | P1 / HIGH | Closed by mandatory daemon and model evidence |
| Cloud-enabled/unsupported daemon reaches chat | HIGH | Closed fail-closed; no compatibility fallback |
| Copied cloud alias appears installed | HIGH | Closed by exact identity, remote markers, details and stable evidence |
| Stale UI authorization | HIGH hypothesis | UI remains projection-only; each chat revalidates before prompt construction |
| Prompt transmitted before verification | HIGH | Closed; rejected fixture paths make zero chat calls and contain no sentinel |
| Preparation reports unverified success | HIGH | Closed by postflight before command success and UI completion |
| Verification hangs or ignores cancellation | MEDIUM | Closed by existing absolute service deadlines and shared control |
| Raw provider metadata leaks | MEDIUM | Closed by controlled codes/messages and sanitized fixture records |
| Shell, proxy, redirect or fallback expansion | HIGH hypothesis | Not introduced; existing denials and checks remain passing |

No unresolved Critical or High correction finding remains at repository level.

## Residual Trust Limitations

This correction protects the supported Phase 1A path against standard Ollama
cloud models, aliases, remote manifests, and ambiguous entries represented by
the official API metadata. It does not cryptographically attest:

- a malicious custom Ollama daemon;
- a compromised local process;
- a daemon deliberately lying in its API responses;
- local tampering after the final evidence read;
- the model file contents, signature, provenance, or supply chain.

A real Ollama/model smoke, platform packaging verification, artifact manifest,
artifact hashing/signatures, malicious-daemon defenses, and production
readiness remain separate gates. No real model was downloaded and no real
prompt was sent in this task.

## Rollback

Before merge, revert this correction commit on the existing PR branch. A safe
rollback must also disable the mounted inference and preparation actions; it
cannot restore name-only inventory authorization while retaining Phase 1A
repository PASS. There is no data, dependency, lockfile, migration, deployment,
or production rollback.

## Release Gate and Next Action

**PASS at repository level.** After this evidence is committed and pushed to
the existing branch, reply to top-level review comment `3581017169`, re-fetch
thread `PRRT_kwDOR7OvXc6Q1LOX`, resolve it only against the pushed executable
evidence, update PR #24, and trigger exactly one fresh `@codex review`. Keep the
PR open and unmerged. Until that exact-head review completes cleanly, the
external classification is **CONDITIONAL_PASS**, merge is blocked, and Phase 1B
remains **NO_GO**.
