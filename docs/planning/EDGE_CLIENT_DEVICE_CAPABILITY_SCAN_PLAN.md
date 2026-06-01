# DAARION Edge Client — Device Capability Scan Plan

This document outlines the architecture, resource thresholds, and user interface flow for the post-launch device capability scanner and operating mode recommendation engine.

## 1. Operating Mode Classifications

The system profiles the host system's hardware to recommend one of four operational profiles:

| Mode | Target Hardware | Memory Range (RAM) | Execution Profile |
| :--- | :--- | :--- | :--- |
| **Light Client** | Low resource or mobile device | `< 6 GB` | Operates as a lightweight node, fetching data from remote consensus / Sofiia orchestrator. No local LLM is loaded. |
| **Local Micro Agent** | Standard consumer notebook | `6 GB - 11.9 GB` | Capable of launching small, high-efficiency local models (0.4B - 0.8B parameter GGUF models) for private offline commands. |
| **Local LLM Runtime** | Power-user laptop or desktop | `≥ 12 GB` | Able to run 2B - 9B GGUF models locally with active hardware/GPU acceleration (Metal / CUDA / Vulkan). |
| **Worker Candidate** | High-performance workstation | `≥ 16 GB` (With discrete GPU) | Eligible to apply to become an advisory worker node (consensus worker). This requires gated operator approval and is **never** auto-enabled. |

---

## 2. Capability Fields Gathered (Local Only)

The Tauri Rust command `get_device_capability_profile` collects the following details:
- **System OS & Architecture**: `os` (e.g. windows, macos, linux, android), `arch` (e.g. x86_64, aarch64).
- **CPU Inventory**: Core count (`cpu_count`), CPU branding name (`cpu_brand`).
- **Memory (RAM)**: Total RAM (`ram_total_gb`), currently available free memory (`ram_available_gb`).
- **Storage**: Free space on primary disk volume (`disk_free_gb`).
- **GPU Acceleration**: GPU vendor, description, and API availability (`Metal` for macOS Silicon, `CUDA` for NVIDIA/Linux, `Vulkan` for Android/Linux, `CPU` fallback).
- **Battery Status**: Battery presence and power state (AC / discharging / charging).
- **Application Context**: App version from cargo manifest, scan timestamp, and persisted `worker_opt_in` status.

---

## 3. Recommended GGUF Models

Models mapped to device categories:
- **Smartphone / Ultra-lite**: Gemma 4 Tiny (0.4B, Q4_0, ~0.3 GB) / Qwen 3.5 (0.8B, Q4_K_M, ~0.5 GB).
- **Tablet / Lite**: Gemma 4 (2B, Q4_K_M, ~1.5 GB).
- **Laptop / Balanced**: Qwen 3.5 (2B, Q8_0, ~2.1 GB) / Gemma 4 (2B, Q4_K_M).
- **Workstation / Full**: Qwen 3.5 (9B, Q8_0, ~9.5 GB) / Gemma 4 (4B, Q8_0, ~4.5 GB).

---

## 4. UI/UX Flow & Consent Gate

- **First Launch / Dashboard Card**: A prominent dashboard section asks the user to run a device capability scan.
- **Check Device Readiness**: Initiates a scan animation, invokes the Rust command, and prints results.
- **Privacy Notice**: Explicit copy confirms the scan runs entirely local, and no models or worker loops are activated without explicit user consent.
