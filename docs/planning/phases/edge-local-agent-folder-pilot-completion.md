# Local agent and folder pilot — completion

## Result

The local-first slice is demonstrated on an Apple Silicon Mac: a native Edge window created a personal agent profile and distinct local node reference, obtained an explicit folder grant through the OS picker, completed a read-only folder inspection, produced a real local-model overview, and displayed both saved results after reopening. The employee-to-City network path remains PARTIAL.

The delivered node reference is a stable local UUID, not a cryptographic network identity. This is a manually triggered bounded local agent workflow, not the canonical Phase 1C Supervisor or a complete autonomous agent platform.

## Implementation and ownership

The isolated `local-agent-pilot` feature, native binary and app identifier separate this pilot from normal Edge enrollment, heartbeats, identity keys, wallets and private runtime-store commands. The UI adds one profile, native folder selection/revocation, actual task status, local model selection and saved history. Local SQLite stores profile, tasks/results and bounded metadata audit rows. Native commands do not accept filesystem paths, provider URLs or model-generated tools from the frontend.

A pinned cap-std dependency provides directory-relative file access through a held capability. The task reads a single directory level, excludes hidden entries and symlinks, and bounds directory entries, file size and excerpts. It writes only to the pilot store. Closing or revoking drops the folder grant. Restart recovery marks unfinished tasks interrupted.

Inference reuses the existing Edge local-only provider/service, including cloud-policy checks, local-artifact verification, cancellation and deadlines. The pilot starts its own temporary Ollama process and issues CPU-only, limited-context requests with unload-after-completion. No model was downloaded. Model responses remain untrusted text. Session-discovered installed-model references do not modify the canonical model registry.

The initial native model check exposed a real compatibility blocker: Edge accepted only `sha256:`-prefixed digests, while installed Ollama and the [official tags API](https://docs.ollama.com/api/tags) return a bare 64-character SHA-256 digest. The bounded remediation accepts both forms and normalizes them; it still rejects malformed/short/non-hex evidence. Default main-app request options remain unchanged, covered by a regression test.

MicroDAO, City Control Plane/OpenBot, NODA4, public installation pages and the previous OpenFabric document pilot were not modified. OpenFabric remains the previously selected candidate for the later network connection; this local slice does not invoke it or establish DAGI membership.

## Executed acceptance

| Check | Evidence and outcome |
| --- | --- |
| Agent creation through native UI | PASS: synthetic `Local helper` profile; distinct stable agent and node references. |
| Folder grant through OS picker | PASS: one synthetic folder selected; frontend supplied no path. |
| Exact folder task | PASS: two visible text files and one folder; hidden control skipped; nested content absent. |
| Actual local AI | PASS: installed `qwen3.5:9b` returned a Ukrainian overview consistent with the supplied materials and explicitly noted the unseen subfolder contents. No mock response or remote fallback. |
| Local policy | PASS: temporary provider reported cloud disabled; existing shared Ollama's cloud setting remained unchanged. CPU-only options are verified in the request regression test; live CPU/GPU utilization was not sampled during the brief request. |
| Persistence and visibility | PASS: both completed tasks, the same profile and the full AI response were read back from SQLite and visible through native accessibility after reopening. |
| Folder revocation/restart | PASS: explicit revocation audited; reopening left task execution disabled until another native folder choice. |
| Source preservation | PASS: hashes of every synthetic source file, including hidden/nested controls, were unchanged. |
| Process lifecycle | PASS: closing the app stopped its temporary listener; reopening did not restart the model automatically. |
| Native package | PASS: Apple Silicon app packaged, ad-hoc signed, strict signature verification passed. |
| Windows employee path | NOT RUN: no Windows host available; no Windows installer or execution proof is claimed. |
| Employee membership and City visibility | NOT IMPLEMENTED in this slice; no SSO, network enrollment or shared compute claim. |

Local execution evidence is retained in ignored `.pilot-evidence/`: `native-receipt.json`, `native-first-result.json`, `native-ai-result.txt`, `native-ai-reopen.txt`, `native-saved-result.png`, and source-file hash evidence. The package and final signed-binary receipt are in ignored `dist-local-agent/`. These files contain synthetic demonstration data only. Personal machine paths are not committed.

## Checks and scoped review

- Two focused folder/store tests passed: traversal and symlink escape rejection, non-recursion, bounds, distinct profile IDs, duplicate profile rejection, idempotent result storage and interrupted-task recovery.
- All 69 inference-filtered Rust tests passed, including new real-format digest and unchanged-default-options cases, existing local-only refusal, timeout and cancellation contracts.
- TypeScript/Vite production frontend build passed.
- Existing frontend/Rust inference contract check passed.
- Native pilot build with the embedded custom protocol passed.
- Normal main-profile library check passed without the pilot feature.
- `git diff --check` passed. Targeted Rust formatting was applied during implementation; repository-wide formatting and unrelated unused-code warnings were not made into a workstream.
- One scoped source review covered registered commands, filesystem scope, private persistence, model policy, owned child cleanup, build isolation, dependency lockfile and repository boundaries. The digest compatibility failure was remediated and the native AI scenario rerun successfully. No unresolved P0/P1 was identified in the demonstrated local path; this is not a full security audit.

## Release boundary and rollback

The canonical main SHA remained `eafb30d8548e2b1e3bc36fc34b2c8f0b36f633b7`. The task branch is `codex/edge-local-agent-folder-pilot`; all task source changes remain uncommitted for review. Active PR overlap was checked against the canonical repository; the existing release-documentation PR is outside these source files.

The Mac package is ad-hoc signed and not notarized for general employee distribution. Windows installation/execution, employee authentication/entitlements and City adapter enrollment remain required before claiming the complete employee path. Resource sharing and general agent autonomy are deferred. Rollback is closing this separate app and using the normal Edge app; no production rollback or data deletion is needed.

NEXT_SINGLE_ACTION = Connect this same local agent through MicroDAO membership to the existing City agent interface, using the existing City adapter and an explicit employee access contract.

## Required result record

```text
TASK = Local agent and folder pilot
PRIMARY_OUTCOME = Native personal agent profile, explicit folder access, real local AI and persistent visible result
RESULT = PARTIAL (local Mac slice PASS; full employee-to-City path incomplete)
IMPLEMENTED = Isolated native pilot, local profile/node references, read-only folder capability, local tasks/results, Ollama reuse
FILES_CHANGED = 22 source/documentation files
TESTS_RUN = 2 folder/store + 69 inference tests; frontend build; inference contract; pilot build; default library check; signature verification
CORE_USER_PATH_SMOKE = PASS on this Mac, including native UI and saved AI result after reopening
P0 = None identified in scoped review
P1 = None in demonstrated local path; Windows installation and authenticated City enrollment remain unimplemented for the full milestone
P2_BACKLOG = General distribution signing and Windows runtime acceptance
P3_BACKLOG = General agent autonomy and optional shared compute
PRODUCTION_WRITES = NO
MIGRATIONS_APPLIED = NO to existing or production stores; a separate private pilot database was initialized
DEPLOYMENT_PERFORMED = NO; local native prototype only
CURRENT_BRANCH = codex/edge-local-agent-folder-pilot
STARTING_MAIN_SHA = eafb30d8548e2b1e3bc36fc34b2c8f0b36f633b7
FINAL_HEAD_SHA = eafb30d8548e2b1e3bc36fc34b2c8f0b36f633b7; task changes uncommitted
PR = NONE
NEXT_SINGLE_ACTION = Bind this local agent to MicroDAO membership and the existing City interface
```
