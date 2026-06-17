# Rust Warning Baseline - 2026-06-17

## Scope

This baseline is intentionally narrow. It documents the current Rust warning debt and protects modules touched by PR #9 without applying broad `cargo fix` or repo-wide cleanup.

Protected modules:

- `src/config.rs`
- `src/enrollment.rs`
- `src/heartbeat.rs`
- `src/lib.rs`
- `src/messaging.rs`
- `src/provisioning.rs`
- `src/registry_client.rs`
- `src/worker/mod.rs`
- `src/worker/onboarding.rs`

## Initial Snapshot

Initial `cargo check` output produced 319 warnings before the scoped cleanup.

Category breakdown:

| Category | Count |
|---|---:|
| Dead code / unused API surface | 260 |
| Unused imports | 44 |
| Unused variables | 13 |
| Unnecessary mutability | 1 |
| Unread fields | 1 |

Module-family breakdown:

| Module family | Count |
|---|---:|
| `src/models` | 94 |
| `src/agents` | 41 |
| `src/worker` | 40 |
| `src/authorities` | 22 |
| `src/market` | 19 |
| `src/districts` | 17 |
| `src/evolution` | 17 |
| `src/coordination` | 15 |
| `src/intelligence` | 15 |
| `src/observability` | 12 |
| `src/trust` | 12 |
| `src/metacognition` | 9 |
| `src/messaging.rs` | 3 |
| `src/genesis.rs` | 2 |
| `src/identity.rs` | 1 |

Warnings found in protected modules before cleanup:

| Module | Warning |
|---|---|
| `src/messaging.rs` | unused import: `Manager` |
| `src/messaging.rs` | unused variable: `node_token` |
| `src/messaging.rs` | unused variable: `state` |
| `src/worker/mod.rs` | unused public re-export group: admission types |
| `src/worker/mod.rs` | unused public re-export: `LocalQueue` |
| `src/worker/mod.rs` | unused public re-export: `start_worker_loop` |
| `src/worker/mod.rs` | unused public re-export group: specialization types |
| `src/worker/mod.rs` | unused private import: `MockRelayClient` |

## Cleanup Applied

- Removed the unused `Manager` import from `src/messaging.rs`.
- Renamed intentionally unused messaging placeholders to `_node_token` and `_state`.
- Removed the unused private `MockRelayClient` import from `src/worker/mod.rs`.
- Marked legacy public re-exports in `src/worker/mod.rs` with targeted `#[allow(unused_imports)]` to avoid changing the module API surface in this PR.

## Final Baseline

Final `cargo check` output is saved in `audit/rust-warning-snapshot-2026-06-17.txt`.

After the scoped cleanup, `cargo check` reports 311 warnings and zero warnings in protected modules.

Final category breakdown:

| Category | Count |
|---|---:|
| Dead code / unused API surface | 260 |
| Unused imports | 38 |
| Unused variables | 11 |
| Unnecessary mutability | 1 |
| Unread fields | 1 |

Final module-family breakdown:

| Module family | Count |
|---|---:|
| `src/models` | 94 |
| `src/agents` | 41 |
| `src/worker` | 35 |
| `src/authorities` | 22 |
| `src/market` | 19 |
| `src/districts` | 17 |
| `src/evolution` | 17 |
| `src/coordination` | 15 |
| `src/intelligence` | 15 |
| `src/observability` | 12 |
| `src/trust` | 12 |
| `src/metacognition` | 9 |
| `src/genesis.rs` | 2 |
| `src/identity.rs` | 1 |

## Guardrail

Run:

```bash
bash scripts/check-rust-touched-warnings.sh
```

The guardrail allows existing repo-wide warning debt, but fails when `cargo check` reports warnings in the protected hardening modules listed above.

## Non-Goals

- No broad `cargo fix`.
- No repo-wide dead-code cleanup.
- No runtime behavior changes beyond safe unused warning cleanup.
