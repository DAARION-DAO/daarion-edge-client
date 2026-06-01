# EDGE CLIENT RELEASE TRUTH

This document describes the canonical release artifact definitions, build output schemas, and platform validation rules for the **DAARION Edge Client**.

---

## 1. Canonical Artifact Names

To maintain compatibility with public download links, installation wizards, and automated scripts (such as `/install` on `DAARION.city`), the GitHub Actions release workflow (`release.yml`) must build and release artifacts using the following canonical naming patterns:

| Platform | Target Architecture | Canonical Filename Pattern | Build Utility |
|---|---|---|---|
| **Windows** | x86_64 | `Daarion.Edge_<version>_x64-setup.exe` <br> `Daarion.Edge_<version>_x64_en-US.msi` | Tauri NSIS / WiX |
| **macOS Apple Silicon** | ARM64 (`aarch64`) | `Daarion.Edge_<version>_aarch64.dmg` | Tauri Bundle DMG |
| **macOS Intel** | x86_64 | `Daarion.Edge_<version>_x64.dmg` | Tauri Bundle DMG |
| **Linux** | x86_64 | `Daarion.Edge_<version>_amd64.AppImage` | Tauri Bundle AppImage |
| **Android** | arm64-v8a | `Daarion.Edge_<version>_android_universal_release.apk` | Gradle Assembler + CI Stager |

> **⚠️ NOTE ON ANDROID**: The APK is initially output by the Gradle build system as `app-universal-release.apk` (signed using the Google Play Keystore Base64 secret), and is subsequently renamed by the CI script `scripts/prepare-release-artifacts.js` to match the canonical versioned pattern before upload.

---

## 2. Release Validation Ladder

An artifact is only marked **Green (Verified)** if it completes all verification gates. Under no circumstances should an artifact be advertised as "production-ready" without passing the complete chain:

```
[ Build Success ] ──> [ Sideload / Install Proof ] ──> [ Process Boot Proof ] ──> [ Network/Worker Proof ]
```

1. **Build Success**: The compiler finishes and exports the binary output without syntax or link errors.
2. **Install Proof**: The package compiles and installs cleanly on a standard clean operating system without triggering malware flags or corrupt format blocks.
3. **Process Boot Proof**: The application launches, initializes the web view wrapper, starts the heartbeat loop, and remains active for at least 60 seconds without unexpected panics or process death.
4. **Network/Worker Proof**: The client successfully executes the Ed25519 signature handshake, passes Operator Gate 1/2 verification, connects to the WebSocket mesh, and runs sandboxed advisory tasks natively.

---

## 3. Current Release Validation Status

### Windows (Yellow)
- **Status**: NSIS Setup executable and MSI compile successfully.
- **Diagnostics**: Hardened with native boot logging to `%APPDATA%/DAARION Edge/logs/boot.log`.
- **Blockers**: In-app execution durability on non-developer environments is still undergoing tester verification.

### macOS Apple Silicon & Intel (Green)
- **Status**: DMGs are built, signed, and verified in active development environments.
- **Diagnostics**: Writes to `~/.daarion-edge/logs/boot.log`.
- **Blockers**: None. Fully validated for the current pilot Wave.

### Linux (Yellow)
- **Status**: AppImage builds successfully under Ubuntu GitHub runners.
- **Diagnostics**: Writes to `~/.daarion-edge/logs/boot.log`.
- **Blockers**: Sideload and Boot testing are currently unverified.

### Android (Yellow)
- **Status**: Release-signed APK compiles successfully under NDK toolchains in CI.
- **Diagnostics**: Monitored via standard `adb logcat -s "CrashAnrDetector"`.
- **Blockers**: Awaiting test report validation (`adb install` verification on pure Android 14+ targets).

---

## 4. Update Policy

* **No Auto-Updates**: The Tauri auto-update plugin is omitted from `tauri.conf.json` and Rust dependencies.
* **Manual Upgrade Model**: All client updates are strictly manual. Users must navigate to the GitHub Releases page, download the updated artifact, and reinstall it to overwrite existing application folders.
