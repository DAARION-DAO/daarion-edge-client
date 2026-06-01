# EDGE INSTALLER READONLY AUDIT — 2026-05-31

## Executive Summary

The **DAARION Edge Client** serves as the sovereign entry point into the DAARION AgentOS ecosystem (DAGI). It is built as a Tauri v2 / React / TypeScript application designed to run on macOS, Windows, Linux, and Android. 

The project has transitioned from a transitional identity-service-v2 structure to a canonical **SOFIIA Worker Registry** registration flow (`POST /api/v1/nodes/register`) that uses client Ed25519 signatures and remains admin-gated by default. 

### Verdict: YELLOW (Progress from RED)
- **macOS (Arm/Intel)**: Stable and verified in local development. Manual DMG builds are functional.
- **Android**: Significant progress has been made since the previous audit. GitHub Actions release workflows have been patched to reconstruct a release keystore from base64 secrets and apply signing configurations to the Gradle Kotlin build script. However, real-device installation verification (Install Proof) is still pending.
- **Windows**: The blank-window crash issue has been mitigated by introducing native boot diagnostics logging to `%APPDATA%/DAARION Edge/logs/boot.log`. However, process longevity (Boot Proof) on clean target machines is still being validated.
- **Linux**: Build pipelines produce AppImage outputs, but verification remains manual-only.
- **iOS**: Target platform is defined, but Matrix builds are currently skipped in CI due to Xcode free-tier restrictions.

---

## 1. Current Repository Shape

The `daarion-edge-client` repository is structured as a Tauri v2 application wrapping a Vite React frontend:
- `/src-tauri/`: Rust backend architecture.
  - [src-tauri/src/lib.rs](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src-tauri/src/lib.rs): App entry point, boot logging, and command registration.
  - [src-tauri/src/identity.rs](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src-tauri/src/identity.rs): Cryptographic Ed25519 key generation and secure OS keyring storage.
  - [src-tauri/src/enrollment.rs](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src-tauri/src/enrollment.rs): Enrollment logic for registering nodes with the SOFIIA registry.
  - [src-tauri/src/capabilities.rs](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src-tauri/src/capabilities.rs): Hardware inspection, device classification, and local LLM recommendation logic.
  - [src-tauri/src/worker/](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src-tauri/src/worker): Worker loop implementation, NATS/WebSocket client connection, and sandboxed Docker runners.
- `/src/`: Frontend React interface.
  - [src/App.tsx](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src/App.tsx): Routing gate (GenesisWizard for L1 client registration, main Dashboard, pending screen, or revocation lock screen).
  - [src/components/EdgeActivation.tsx](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src/components/EdgeActivation.tsx): Multi-step L3 worker onboarding (Environment verify, operator approval verification, tunnel activation).
  - [src/components/GenesisWizard.tsx](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src/components/GenesisWizard.tsx): In-app cryptographic setup and personal agent directive definition.

---

## 2. Installer Technology

- **Framework**: Tauri v2 (`@tauri-apps/api: "^2"`).
- **Frontend Stack**: React 19, TypeScript, Vite 7, Tailwind CSS v4.
- **CI Packaging**: Builds MSI and NSIS for Windows, DMG for macOS, and AppImage for Linux via `tauri-apps/tauri-action@v0`.
- **Mobile Packing**: Builds signed Android APKs via `cargo tauri android build --apk`.

---

## 3. Supported Platforms

| OS / Target | Architecture | Format | Status |
|-------------|--------------|--------|--------|
| macOS | Apple Silicon (`aarch64-apple-darwin`) | `.dmg` | ✅ Verified |
| macOS | Intel (`x86_64-apple-darwin`) | `.dmg` | ✅ Verified |
| Windows | x86_64 | `.exe` (NSIS) / `.msi` | ⚠️ Built / Boot proof pending |
| Linux | x86_64 | `.AppImage` | ✅ Built / Untested |
| Android | arm64-v8a | `.apk` (Signed) | ⚠️ Signed preview / Install proof pending |
| iOS | arm64 | Native App | ❌ Skipped in CI |

---

## 4. Release Artifact Truth Table

| Target | Build Status | Signing Status | Install Proof | Boot Proof |
|--------|--------------|----------------|---------------|------------|
| **macOS (Arm)** | Pass | Developer Signed | Pass | Pass (Dev machine) |
| **macOS (Intel)** | Pass | Developer Signed | Pass | Pass (Dev machine) |
| **Windows** | Pass | Unsigned / Self | Pass | Pending (check `%APPDATA%/logs/boot.log` on crash) |
| **Linux** | Pass | Unsigned | Pending | Pending |
| **Android** | Pass | Release Key Signed | Pending | Pending |
| **iOS** | Skipped | N/A | N/A | N/A |

---

## 5. Public UX / README Truth

### README Gap
- The `daarion-edge-client/README.md` is currently a boilerplate template. It contains no documentation on download links, setup procedures, WSL2 activation, or debugging commands.

### install Page Mismatches (daarion-ai-city)
On the frontend page [/Users/apple/daarion-ai-city/src/pages/Install.tsx](file:///Users/apple/daarion-ai-city/src/pages/Install.tsx):
- **Android Download**: Marked as "Disabled" and "Signing pending" (lines 209-217). This does not match reality, as signed Android builds are now successfully produced in the CI workflow.
- **Version Skew**: Download links for macOS and Linux point to version `0.1.0` (lines 177, 186, 205), whereas the project is currently tagged as `v0.2.0-beta` / `v0.2.1-canary.1`.
- **Windows Link**: Hardcoded to download `v0.2.1-canary.1/Daarion.Edge_0.2.0_x64-setup.exe` (line 195).

---

## 6. Onboarding Flow Findings

The codebase implements a tiered access onboarding model:
1. **L1 Personal Agent**: Triggered via `GenesisWizard.tsx`. Generates Ed25519 identity keys and registers EVM/Solana addresses to the backend via `provision_sovereign_genesis` in `provisioning.rs`.
2. **L1 Local Runtime**: Initialized automatically on app start. Sets up local databases and boots the background heartbeat loop.
3. **L3 Worker Node (Advisory)**: Optional flow triggered in `EdgeActivation.tsx`.
   - Checks if WSL2 (Windows) or Colima/Docker (macOS) is available.
   - Looks for `operator_token.txt` at `{HOME}/.daarion/operator_token.txt` to verify admin-gated whitelisting.
   - Connects to relay endpoints via a WebSocket client.

---

## 7. Security and Identity Findings

- **Identity Storage**: On first run, `identity.rs` generates a 256-bit Ed25519 keypair. The private key is stored in the OS secure storage (Keychain on macOS, Credential Manager on Windows) under `com.daarion.edge.identity` using the `keyring` crate. The public key and node UUID are stored in `identity.json`.
- **Worker Registration**: The node signs its registration request (`node_id|public_key|invite_code`) with its private key, which is verification-gated on the registry backend.
- **Revocation Check**: The heartbeat loop updates local enrollment state. If the backend marks the node as revoked (`revoked: true`), `App.tsx` immediately blocks access and displays the "Node Revoked" lock screen.
- **Logs Privacy**: System logs are limited to boot sequences written to the local log folder. Plaintext private keys are never written to disk or console.

---

## 8. Compute / Expertise Contribution Readiness

- The worker loop is configured to request tasks from the WebSocket relay client.
- Bounded sandboxing is implemented in `runner.rs` via Docker/WSL2 containers with `--network none` and `env_clear()`.
- Currently, tasks are restricted to deterministic advisory functions: `ping_math` and `text_hash` running in lightweight Alpine Python containers.

---

## 9. Auto-update / Manual-update Status

- **Tauri Auto-Updater**: Lacks configuration in `tauri.conf.json`. No updater plugin is imported or compiled.
- **Status**: **Manual-only**. The user is warned on the UI and onboarding screens that they must download and install updates manually from GitHub.

---

## 10. Windows WSL2 Bootstrap Status

- The WSL2 check is implemented in `onboarding.rs` via a native shell check: `wsl --status`.
- If WSL2 is missing, `enable_wsl_windows` triggers an elevated PowerShell command:
  ```powershell
  Start-Process wsl -ArgumentList '--install' -Verb RunAs
  ```
  This prompts the user for UAC permissions to enable the WSL subsystem.

---

## 11. Critical Gaps

1. **Empty README**: Lacks basic operator runbooks and architecture overviews.
2. **AI-City install Page Outdated**: Mismatched version tags, disabled Android link despite signed builds in CI, and manually hardcoded canary assets.
3. **No In-App Log Viewer**: If the Windows client crashes or starts as a blank screen, users must manually search `%APPDATA%/DAARION Edge/logs/boot.log` in Explorer. A diagnostic view or error-handling overlay is missing in the React app itself.

---

## 12. Safe Next Implementation Plan (PR Plan)

### Stage 1: Document & Instruct (daarion-edge-client)
- **Task**: Replace the placeholder `README.md` in `daarion-edge-client` with a functional document describing:
  1. Quickstart commands (`npm run dev`, `npm run tauri android dev`).
  2. Location of configuration directories and key storage backends.
  3. Diagnostic logging locations (`boot.log`) for Windows and macOS.
  4. Manual update instructions.

### Stage 2: Align UI Install Page (daarion-ai-city)
- **Task**: Update `src/pages/Install.tsx` in `daarion-ai-city` to:
  1. Reference the latest `v0.2.1-canary.1` (or release variables) instead of the outdated `0.1.0`.
  2. Enable the Android download link and point it directly to the signed `.apk` released on GitHub.
  3. Clear up translations to reflect that signed Android APKs are ready for sideloading.

### Stage 3: UI Boot Diagnostics Access (daarion-edge-client)
- **Task**: Add an diagnostic debug console inside `daarion-edge-client`'s dashboard tab or as an error screen fallback so that testers can view or copy log lines from `boot.log` directly in the UI.

---

## 13. Do Not Touch List

- **Backend / SOFIIA**: Do not modify registration/approval APIs or the central registry logic.
- **Relay Transport**: Do not alter NATS routing layers or Octelium secure tunnels.
- **Consensus**: Do not introduce any write paths to network consensus.

---

## 14. Evidence Appendix

- **Boot Logging**: [src-tauri/src/lib.rs:32-86](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src-tauri/src/lib.rs#L32-L86)
- **Keyring Storage**: [src-tauri/src/identity.rs:69-72](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src-tauri/src/identity.rs#L69-L72)
- **WSL Bootstrap**: [src-tauri/src/worker/onboarding.rs:80-97](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src-tauri/src/worker/onboarding.rs#L80-L97)
- **Docker Sandbox Boundary**: [src-tauri/src/worker/runner.rs:36-81](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src-tauri/src/worker/runner.rs#L36-L81)
- **Revocation Gate**: [src/App.tsx:123-138](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src/App.tsx#L123-L138)
