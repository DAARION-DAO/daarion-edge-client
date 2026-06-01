# EDGE CLIENT TESTER CHECKLIST

This checklist is for Canary and Beta testers validating the **DAARION Edge Client** release candidate builds on their local hardware.

---

## 📋 Pre-Flight Requirements

Before beginning tests, ensure your local hardware environment matches:
* **macOS**: Catalina (10.15) or newer. Apple Silicon (M1/M2/M3) or Intel Core processor.
* **Windows**: Windows 10 (Build 19041+) or Windows 11. 
* **Linux**: Ubuntu 22.04 LTS (or compatible Debian derivative) with desktop environment.
* **Android**: Android 10 or newer (ARM64 device). Sideloading permissions enabled for your browser/file explorer.
* **Docker Context**: Docker or Colima installed and running if you plan to verify L3 Worker Node Mode.

---

## 🧪 Phase 1: Installation & Sideloading

Verify that the installer finishes the package delivery successfully.

- [ ] **Windows Installer**: Double-click `Daarion.Edge_<version>_x64-setup.exe` or run MSI alternative. Check that the install completes without corrupt CAB file errors.
- [ ] **macOS DMG**: Open the DMG file, drag the application icon to the `/Applications` folder, and ensure the copy operation completes successfully.
- [ ] **Linux AppImage**: Apply execute permissions (`chmod +x <file>.AppImage`) and launch the file.
- [ ] **Android APK Sideload**: Download `Daarion.Edge_<version>_android_universal_release.apk` directly on the device. Open and accept the sideload warning. Verify that the system does not show an "App not installed: Package appears to be invalid or corrupted" popup.

---

## ⚡ Phase 2: App Boot Verification

Verify that the application launches, initializes its WebView runtime, and establishes the local database.

- [ ] **Launch**: Open the application. Check that the initializing spinner appears and fades away.
- [ ] **WebView Load**: Verify that the main UI renders (i.e. no blank white or blank black screen).
- [ ] **Longevity Check**: Leave the application open for at least 60 seconds. Verify that the process does not terminate unexpectedly.
- [ ] **Log Creation**:
  - Open file explorer/terminal and check if `boot.log` was successfully created.
  - **Windows Location**: `%APPDATA%\DAARION Edge\logs\boot.log`
  - **macOS / Linux Location**: `~/.daarion-edge/logs/boot.log`
  - Open `boot.log` and confirm that the final lines show `setup() completed successfully` and `Running Tauri application...`.

---

## 🔑 Phase 3: L1 Identity & Genesis Onboarding

Verify that the cryptographic setup and initial wallet mapping functions work.

- [ ] **Identity status**: Upon first launch (clean state), verify that the app displays the **Sovereign Genesis Wizard**.
- [ ] **Key Generation**: Complete the identity steps. Ensure the secure storage (keyring) generation completes.
- [ ] **Plaintext JSON Verification**: Verify that the metadata file `identity.json` is created in the application's configuration directory.
- [ ] **Registry Onboarding**: Put in a valid invite token. Verify that the request is sent, registering the public key, EVM address, Solana address, and hardware capabilities with the SOFIIA registry.
- [ ] **Pending Approval Screen**: Verify that after registration, the app redirects to the **Pending Approval** screen showing your Node ID and "Heartbeat Active" status.

---

## ⚙️ Phase 4: L3 Worker Activation (Optional / Gated)

Verify the environment sandbox setup for compute tasks.

- [ ] **Operator Whitelist**: Place your eligibility token file `operator_token.txt` at `{HOME}/.daarion/operator_token.txt`.
- [ ] **Environment Probe**: Click "Verify Environment" inside the Worker Mode setup panel. Ensure the app successfully detects Colima (macOS) or WSL2 (Windows).
- [ ] **WSL2 Windows Auto-Install** *(Windows Only)*: If WSL2 is missing, click "Enable WSL2". Confirm that an elevated PowerShell prompt appears asking for UAC authorization.
- [ ] **Tunnel Connection**: Verify that the TCP probe succeeds, attachments are active, and the status changes to "Worker Active" or "Worker Opted In".
