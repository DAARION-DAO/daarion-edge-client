---
name: daarion-rust-tauri
description: "Use for DAARION Rust or Tauri v2 commands, managed state, native IPC, keyring, filesystem, process, and platform-specific implementation work."
---

# DAARION Rust and Tauri

Follow the mandatory lifecycle in the active `AGENTS.md`: require a reviewed baseline audit and written phase plan with an allowed verdict before implementation, then require tests, security review, full diff review, documentation, a completion report, and the release gate. Never claim completion when a required check failed.

- Keep Tauri commands as thin validated adapters over cohesive services. Treat `lib.rs` as a composition root, not a growing application module.
- Validate and deserialize every frontend-to-Rust input with explicit bounds. Never trust browser state as authorization.
- Avoid `unwrap()`, `expect()`, and panic on untrusted or recoverable paths. Use typed errors that do not leak secrets.
- Protect shell, process, filesystem, microphone, model download, IPC, and network access with explicit allowlists and policy checks. Normalize paths before authorization.
- Use managed state with clear ownership, concurrency limits, cancellation, and shutdown behavior; do not hold blocking work on the UI thread.
- Keep key material in OS-supported secure storage where practical and never return it to the frontend.
- Analyze Windows, macOS, Linux, and mobile behavior; gate unsupported capabilities explicitly rather than simulating success.
- Add unit tests for services and validation, command-level tests where practical, and failure-path coverage. Run format, clippy, Rust tests, and the repository build checks required by the phase.

Record any native packaging, permissions, signing, or mobile constraint in the phase plan. Do not add a native/production dependency without explicit justification and compatibility review.
