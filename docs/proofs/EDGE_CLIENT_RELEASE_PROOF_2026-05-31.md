# DAARION Edge Client Release Proof — v0.2.2-canary.2

## 1. Release Metadata
- **Repository**: DAARION-DAO/daarion-edge-client
- **Release tag**: v0.2.2-canary.2
- **Commit SHA**: Pending
- **GitHub Actions workflow run**: Pending
- **Release URL**: Pending
- **Date**: 2026-05-31
- **Operator**: Antigravity (Advanced Agentic Coding Assistant)

---

## 2. Expected Artifact Contract

| Platform | Expected artifact | Present | Notes |
|---|---|---|---|
| Windows Setup | `Daarion.Edge_0.2.2-canary.2_x64-setup.exe` | Pending | |
| Windows MSI | `Daarion.Edge_0.2.2-canary.2_x64_en-US.msi` | Pending | |
| macOS Apple Silicon | `Daarion.Edge_0.2.2-canary.2_aarch64.dmg` | Pending | |
| macOS Intel | `Daarion.Edge_0.2.2-canary.2_x64.dmg` | Pending | |
| Linux AppImage | `Daarion.Edge_0.2.2-canary.2_amd64.AppImage` | Pending | |
| Android APK | `Daarion.Edge_0.2.2-canary.2_android_universal_release.apk` | Pending | |
| Manifest | `release-manifest-*.json` | Pending | |

---

## 3. Live Installer Proof Matrix

| Platform | Download | Install | Launch | Logs | Identity | Worker default | Uninstall | Verdict |
|---|---|---|---|---|---|---|---|---|
| **Windows 11 x64** | Pending | Pending | Pending | Pending | Pending | Pending | Pending | Pending |
| **macOS Apple Silicon** | Pending | Pending | Pending | Pending | Pending | Pending | Pending | Pending |
| **macOS Intel** | Pending | Pending | Pending | Pending | Pending | Pending | Pending | Pending |
| **Ubuntu Linux x64** | Pending | Pending | Pending | Pending | Pending | Pending | Pending | Pending |
| **Android arm64** | Pending | Pending | Pending | Pending | Pending | N/A | Pending | Pending |

---

## 4. Runtime Checks

- **App starts without crash**: Pending
- **Local log file created**: Pending (`boot.log` checked)
- **Local identity created or detected**: Pending (`identity.json` and keyring entry verified)
- **No private key printed in logs**: Pending
- **Device capabilities detected**: Pending (`sysinfo` diagnostics check)
- **Network state shown honestly**: Pending (heartbeat status sync checked)
- **Worker mode disabled by default**: Pending
- **Worker activation gated**: Pending (`operator_token.txt` presence and verify checks)
- **Uninstall/revoke path documented**: Pending

---

## 5. Screenshots / Evidence

*(Add screenshots of native window, system tray icon, boot.log contents, and Android sideload screen below)*

---

## 6. Known Issues

- **v0.2.2-canary.1 Initial Diagnostic Run**: Desktop build outputs (Windows NSIS/MSI, macOS DMG, Linux AppImage) were skipped because `.github/workflows/release.yml` had `includeRelease: false` configured for the `tauri-action` step. The Android arm64 build successfully completed compilation and uploaded the signed APK: `Daarion.Edge_0.2.2-canary.1_android_universal_release.apk`. Fix applied in `v0.2.2-canary.2` to set `includeRelease: true`.

---

## 7. Final Verdict

- **Release Status**: **Pending**
