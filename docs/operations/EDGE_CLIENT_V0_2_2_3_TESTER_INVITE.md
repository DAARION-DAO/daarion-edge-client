# 🌌 Join the DAARION Edge Client Test Round (v0.2.2-3)

Hi there! You are invited to participate in the **first controlled testing round** for the DAARION Edge Client (`v0.2.2-3` Canary/Beta). 

Our goal is to verify that the core client installer package installs cleanly, launches successfully, initializes a secure local identity, and stays inactive (no background compute) by default.

---

## 🚀 Quick Start for Testers

### Step 1: Download the Client
Go to the download page and grab the installer package for your operating system:
👉 **[daarion.city/install](https://daarion.city/install)**

*(Alternatively, you can access the files directly from the [GitHub Release Page](https://github.com/DAARION-DAO/daarion-edge-client/releases/tag/v0.2.2-3))*

### Step 2: Install & Launch
- **macOS**: Drag the app to `/Applications`. Right-click the app in Finder and select **Open** to bypass the Gatekeeper developer warning.
- **Windows**: Run the `.msi` or `.exe` installer. Click **More info** -> **Run anyway** if Windows SmartScreen displays a warning.
- **Linux**: Grant execute permissions (`chmod +x <filename>.AppImage`) and run the AppImage.
- **Android**: Install the signed sideload `.apk` (requires enabling "Install from unknown sources" in settings).

### Step 3: Run the Verification Checklist
Once the application opens, check for the following:
1. **Welcome Screen**: Confirm that the main user interface loads and renders cleanly.
2. **Tray Menu**: Check if the DAARION Edge icon appears in your system tray or menu bar.
3. **Secure Identity**: Ensure that the UI displays a generated local identity (Node ID).
4. **Worker Mode is OFF**: Confirm that **Worker Mode is disabled** by default. The client should not execute any background processing without your opt-in.

### Step 4: Take a Screenshot & Collect Logs
- Capture a screenshot of the main screen and tray menu.
- Locate your boot log file:
  - **macOS/Linux**: `~/.daarion-edge/logs/boot.log`
  - **Windows**: `%APPDATA%\DAARION Edge\logs\boot.log`

### Step 5: Uninstall
- Remove the application using your system's default uninstall flow.
- Confirm that the application closed cleanly and did not leave any active background processes running.

---

## 📝 How to Report Your Results

Please reply back to this channel with your findings using the template below:

```text
Platform: [e.g., Windows 11 x64, macOS Apple Silicon, Android 14]
Downloaded from website: [Yes / No]
Install successful: [Yes / No]
App opened without crash: [Yes / No]
Onboarding screen visible: [Yes / No]
Local identity generated: [Yes / No]
Worker Mode OFF by default: [Yes / No]
Uninstall successful: [Yes / No]

Attached: [Screenshots / boot.log]
Any issues/bugs encountered: 
```

Thank you for helping us verify the DAARION Edge Client installer! For detailed technical file paths and troubleshooting instructions, please consult the [Manual Tester Pack](EDGE_CLIENT_V0_2_2_3_MANUAL_TESTER_PACK.md).
