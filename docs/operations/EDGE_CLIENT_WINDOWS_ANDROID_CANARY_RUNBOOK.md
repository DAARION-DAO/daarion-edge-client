# DAARION Edge Client — Windows & Android Canary Runbook

## Purpose
This runbook provides step-by-step instructions for testers to validate Windows and Android release artifacts.

---

## Windows Testing Procedure

### Prerequisites
- Windows 10 or Windows 11 machine
- Internet connection for WebView2 download (first run)

### Steps

1. **Download** the `*-setup.exe` or `*.msi` from the GitHub Release page.
2. **Install** by double-clicking the installer and following the wizard.
3. **Launch** the application from the Start Menu or Desktop shortcut.

### Expected Behavior
- The DAARION Edge window should open and display the main UI.
- The system tray should show a DAARION Edge icon.
- The app should remain alive for at least 60 seconds without closing.

### If the App Closes Immediately (Blank White Window)
1. Navigate to:
   ```
   %APPDATA%\DAARION Edge\logs\boot.log
   ```
   (In Explorer, paste this path into the address bar)
2. Open `boot.log` in Notepad.
3. **Take a screenshot** of the log contents.
4. Note the **last line** — it indicates where the startup failed.
5. Send the screenshot and last line to the development team.

### What to Report
| Field | Value |
|-------|-------|
| Windows version | (e.g., Windows 11 23H2) |
| Architecture | (e.g., x64, ARM64) |
| Installer file used | (exact filename) |
| Did app window appear? | Yes / No |
| Did app stay open 60s? | Yes / No |
| boot.log last line | (paste here) |
| boot.log full content | (attach file) |
| Task Manager: process visible? | Yes / No |
| System tray icon visible? | Yes / No |

### How to Confirm Success
The app is **Green** only if:
- [x] App installs without error
- [x] App window opens and renders UI
- [x] App remains alive for 60+ seconds
- [x] boot.log shows "setup() completed successfully"
- [x] boot.log shows "Running Tauri application..."
- [x] No crash or blank screen

---

## Android Testing Procedure

### Prerequisites
- Android phone (ARM64, Android 10+)
- "Install Unknown Apps" enabled for your browser or file manager
- (Optional) ADB access for detailed testing

### Steps

1. **Download** the `*.apk` from the GitHub Release page directly on the phone.
2. **Open** the downloaded APK file.
3. **Accept** the "Install Unknown Apps" permission if prompted.
4. **Install** the APK.

### Known Issue: "Application Not Installed"
The current APK may fail installation with:
> "Application not installed because its package appears to be invalid / corrupted."

This is expected — the APK is not properly signed for release distribution.

### What to Report
| Field | Value |
|-------|-------|
| Android version | (e.g., Android 14) |
| Device model | (e.g., Samsung Galaxy S24) |
| Architecture | (e.g., arm64-v8a) |
| APK filename | (exact name) |
| Installation result | Success / "Invalid/Corrupted" / Other error |
| If installed, does app open? | Yes / No |
| If opened, does UI render? | Yes / No |

### ADB Testing (Optional, for developers)
```bash
# Install APK
adb install -r path/to/app-universal-release.apk

# Launch app
adb shell monkey -p city.daarion.edge 1

# Check if process is running
adb shell pidof city.daarion.edge

# View crash logs
adb logcat -s "CrashAnrDetector" | head -50

# Uninstall
adb uninstall city.daarion.edge
```

---

## Release Validation Ladder

```
build exists       ≠  install proof
install proof      ≠  boot proof
boot proof         ≠  worker proof
worker proof       ≠  lease/receipt proof
```

A platform is **Green** only after all levels pass.
Current status:
- **macOS:** Boot proof confirmed (development environment)
- **Windows:** RED — blank window closes immediately
- **Android:** RED — APK rejected at installation
- **Linux:** Untested
