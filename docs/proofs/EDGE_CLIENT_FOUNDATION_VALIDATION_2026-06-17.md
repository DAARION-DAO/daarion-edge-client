# Edge Client Foundation Validation - 2026-06-17

## Scope

This report closes the current Edge Client foundation cycle after the backend
contract and runtime connectivity work landed on `main`.

Merged PRs covered by this validation:

| PR | Title | Result |
|---|---|---|
| #13 | `docs: define backend health contract` | Merged into `main` |
| #14 | `feat: add backend connectivity and health diagnostics` | Merged into `main` |

No new product milestone was opened during this validation. Local Agent Runtime
Foundation remains out of scope until this foundation path is reviewed.

## Executed Validation

Post-merge validation was run against updated `main` after PR #13 and PR #14
were merged.

| Check | Result | Notes |
|---|---|---|
| `cargo check --manifest-path src-tauri/Cargo.toml` | Pass | Existing warning baseline remains at 310 warnings. |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Pass | 49 Rust tests passed. |
| `npm run build` | Pass | TypeScript and Vite production build completed. |
| `git diff --check` | Pass | No whitespace errors. |
| `bash scripts/check-no-secrets.sh` | Pass | No forbidden patterns found. |
| `bash scripts/check-rust-touched-warnings.sh` | Pass | No Rust warnings reported in protected hardening modules. |

## Smoke Matrix

The runtime health states are covered by the merged Rust test suite and the
frontend integration is covered by the production build.

| Scenario | Expected state | Observed evidence |
|---|---|---|
| Unpaired backend | `pairing_required` | `backend_health::tests::no_configured_backend_maps_to_pairing_required` passed. |
| Paired valid backend, `status: ok` | `online` | `backend_health::tests::valid_ok_maps_to_online` passed. |
| Paired valid backend, `status: degraded` | `degraded` | `backend_health::tests::valid_degraded_maps_to_degraded` passed. |
| Paired valid backend, `status: maintenance` | `maintenance` | `backend_health::tests::valid_maintenance_maps_to_maintenance` passed. |
| Bad backend HTTP response | `offline` | `backend_health::tests::non_success_maps_to_offline` passed. |
| Bad contract: invalid JSON | `contract_invalid` | `backend_health::tests::invalid_json_maps_to_contract_invalid` passed. |
| Bad contract: missing required fields | `contract_invalid` | `backend_health::tests::missing_required_field_maps_to_contract_invalid` passed. |
| Bad contract: public health requires auth | `contract_invalid` | `backend_health::tests::unauthorized_maps_to_contract_invalid` passed. |
| Old backend: unsupported schema | `version_mismatch` | `backend_health::tests::unsupported_schema_maps_to_version_mismatch` passed. |
| Old backend: incompatible protocol | `version_mismatch` | `backend_health::tests::incompatible_protocol_maps_to_version_mismatch` passed. |
| Old backend: client below minimum version | `version_mismatch` | `backend_health::tests::too_high_min_client_maps_to_version_mismatch` passed. |

## First-Run Path

Target user path:

```text
Fresh install
Identity initialization
Unpaired state
Pair backend
Health diagnostics
Genesis
Dashboard
```

Observed automated coverage:

| Step | Validation evidence |
|---|---|
| Identity initialization | Identity generation, loading, signing, secure-key failure, and legacy migration tests passed in the full Rust suite. |
| Unpaired state | Config and backend health tests verify fail-closed unconfigured state and `pairing_required`. |
| Pair backend | Pairing tests verify invite parsing, manual advanced constraints, env import, paired/unpaired transitions, PairingState precedence, and debug localhost non-persistence. |
| Health diagnostics | Backend health tests verify contract parsing and all required state mappings. |
| Genesis | Production frontend build passed with Genesis health diagnostics integration. |
| Dashboard | Production frontend build passed with dashboard health diagnostics integration. |

## Failures Or Regressions

No validation command failed.

No regression was found in the signed-only enrollment tests, IdentityService
tests, PairingState tests, or backend health state mapping tests.

The existing Rust warning baseline still exists at 310 warnings. The warning
guardrail for protected hardening modules passes.

## UX Gaps

The complete packaged Tauri GUI first-run journey was not executed in this
session. The repository does not include Playwright, and no browser/Tauri
automation tool was available in the session after tool discovery.

Remaining manual smoke gap:

```text
Fresh install
Identity initialization
Unpaired state
Pair backend
Health diagnostics
Genesis
Dashboard
```

This manual pass should use a real backend or deterministic mock backend that
can return:

```text
ok
degraded
maintenance
invalid JSON
unsupported schema_version
incompatible edge_protocol_version
too-high min_edge_client_version
```

## Recommendation

The code foundation is complete and merged:

```text
Identity
Pairing
Backend Contract
Connectivity and Health
```

Automated validation passes for the foundation components and their integration
points. Do not open Local Agent Runtime Foundation or another product milestone
until the remaining packaged Tauri first-run smoke is completed and reviewed.

Foundation status:

```text
Implementation complete.
Automated validation passed.
Product journey validation pending one packaged GUI smoke.
```
