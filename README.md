# DAARION Sovereign Agent Edge Client

[![Release](https://img.shields.io/github/v/release/DAARION-DAO/daarion-edge-client?color=blue&style=flat-square)](https://github.com/DAARION-DAO/daarion-edge-client/releases)
[![License](https://img.shields.io/github/license/DAARION-DAO/daarion-edge-client?style=flat-square)](LICENSE)

The **DAARION Edge Client** is the public desktop and mobile gateway into the **DAARION.city / DAGI** (Decentralized Agentic Governance Infrastructure) ecosystem. It serves as a sovereign interface and edge-client runtime that allows users to birth, manage, and coordinate personal AI agents locally on their own hardware.

---

## 🌌 Core Architecture Levels

The Edge Client operates across three distinct functional layers:

### 🛡️ L1 — Client Device
* **Sovereign Entry**: Users install the client app natively on their local hardware.
* **Identity Generation**: Automatically generates a secure, localized cryptographic Ed25519 identity keypair upon initialization. The private key is strictly isolated in the operating system's secure storage (macOS Keychain, Windows Credential Manager) via the native `keyring` API.
* **Basic Synchronization**: Connects to the DAARION network to report basic telemetry and sync capability configurations.

### 🤖 L2 — Personal Agent & Local Runtime
* **Sovereign Agent Onboarding**: Provides an interactive workspace (via the Genesis Wizard) for personal agent creation, wallet management, and localized prompt directives.
* **Local Compute Integration**: Automatically detects and catalogs local compute capacity, identifying available hardware resources (CPU cores, RAM total, and GPU acceleration APIs like Apple Metal or CUDA).
* **Model Inference Shell**: Downloads, manages, and executes approved local LLMs (Gemma, Qwen) in GGUF format natively on the user's device.

### ⚙️ L3 — Worker Node (Gated)
* **Compute Contribution**: Enables the option to lease local compute resources to the network for processing deterministic edge tasks (`ping_math` and `text_hash`).
* **Hard Sandboxing**: All edge tasks are executed strictly inside a closed-envelope container (Docker/Colima on macOS, WSL2 on Windows) with zero network egress (`--network none`) and cleared environment variables.
* **Access Gating**: Active worker participation is **disabled by default** until cryptographic operator-token validation is implemented. A local `~/.daarion/operator_token.txt` file is treated only as a legacy marker, not as a security boundary.

---

## 📱 Supported Platforms & Releases

The client is compiled for cross-platform availability, leveraging Tauri v2:

| Platform | Target Architecture | Release Format | Release Status |
|---|---|---|---|
| **Windows** | x86_64 | `.exe` (NSIS) / `.msi` | ⚠️ Beta / Canary (Execution Proof Pending) |
| **macOS Apple Silicon** | ARM64 (`aarch64`) | `.dmg` | ✅ Beta / Canary (Active) |
| **macOS Intel** | x86_64 | `.dmg` | ✅ Beta / Canary (Active) |
| **Linux** | x86_64 | `.AppImage` | ⚠️ Beta / Canary (Untested) |
| **Android** | arm64-v8a | `.apk` (Signed) | ⚠️ Release-Signed Sideload (Install Proof Pending) |
| **iOS** | arm64 | Native App | ❌ Future / Coming Soon |

---

## 📢 Public Release & Canary Status

* **Current Status**: **Beta / Canary**
* **Update Model**: **Manual updates only**. Automatic update pipelines are disabled; users must manually download latest releases from [GitHub Releases](https://github.com/DAARION-DAO/daarion-edge-client/releases).
* **Production Readiness**: Requires live proof of performance, stability, and security on a per-platform basis.
* **Security & Sandboxing**: Desktop Worker Mode is blocked until cryptographic operator-token validation is available; sandbox execution remains advisory-only test infrastructure.

---

## 🛠️ Developer Setup & Dev-Run

Ensure you have Rust stable and Node.js v20+ installed.

### 1. Install Dependencies
```bash
npm install
```

### 2. Run in Development Mode (Vite + Tauri)
```bash
npm run dev
# In another terminal to launch the Tauri window:
npm run tauri dev
```

### 3. Build Release Packages
```bash
# Build desktop packages (DMG, MSI, AppImage depending on Host OS)
npm run build
npm run tauri build
```

---

## 🗂️ Diagnostic Logs & Telemetry
If the application crashes, closes unexpectedly, or exhibits a blank screen on startup, collect and submit the diagnostic log file:
* **Windows**: `%APPDATA%\DAARION Edge\logs\boot.log`
* **macOS / Linux**: `~/.daarion-edge/logs/boot.log`
