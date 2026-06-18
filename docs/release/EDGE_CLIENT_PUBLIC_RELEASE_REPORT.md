# DAARION Edge Client Public Release Report

Date: 2026-06-18

Branch: `codex/foundation-aligned-public-release`

Milestone: `release: foundation-aligned signed public Edge Client`

## Final Verdict

```text
READY FOR TESTER RELEASE ONLY
```

The current branch prepares a foundation-aligned `0.2.2-4` release candidate
and fixes release-truth gaps, but it is not ready for signed public onboarding.
The only locally produced installer candidate in this run is the macOS Apple
Silicon DMG. It builds and boots as a tester/canary artifact, but macOS trust
checks fail because the app is ad-hoc/linker-signed and not notarized.

No public download links were changed. No GitHub release was published. No
Local Agent Runtime, Worker Node, Genesis, onboarding, or backend scope was
started.

## Release Baseline

| Area | Observed state |
|---|---|
| Baseline main | `c68a67d Merge pull request #18 from DAARION-DAO/codex/release-readiness-gate` |
| Required readiness commit | `bc11d64 docs: add Edge Client release readiness gate` |
| Latest public install target observed in release audit | `v0.2.2-3` |
| Latest GitHub prerelease observed during this milestone | `v0.2.2-3` |
| GitHub Latest badge inconsistency | `v0.2.0-beta` remains marked Latest while newer prereleases exist |
| Target release version | `0.2.2-4` |
| Target tag | `v0.2.2-4` |
| Release mode | tester/canary first; public links remain blocked |

`v0.2.2-3` predates the merged foundation state now on `main`, including
signed-only enrollment hardening, warning baseline, IdentityService keyring
boundary, PairingState, backend health contract, backend connectivity
diagnostics, and validation reports.

## Version Truth

This branch aligns version truth to `0.2.2-4` across:

| File | Version |
|---|---|
| `package.json` | `0.2.2-4` |
| `package-lock.json` | `0.2.2-4` |
| `package-lock.json` root package | `0.2.2-4` |
| `src-tauri/Cargo.toml` | `0.2.2-4` |
| `src-tauri/Cargo.lock` package entry | `0.2.2-4` |
| `src-tauri/tauri.conf.json` | `0.2.2-4` |

## Release Version Strategy

Use:

```text
Version: 0.2.2-4
Tag: v0.2.2-4
Channel: prerelease/tester first
```

Rationale:

- prior release docs selected `0.2.2-4` as the next WiX-compatible numeric
  prerelease version;
- no newer GitHub prerelease was observed before starting this branch;
- the version bump is narrow and does not imply runtime feature changes;
- public onboarding remains blocked until trust and full platform proof pass.

Release notes must be proof-based. They must not claim `Release-Signed`,
notarized, install-proofed, boot-proofed, or public-ready status unless the
corresponding evidence is attached.

## Publication Strategy

1. Publish `v0.2.2-4` only as prerelease/tester unless all public gates pass.
2. Attach official checksum assets generated from release artifacts.
3. Mark macOS artifacts unsigned/untrusted unless Developer ID signing and
   notarization proof is present.
4. Keep `1.daarion.city` download links unchanged until public gates pass.
5. If tester artifacts fail, leave the release prerelease/tester-only and
   document the blocker.

## Artifact Inventory

### Locally Produced Candidate

| Platform | Artifact | Version | Size | SHA-256 | Build timestamp | Status |
|---|---|---:|---:|---|---|---|
| macOS Apple Silicon | `Daarion.Edge_0.2.2-4_aarch64.dmg` | `0.2.2-4` | 7,928,454 bytes | `ee343893b9db500a9f373453d3f8b618493fa2f0a857c32ea89cedd882d655f6` | 2026-06-18T15:55:48Z | Built locally; tester/canary only |
| macOS Apple Silicon manifest | `release-manifest-macos-arm64.json` | `0.2.2-4` | 330 bytes | `e849f739a6c8987c98637d8ff260d220978cfcec0432c5392cd8027f829bfed7` | 2026-06-18T15:55:48Z | Built locally |
| macOS Apple Silicon checksums | `SHA256SUMS-macos-arm64.txt` | `0.2.2-4` | 199 bytes | n/a | 2026-06-18T15:55:48Z | Generated locally |

Checksum payload:

```text
ee343893b9db500a9f373453d3f8b618493fa2f0a857c32ea89cedd882d655f6  Daarion.Edge_0.2.2-4_aarch64.dmg
e849f739a6c8987c98637d8ff260d220978cfcec0432c5392cd8027f829bfed7  release-manifest-macos-arm64.json
```

### Not Locally Produced In This Run

| Platform | Expected artifact | Status | Reason |
|---|---|---|---|
| macOS Intel | `Daarion.Edge_0.2.2-4_x64.dmg` | Not built locally | `x86_64-apple-darwin` Rust target is not installed; `rustup` is unavailable in this environment |
| Windows setup.exe | `Daarion.Edge_0.2.2-4_x64-setup.exe` | Not built locally | Requires Windows release runner and real Windows install proof |
| Windows MSI | `Daarion.Edge_0.2.2-4_x64_en-US.msi` | Not built locally | Requires Windows release runner and real Windows install proof |
| Linux AppImage | `Daarion.Edge_0.2.2-4_amd64.AppImage` | Not built locally | Requires Linux release runner and desktop/AppImage proof |
| Android APK | `Daarion.Edge_0.2.2-4_android_universal_release.apk` | Not built locally | Android remains tester-only until signature, `adb install`, launch, and crash-log proof are attached |

## Release Workflow Changes

This branch updates release publication preparation:

- `scripts/prepare-release-artifacts.cjs` now generates per-platform
  `SHA256SUMS-<platform>.txt` checksum assets next to staged artifacts;
- `.github/workflows/release.yml` uploads those checksum assets through the
  existing `dist/artifacts/*` upload path;
- release body copy no longer claims macOS `Release-Signed` status without
  signing/notarization evidence;
- Android release text no longer claims signature proof without APK signature
  verification evidence.

## Trust Validation

### macOS Apple Silicon

Artifact: `Daarion.Edge_0.2.2-4_aarch64.dmg`

| Check | Result | Evidence |
|---|---|---|
| `hdiutil verify` | Pass | DMG checksum is valid |
| DMG code signature | Fail | `code object is not signed at all` |
| `spctl --assess --type open` on DMG | Fail | `rejected`, `source=Insufficient Context` |
| App signature detail | Fail for public trust | `Signature=adhoc`, `TeamIdentifier=not set`, `Info.plist=not bound` |
| `codesign --verify --deep --strict` on app | Fail | `code has no resources but signature indicates they must be present` |
| `spctl --assess --type execute` on app | Fail | same strict signature failure |
| `stapler validate` on app | Fail | no stapled ticket |
| `stapler validate` on DMG | Fail | no stapled ticket |

Conclusion: macOS Apple Silicon is usable only as an explicitly unsigned or
untrusted tester/canary artifact. It is not public-trust ready.

### macOS Intel

Not validated in this run. Local cross-build failed because the
`x86_64-apple-darwin` Rust target is missing and the environment does not have
`rustup` available to install it. Public release requires a macOS Intel build
from CI or a prepared macOS runner, plus the same signing, notarization,
Gatekeeper, and first-run proof as Apple Silicon.

### Windows

Not validated in this run. Public release requires setup.exe and MSI artifacts
from the Windows release runner, signing status, SmartScreen observation,
WebView2 bootstrapper behavior, install proof, launch proof, boot log proof,
shortcut proof, and uninstall proof.

### Linux

Not validated in this run. Public release requires an AppImage from the Linux
release runner, checksum verification, dependency/FUSE behavior, desktop launch
proof, process liveness, and boot log proof.

### Android

Not validated in this run. Android remains tester-only unless the APK signature,
`adb install`, launch, and logcat/crash proof are attached. Public onboarding
does not depend on Android in this release gate unless Android is explicitly
kept in the public platform set.

## Installer And First-Run Validation

### macOS Apple Silicon Local Smoke

Performed against the locally built `0.2.2-4` DMG.

| Step | Result |
|---|---|
| Build app and DMG | Pass |
| Stage canonical artifact name | Pass |
| Generate checksum file | Pass |
| Verify DMG with `hdiutil verify` | Pass |
| Mount DMG read-only | Pass |
| Copy app into temporary Applications directory | Pass |
| Launch app executable with isolated temporary HOME | Pass |
| Remain alive for more than 60 seconds | Pass |
| Boot log generated | Pass |
| Blank-screen visual verification | Not proven in this automated run |
| Pairing gate visual verification | Not proven in this automated run |
| Health diagnostics visual verification | Not proven in this automated run |

Boot log/stdout tail showed:

```text
Version: 0.2.2-4
OS: macos
Arch: aarch64
Managing state: HeartbeatManager, BackendHealthManager, MessagingState, WorkerModeState
Invoke handlers registered
System tray: OK
Heartbeat loop started
Worker opt-in loaded: false
setup() completed successfully
```

This proves process boot and diagnostic logging for a local tester candidate. It
does not prove Finder launch, Gatekeeper acceptance, nonblank UI rendering, or
the full product journey.

## Public Onboarding Validation

Public onboarding is still blocked.

The required path:

```text
1.daarion.city
-> Connect Device
-> Download installer
-> Install
-> Launch
-> Reach usable state
```

does not pass yet because:

- no `v0.2.2-4` GitHub release has been published;
- public download links have not been updated;
- macOS artifacts are not signed/notarized;
- Windows/Linux artifacts were not produced and validated in this run;
- nonblank UI, pairing gate, health diagnostics, Genesis, and Dashboard were
  not proven from public release links;
- GitHub release metadata still has an older `Latest` badge state that should
  be corrected during publication.

## Risks And Blockers

| Blocker | Severity | Required action |
|---|---:|---|
| macOS artifacts are unsigned/unnotarized | High | Add Developer ID signing and notarization, or mark release tester-only |
| No public `0.2.2-4` GitHub release yet | High | Tag and publish only after review |
| Windows install proof missing | High | Validate setup.exe and MSI on real Windows |
| Linux launch proof missing | High | Validate AppImage on Linux desktop |
| No public-link product smoke | High | Run smoke from published release links before updating `1.daarion.city` |
| Nonblank UI and pairing/health screens not visually captured | Medium | Run packaged GUI smoke with screenshot/manual proof |
| GitHub Latest badge points to older `v0.2.0-beta` | Medium | Correct release metadata during publication |
| Android policy unresolved | Medium | Keep Android tester-only or attach full APK proof |

## Validation Commands

Executed in this milestone:

```text
git fetch origin --prune
git merge-base --is-ancestor bc11d64 origin/main
git cat-file -e origin/main:docs/release/EDGE_CLIENT_RELEASE_READINESS_PLAN.md
npm version 0.2.2-4 --no-git-tag-version
cargo check --manifest-path src-tauri/Cargo.toml
npm run tauri build -- --target aarch64-apple-darwin
ARTIFACT_LABEL=macos-arm64 RUST_TARGET=aarch64-apple-darwin node scripts/prepare-release-artifacts.cjs
hdiutil verify Daarion Edge_0.2.2-4_aarch64.dmg
codesign --verify --deep --strict Daarion Edge.app
codesign -dv --verbose=4 Daarion Edge.app
spctl --assess --type execute Daarion Edge.app
spctl --assess --type open Daarion Edge_0.2.2-4_aarch64.dmg
xcrun stapler validate Daarion Edge.app
xcrun stapler validate Daarion Edge_0.2.2-4_aarch64.dmg
npm run tauri build -- --target x86_64-apple-darwin
```

Completed after report creation:

```text
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
bash scripts/check-no-secrets.sh
bash scripts/check-rust-touched-warnings.sh
```

## Recommendation

Proceed with this PR as release-preparation work, not as a signed public
release. After merge, publish `v0.2.2-4` only as a clearly labeled
tester/canary release unless signing, notarization, Windows, Linux, and
public-link product smoke all pass.

Do not start Local Agent Runtime Foundation or Worker Node onboarding until the
public installer channel is current, trusted, and validated.
