# DAARION Edge Client Tester Pack — v0.2.2-3

Welcome to the **DAARION Edge Client Live Installer Verification Phase** for version `v0.2.2-3` (Canary/Beta). 

This tester pack provides all details, links, commands, and expectations required for manual test coordination.

---

## 1. Download & Release Metadata

- **Release Page**: [DAARION Edge Client v0.2.2-3 Releases](https://github.com/DAARION-DAO/daarion-edge-client/releases/tag/v0.2.2-3)
- **Manifest / Updates endpoint**: Integrated on [DAARION.city/install](https://daarion.city/install)
- **Direct Download Links**:
  - **Windows NSIS Setup**: [Daarion.Edge_0.2.2-3_x64-setup.exe](https://github.com/DAARION-DAO/daarion-edge-client/releases/download/v0.2.2-3/Daarion.Edge_0.2.2-3_x64-setup.exe)
  - **Windows MSI Installer**: [Daarion.Edge_0.2.2-3_x64_en-US.msi](https://github.com/DAARION-DAO/daarion-edge-client/releases/download/v0.2.2-3/Daarion.Edge_0.2.2-3_x64_en-US.msi)
  - **macOS Apple Silicon (ARM)**: [Daarion.Edge_0.2.2-3_aarch64.dmg](https://github.com/DAARION-DAO/daarion-edge-client/releases/download/v0.2.2-3/Daarion.Edge_0.2.2-3_aarch64.dmg)
  - **macOS Intel (x64)**: [Daarion.Edge_0.2.2-3_x64.dmg](https://github.com/DAARION-DAO/daarion-edge-client/releases/download/v0.2.2-3/Daarion.Edge_0.2.2-3_x64.dmg)
  - **Linux AppImage**: [Daarion.Edge_0.2.2-3_amd64.AppImage](https://github.com/DAARION-DAO/daarion-edge-client/releases/download/v0.2.2-3/Daarion.Edge_0.2.2-3_amd64.AppImage)
  - **Android APK**: [Daarion.Edge_0.2.2-3_android_universal_release.apk](https://github.com/DAARION-DAO/daarion-edge-client/releases/download/v0.2.2-3/Daarion.Edge_0.2.2-3_android_universal_release.apk)

---

## 2. Platform-Specific Installation Steps

### A. macOS (Apple Silicon & Intel)
1. Double-click the downloaded `.dmg` file to mount it.
2. Drag `Daarion Edge.app` to your `/Applications` directory.
3. Due to ad-hoc code-signing:
   - Double-clicking the app in Finder will trigger a security popup: *“Daarion Edge” cannot be opened because the developer cannot be verified.*
   - **Bypass**: Right-click `Daarion Edge.app` in Finder, click **Open**, and click **Open** in the confirmation dialog.

### B. Windows 11 / 10
1. Download the `.msi` or `.exe` installer.
2. Run the installer. Since the package is ad-hoc signed:
   - Windows SmartScreen may show: *Windows protected your PC*.
   - **Bypass**: Click **More info**, then click **Run anyway**.
3. Follow the installation wizard steps to complete the setup.

### C. Linux (Ubuntu / Debian / Fedora)
1. Download the `.AppImage` file.
2. Set execution permissions via terminal:
   ```bash
   chmod +x Daarion.Edge_0.2.2-3_amd64.AppImage
   ```
3. Run the AppImage:
   ```bash
   ./Daarion.Edge_0.2.2-3_amd64.AppImage
   ```

### D. Android (sideload APK)
1. Download the `.apk` file directly to your Android device or emulator.
2. Open the file to start installation.
3. If prompted, enable **“Install from unknown sources”** in your system settings.
4. Follow the installer instructions to complete sideloading.

---

## 3. What to Verify & Evidence Checklist

To prove a successful live installation, please collect the following evidence for each platform tested:

### 1. Visual Proof (Screenshots)
- **First Visible Screen**: Capture the onboarding/welcome screen when the app opens.
- **System Tray Icon**: Screenshot the system tray / menu bar icon showing that the DAARION Edge menu items are registered.
- **Onboarding Interface**: Verify if identity status is displayed as uninitialized (or ready to generate).

### 2. Log File Collection (`boot.log`)
Inspect and submit the contents of `boot.log`. Ensure it does not contain private keys or passwords.
- **macOS Path**: `~/.daarion-edge/logs/boot.log`
- **Linux Path**: `~/.daarion-edge/logs/boot.log`
- **Windows Path**: `%APPDATA%\DAARION Edge\logs\boot.log` (copy-paste in File Explorer)

### 3. Local Identity Creation (`identity.json`)
Verify that launching the UI successfully triggers the local identity generation.
- **macOS Path**: `~/Library/Application Support/city.daarion.edge/identity.json`
- **Windows Path**: `%APPDATA%\city.daarion.edge\identity.json`
- **Linux Path**: `~/.config/city.daarion.edge/identity.json`
- **Verification Command**:
  Verify the file is a JSON object containing a `node_id` (UUID format) and a `public_key` (base64 string). Do not share the private key (which is stored safely inside the OS secure storage keyring).

### 4. Worker Mode Default Check
Confirm that **Worker Mode is OFF** by default.
- In `boot.log`, verify that you see:
  `Worker opt-in loaded: false`
- Confirm that the UI shows the Worker Mode toggle button as inactive (disabled).
- Confirm that no background processing loops are active without explicit operator activation.

### 5. Uninstall Verification
- Verify that uninstalling the application via the standard system paths (dragging app to Trash on Mac, Settings -> Installed Apps -> Uninstall on Windows, removing AppImage file on Linux) successfully stops background processes.
- Ensure you can cleanly delete the configuration caches:
  - macOS/Linux: `rm -rf ~/.daarion-edge`
  - Windows: Remove `%APPDATA%\DAARION Edge` and `%APPDATA%\city.daarion.edge`.

---

## 4. Known Limitations & Scope Boundaries

- **Canary/Beta Gate**: This release is designed for manual testing and early staging validation. It is not labeled or intended for production rollout.
- **Manual Updates**: Automatic updates are disabled for this Canary phase. Retroactive bumps require manual download of newer binaries.
- **Android Sideloading**: Sideloading requires developer mode / permission adjustments on the test device. Sideload runs are restricted to personal sandbox use.
- **Worker Mode Restrictions**: Activation of Worker Mode requires operator approval, system enrollment validation, and admin gating. Simply toggling it on without backend registration will defer runtime activities.

---

## 5. How to Report Test Results

Post your verification findings back to the engineering channel under the following format:
```text
Platform: [macOS Apple Silicon / Windows 11 x64 / etc.]
Install Status: [PASS / FAIL]
UI Opens Status: [PASS / FAIL]
Identity Created: [YES / NO]
Worker Default OFF: [YES / NO]
Uninstall Status: [PASS / FAIL]
Boot Logs attached: [Yes / No]
Screenshots attached: [Yes / No]
Issues/Crashes: [Detail any errors or unexpected behavior]
```
