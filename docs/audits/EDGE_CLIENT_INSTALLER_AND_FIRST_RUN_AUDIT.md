# DAARION Edge Client Installer And First-Run Audit

Audit date: 2026-06-18

Scope: audit only. No source code changes, commits, or pull requests were
created. The audit checked the public DAARION device setup surface linked from
`https://1.daarion.city/install`, the GitHub release artifacts for
`DAARION-DAO/daarion-edge-client`, and the first-run behavior that could be
validated from a macOS Apple Silicon host.

## Executive Summary

Overall verdict: **FAIL for public onboarding**.

The public download links on `https://1.daarion.city/install` now point to
existing GitHub release assets for `v0.2.2-3`, and those assets are
downloadable. The Apple Silicon build also boots and stays alive for more than
60 seconds when launched directly from the mounted DMG.

The blocker is product readiness, not basic file availability:

- the public release `v0.2.2-3` is older than current `main` and does not
  contain the merged foundation work for hardening, IdentityService,
  PairingState, and backend health;
- macOS public install trust is not ready: DMGs have no usable signature,
  Apple Silicon is ad-hoc/linker signed and fails strict verification, Intel
  is unsigned, and neither app has a stapled notarization ticket;
- Windows, Linux, Android, and macOS Intel still need real-device install and
  first-run proof;
- the live Edge PWA at `https://edge.daarion.city/` exists, but still presents a
  technical `Sovereign Genesis` experience rather than a clean non-technical
  "Connect device" flow.

## Environment

| Item | Observed value |
|---|---|
| Repository | `DAARION-DAO/daarion-edge-client` |
| Local branch | `main` |
| Local HEAD | `37015ca docs: add manual product journey smoke result` |
| Public release audited | `v0.2.2-3` |
| Release commit | `d00d825 chore: bump edge client to 0.2.2-3 after startup fix` |
| Host OS | macOS 26.4.1 |
| Host architecture | Apple Silicon / `arm64` |
| Native first-run coverage | Apple Silicon process boot only |
| Windows/Linux/Android coverage | Download/static metadata only |

## Evidence Sources

- Live install page: `https://1.daarion.city/install`
- Edge PWA: `https://edge.daarion.city/`
- GitHub release: `https://github.com/DAARION-DAO/daarion-edge-client/releases/tag/v0.2.2-3`
- Release truth document: `docs/operations/EDGE_CLIENT_RELEASE_TRUTH.md`
- Tester checklist: `docs/operations/EDGE_CLIENT_TESTER_CHECKLIST.md`
- Previous manual journey smoke: `docs/proofs/EDGE_CLIENT_MANUAL_PRODUCT_JOURNEY_SMOKE_2026-06-17.md`
- Current source evidence: `src/App.tsx`, `src/components/PairingGate.tsx`,
  `src-tauri/src/pairing.rs`, `src-tauri/src/backend_health.rs`

## Public Install Surface

The live `https://1.daarion.city/install` page was serving the updated bundle
and rendered the new device language:

- `Connect your device to DAARION`
- `Prepare this device`
- `Prepare Apple Silicon`
- `Prepare Intel Mac`
- `Prepare Windows`
- `Prepare Linux`
- `Prepare Android`

The page exposes these release links:

| Surface | Link target |
|---|---|
| Web/PWA setup | `https://edge.daarion.city/` |
| macOS Apple Silicon | `Daarion.Edge_0.2.2-3_aarch64.dmg` |
| macOS Intel | `Daarion.Edge_0.2.2-3_x64.dmg` |
| Windows setup.exe | `Daarion.Edge_0.2.2-3_x64-setup.exe` |
| Windows MSI | `Daarion.Edge_0.2.2-3_x64_en-US.msi` |
| Linux AppImage | `Daarion.Edge_0.2.2-3_amd64.AppImage` |
| Android APK | `Daarion.Edge_0.2.2-3_android_universal_release.apk` |

The UI copy is improved, but the linked release is still `v0.2.2-3`, which
predates the current foundation state on `main`.

## Release Artifact Matrix

GitHub Release API confirmed all expected assets are present and in `uploaded`
state. The files were also downloaded successfully for local static checks.

| Platform | Artifact | API content type | Size | Local SHA-256 | Download status |
|---|---|---:|---:|---|---|
| macOS Apple Silicon | `Daarion.Edge_0.2.2-3_aarch64.dmg` | `application/x-apple-diskimage` | 7,803,223 bytes | `ef428556b58ece4529e535f3ec8a2e9e356f905d0df02b4907af2de06d317aa1` | Pass |
| macOS Intel | `Daarion.Edge_0.2.2-3_x64.dmg` | `application/x-apple-diskimage` | 8,081,498 bytes | `4a6a46ea4e1ec2f8dd32800850a8f2eae436bc10da1dc78abe50f51e77dd1519` | Pass |
| Windows setup.exe | `Daarion.Edge_0.2.2-3_x64-setup.exe` | `application/x-msdos-program` | 5,206,713 bytes | `f9deec38e72441688785fd0ba57d2df3fadedfea41375de5432530808cbd55e0` | Pass |
| Windows MSI | `Daarion.Edge_0.2.2-3_x64_en-US.msi` | `application/x-msdownload` | 7,290,880 bytes | `c88b54f5b5a8ff9c15b03c5bc4c35028a4b6dd25cde2aed4edd20018a6b439ac` | Pass |
| Linux AppImage | `Daarion.Edge_0.2.2-3_amd64.AppImage` | `application/octet-stream` | 86,129,144 bytes | `75d5cb6eb9176858d4a812df706182dca59f501bf61da453e7dea337289b22d1` | Pass |
| Android APK | `Daarion.Edge_0.2.2-3_android_universal_release.apk` | `application/vnd.android.package-archive` | 75,988,408 bytes | `afbc274e4bb0a75b6d50129cdc3ab9f40a7ffec0de5c407f24370366ef198e24` | Pass |

Checksum status: no official checksum or signature assets were found in the
release. The hashes above are locally computed audit hashes and should not be
treated as a publisher-provided trust anchor.

## macOS Installation And Trust Checks

Both DMGs pass structural verification and mount successfully.

| Check | Apple Silicon DMG | Intel DMG |
|---|---|---|
| `hdiutil verify` | Pass | Pass |
| Mounted read-only | Pass | Pass |
| Contains `Daarion Edge.app` | Pass | Pass |
| Contains Applications alias | Pass | Pass |
| DMG Gatekeeper assessment | Fail: `rejected`, `source=no usable signature` | Fail: `rejected`, `source=no usable signature` |
| App binary architecture | `arm64` | `x86_64` |
| App bundle identifier | `city.daarion.edge` | `city.daarion.edge` |
| App version | `0.2.2-3` | `0.2.2-3` |
| Code signing | Ad-hoc/linker signed; strict verify fails | Not signed |
| Notarization stapled ticket | Missing | Missing |

Detailed signing findings:

- Apple Silicon app:
  - `Signature=adhoc`
  - `TeamIdentifier=not set`
  - `codesign --verify --deep --strict` fails with:
    `code has no resources but signature indicates they must be present`
  - `stapler validate` reports no stapled ticket.
- Intel app:
  - `codesign` reports the code object is not signed.
  - `spctl` rejects the app with `source=no usable signature`.

True copy-to-Applications install was not performed in this audit because the
host already had an existing `Daarion Edge.app` installation and the audit did
not overwrite the user's installed app. The mounted DMG layout contains the
expected app bundle and Applications alias, but a clean-machine Applications
copy still needs to be recorded as separate manual proof.

## Apple Silicon First Launch

Apple Silicon launch was tested by running the binary directly from the mounted
DMG with an isolated temporary home directory. This intentionally avoids
modifying the user's real identity/config state and bypasses Finder/Gatekeeper.

| Check | Result |
|---|---|
| Process starts | Pass |
| Still alive after 8 seconds | Pass |
| Still alive after 65 seconds | Pass |
| Window appears | Pass |
| Blank screen | Not observed |
| Boot log created | Pass |
| `setup() completed successfully` in log | Pass |
| Worker mode default | `Worker opt-in loaded: false` |

The visible first screen was the technical Genesis flow:

- `Sovereign Genesis`
- `Hardware Audit`
- `Scanning Creator's device`
- detected hardware class and local model recommendation content

This proves process boot and basic UI rendering for Apple Silicon, but it does
not prove the current foundation product journey because this public release is
older than the current pairing/health work on `main`.

## Windows Static Checks

Windows artifacts were downloadable and have plausible installer formats:

- setup.exe: PE32 GUI executable, Nullsoft Installer / NSIS.
- MSI: Windows Installer database, ProductName `Daarion Edge`,
  ProductVersion `0.2.2.3`.

Static MSI strings indicate:

- desktop shortcut support;
- Start Menu shortcut support;
- uninstall shortcut support;
- WebView2 bootstrapper action;
- install/uninstall UI text from WiX.

Not validated from this macOS host:

- installer launch;
- SmartScreen behavior;
- install completion;
- real desktop/start menu shortcut creation;
- Windows Apps uninstall entry;
- first launch;
- `%APPDATA%/DAARION Edge/logs/boot.log`;
- WebView2 bootstrapper behavior on a clean Windows machine.

## Linux Static Checks

The Linux artifact is downloadable and identified as:

```text
ELF 64-bit LSB pie executable, x86-64, static-pie linked, stripped
```

Not validated from this macOS ARM host:

- `chmod +x`;
- AppImage launch;
- FUSE dependency behavior;
- desktop integration;
- first-run UI;
- log creation;
- local config creation.

## Android Static Checks

The Android APK is downloadable and ZIP integrity passes with no compressed data
errors.

Not validated from this host:

- `adb install`;
- Android 14+ sideload behavior;
- Play Protect or unknown-source prompts;
- runtime launch;
- Android logs.

Additional limitation: `apksigner` was unavailable in the audit environment,
and local Java tooling was not usable, so APK signature verification was not
completed.

## Edge PWA Check

The `https://edge.daarion.city/` web setup path is live:

- HTTP 200;
- `manifest.webmanifest` exists;
- `manifest.json` exists;
- `sw.js` exists;
- `registerSW.js` is referenced;
- app shell assets are served.

The manifest still describes the app as:

```text
DAARION Edge
Sovereign Edge Client
Sovereign Genesis
```

Rendered live PWA state included:

- `DAARION Protocol`
- `Sovereign Genesis`
- `Pairing required. Enter your DAARION invitation code before Genesis registration.`
- `Hardware Audit`
- `Confirm Device`

Finding: this is not yet the desired non-technical "Connect device" product
flow. It exposes internal Genesis language and begins the hardware-audit mental
model before the user has a clear MicroDAO-to-device connection path.

## Runtime And Product Validation

| Area | Result | Finding |
|---|---|---|
| Pairing screen in public native release | Fail | Public `v0.2.2-3` opened directly into Genesis/hardware audit, not the current PairingState gate. |
| Backend resolution / health diagnostics in public native release | Fail | The linked release predates current backend pairing and health diagnostics work on `main`. |
| Genesis flow availability | Partial | Genesis UI appears and hardware audit renders, but this is not the validated current product journey. |
| Local storage initialization | Partial | Boot log was created under isolated home. Identity/config creation was not proven in this pass. |
| Configuration creation | Not proven | No full pairing or provisioning path was completed. |
| Complete new-user journey | Fail | A new user still needs manual help around macOS trust prompts, stale native release behavior, invite/pairing semantics, and Genesis-to-Dashboard completion. |

## Current Main vs Public Release Gap

The public installer links point to release tag `v0.2.2-3`.

That tag is an ancestor of current `main`, but current `main` contains later
foundation commits including:

- `hardening: enforce signed enrollment and explicit config`;
- `chore: establish Rust warning cleanup baseline`;
- `feat: add IdentityService keyring signing boundary`;
- `feat: add backend pairing foundation`;
- `docs: define backend health contract`;
- `feat: add backend connectivity and health diagnostics`;
- foundation validation and manual product journey smoke documents.

This means the live installer page is not distributing the current foundation
state. Even if the old release boots, it is not the release that should be used
to validate the current product foundation.

## Can A New User Install And Know What To Do?

Answer: **No, not yet**.

A new non-technical user can reach the download page and download the package,
but the next steps are still fragile:

- macOS users will hit unsigned/unnotarized app trust friction;
- the native release does not represent the current pairing/health foundation;
- the first visible native flow is `Sovereign Genesis` and hardware audit, not
  a simple "Connect this device to your MicroDAO" experience;
- web/PWA setup exists but still exposes technical Genesis language;
- Windows/Linux/Android are not backed by current real-device first-run proof.

## Top 10 Blockers Before Public Onboarding

1. **Publish a new Edge Client release from current `main`** after the merged
   foundation work. Public `/install` links must not point to a stale release.
2. **Fix macOS signing and notarization** for both Apple Silicon and Intel DMGs.
   Current artifacts are rejected by Gatekeeper assessment.
3. **Add official checksums/signature artifacts** to GitHub Releases and expose
   them in release notes or manifests.
4. **Run clean-machine macOS install proof** by copying the app from DMG into
   Applications and launching via Finder, not only by direct binary execution.
5. **Run Windows real-device install proof** for setup.exe and MSI, including
   SmartScreen, WebView2 bootstrapper, shortcuts, uninstall entry, first launch,
   and boot log.
6. **Run Linux real-device AppImage proof** on Ubuntu/Debian desktop, including
   FUSE/dependency behavior and first launch.
7. **Run Android sideload proof** with `adb install` or a real device, including
   signature verification and first-launch behavior.
8. **Unify the Edge PWA first screen** with the approved "Connect device"
   product language. Avoid making `Sovereign Genesis` the first mental model.
9. **Validate real invite-code onboarding** with a backend capable of completing
   pairing, health, Genesis provisioning, and Dashboard arrival.
10. **Keep Local Agent Runtime Foundation blocked** until the public installer
    and first-run journey pass from current release artifacts.

## Recommended Next Action

Do not open Local Agent Runtime Foundation yet.

The next narrow milestone should be release-readiness, not a new product
feature:

```text
release: publish foundation-aligned Edge Client installer artifacts
```

Minimum acceptance criteria for that milestone:

- build native artifacts from current `main`;
- sign and notarize macOS Apple Silicon and Intel DMGs;
- publish official checksums;
- update `/install` links only after the new release exists;
- run clean first-run smoke from public links:
  - unpaired -> pairing required;
  - paired valid -> online/degraded/maintenance;
  - bad backend -> offline;
  - bad contract -> contract invalid;
  - old backend -> version mismatch;
  - Genesis -> Dashboard path with real invite/provisioning backend.

Until then, the current state should be recorded as:

```text
Download links: AVAILABLE
Apple Silicon process boot: PASS
Public installer trust: FAIL
Foundation-aligned public release: MISSING
Full product journey: PENDING
Overall public onboarding verdict: FAIL
```
