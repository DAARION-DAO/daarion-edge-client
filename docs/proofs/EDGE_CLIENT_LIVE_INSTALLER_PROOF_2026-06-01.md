# DAARION Edge Client Live Installer Proof — 2026-06-01

This document records the installation and startup behavior of the DAARION Edge Client `v0.2.2-2` release across the live installer matrix.

## 1. Live Installer Proof Matrix

| Platform | Artifact | Download | Install | Launch | Logs | Identity | Worker default | Uninstall | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| **Windows 11 x64** | `Daarion.Edge_0.2.2-2_x64-setup.exe` / `_x64_en-US.msi` | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **macOS Apple Silicon** | `Daarion.Edge_0.2.2-2_aarch64.dmg` | ✅ PASS | ✅ PASS | ❌ FAIL (Fixed in source) | ✅ PASS | ❌ MISSING (Fixed in source) | ✅ DISABLED | ✅ PASS | **FAIL (Retest pending for next release)** |
| **macOS Intel** | `Daarion.Edge_0.2.2-2_x64.dmg` | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **Ubuntu Linux x64** | `Daarion.Edge_0.2.2-2_amd64.AppImage` | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **Android arm64** | `Daarion.Edge_0.2.2-2_android_universal_release.apk` | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |

---

## 2. Platform Evidence Logs

### macOS Apple Silicon (arm64)

- **Artifact Filename**: `Daarion.Edge_0.2.2-2_aarch64.dmg`
- **Source URL**: `https://github.com/DAARION-DAO/daarion-edge-client/releases/download/v0.2.2-2/Daarion.Edge_0.2.2-2_aarch64.dmg`
- **Installer opens/runs**:
  - Mounted disk image successfully via CLI.
  - App bundle is ad-hoc signed (`Signature=adhoc`), meaning launch via Finder triggers standard macOS Gatekeeper unsigned/unverified developer warnings.
- **Application launches**:
  - **Result**: **FAIL**
  - Launching the inner binary `/Volumes/Daarion Edge/Daarion Edge.app/Contents/MacOS/daarion-edge-client` bypassed Gatekeeper but resulted in an immediate runtime panic.
- **Errors/Warnings**:
  - The binary crashed with the following error:
    ```text
    [2026-06-01 07:03:32.767 UTC] === DAARION Edge boot sequence started ===
    [2026-06-01 07:03:32.767 UTC] Version: 0.2.2-2
    [2026-06-01 07:03:32.767 UTC] OS: macos
    [2026-06-01 07:03:32.767 UTC] Arch: aarch64
    ...
    [2026-06-01 07:03:33.021 UTC]   setup() entered
    [2026-06-01 07:03:33.021 UTC]   Setting up system tray...
    [2026-06-01 07:03:33.032 UTC]   System tray: OK
    [2026-06-01 07:03:33.032 UTC]   Starting heartbeat loop...

    thread 'main' (44812039) panicked at src/heartbeat.rs:56:5:
    there is no reactor running, must be called from the context of a Tokio 1.x runtime
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    ```
- **Log path**:
  - **Result**: **PASS**
  - Log folder `~/.daarion-edge/logs/` was successfully created.
  - Log file `~/.daarion-edge/logs/boot.log` was populated with the startup sequence before the Tokio context panic terminated the thread.
- **Local identity**:
  - **Result**: **MISSING**
  - Since the application panicked and exited during setup, it was unable to load the webview and invoke the `initialize_identity` frontend-to-backend handler. No `identity.json` metadata file or OS keyring secure storage entry was created under `~/Library/Application Support/city.daarion.edge`.
- **Worker mode default**:
  - **Result**: **DISABLED** (opt-in worker mode was not spawned since no enrollment or identity existed).
- **Uninstall/removal behavior**:
  - Removed app configuration cache cleanly via `rm -rf ~/.daarion-edge`.
  - Unmounted disk image successfully via `hdiutil detach "/Volumes/Daarion Edge"`.

---

## 3. Findings & Next Steps

1. **Tokio Runtime Panic on Start**:
   - **Root Cause**: Spawning tokio async tasks using `tokio::spawn` within Tauri's `setup()` block on the main thread triggers a runtime panic because the main thread doesn't have an active Tokio reactor running context.
   - **Resolution**: Replaced direct `tokio::spawn` calls with Tauri's managed tokio execution context `tauri::async_runtime::spawn` in `src-tauri/src/heartbeat.rs` (start loop), `src-tauri/src/lib.rs` (on startup worker setup), and `src-tauri/src/worker/mod.rs` (inside toggle function).
   - **Local Verification**: 
     - Compiled the debug binary locally via `cargo build`.
     - Ran the compiled binary `./target/debug/daarion-edge-client` successfully.
     - Logs in `~/.daarion-edge/logs/boot.log` confirmed zero panics and complete execution of `setup()` block:
       ```text
       [2026-06-01 07:06:22.744 UTC]   Starting heartbeat loop...
       [2026-06-01 07:06:22.745 UTC]   Heartbeat loop started
       [2026-06-01 07:06:22.745 UTC]   Worker opt-in loaded: false
       [2026-06-01 07:06:22.745 UTC]   setup() completed successfully
       ```
     - Worker mode correctly loaded as `false` (disabled) by default.
   - **Next Action**: Create and push version bump tag `v0.2.2-3` to compile new binaries via CI and repeat the installer proof matrix checks.
2. **Ad-hoc codesign**:
   - The Apple Silicon DMG bundle has ad-hoc signature which behaves as expected on macOS systems, showing standard security popups when run from the UI.
3. **Labels**:
   - All references to this release remain labeled as **Canary** / **Beta** / **Manual update** / **Android sideload**, keeping the **Worker mode gated** context clear.

