# EDGE CLIENT RELEASE CANARY AUDIT — 2026-05-01

## Executive Verdict: RED

Both Windows and Android release artifacts are **non-functional** in live testing.
Neither artifact should be described as "validated" or "ready" until the proof block passes.

---

## 1. Release Workflow Analysis (`release.yml`)

### Findings

| Area | Status | Detail |
|------|--------|--------|
| Trigger | OK | Runs on `v*` tags and `workflow_dispatch`. |
| Desktop build (macOS/Windows/Linux) | PARTIAL | Uses `tauri-apps/tauri-action@v0`. Builds and uploads artifacts, but **no install/boot smoke test.** |
| Android build | BROKEN | Runs `cargo tauri android init` to generate android project freshly in CI, then `cargo tauri android build --apk`. **No signing config** is provided. The APK output path assumes `universal/release/*.apk` — but without a keystore, the APK may be unsigned or debug-signed, making it **invalid for sideloading** on real devices. |
| Android installability test | MISSING | No `adb install` or emulator test exists. |
| Windows boot smoke test | MISSING | No post-build validation that the `.exe` launches successfully. |
| Release notes | MISLEADING | Release body lists macOS/Windows/Linux/Android as if all are validated. No distinction between "builds" and "installs and runs". |

### Root Cause: Android APK "Invalid/Corrupted"

1. **No release signing keystore.** `cargo tauri android build --apk` without a signing config produces a debug-signed APK. Many Android 14+ devices reject installation of unsigned or debug-signed release APKs from unknown sources.
2. **The `gen/android` directory is not committed** — `src-tauri/gen/` only contains `schemas/`. The CI runs `cargo tauri android init` to create it fresh each build. This is fragile and may produce inconsistent `AndroidManifest.xml` or build.gradle configurations.
3. **No APK verification step.** The APK is uploaded directly to the GitHub Release without any installability check.

### Root Cause: Windows "Blank Window Then Closes"

1. **`windows_subsystem = "windows"` hides all console output** in release builds (line 2 of `main.rs`). If any panic or early error occurs, the user sees nothing.
2. **Startup path has multiple crash vectors:**
   - `identity.rs:32` — `get_app_dir()` calls `.expect("Failed to get app data dir")` — **panic on failure**.
   - `lib.rs:107` — `TrayIconBuilder::new().build(app)` — the return value is silently discarded. On Windows, if the icon file is missing or malformed, this may silently fail, but the real problem is likely elsewhere.
   - `lib.rs:113` — `worker::load_worker_optin(&handle)` — safe (returns false on error).
   - `lib.rs:124` — `crate::config::resolve_relay_endpoint()` — safe (returns None).
   - `lib.rs:142` — `.expect("error while running tauri application")` — **panic on Tauri init failure, invisible in release**.
3. **`keyring` crate behavior on Windows:** The `keyring` crate on Windows uses Windows Credential Manager. On first launch with no prior identity, `load_or_create_identity` will attempt to create a new keyring entry. This usually works but may fail on restrictive corporate machines. However, this is NOT called on startup unless worker mode was previously opted in.
4. **`cpal` crate (audio):** Listed as a dependency in `Cargo.toml`. CPAL initializes system audio interfaces at link time on Windows. If the Windows audio subsystem is unavailable or the WASAPI COM initialization fails, this can cause startup panics. **This is the strongest suspected crash vector**, especially on CI-built artifacts that may lack audio driver context.
5. **WebView2 runtime:** Tauri v2 on Windows requires WebView2. If the target Windows machine doesn't have it, the app will show a blank window and close. The `tauri-action` builder should bundle WebView2 bootstrapper, but this needs verification.

---

## 2. Tauri Configuration Analysis (`tauri.conf.json`)

| Field | Value | Issue |
|-------|-------|-------|
| `productName` | `"Daarion Edge"` | OK |
| `version` | `"0.2.0"` | Matches `package.json` and `Cargo.toml`. **No drift.** |
| `identifier` | `"city.daarion.edge"` | OK for desktop. Android requires this to be a valid Java package identifier. The dots are fine. |
| `bundle.targets` | `"all"` | Builds all available targets for the platform. On Windows: MSI + NSIS. |
| `bundle.icon` | Standard icon set | Includes `.ico` for Windows and `.icns` for macOS. Assumes files exist. |
| `app.security.csp` | `null` | Permissive — OK for dev/pilot, not production. |
| Mobile config | **MISSING** | No `tauri.conf.json` section for Android-specific configuration (signing, ABI, etc.). |

---

## 3. Rust/Tauri Startup Path Analysis

### Panic Points (invisible in release)

| Location | Code | Risk |
|----------|------|------|
| `main.rs:2` | `windows_subsystem = "windows"` | Hides ALL stdout/stderr in release. Any panic = silent death. |
| `lib.rs:142` | `.expect("error while running tauri application")` | If Tauri builder fails (e.g., plugin init, WebView2 missing), this panics silently. |
| `identity.rs:32` | `.expect("Failed to get app data dir")` | Panic if `app_data_dir()` returns `None`. |

### Worker Mode Startup Safety (lib.rs:111-138)

**This section is already hardened.** If the user previously opted in:
- It checks `resolve_relay_endpoint()` first. If `None`, it logs and defers — no crash.
- If relay exists, `toggle_worker_mode` is spawned in a `tokio::spawn` — failures are contained.
- The blocking_lock on line 118 is in setup(), which runs on the main thread. This is generally safe in Tauri setup but could theoretically deadlock if combined with other async state access.

**Verdict on worker startup safety: SAFE.** The worker path defers gracefully when relay is absent.

### Heartbeat Loop (heartbeat.rs)

Starts unconditionally in `setup()`. It:
1. Polls `load_enrollment_state()` — reads from disk, safe fallback.
2. Uses `resolve_backend_url()` which defaults to `localhost:8010` — will fail silently on production machines with no backend.
3. Consecutive failure counter caps at 20 warnings — never panics.

**Verdict: SAFE.** The heartbeat loop is resilient.

---

## 4. Frontend Boot Path Analysis

| Component | Status |
|-----------|--------|
| `index.html` | Minimal — `<div id="root">` + `<script src="/src/main.tsx">`. OK. |
| `main.tsx` | Simple path router. No async deps, no API calls on mount. OK. |
| `App.tsx` | 28KB. Loads many components. Could fail if a Tauri invoke crashes before React mounts, but React should render independently. |
| Vite build | **Successful.** `npm run build` completes with clean output. Frontend dist is 315KB JS + 84KB CSS. |
| PWA service worker | Built by `vite-plugin-pwa`. In Tauri desktop context, service worker registration will be skipped (not applicable to `tauri://` protocol). Should be harmless. |

**Verdict: Frontend is NOT the crash source.** The blank white window likely shows the WebView initializing (HTML loads) but Rust panics before the IPC bridge is established, causing the window to close when the process dies.

---

## 5. Android Generated Project State

| Item | Status |
|------|--------|
| `src-tauri/gen/android/` | **DOES NOT EXIST** locally. Only `gen/schemas/` is committed. |
| CI behavior | `cargo tauri android init` generates the project fresh each build. |
| Signing config | **NONE.** No keystore, no `key.properties`, no signing block in CI env. |
| ABI targets | CI specifies `--target aarch64-linux-android` only. No x86/arm32. |
| Package identifier | Inherits `city.daarion.edge` from `tauri.conf.json`. |
| APK upload path | `src-tauri/gen/android/app/build/outputs/apk/universal/release/*.apk` — assumes universal APK. |

**Verdict: Android APK is broken because:**
1. No release keystore → APK is debug-signed or unsigned.
2. Android 14+ rejects installation of unsigned release APKs.
3. No CI validation of installability.

---

## 6. Suspected Root Causes — Ranked

### Windows (Blank Window → Close)

| # | Suspect | Confidence | Evidence |
|---|---------|------------|----------|
| 1 | **Missing WebView2 runtime** | HIGH | Tauri v2 requires Edge WebView2. If not present, app shows empty frame and exits. |
| 2 | **Silent panic from `.expect()` calls** | MEDIUM | `lib.rs:142` catches Tauri builder errors with panic. `windows_subsystem = "windows"` hides the message. |
| 3 | **`cpal` audio initialization panic** | MEDIUM | CPAL links against WASAPI. If COM init fails or no audio device is present, it can panic at runtime. |
| 4 | **`keyring` crate failure** | LOW | Only triggered if worker was previously opted in AND identity exists. Unlikely on fresh install. |

### Android (Invalid/Corrupted APK)

| # | Suspect | Confidence | Evidence |
|---|---------|------------|----------|
| 1 | **No release signing** | VERY HIGH | No keystore in CI. Debug-signed APKs are rejected by modern Android. |
| 2 | **ABI mismatch** | LOW | Build targets `aarch64`. Should match most modern phones. |

---

## 7. Files Inspected

- `.github/workflows/release.yml` — release pipeline
- `src-tauri/tauri.conf.json` — Tauri config
- `src-tauri/Cargo.toml` — Rust dependencies
- `src-tauri/build.rs` — build script
- `src-tauri/src/main.rs` — binary entry point
- `src-tauri/src/lib.rs` — Tauri app builder + setup
- `src-tauri/src/config.rs` — configuration resolution
- `src-tauri/src/identity.rs` — node identity / keyring
- `src-tauri/src/heartbeat.rs` — heartbeat loop
- `src-tauri/src/worker/mod.rs` — worker mode state + toggle
- `src-tauri/capabilities/default.json` — Tauri permissions
- `src/main.tsx` — React entry point
- `src/App.tsx` — main app shell
- `vite.config.ts` — Vite build config
- `package.json` — frontend dependencies
- `index.html` — HTML shell
- `src-tauri/gen/` — only `schemas/` exists; no android project committed

---

## 8. Minimal Patch Plan

### A. Windows Boot Logging (durable, survives `windows_subsystem = "windows"`)
Add file-based boot logging to `%APPDATA%/DAARION Edge/logs/boot.log`. Log every stage: startup, plugin init, tray setup, heartbeat start, worker opt-in load, relay resolution, window creation. Replace `.expect()` on line 142 with a logged error dialog.

### B. Windows Canary Visibility
If Tauri builder fails, show a native error dialog on Windows (via `msgbox` or `rfd`). Do NOT let the app die silently.

### C. Worker Startup Safety
Already safe per audit. No changes needed beyond adding boot logging.

### D. Android Installability Gate
Document the exact signing requirements. Add a placeholder CI step for future signing. Mark Android as "Build Only — Not Validated" in release notes.

### E. Release Workflow Truth
Add release checklist. Do not describe artifacts as validated without proof.

### F. Documentation
Create runbook with tester instructions for Windows boot.log recovery and Android APK testing.

---

## 9. Proven vs Suspected

| Item | Status |
|------|--------|
| Frontend builds successfully | **PROVEN** (npm run build succeeds) |
| Rust compiles on macOS | **PROVEN** (cargo check passes) |
| Windows release hides all errors | **PROVEN** (`windows_subsystem = "windows"`) |
| Android APK has no signing | **PROVEN** (no keystore in CI or repo) |
| WebView2 missing on test machine | **SUSPECTED** (cannot verify from macOS) |
| `cpal` audio panic on Windows | **SUSPECTED** (requires Windows runtime test) |
| `.expect()` panic on builder init | **SUSPECTED** (requires boot.log to diagnose) |
