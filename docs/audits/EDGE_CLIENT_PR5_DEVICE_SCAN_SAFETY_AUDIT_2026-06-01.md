# Safety Audit & Code Review — PR 5 Device Capability Scan

**Audit Date**: 2026-06-01  
**Project**: `daarion-edge-client`  
**Branch**: `feat/device-capability-scan-recommendation`  
**Status**: **PASSED (GO FOR MERGE)**

---

## 1. Files Reviewed
- **Rust Backend**:
  - [capabilities.rs](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src-tauri/src/capabilities.rs) (New struct and `get_device_capability_profile` command)
  - [lib.rs](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src-tauri/src/lib.rs) (Tauri command registration)
- **Frontend TSX**:
  - [App.tsx](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/src/App.tsx) (Scan button, state management, and profile display layout)
- **Docs/Planning**:
  - [EDGE_CLIENT_DEVICE_CAPABILITY_SCAN_PLAN.md](file:///Users/apple/github-projects/microdao-daarion/daarion-edge-client/docs/planning/EDGE_CLIENT_DEVICE_CAPABILITY_SCAN_PLAN.md) (Design spec)

---

## 2. Safety Findings & Checklist

| # | Check Item | Status | Finding / Evidence |
| :-: | :--- | :---: | :--- |
| **1** | Local-Only Operation | **CONFIRMED** | `get_device_capability_profile` uses only `sysinfo::System` and `sysinfo::Disks` to query local hardware specifications. |
| **2** | No Network Calls | **CONFIRMED** | No networking imports (`reqwest`, sockets, or HTTP request logic) are present or invoked in the capability scan functions. |
| **3** | Read-Only Profile (No write to disk) | **CONFIRMED** | No `std::fs::write` or write streams are used; the hardware data resides purely in memory and is passed back as serializable JSON payload to the Webview. |
| **4** | Safe Worker Opt-in Read | **CONFIRMED** | It queries `worker::load_worker_optin(&app)` to display worker status. It has no capabilities to mutate or toggle the opted_in status on disk. |
| **5** | Safe UI Failure Handling | **CONFIRMED** | Command returns a `Result<DeviceCapabilityProfile, String>`. In `App.tsx`, `runDeviceScan` wraps the invoke in `try...catch` and displays scan errors gracefully in the UI without crashing the application. |
| **6** | Fail-Safe Default Fields | **CONFIRMED** | Missing hostnames or CPU brands fall back to `"sovereign-node"` and `"Unknown"`. Disk list empty state sums to `0.0` rather than causing an out-of-bounds panic. |
| **7** | Model Recommendation Labels | **CONFIRMED** | The UI explicitly states *"Model Recommendation"*, separating recommendations from installed runtimes. |
| **8** | Installation Safeguard Copy | **CONFIRMED** | The card displays: *"Local Model Installation: Coming Soon / Requires Confirmation"* with a warning state. |
| **9** | Default Worker OFF Guard | **CONFIRMED** | Worker Mode status is displayed as `Disabled (OFF)` by default (loaded from the persisted config) and is not altered by scanning. |
| **10** | No Auto-Downloads / Fetches | **CONFIRMED** | There are zero network hooks or triggering commands for fetching Ollama dependencies or GGUF assets. |

---

## 3. Build & Compilation Results
- **Rust Backend (`cargo check`)**: **SUCCESS** with 0 errors.
- **Frontend App (`npm run build`)**: **SUCCESS** with 0 errors.

---

## 4. Privacy & Worker Boundary Verification
- The privacy disclaimer copy is prominent:
  > *Device scan runs locally. We do not enable Worker Mode, transfer system specs, or download any LLM model automatically.*
- Clicking the "Apply for Worker Node" button only acts as a tab navigation trigger (`setActiveTab("activation")`) pointing to the existing gated activation form, preserving the operator-checked workflow.

---

## 5. Recommendation
The code is verified as safe, performant, and conforms to all strict privacy guidelines and default-off worker bounds. **Recommended to merge into `main` after this audit review.**
