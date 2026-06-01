# DAARION Edge Client Release Proof — v0.2.2-3

## 1. Release Metadata
- **Repository**: DAARION-DAO/daarion-edge-client
- **Release tag**: v0.2.2-3
- **Commit SHA**: d00d825944b9fabbed6cfd23d5317db3e6c8f7c4
- **GitHub Actions workflow run**: [Run #26740554535](https://github.com/DAARION-DAO/daarion-edge-client/actions/runs/26740554535)
- **Release URL**: [Release v0.2.2-3](https://github.com/DAARION-DAO/daarion-edge-client/releases/tag/v0.2.2-3)
- **Date**: 2026-06-01
- **Operator**: Antigravity (Advanced Agentic Coding Assistant)

---

## 2. Expected Artifact Contract

| Platform / Artifact Type | Exact Asset Name | Present | Size (Bytes) | Size (MB) | Verified HTTP 200 | SHA256 Checksum |
|---|---|---|---|---|---|---|
| Windows Setup | `Daarion.Edge_0.2.2-3_x64-setup.exe` | ✅ Present | 5,206,713 | 4.97 MB | ✅ Downloadable | `f9deec38e72441688785fd0ba57d2df3fadedfea41375de5432530808cbd55e0` |
| Windows MSI | `Daarion.Edge_0.2.2-3_x64_en-US.msi` | ✅ Present | 7,290,880 | 6.95 MB | ✅ Downloadable | `c88b54f5b5a8ff9c15b03c5bc4c35028a4b6dd25cde2aed4edd20018a6b439ac` |
| macOS Apple Silicon | `Daarion.Edge_0.2.2-3_aarch64.dmg` | ✅ Present | 7,803,223 | 7.44 MB | ✅ Downloadable | `ef428556b58ece4529e535f3ec8a2e9e356f905d0df02b4907af2de06d317aa1` |
| macOS Intel | `Daarion.Edge_0.2.2-3_x64.dmg` | ✅ Present | 8,081,498 | 7.71 MB | ✅ Downloadable | `4a6a46ea4e1ec2f8dd32800850a8f2eae436bc10da1dc78abe50f51e77dd1519` |
| Linux AppImage | `Daarion.Edge_0.2.2-3_amd64.AppImage` | ✅ Present | 86,129,144 | 82.14 MB | ✅ Downloadable | `75d5cb6eb9176858d4a812df706182dca59f501bf61da453e7dea337289b22d1` |
| Android APK | `Daarion.Edge_0.2.2-3_android_universal_release.apk` | ✅ Present | 75,988,408 | 72.47 MB | ✅ Downloadable | `afbc274e4bb0a75b6d50129cdc3ab9f40a7ffec0de5c407f24370366ef198e24` |
| Windows Manifest | `release-manifest-windows-x86_64.json` | ✅ Present | 498 | <0.01 MB | ✅ Downloadable | `e470b736653e0748766802bbdd1658e77ee2ec41dcdd87360957ba43bea6eb61` |
| macOS Apple Silicon Manifest | `release-manifest-macos-arm64.json` | ✅ Present | 330 | <0.01 MB | ✅ Downloadable | `44d29c9cb4337b44128874524c9314b31595cd11c95c6cf642eb072bf2715b14` |
| macOS Intel Manifest | `release-manifest-macos-x86_64.json` | ✅ Present | 325 | <0.01 MB | ✅ Downloadable | `ef9f1cfaccdd1664d117d7ac5fe91d9dc0c396d021571501e61d7add04d2b4da` |
| Linux Manifest | `release-manifest-linux-x86_64.json` | ✅ Present | 337 | <0.01 MB | ✅ Downloadable | `f7ea733691cb3bc3821c9e5caca0afe2be2c29b0ca047c767c42b422a8900e06` |
| Android Manifest | `release-manifest-android-arm64.json` | ✅ Present | 352 | <0.01 MB | ✅ Downloadable | `2d8d8de41d920c47f5b691352847eaf2f08384c9106d471ea2a5e369a04bc61e` |

---

## 3. Live Installer Proof Matrix

| Platform | Download | Install | Launch | Logs | Identity | Worker default | Uninstall | Verdict |
|---|---|---|---|---|---|---|---|---|
| **Windows 11 x64** | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **macOS Apple Silicon** | ✅ PASS | ✅ PASS | ✅ PASS | ✅ PASS | ➖ Pending | ✅ DISABLED | ✅ PASS | **PASS (Launch proof passed)** |
| **macOS Intel** | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **Ubuntu Linux x64** | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | **BLOCKED / NOT TESTED** |
| **Android arm64** | ✅ PASS | ➖ Pending | ➖ Pending | ➖ Pending | ➖ Pending | N/A | ➖ Pending | **BLOCKED / NOT TESTED** |

---

## 4. Runtime Checks

- **App starts without crash**: ✅ PASS (verified on macOS Apple Silicon, startup panic fixed)
- **Local log file created**: ✅ PASS (`~/.daarion-edge/logs/boot.log` verified)
- **Local identity created or detected**: ➖ Pending (requires loading UI to invoke command)
- **No private key printed in logs**: ✅ PASS (No secret keys printed in `boot.log`)
- **Device capabilities detected**: ➖ Pending
- **Network state shown honestly**: ➖ Pending
- **Worker mode disabled by default**: ✅ PASS (Disabled by default, loop did not run)
- **Worker activation gated**: ➖ Pending
- **Uninstall/revoke path documented**: ✅ PASS (Removal paths validated)

---

## 5. Screenshots / Evidence

- **macOS Apple Silicon (v0.2.2-3) Successful Boot Log**:
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

---

## 6. Known Issues

- **v0.2.2-canary.1 Initial Diagnostic Run**: Desktop build outputs (Windows NSIS/MSI, macOS DMG, Linux AppImage) were skipped because `.github/workflows/release.yml` had `includeRelease: false` configured for the `tauri-action` step. The Android arm64 build successfully completed compilation and uploaded the signed APK: `Daarion.Edge_0.2.2-canary.1_android_universal_release.apk`. Fix applied in `v0.2.2-canary.2` to set `includeRelease: true`.
- **v0.2.2-canary.2 Windows Bundle Fail**: The Windows WiX bundler failed with `failed to bundle project: optional pre-release identifier in app version must be numeric-only and cannot be greater than 65535 for msi target`. This was caused by the alphanumeric pre-release identifier `canary.2`. WiX requires a purely numeric-only identifier for pre-releases (e.g. `0.2.2-2` which translates to product version `0.2.2.2`). Fixed in `v0.2.2-2` by switching version format to `0.2.2-2`.
- **v0.2.2-2 macOS Startup Panic**: Spawning tokio async tasks using `tokio::spawn` within Tauri's `setup()` block triggers a runtime panic due to missing Tokio reactor context. Fixed in `v0.2.2-3` by using `tauri::async_runtime::spawn` instead of `tokio::spawn`.

---

## 7. Final Verdict

- **Release Status**: **PASS (verified macOS Apple Silicon launch proof)**

