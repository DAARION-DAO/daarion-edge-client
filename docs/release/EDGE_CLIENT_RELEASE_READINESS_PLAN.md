# DAARION Edge Client Release Readiness Plan

Date: 2026-06-18

Status: docs-only release-readiness gate.

## Executive Summary

Final recommendation:

```text
Official public release: NOT READY
Unsigned tester/canary release: READY WITH CONDITIONS
Local Agent Runtime Foundation: BLOCKED
```

The public install page now uses the correct "Connect device" language, but the
download links still point to `v0.2.2-3`, which predates the current foundation
state on `main`. The next release milestone is therefore not a feature
milestone. It is a release-readiness milestone for publishing
foundation-aligned Edge Client artifacts.

Use a two-track strategy:

1. Publish a foundation-aligned tester/canary release only if it is clearly
   marked as unsigned or untrusted where applicable.
2. Keep public onboarding blocked until signed/trusted distribution, checksums,
   and real-device first-run proof are complete.

Do not open Local Agent Runtime, Worker Node UX, Genesis expansion, or new
feature work until this release gate is closed.

## Current State

Current repo evidence:

| Area | Current state |
|---|---|
| Branch baseline | `main` at `37015ca docs: add manual product journey smoke result` |
| Public install release evidence | `v0.2.2-3` |
| Latest GitHub prerelease verified | `v0.2.2-3` |
| GitHub `Latest` badge | `v0.2.0-beta`; release metadata is older than the current public install target |
| Public release commit | `d00d825 chore: bump edge client to 0.2.2-3 after startup fix` |
| Current app version in `package.json` | `0.2.2-3` |
| Current app version in `src-tauri/Cargo.toml` | `0.2.2-3` |
| Current app version in `src-tauri/tauri.conf.json` | `0.2.2-3` |
| Root version in `package-lock.json` | `0.1.0` |
| Existing installer audit | `docs/audits/EDGE_CLIENT_INSTALLER_AND_FIRST_RUN_AUDIT.md` |

The `package-lock.json` version drift is a release hygiene blocker for the next
version bump. The future release commit must align version truth across:

- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

## Main vs Public Release Gap

The public `v0.2.2-3` release is missing the current foundation work now merged
on `main`:

| Capability | Evidence on `main` after `v0.2.2-3` |
|---|---|
| Signed-only hardening | `b0d0c36 hardening: enforce signed enrollment and explicit config` |
| Warning debt baseline | `840f8e5 chore: establish Rust warning cleanup baseline` |
| IdentityService boundary | `09cfc55 feat: add IdentityService keyring signing boundary` |
| PairingState foundation | `0975d43 feat: add backend pairing foundation` |
| Backend health contract | `2acc105 docs: define backend health contract` |
| Backend connectivity diagnostics | `e73291d feat: add backend connectivity and health diagnostics` |
| Foundation validation | `bcd8df2`, `8f445b3`, `37015ca` validation docs |

Runtime consequence: the public installer artifacts do not represent the
current intended product path:

```text
Identity
Pairing
Backend health
Genesis
Dashboard
```

## Release Inventory

Current public artifacts are downloadable, but they are not sufficient for
public onboarding because they come from stale release `v0.2.2-3` and lack
complete trust/install proof.

| Platform | Current public artifact | Status | Known blockers |
|---|---|---|---|
| macOS Apple Silicon | `Daarion.Edge_0.2.2-3_aarch64.dmg` | Boots in direct-binary smoke | Stale release, DMG rejected by Gatekeeper, ad-hoc/linker signature, strict codesign failure, no stapled notarization ticket |
| macOS Intel | `Daarion.Edge_0.2.2-3_x64.dmg` | Download/static DMG proof only | Stale release, DMG rejected by Gatekeeper, app unsigned, no stapled notarization ticket, no native Intel first-run proof |
| Windows setup.exe | `Daarion.Edge_0.2.2-3_x64-setup.exe` | Download/static installer proof only | Stale release, SmartScreen unknown, WebView2 bootstrapper not proven, no install/launch/uninstall proof |
| Windows MSI | `Daarion.Edge_0.2.2-3_x64_en-US.msi` | Download/static MSI proof only | Stale release, SmartScreen unknown, no install/launch/uninstall proof |
| Linux AppImage | `Daarion.Edge_0.2.2-3_amd64.AppImage` | Download/static ELF proof only | Stale release, no FUSE/dependency proof, no launch proof |
| Android APK | `Daarion.Edge_0.2.2-3_android_universal_release.apk` | Download/ZIP integrity proof only | Stale release, APK signature not verified in latest audit, no `adb install`, no launch proof |

## Trust And Distribution Gates

Public release is blocked until these gates pass:

1. **Truthful release notes**
   - Current workflow release body says macOS artifacts are `Release-Signed`.
   - Latest audit found no usable DMG signature, ad-hoc Apple Silicon signing,
     unsigned Intel app, and no notarization ticket.
   - Future release notes must distinguish `build exists`, `signed`,
     `notarized`, `install proof`, `boot proof`, and `product journey proof`.

2. **macOS signing and notarization**
   - Public macOS onboarding requires Developer ID signing and notarization for
     both Apple Silicon and Intel artifacts.
   - If credentials are unavailable, artifacts may only be tester/canary and
     must be explicitly labeled unsigned or untrusted.

3. **Official checksums**
   - Release must publish official SHA-256 checksums as release assets.
   - Locally computed audit hashes are not a publisher trust anchor.

4. **Windows real-device proof**
   - Static MSI strings are not enough.
   - Windows release requires setup.exe and MSI install, SmartScreen
     observation, WebView2 behavior, shortcuts, uninstall entry, launch, and
     boot log proof.

5. **Linux and Android proof**
   - Linux needs AppImage execution and dependency/FUSE proof.
   - Android needs APK signature verification, `adb install`, launch, and crash
     log check.

## Two-Track Release Strategy

### Track A: Foundation-Aligned Tester/Canary

Allowed only with conditions:

- versioned as `0.2.2-4` / `v0.2.2-4` unless a newer tag already exists;
- built from current `main`;
- marked prerelease/tester-only;
- release notes explicitly say macOS signing/notarization and public
  onboarding are not complete if signing is unavailable;
- official checksums are attached;
- public `1.daarion.city` download links are not switched to this release until
  public gates pass.

Track A can prove that the current foundation work packages and boots.

### Track B: Official Public Release

Required before public onboarding:

- signed/notarized macOS DMGs;
- real-device Windows, Linux, Android validation as applicable;
- official checksums;
- accurate release notes;
- product smoke from public links;
- updated `1.daarion.city` links only after validation passes.

Track B is the release that can unblock public onboarding.

## Future Release Execution Plan

Do not execute these steps in this docs-only PR. This is the required sequence
for the future release milestone.

1. Confirm no newer release/tag supersedes `v0.2.2-3`.
2. Create release branch from latest `main`.
3. Bump version to `0.2.2-4` in all version files.
4. Update release notes to remove unproven `Release-Signed` claims.
5. Add checksum generation/publishing to the release process.
6. If macOS signing credentials exist, wire Developer ID signing and
   notarization before public release.
7. Run pre-release validation:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
   - `cargo test --manifest-path src-tauri/Cargo.toml`
   - `git diff --check`
   - `bash scripts/check-no-secrets.sh`
   - `bash scripts/check-rust-touched-warnings.sh`
8. Create tag `v0.2.2-4`.
9. Let release workflow build artifacts.
10. Download artifacts from GitHub Releases.
11. Verify artifact names, sizes, content types, and checksums.
12. Run platform install and first-run validation.
13. Record validation results in docs.
14. If only tester gates pass, keep release prerelease/tester-only.
15. If public gates pass, update `1.daarion.city` download links.
16. Rerun public product smoke from the live install page.

## First-Run Validation Requirements

Required before public onboarding:

| Scenario | Expected result |
|---|---|
| Fresh install with no pairing | `pairing_required` |
| Paired valid backend, health `ok` | `online` |
| Paired valid backend, health `degraded` | `degraded` |
| Paired valid backend, health `maintenance` | `maintenance` |
| Bad backend | `offline` |
| Bad health contract | `contract_invalid` |
| Old backend contract | `version_mismatch` |
| Real invite/provisioning backend | Genesis completes and Dashboard is reachable |

Each platform smoke must also record:

- package/build used;
- install path;
- launch behavior;
- whether the process stays alive for more than 60 seconds;
- visible first screen;
- boot log location and final lines;
- identity/pairing/config files created;
- any OS security prompt.

## Rollback Plan

- Do not update public download links until new artifacts pass validation.
- If a tester release fails, leave it prerelease/tester-only and document the
  exact blocker.
- If public links are already updated and a blocker is found, revert links to
  the previous known safe target or to the GitHub Releases page with warning
  copy.
- Do not delete failed release artifacts; mark them superseded or failed in
  release notes for auditability.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Public links point to stale release | Users install a client without current pairing/health foundation | Publish foundation-aligned release before public onboarding |
| macOS trust failure | Users cannot launch without bypassing Gatekeeper | Developer ID signing and notarization before public release |
| Misleading release notes | Operators assume unvalidated artifacts are safe | Replace `Release-Signed` claims with proof-based status |
| Windows silent launch failure | Users see blank window or app closes | Real-device boot proof and boot log validation |
| Missing checksums | Users cannot verify downloads | Attach official SHA-256 checksum asset |
| Version drift | Wrong artifact names or backend/version reporting | Align all version files in release commit |

## Next Release Milestone

Recommended milestone:

```text
release: publish foundation-aligned Edge Client tester artifacts
```

Acceptance for that milestone:

```text
Foundation-aligned artifacts: built from current main
Release notes: truthful
Checksums: published
macOS public trust: documented as pass or blocker
Windows/Linux/Android: real-device validation plan assigned
Public onboarding: remains blocked unless all public gates pass
Local Agent Runtime Foundation: remains blocked
```
