# Windows installer preparation — same local-agent pilot

TASK = Windows installer and restored basic device check
PRIMARY_OUTCOME = Build and verify an installer for the existing pilot on Windows
RESULT = PARTIAL
IMPLEMENTED = Pilot-specific NSIS build script, artifact-only Windows workflow, disposable install/UI/restart/uninstall check, native basic device-readiness card
FILES_CHANGED = 11 files relative to the inherited pilot; 25 prior uncommitted pilot files carried forward without changing their original checkout
TESTS_RUN = Packaging config/schema and JavaScript syntax PASS; TypeScript/frontend build PASS; native Mac build PASS; four focused Rust tests PASS, one live City/model test intentionally ignored; clippy exit 0 with no diagnostics in the changed native modules; inference contract PASS; inherited storage inventory check FAIL in both copies; Windows tests NOT RUN
CORE_USER_PATH_SMOKE = Actual Mac native device scan PASS; existing agent and saved result visible; Windows installation and employee-device path NOT RUN
P0 = None found in the scoped change review; no general security-audit claim
P1 = Windows execution blocked by GitHub account billing lock before any step started; no installer artifact or Windows acceptance yet
P2_BACKLOG = Inherited fixed Tauri-importer inventory rejects the pilot; GPU/driver/model-fit validation is not implemented by this basic scan
P3_BACKLOG = Signing/distribution polish and additional Windows architectures
PRODUCTION_WRITES = NO
MIGRATIONS_APPLIED = NO
DEPLOYMENT_PERFORMED = NO
CURRENT_BRANCH = codex/edge-windows-installer-pilot
STARTING_MAIN_SHA = eafb30d8548e2b1e3bc36fc34b2c8f0b36f633b7
FINAL_HEAD_SHA = Application/build source pushed at a633aeac3e24b638e152e32aeba6f8f73fa80cb9; this evidence-only follow-up is recorded in Git history
PR = NONE
NEXT_SINGLE_ACTION = Resolve the GitHub account billing lock, then rerun the existing Windows workflow

## Authorized push and actual Windows attempt

On 2026-09-05 the operator authorized committing/pushing this isolated branch to build and verify the Windows package without publishing a release. Commit `a633aeac3e24b638e152e32aeba6f8f73fa80cb9` was pushed and read back from the remote task branch. Canonical main stayed at `eafb30d8548e2b1e3bc36fc34b2c8f0b36f633b7`; no PR, merge or release was created.

[Windows run 33975555586](https://github.com/DAARION-DAO/daarion-edge-client/actions/runs/33975555586) failed before starting any step. The `windows-pilot` job has an empty steps list; the run has zero artifacts. GitHub's check annotation says: "The job was not started because your account is locked due to a billing issue."

Classification: **CI_INFRA_BLOCKED**, not a compilation/test failure and not Windows PASS. No repeated retry was attempted because the account condition had not changed. The prepared build and smoke scripts remain ready. Compilation, installer creation, installation, native Windows UI and restart checks did not run. No account billing setting, payment method or spending limit was changed.

## What was reused

The old Windows setup is not unsuitable as an installer format. Tauri/NSIS is retained. The published v0.2.2-3 binary predates the uncommitted pilot, so it cannot represent this new local-agent path. The new script explicitly builds `edge-local-agent` with the pilot feature and identity, checks its PE x64 header, bundles only the expected main executable, and writes installer/executable SHA-256 hashes plus source commit/dirty status. An unsigned package is not a signed production release.

The native readiness card reuses `get_device_capability_profile` for measured CPU/RAM/OS facts and shares `ModelProcess::installed_binary` with the existing model start path. It filters out the older heuristic GPU/model catalogue. The card does not choose a model, silently install software or enroll the device. The missing full scanner/installation-advisor integration is explicit in the UI and guide. The user's prior Hardware Checker and other OSS references are retained in that guide; no additional engine or scanner dependency was adopted.

## Verification evidence

| Check | Result and practical limit |
| --- | --- |
| `node scripts/package-local-agent-windows.mjs --check` | PASS; validates pilot identity, entrypoint, NSIS scope and matching version. Does not build Windows. |
| Installed Tauri config JSON schema | PASS for merged pilot/Windows config. |
| `node --check scripts/package-local-agent-windows.mjs` | PASS. |
| `tsc --noEmit`, `npm run build` with Node 22 | PASS; existing Browserslist age notice retained. |
| `cargo test --locked --lib --features local-agent-pilot local_agent::` | Four PASS on Mac: bounded folder read, persistence/recovery and two City protocol tests. One test requiring live model/City ports remains intentionally ignored. |
| `cargo build --locked --bin edge-local-agent --features local-agent-pilot,tauri/custom-protocol` | PASS on Mac. This is not a Windows binary. |
| `cargo clippy --locked --lib --features local-agent-pilot` | Exit 0; inherited repository warnings remain; no diagnostic in the changed native modules. |
| Real native UI | New basic scan button returned actual OS/architecture, CPU, memory and Ollama presence. Existing profile and saved task survived reopening the new build. No model/City connection action was used. |
| `validate-inference-contract.mjs` | PASS. |
| `validate-storage-runtime-contract.mjs` | FAIL: fixed Tauri core importer inventory. Reproduced against the unchanged preceding City-join pilot; deferred, not caused by this Windows step. |
| `git diff --check`, changed-file secret/private-path scan | PASS. No new dependencies, production endpoint or credential. |
| Workflow structure | Windows runner, read-only contents permission, artifact upload only; no secrets/release/deployment. Existing Pages runs only on main, release only on tags/manual dispatch. |
| Windows build guard | Refuses execution on this Mac rather than producing a mislabeled artifact. |
| PowerShell script, NSIS install, Windows UI | NOT RUN: branch is pushed, but GitHub stopped the job before its first step due to the account billing lock. No local Windows host/VM is available. |

The locally rebuilt Mac app has an ad-hoc signed binary SHA-256 of `a8170453ba915a9e113660c09c0805b259ce0d27e0fff398a99ae4a8ef6c301d`. Its receipt remains in ignored local artifacts. It was used only to verify the shared scanner/UI code. The existing MicroDAO preview/download package was not replaced and still cannot offer a Windows pilot.

## One scoped review and remediation

Reviewed the new packaging/config/workflow/test code and native/UI delta against the inherited pilot. Corrected PowerShell automatic-variable collisions before execution. The build uses fixed argv, a fixed Windows target, locked Rust dependencies and the pilot's separate identifier. The Windows smoke refuses pre-existing pilot profiles and runs only in the disposable Actions environment. It installs the exact hashed artifact, checks UI through Windows UI Automation, creates a synthetic local agent, reopens that agent, then uninstalls. It does not claim a Windows folder-task/model/City proof. Runtime UI automation availability remains a real verification question until the workflow runs.

## Release gate and rollback

Release gate at local preparation: FAIL for claiming the requested Windows installer verified, because the Windows execution evidence is absent. Local preparation result: PARTIAL. On 2026-09-05 the operator explicitly authorized committing/pushing this task branch and running its artifact-only Windows workflow, without merging or publishing a release. Windows results will be recorded after that run.

Close the new pilot and open the unchanged prior build to roll back locally. The app identity and private store schema are unchanged, and saved results remain. No changes were made to the City server, its active parallel task, MicroDAO production, source release workflow, authentication or real employee permissions.
