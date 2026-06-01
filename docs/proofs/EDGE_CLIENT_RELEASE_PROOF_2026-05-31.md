# DAARION Edge Client Release Proof — v0.2.2-2

## 1. Release Metadata
- **Repository**: DAARION-DAO/daarion-edge-client
- **Release tag**: v0.2.2-2
- **Commit SHA**: 4e99823a2d72b293058e3fb7c288a27211eeb306
- **GitHub Actions workflow run**: [Run #26738969666](https://github.com/DAARION-DAO/daarion-edge-client/actions/runs/26738969666)
- **Release URL**: [Release v0.2.2-2](https://github.com/DAARION-DAO/daarion-edge-client/releases/tag/v0.2.2-2)
- **Date**: 2026-05-31
- **Operator**: Antigravity (Advanced Agentic Coding Assistant)

---

## 2. Expected Artifact Contract

| Platform / Artifact Type | Exact Asset Name | Present | Size (Bytes) | Size (MB) | Verified HTTP 200 | SHA256 Checksum |
|---|---|---|---|---|---|---|
| Windows Setup | `Daarion.Edge_0.2.2-2_x64-setup.exe` | ✅ Present | 5,214,257 | 4.97 MB | ✅ Downloadable | `1fb022eac2ff00ba1bf3565dc56ba2556feb6e7dbdc0c423ac6654537eae012b` |
| Windows MSI | `Daarion.Edge_0.2.2-2_x64_en-US.msi` | ✅ Present | 7,286,784 | 6.95 MB | ✅ Downloadable | `627c324157557ee279a8d3233508a79a63babe0b1c18961cf7f769fb0f3a26e3` |
| macOS Apple Silicon | `Daarion.Edge_0.2.2-2_aarch64.dmg` | ✅ Present | 7,796,727 | 7.44 MB | ✅ Downloadable | `270b17be3ebfbe3323d6b92066d95e0657c85c83459ad262d85294a6528224e4` |
| macOS Intel | `Daarion.Edge_0.2.2-2_x64.dmg` | ✅ Present | 8,074,201 | 7.70 MB | ✅ Downloadable | `df09a001be70ab316d0ed1b9d34a152f329311aee063e693f68c50a81aeb29c3` |
| Linux AppImage | `Daarion.Edge_0.2.2-2_amd64.AppImage` | ✅ Present | 86,104,568 | 82.12 MB | ✅ Downloadable | `4ab14bb35e35d0df5970e1a13aec457f700fefa0519ac03889cf3862caeabf00` |
| Android APK | `Daarion.Edge_0.2.2-2_android_universal_release.apk` | ✅ Present | 75,993,724 | 72.47 MB | ✅ Downloadable | `250e0a4dfd3538209accfc7921d887e703746496dc86577f19b0f014b2678888` |
| Windows Manifest | `release-manifest-windows-x86_64.json` | ✅ Present | 498 | <0.01 MB | ✅ Downloadable | `5d2da61b4803cf35c765ffc39a753c9a24eeb3b42503179f7af232ccf735cd6b` |
| macOS Apple Silicon Manifest | `release-manifest-macos-arm64.json` | ✅ Present | 330 | <0.01 MB | ✅ Downloadable | `401520b56d8a826086745ffb46bcb91398a741e1524173271832bf5d08cf0df4` |
| macOS Intel Manifest | `release-manifest-macos-x86_64.json` | ✅ Present | 325 | <0.01 MB | ✅ Downloadable | `7587689d55409fd2f8f8f920d02d1c04614e664e4bcc7fd612970a50a1cf95e0` |
| Linux Manifest | `release-manifest-linux-x86_64.json` | ✅ Present | 337 | <0.01 MB | ✅ Downloadable | `b65a1bf70731e0948080d7622c1ab892b062e727f2a87362e5bb048f621e634f` |
| Android Manifest | `release-manifest-android-arm64.json` | ✅ Present | 352 | <0.01 MB | ✅ Downloadable | `3529a95c41226ac44e79d39982780e68f1f73a83221e5f0ceb9e157d6949768a` |

---

## 3. Live Installer Proof Matrix

| Platform | Download | Install | Launch | Logs | Identity | Worker default | Uninstall | Verdict |
|---|---|---|---|---|---|---|---|---|
| **Windows 11 x64** | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **macOS Apple Silicon** | ✅ PASS | ✅ PASS | ❌ FAIL | ✅ PASS | ❌ MISSING | ✅ DISABLED | ✅ PASS | **FAIL** |
| **macOS Intel** | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **Ubuntu Linux x64** | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **Android arm64** | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | N/A | ➖ Pending | **BLOCKED / NOT TESTED** |

---

## 4. Runtime Checks

- **App starts without crash**: ❌ FAIL (Tokio reactor runtime error on main thread)
- **Local log file created**: ✅ PASS (`~/.daarion-edge/logs/boot.log` verified)
- **Local identity created or detected**: ❌ MISSING (App crashed before frontend loaded to trigger initialization invoke command)
- **No private key printed in logs**: ✅ PASS (No secret keys printed in `boot.log`)
- **Device capabilities detected**: ➖ Pending
- **Network state shown honestly**: ➖ Pending
- **Worker mode disabled by default**: ✅ PASS (Disabled by default, loop did not run)
- **Worker activation gated**: ➖ Pending
- **Uninstall/revoke path documented**: ✅ PASS (Removal paths validated)

---

## 5. Screenshots / Evidence

- **macOS Apple Silicon Crash Output**:
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
  ```

---

## 6. Known Issues

- **v0.2.2-canary.1 Initial Diagnostic Run**: Desktop build outputs (Windows NSIS/MSI, macOS DMG, Linux AppImage) were skipped because `.github/workflows/release.yml` had `includeRelease: false` configured for the `tauri-action` step. The Android arm64 build successfully completed compilation and uploaded the signed APK: `Daarion.Edge_0.2.2-canary.1_android_universal_release.apk`. Fix applied in `v0.2.2-canary.2` to set `includeRelease: true`.
- **v0.2.2-canary.2 Windows Bundle Fail**: The Windows WiX bundler failed with `failed to bundle project: optional pre-release identifier in app version must be numeric-only and cannot be greater than 65535 for msi target`. This was caused by the alphanumeric pre-release identifier `canary.2`. WiX requires a purely numeric-only identifier for pre-releases (e.g. `0.2.2-2` which translates to product version `0.2.2.2`). Fixed in `v0.2.2-2` by switching version format to `0.2.2-2`.
- **v0.2.2-2 macOS Startup Panic**: Spawning tokio async tasks using `tokio::spawn` within Tauri's `setup()` block triggers a runtime panic due to missing Tokio reactor context. Fix required: use `tauri::async_runtime::spawn` instead of `tokio::spawn`.

---

## 7. Final Verdict

- **Release Status**: **FAIL (due to macOS startup panic)**

