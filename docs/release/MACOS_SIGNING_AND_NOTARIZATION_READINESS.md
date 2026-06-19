# macOS Signing And Notarization Readiness

Date: 2026-06-19

Branch: `codex/macos-signing-notarization-readiness`

Milestone: `release: macOS signing and notarization readiness`

## Final Classification

```text
BLOCKED BY MISSING APPLE ASSETS
```

The current `0.2.2-4` foundation-aligned release path is suitable for
tester/canary use only. The repository now labels macOS artifacts truthfully as
tester/canary, but it cannot produce a signed public macOS release until Apple
Developer assets, notarization credentials, CI secret wiring, and proof
artifacts are available.

This milestone does not publish releases, create tags, change runtime behavior,
or start Local Agent Runtime / Worker Node work.

## Source Documents Reviewed

Repository evidence:

| Area | Evidence |
|---|---|
| Tauri app config | `src-tauri/tauri.conf.json` |
| Release workflow | `.github/workflows/release.yml` |
| Release truth | `docs/operations/EDGE_CLIENT_RELEASE_TRUTH.md` |
| Release readiness plan | `docs/release/EDGE_CLIENT_RELEASE_READINESS_PLAN.md` |
| Release checklist | `docs/release/EDGE_CLIENT_RELEASE_CHECKLIST.md` |
| Public release report | `docs/release/EDGE_CLIENT_PUBLIC_RELEASE_REPORT.md` |
| README release language | `README.md` |

External reference material checked on 2026-06-19:

- Apple Developer ID support:
  <https://developer.apple.com/support/developer-id/>
- Apple Developer ID / Gatekeeper distribution:
  <https://developer.apple.com/developer-id/>
- Apple notarization documentation:
  <https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution>
- Tauri v2 macOS signing documentation:
  <https://v2.tauri.app/distribute/sign/macos/>

## Current macOS Signing State

| Item | Current state | Public-release impact |
|---|---|---|
| macOS artifacts | DMG artifacts are built by `tauri-apps/tauri-action` on `macos-latest` for `aarch64-apple-darwin` and `x86_64-apple-darwin`. | Build capability exists, but build success is not trust readiness. |
| `src-tauri/tauri.conf.json` | No macOS signing identity, entitlements, hardened runtime, notarization, or updater signing configuration is present. | Public release cannot be classified as signed or notarized from config alone. |
| Release workflow macOS env | Desktop build step passes only `GITHUB_TOKEN` into `tauri-action`. | No Developer ID certificate or notarization credentials are available to the macOS build. |
| Certificate import | No workflow step imports a Developer ID certificate into a temporary keychain. | CI cannot prove `codesign` uses a real Developer ID identity. |
| Notarization | No `APPLE_API_*`, `APPLE_ID`, `APPLE_PASSWORD`, or `APPLE_TEAM_ID` variables are wired for macOS builds. | CI cannot submit artifacts to Apple notarization. |
| Stapling | No explicit stapler validation step records proof for app or DMG. | Gatekeeper offline/notarization ticket proof is missing. |
| Validation | Existing `0.2.2-4` local proof records ad-hoc/linker signing and failed `codesign`, `spctl`, and `stapler` checks. | Current macOS artifact remains tester/canary only. |
| Release language | Workflow release notes now say macOS signing and notarization proof is required. README says Beta/Canary. | No current public trust claim was found after PR #19. |

## Apple Asset Inventory

| Asset | Required for signed public macOS release | Current repo status | Required action |
|---|---:|---|---|
| Apple Developer Program membership | Required | Not inferable from repo. | Confirm active membership for the publishing team. |
| Apple Team ID | Required | Not present in repo. | Store as GitHub secret or environment variable used only by release workflow. |
| Developer ID Application certificate | Required | Not present in repo and should not be committed. | Generate/export certificate as password-protected `.p12`; store base64 and password in GitHub secrets. |
| Developer ID Installer certificate | Conditional | Not present in repo and should not be committed. | Required if DAARION ships signed `.pkg` installer packages. Current DMG flow does not prove a need for it, but it should be tracked before adding `.pkg`. |
| App Store Connect API key | Required for preferred CI notarization path | Not present in repo and should not be committed. | Create API key with appropriate access; store issuer ID, key ID, and private key material as secrets. |
| Apple ID app-specific password | Alternative notarization path | Not present in repo. | Avoid unless API key path is unavailable; if used, store `APPLE_ID`, app-specific password, and Team ID as secrets. |
| Dedicated CI keychain | Required implementation detail | Not configured. | Create temporary keychain during GitHub Actions run, import certificate, unlock keychain, delete keychain after build. |
| Hardened runtime / entitlements decision | Required release decision | Not documented in repo-specific terms. | Add explicit entitlements file only if the app needs capabilities beyond default Tauri behavior. |
| Public proof machine | Required for final classification | Not provided. | Validate on clean Apple Silicon and Intel Macs, not only on the build runner. |

## Required GitHub Secrets

Preferred App Store Connect API path:

| Secret | Purpose |
|---|---|
| `APPLE_CERTIFICATE` | Base64-encoded Developer ID Application `.p12`. |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the exported `.p12`. |
| `APPLE_SIGNING_IDENTITY` | Developer ID Application identity name or resolved identity injected into the Tauri build. |
| `APPLE_TEAM_ID` | Apple Developer Team ID. |
| `APPLE_API_ISSUER` | App Store Connect issuer ID. |
| `APPLE_API_KEY` | App Store Connect key ID. |
| `APPLE_API_PRIVATE_KEY` | Private key contents for the App Store Connect key; workflow should write it to a temporary file. |

Derived CI variable:

| Variable | Purpose |
|---|---|
| `APPLE_API_KEY_PATH` | Path to the temporary `.p8` file written during the workflow run. Do not store this as a static repo value. |

Alternative Apple ID path if API key is not available:

| Secret | Purpose |
|---|---|
| `APPLE_ID` | Apple account email for notarization. |
| `APPLE_PASSWORD` | App-specific password. |
| `APPLE_TEAM_ID` | Apple Developer Team ID. |

The API key path is preferred for CI because it avoids tying release automation
to an individual Apple ID password.

## Notarization Readiness Audit

Current workflow gaps:

1. No macOS-only preflight fails the build when public-release signing secrets
   are missing.
2. No Developer ID certificate is imported into a temporary keychain.
3. No `APPLE_SIGNING_IDENTITY` is passed to `tauri-action` for macOS builds.
4. No App Store Connect API key file is reconstructed from GitHub secrets.
5. No notarization credentials are passed to `tauri-action`.
6. No workflow step records `codesign`, `spctl`, `stapler`, or `hdiutil`
   outputs as release evidence.
7. No public release gate prevents a macOS artifact from being promoted from
   tester/canary to public if notarization proof is absent.

Implementation-ready sequence once Apple assets exist:

1. Add GitHub Actions secrets listed above.
2. For macOS matrix jobs, create and unlock a temporary keychain.
3. Import the Developer ID Application certificate into that keychain.
4. Resolve and export the signing identity.
5. Write the App Store Connect private key to `$RUNNER_TEMP`.
6. Pass signing and notarization environment variables to `tauri-action`.
7. Build both `aarch64-apple-darwin` and `x86_64-apple-darwin` DMGs.
8. Run validation commands against the produced app bundles and DMGs.
9. Upload validation logs and checksum assets with the release artifacts.
10. Keep release marked prerelease/tester unless all proof gates pass.

## Gatekeeper Validation Plan

Run these commands against each architecture-specific artifact from the GitHub
release, not against a local development bundle.

Set variables:

```bash
DMG="Daarion.Edge_0.2.2-4_aarch64.dmg"
APP="/Applications/Daarion Edge.app"
MOUNT="/Volumes/Daarion Edge"
```

### 1. DMG integrity

```bash
shasum -a 256 "$DMG"
hdiutil verify "$DMG"
```

Expected evidence:

- checksum matches the published `SHA256SUMS-macos-*.txt`;
- `hdiutil verify` exits `0`;
- output is saved with artifact name, version, macOS version, and machine arch.

### 2. Mount and install

```bash
hdiutil attach "$DMG" -readonly -nobrowse
cp -R "$MOUNT/Daarion Edge.app" /Applications/
hdiutil detach "$MOUNT"
```

Expected evidence:

- mount succeeds;
- app copies to `/Applications`;
- no quarantine bypass command is used.

### 3. Signature inspection

```bash
codesign -dv --verbose=4 "$APP"
codesign --verify --deep --strict --verbose=4 "$APP"
```

Expected public-release evidence:

- authority chain includes Developer ID Application;
- TeamIdentifier matches the DAARION Apple team;
- no `Signature=adhoc`;
- strict verification exits `0`.

### 4. Gatekeeper assessment

```bash
spctl --assess --type execute --verbose=4 "$APP"
spctl --assess --type open --verbose=4 "$DMG"
```

Expected public-release evidence:

- app assessment exits `0`;
- DMG assessment exits `0` or the accepted platform-specific result is
  documented with Apple tooling output;
- no `rejected`, `Insufficient Context`, or unsigned developer warning remains.

### 5. Stapled ticket validation

```bash
xcrun stapler validate "$APP"
xcrun stapler validate "$DMG"
```

Expected public-release evidence:

- stapler validates a ticket for the app and distributed DMG where applicable;
- output is attached to the release proof.

### 6. First-run proof

```bash
open "$APP"
sleep 60
pgrep -fl "Daarion|daarion-edge-client"
tail -120 "$HOME/.daarion-edge/logs/boot.log"
```

Expected public-release evidence:

- Finder launch opens without requiring Privacy & Security override;
- app remains alive for more than 60 seconds;
- UI is nonblank;
- pairing gate and backend health diagnostics are reachable;
- boot log shows setup completed successfully;
- screenshots or manual notes are attached for Apple Silicon and Intel.

## Release Workflow Truth Audit

| File | Finding | Action |
|---|---|---|
| `.github/workflows/release.yml` | macOS release body says tester/canary and signing/notarization proof required. No unproven public trust claim found. | No workflow text change required in this audit. |
| `README.md` | Platform table marks macOS as Beta/Canary with signing and notarization proof required. | No README text change required in this audit. |
| `docs/operations/EDGE_CLIENT_RELEASE_TRUTH.md` | macOS status says DMGs build but Developer ID signing, notarization, Gatekeeper, and first-run proof are blockers. | No text change required in this audit. |
| `docs/release/EDGE_CLIENT_PUBLIC_RELEASE_REPORT.md` | Final verdict is `READY FOR TESTER RELEASE ONLY` and records failed macOS trust checks. | No text change required in this audit. |
| `docs/release/EDGE_CLIENT_RELEASE_READINESS_PLAN.md` | Historical wording still said the current workflow claimed `Release-Signed`. | Updated in this PR to distinguish the previous mismatch from the corrected current workflow. |

## Ready For Signed Public Release Checklist

The macOS release can be classified as:

```text
READY FOR SIGNED PUBLIC RELEASE
```

only when all of these are complete:

1. Apple Developer Program membership is active.
2. Developer ID Application certificate is valid and available to CI only as a
   protected secret.
3. App Store Connect notarization credentials are configured through protected
   secrets.
4. `tauri-action` signs the app with Developer ID for Apple Silicon and Intel.
5. Notarization succeeds for each distributed macOS artifact.
6. Stapling succeeds or an explicit Apple-supported ticket validation path is
   documented for the distributed format.
7. `codesign --verify --deep --strict --verbose=4` exits `0`.
8. `spctl --assess --type execute --verbose=4` exits `0`.
9. `hdiutil verify` exits `0`.
10. Published checksums match downloaded artifacts.
11. Finder launch on clean Apple Silicon and Intel machines does not require a
    manual Gatekeeper bypass.
12. App remains alive for more than 60 seconds.
13. UI is nonblank.
14. Pairing gate is reachable.
15. Backend health diagnostics are reachable.
16. Boot log exists and shows successful setup.
17. Release notes name the exact signing, notarization, stapling, and
    first-run proof artifacts.
18. `1.daarion.city` public download links are updated only after this proof is
    complete.

## Recommendation

Do not publish a signed public macOS release yet.

Next narrow implementation milestone:

```text
release: implement macOS Developer ID signing and notarization workflow
```

Acceptance for that implementation milestone:

- Apple assets are available as protected GitHub secrets;
- release workflow imports the certificate and submits notarization;
- macOS Apple Silicon and Intel artifacts are signed, notarized, and validated;
- validation logs are attached to release evidence;
- final verdict can move from `BLOCKED BY MISSING APPLE ASSETS` to either
  `READY FOR SIGNED PUBLIC RELEASE` or a specific implementation blocker.

Until then:

```text
Foundation-aligned tester build: READY
Public signed release: BLOCKED
Public onboarding: BLOCKED
Local Agent Runtime: BLOCKED
Worker Node onboarding: BLOCKED
```
