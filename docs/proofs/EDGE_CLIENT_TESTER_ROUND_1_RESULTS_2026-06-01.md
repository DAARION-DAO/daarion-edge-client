# DAARION Edge Client Tester Round 1 Results — v0.2.2-3

This document aggregates and summarizes the results from the **first controlled tester round** for the DAARION Edge Client `v0.2.2-3` release.

## 1. Summary
- **Release Version**: `v0.2.2-3`
- **Test Round**: Round 1 (Controlled group of 3–5 testers)
- **Tester Count**: Pending / 0 reported
- **Platforms Covered**: Windows 11, macOS Apple Silicon, macOS Intel, Ubuntu Linux, Android
- **Overall Verdict**: **Pending / In Progress**

---

## 2. Results Matrix

| Tester | Platform | Install | Launch | UI | Identity | Logs | Worker OFF | Uninstall | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| *T1* | **Windows 11 x64** | Pending | Pending | Pending | Pending | Pending | Pending | Pending | **Pending** |
| *T2* | **macOS Apple Silicon** | Pending | Pending | Pending | Pending | Pending | Pending | Pending | **Pending** |
| *T3* | **macOS Intel** | Pending | Pending | Pending | Pending | Pending | Pending | Pending | **Pending** |
| *T4* | **Ubuntu Linux x64** | Pending | Pending | Pending | Pending | Pending | Pending | Pending | **Pending** |
| *T5* | **Android arm64** | Pending | Pending | Pending | Pending | Pending | N/A | Pending | **Pending** |

---

## 3. Platform Findings

### Windows 11 (x64)
- **Artifact**: `Daarion.Edge_0.2.2-3_x64-setup.exe` / `Daarion.Edge_0.2.2-3_x64_en-US.msi`
- **Onboarding Experience**: *(Document Windows SmartScreen behavior, installation speed, and shortcut creation)*
- **Key Logs**: `%APPDATA%\DAARION Edge\logs\boot.log`
- **Identity File**: `%APPDATA%\city.daarion.edge\identity.json`

### macOS Apple Silicon (ARM64)
- **Artifact**: `Daarion.Edge_0.2.2-3_aarch64.dmg`
- **Onboarding Experience**: *(Document Gatekeeper bypass friction, window loading, and tray behavior)*
- **Key Logs**: `~/.daarion-edge/logs/boot.log`
- **Identity File**: `~/Library/Application Support/city.daarion.edge/identity.json`

### macOS Intel (x64)
- **Artifact**: `Daarion.Edge_0.2.2-3_x64.dmg`
- **Onboarding Experience**: *(Document x64 performance, Gatekeeper checks, and tray behavior)*
- **Key Logs**: `~/.daarion-edge/logs/boot.log`
- **Identity File**: `~/Library/Application Support/city.daarion.edge/identity.json`

### Ubuntu Linux (x86_64)
- **Artifact**: `Daarion.Edge_0.2.2-3_amd64.AppImage`
- **Onboarding Experience**: *(Document execution permission requirements, desktop integration issues if any, and window loading)*
- **Key Logs**: `~/.daarion-edge/logs/boot.log`
- **Identity File**: `~/.config/city.daarion.edge/identity.json`

### Android (arm64)
- **Artifact**: `Daarion.Edge_0.2.2-3_android_universal_release.apk`
- **Onboarding Experience**: *(Document sideload warning clicks, startup permissions, and welcome screen layout)*
- **Key Logs**: *(Check logcat / local storage if logged)*
- **Identity File**: N/A

---

## 4. Bugs Found

*(List any crashes, unhandled errors, log exceptions, or installation blockers encountered during the test)*
- **None reported yet**

---

## 5. UX Friction

*(List any difficulties testers encountered regarding code-signing popups, warning dialogs, UI labels, or tray behavior)*
- **None reported yet**

---

## 6. Release Decision

- `[ ]` **Stay on Canary**: More bugfixes required before widening the test pool.
- `[ ]` **Prepare v0.2.2-4**: Direct hotfixes needed for blockers found in Round 1.
- `[ ]` **Promote to v0.2.3-beta.1**: Successful onboarding verified. Open testing pool to wider audience.
