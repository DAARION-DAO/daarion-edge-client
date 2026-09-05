# Windows installer for the existing local-agent pilot

EXECUTION_MODE = RESULT_FIRST_PROTOTYPE_FIRST
PRIMARY_OUTCOME = A reproducible Windows setup for the same local-agent pilot, with an honest device-readiness step and executable Windows installation checks.
CORE_USER_PATH = MicroDAO download -> Windows setup -> local device check -> personal agent -> permitted folder -> saved result.
THIS_TASK_MUST_SHIP = Pilot-specific NSIS packaging, a Windows-only artifact build/check workflow, and the missing basic device scan in the pilot UI.
SHORTEST_SAFE_PATH = Reuse Tauri/NSIS and the existing native capability scan; preserve the proven local task implementation.
RELEASE_BLOCKERS = P0 and real P1 only.
DEFER = Full model/engine auto-installation, GPU benchmarking, additional architectures, signing, cosmetics, unrelated checks.
WORKSTREAM_NAME = edge-windows-installer-pilot
WORKSTREAM_BOUNDARY = Local Edge pilot packaging, readiness UI, and disposable Windows build/install verification.
FILES_IN_SCOPE = scripts/package-local-agent-windows.mjs, scripts/test-local-agent-windows.ps1, .github/workflows/local-agent-windows.yml, src-tauri/tauri.local-agent.windows.conf.json, src-tauri/src/local_agent/{mod,model}.rs, src/pages/LocalAgentPilotPage.tsx, bounded pilot documentation.
FILES_OUT_OF_SCOPE = City server/source/settings, MicroDAO production/source, normal release workflow, existing working pilot copies, identity/permissions, production data.
ACTIVE_PR_OVERLAP = Fresh main is eafb30d8548e2b1e3bc36fc34b2c8f0b36f633b7. Only open Edge PR 21 changes two release documents outside this scope.
OPERATOR_AUTHORIZATION = YES for local implementation/build/tests and, explicitly authorized on 2026-09-05, committing/pushing this isolated task branch to run its Windows artifact checks; NO for deployment, release publishing, merging or shared-server changes.
STOP_ONLY_FOR = Data loss, secrets, unauthorized production action, irreversible operation, unknown canonical baseline, real active-PR conflict, or core scenario failure.

## Objective and current state

The repository's adopted baseline, ownership, roadmap and relevant phase/security/testing/release skills were read. This continues the explicitly requested isolated pilot; it does not authorize Phase 1B.4 or Phase 1C. The 25 uncommitted files from the existing City-join pilot are preserved in an isolated branch at the fresh canonical base. The original working copies remain intact.

The published v0.2.2-3 Windows NSIS/MSI packages predate this uncommitted pilot. Their packaging technology is reusable, but those binaries do not contain its entrypoint or UI. The normal release workflow publishes releases and is unsuitable for an artifact-only test. No Windows host/VM is available locally. Windows compilation, installation, first launch and restart evidence must come from a Windows execution environment; macOS checks cannot substitute for it.

## Scope and repository ownership

Edge owns the native installer and device detection. MicroDAO owns the download page, which must not advertise this artifact as available until it exists. Build the explicitly named `edge-local-agent` binary with the pilot feature and config. Bundle only that executable with the existing isolated pilot identifier, current-user NSIS installation and WebView2 bootstrapper. No City API is called by installer verification.

Restore the existing local scanner's CPU/RAM/OS facts in the pilot UI. Do not expose its unverified static model catalogue or imply that Windows GPU/drivers were validated. Distinguish a basic scan from a full model-installation advisor. Reuse Ollama executable discovery to explain whether the existing model path can be used.

## Contracts affected and non-goals

One private, read-only pilot Tauri command returns a minimal readiness projection, without hostnames, paths, keys or network writes. No shared web/auth contract, database schema, agent identity, City transport or model-download contract changes. No new runtime dependency. Full scanner/installer composition remains a named gap, with the previously proposed OSS candidates recorded in the guide.

## Security, migration and compatibility

Keep the separate pilot identifier and per-user install location so the main Edge installation is preserved. Require the expected executable, PE architecture and hash. Build receipt names source state, hashes and unsigned status. The Windows test installs only this artifact into a disposable directory, verifies launch, closes only its own process and removes only its own test installation. It must refuse a pre-existing pilot profile/install. Do not upload raw profiles or hardware inventories. No schema migration. Models, folders and City connection remain opt-in.

## Implementation steps and tests

1. Add a fixed Windows build/bundle script and NSIS overlay; use the existing locked toolchain/dependencies.
2. Add a Windows artifact-only workflow with read-only repository permissions, no release or secrets, and a scoped PowerShell install/launch/relaunch/uninstall check.
3. Restore the basic scan/readiness card and validate its minimal projection.
4. Run targeted tests, typecheck/build and scoped Rust checks. Review the exact new diff once. Report unavailable Windows evidence explicitly.

## Acceptance criteria

- The build selects the pilot executable/config and emits an installer plus SHA-256 receipt.
- A real Windows run installs that exact artifact, starts the pilot window, restarts it, and removes the test installation without affecting another install.
- The pilot displays a real local CPU/RAM/OS scan and accurately states model-runtime/GPU limits; scan does not install anything or connect to City.
- Existing private profile, folder and saved-result behavior stays intact.
- No deployment, release publication or misleading frontend download link.

## Rollback strategy and documentation updates

Close the new pilot and use the unchanged previous pilot. Remove only the disposable build/test artifacts. Preserve user profiles. Update the pilot guide and add a completion report with required versus unexecuted evidence.

## Open questions and verdict

The operator explicitly authorized committing/pushing this isolated branch and running its artifact-only Windows checks on 2026-09-05. Full automatic model installation needs a pinned scanner/recipe/backend integration and is not established by basic hardware readings.

The parallel City workstream confirmed that Edge can be registered through Agents as a separate `remote-ag-ui` agent, outside Core, after endpoint/protocol/authentication verification. The existing pilot already targets that boundary. This confirmation does not authorize a shared-server restart, endpoint allowlist change, or sharing credentials in chat.

CONDITIONAL_GO for local source, packaging and checks. Windows acceptance and employee-ready distribution remain unproven until an actual Windows build/install test runs. This condition does not block independent local preparation.
