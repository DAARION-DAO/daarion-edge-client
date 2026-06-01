# DAARION Edge Client Live Installer Proof — 2026-06-01

This document records the installation and startup behavior of the DAARION Edge Client `v0.2.2-3` release (which contains the PR 4 startup fix) across the live installer matrix.

## 1. Live Installer Proof Matrix

| Platform | Artifact | Download | Install | Launch | Logs | Identity | Worker default | Uninstall | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| **Windows 11 x64** | `Daarion.Edge_0.2.2-3_x64-setup.exe` / `_x64_en-US.msi` | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **macOS Apple Silicon** | `Daarion.Edge_0.2.2-3_aarch64.dmg` | ✅ PASS | ✅ PASS | ✅ PASS | ✅ PASS | ➖ Pending | ✅ DISABLED | ✅ PASS | **PASS** |
| **macOS Intel** | `Daarion.Edge_0.2.2-3_x64.dmg` | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **Ubuntu Linux x64** | `Daarion.Edge_0.2.2-3_amd64.AppImage` | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **Android arm64** | `Daarion.Edge_0.2.2-3_android_universal_release.apk` | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | N/A | ➖ Pending | **BLOCKED / NOT TESTED** |

---

## 2. Platform Evidence Logs

### macOS Apple Silicon (arm64)

- **Artifact Filename**: `Daarion.Edge_0.2.2-3_aarch64.dmg`
- **Source URL**: `https://github.com/DAARION-DAO/daarion-edge-client/releases/download/v0.2.2-3/Daarion.Edge_0.2.2-3_aarch64.dmg`
- **Installer opens/runs**:
  - Mounted disk image successfully via CLI.
  - App bundle is ad-hoc signed (`Signature=adhoc`), meaning launch via Finder triggers standard macOS Gatekeeper unsigned/unverified developer warnings.
- **Application launches**:
  - **Result**: **PASS**
  - Launching the inner binary `/Volumes/Daarion Edge/Daarion Edge.app/Contents/MacOS/daarion-edge-client` bypassed Gatekeeper and executed successfully with zero runtime panics.
- **Log path**:
  - **Result**: **PASS**
  - Log folder `~/.daarion-edge/logs/` was successfully created.
  - Log file `~/.daarion-edge/logs/boot.log` was populated correctly with the full startup sequence, confirming clean initialization of handlers and completion of the setup hook.
- **Logs Output (v0.2.2-3)**:
  ```text
  [2026-06-01 07:18:03.093 UTC] === DAARION Edge boot sequence started ===
  [2026-06-01 07:18:03.093 UTC] Version: 0.2.2-3
  [2026-06-01 07:18:03.093 UTC] OS: macos
  [2026-06-01 07:18:03.093 UTC] Arch: aarch64
  [2026-06-01 07:18:03.093 UTC] Initializing Tauri builder...
  [2026-06-01 07:18:03.094 UTC]   Tauri builder created
  [2026-06-01 07:18:03.094 UTC]   Plugin: opener initialized
  [2026-06-01 07:18:03.094 UTC]   Plugin: shell initialized
  [2026-06-01 07:18:03.094 UTC]   Managing state: HeartbeatManager, MessagingState, WorkerModeState
  [2026-06-01 07:18:03.094 UTC]   Registering invoke handlers...
  [2026-06-01 07:18:03.094 UTC]   Invoke handlers registered
  [2026-06-01 07:18:03.094 UTC]   Configuring setup()...
  [2026-06-01 07:18:03.094 UTC] Running Tauri application...
  [2026-06-01 07:18:03.328 UTC]   setup() entered
  [2026-06-01 07:18:03.328 UTC]   Setting up system tray...
  [2026-06-01 07:18:03.342 UTC]   System tray: OK
  [2026-06-01 07:18:03.342 UTC]   Starting heartbeat loop...
  [2026-06-01 07:18:03.342 UTC]   Heartbeat loop started
  [2026-06-01 07:18:03.342 UTC]   Worker opt-in loaded: false
  [2026-06-01 07:18:03.342 UTC]   setup() completed successfully
  ```
- **Local identity**:
  - **Result**: **Pending** (requires loading UI to trigger initialization invoke command `initialize_identity` from the frontend).
- **Worker mode default**:
  - **Result**: **DISABLED** (opt-in worker mode was not spawned since no enrollment or identity existed, loading `Worker opt-in loaded: false` as expected).
- **Uninstall/removal behavior**:
  - Removed app configuration cache cleanly via `rm -rf ~/.daarion-edge`.
  - Unmounted disk image successfully via `hdiutil detach "/Volumes/Daarion Edge"`.

---

## 3. Findings & Next Steps

1. **Tokio Runtime Panic on Start**:
   - **Root Cause**: Spawning tokio async tasks using `tokio::spawn` within Tauri's `setup()` block on the main thread triggered a runtime panic because the main thread doesn't have an active Tokio reactor running context.
   - **Resolution**: Replaced direct `tokio::spawn` calls with Tauri's managed tokio execution context `tauri::async_runtime::spawn` in `src-tauri/src/heartbeat.rs` (start loop), `src-tauri/src/lib.rs` (on startup worker setup), and `src-tauri/src/worker/mod.rs` (inside toggle function).
   - **Retest Verification**: 
     - Downloaded and mounted the new release `v0.2.2-3` DMG asset.
     - Ran the compiled binary and verified it executed successfully without Tokio runtime crashes.
     - The boot log correctly completed setup with the tray and heartbeat loops running:
       ```text
       [2026-06-01 07:18:03.342 UTC]   Worker opt-in loaded: false
       [2026-06-01 07:18:03.342 UTC]   setup() completed successfully
       ```
     - Worker mode correctly loaded as `false` (disabled) by default.
2. **Ad-hoc codesign**:
   - The Apple Silicon DMG bundle has ad-hoc signature which behaves as expected on macOS systems, showing standard security popups when run from the UI.
3. **Labels**:
   - All references to this release remain labeled as **Canary** / **Beta** / **Manual update** / **Android sideload**, keeping the **Worker mode gated** context clear.
