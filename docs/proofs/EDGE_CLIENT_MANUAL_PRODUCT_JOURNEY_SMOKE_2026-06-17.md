# Edge Client Manual Product Journey Smoke - 2026-06-17

## Scope

This smoke result records the final foundation gate for the DAARION Edge Client
product journey:

```text
Install
First launch
Invite-code onboarding
Pairing success
Health status resolution
Genesis flow
Dashboard arrival
```

This was a validation-only pass. No new feature milestone was opened.

## Environment

| Item | Value |
|---|---|
| Repository branch | `main` |
| Starting commit | `8f445b3 docs: record packaged foundation smoke outcome` |
| App version | `0.2.2-3` |
| Platform | macOS, Apple Silicon |
| Package command | `npm run tauri -- build --bundles app --no-sign` |
| Backend source | Existing pairing-aware resolver |
| Health contract endpoint | `GET /api/v1/edge/health` |

The packaged app was tested with a deterministic local backend that returned
health contract variants for `ok`, `degraded`, `maintenance`, invalid JSON, and
unsupported `schema_version`. The offline case used a configured backend with no
listener.

## Steps Executed

| Step | Result | Notes |
|---|---|---|
| Build packaged macOS app | Pass | Unsigned `.app` bundle built successfully. |
| First launch with no pairing state | Pass | App rendered the non-technical `Connect DAARION Edge` pairing gate. |
| Unpaired health state | Pass | UI showed `Pairing required`. |
| Pairing through env-imported development backend | Pass | Packaged app imported explicit development backend through the pairing-aware resolver and entered Genesis. |
| Health status resolution | Pass | Packaged Genesis UI rendered all tested health states. |
| Primary invite-code onboarding | Not completed | macOS automation could focus the Tauri window, but text input did not reliably reach the WebView textarea. This requires a human manual pass. |
| Genesis flow | Partial | Genesis loaded and displayed device scan plus backend health diagnostics. Full provisioning was not completed. |
| Dashboard arrival | Not completed | Requires successful Genesis provisioning and enrollment path. |

## Observed States

| Scenario | Expected state | Observed state | Result |
|---|---|---|---|
| No pairing | `pairing_required` | `Pairing required` gate rendered on first launch. | Pass |
| Paired valid backend, `status: ok` | `online` | Genesis rendered `dev.localhost paired - Online`. | Pass |
| Paired valid backend, `status: degraded` | `degraded` | Genesis rendered `dev.localhost paired - Degraded`. | Pass |
| Paired valid backend, `status: maintenance` | `maintenance` | Genesis rendered `dev.localhost paired - Maintenance`. | Pass |
| Configured backend with no listener | `offline` | Genesis rendered `dev.localhost paired - Offline`. | Pass |
| Invalid health contract JSON | `contract_invalid` | Genesis rendered `dev.localhost paired - Contract invalid`. | Pass |
| Unsupported health schema version | `version_mismatch` | Genesis rendered `dev.localhost paired - Version mismatch`. | Pass |

## Blockers

Full product journey validation is still pending.

Blocker 1: primary invite-code onboarding was not completed in this validation
session. The app rendered the correct invite-code gate, but the validation
environment could not reliably send text input into the Tauri WebView textarea.
This is not recorded as an application defect until a human manual pass
reproduces it.

Blocker 2: Genesis-to-Dashboard arrival was not completed. The current Genesis
flow depends on token-gate and provisioning calls, including `/genesis/register`.
The deterministic health mock used for this smoke was intentionally not treated
as a real provisioning backend.

No narrow fix PR was opened because this smoke did not isolate a confirmed
product defect. It isolated remaining validation requirements.

## Cleanup

Generated test pairing state was removed after the smoke pass. No identity
keyring entry was present after cleanup.

## Recommendation

Current state:

```text
Foundation implementation: COMPLETE
Automated validation: COMPLETE
Packaged launch + health smoke: PARTIAL COMPLETE
Full product journey validation: PENDING
```

Do not open Local Agent Runtime Foundation yet.

Next required action is a human manual smoke using a real invitation code and a
backend capable of completing Genesis provisioning:

```text
Install package
First launch
Enter invitation code
Pair backend
Resolve health state
Complete Genesis
Arrive at Dashboard
```

Decision rule:

```text
If the manual run passes:
  Foundation phase = COMPLETE
  Product journey = VALIDATED

If the manual run fails:
  Open a narrow fix PR for the exact observed blocker.
```
