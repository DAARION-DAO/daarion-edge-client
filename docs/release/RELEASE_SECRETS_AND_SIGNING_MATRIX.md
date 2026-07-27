# Release Secrets And Signing Matrix

Date: 2026-06-19

Purpose: define the secrets, assets, and proof required to move DAARION Edge
Client from tester/canary distribution to public trusted installer
distribution.

Do not commit real certificates, private keys, keystores, passwords, tokens, or
API private keys to this repository. This document names required secret slots
only.

## Final Classification

```text
READY WITH MISSING SIGNING ASSETS
```

The repository has artifact naming, staging, checksums, and release docs in
place. Public release remains blocked by missing signing assets, missing CI
secret wiring, and missing real-device proof.

## Platform Secret Matrix

| Platform | Release status | Required assets | Current repo state |
|---|---|---|---|
| macOS | Blocked for public release | Apple Developer Program, Developer ID Application certificate, Apple Team ID, notarization credentials. | No Apple signing/notarization secrets wired into workflow. |
| Windows | Blocked for public release | Azure Artifact Signing account or OV/EV code signing certificate, timestamping, verification. | No Windows signing secrets or signing step wired into workflow. |
| Linux | Blocked for public release proof | Checksum publication plus optional GPG/AppImage signing key and clean desktop runtime proof. | Checksums generated; no Linux publisher signature or launch proof. |
| Android | Tester-only unless explicitly kept public | Android release keystore, passwords, alias, signature verification, device install proof. | Workflow has Android signing secret names and guard; proof is still pending. |

## macOS Secrets

Preferred App Store Connect API notarization path:

| Secret | Required | Description |
|---|---:|---|
| `APPLE_CERTIFICATE` | Yes | Base64-encoded Developer ID Application `.p12`. |
| `APPLE_CERTIFICATE_PASSWORD` | Yes | Password for the exported `.p12`. |
| `APPLE_SIGNING_IDENTITY` | Yes | Developer ID Application identity name or resolved identity used by Tauri/codesign. |
| `APPLE_TEAM_ID` | Yes | Apple Developer Team ID. |
| `APPLE_API_ISSUER` | Yes | App Store Connect issuer ID. |
| `APPLE_API_KEY` | Yes | App Store Connect key ID. |
| `APPLE_API_PRIVATE_KEY` | Yes | Private `.p8` key content; workflow writes it to `$RUNNER_TEMP`. |

Derived runtime value:

| Variable | Description |
|---|---|
| `APPLE_API_KEY_PATH` | Temporary path to the `.p8` file created during CI. Do not store as a static secret. |

Alternative notarization path:

| Secret | Required if using Apple ID path | Description |
|---|---:|---|
| `APPLE_ID` | Conditional | Apple account email. |
| `APPLE_PASSWORD` | Conditional | App-specific password. |
| `APPLE_TEAM_ID` | Conditional | Apple Developer Team ID. |

Proof required:

- `codesign -dv --verbose=4`;
- `codesign --verify --deep --strict --verbose=4`;
- `spctl --assess --type execute --verbose=4`;
- `spctl --assess --type open --verbose=4`;
- `xcrun stapler validate`;
- `hdiutil verify`;
- clean Finder first launch and 60-second liveness proof.

## Windows Secrets

### Option A: Azure Artifact Signing / Trusted Signing

| Secret | Required | Description |
|---|---:|---|
| `AZURE_TENANT_ID` | Yes | Azure tenant ID for signing identity. |
| `AZURE_CLIENT_ID` | Yes | App registration client ID. |
| `AZURE_CLIENT_SECRET` | Yes | App registration client secret. |
| `AZURE_TRUSTED_SIGNING_ENDPOINT` | Yes | Artifact Signing endpoint URI. |
| `AZURE_CODE_SIGNING_NAME` | Yes | Artifact Signing account name. |
| `AZURE_CERT_PROFILE_NAME` | Yes | Certificate profile name. |

Required workflow support:

- install or use supported Artifact Signing tooling;
- sign both `.exe` and `.msi` artifacts;
- timestamp signatures;
- verify signatures with SignTool;
- store verification output as release evidence.

### Option B: OV/EV Certificate

| Secret | Required | Description |
|---|---:|---|
| `WINDOWS_SIGNING_CERTIFICATE` | Yes | Base64-encoded `.pfx` certificate. |
| `WINDOWS_SIGNING_CERTIFICATE_PASSWORD` | Yes | Password for `.pfx`. |
| `WINDOWS_SIGNING_CERTIFICATE_THUMBPRINT` | Optional | Certificate selector if imported into a certificate store. |
| `WINDOWS_TIMESTAMP_URL` | Yes | RFC 3161 timestamp server URL. |

Proof required:

```powershell
signtool verify /pa /v .\Daarion.Edge_0.2.2-4_x64-setup.exe
signtool verify /pa /v .\Daarion.Edge_0.2.2-4_x64_en-US.msi
```

SmartScreen note:

- Signed artifacts may still show SmartScreen prompts for a new publisher until
  the publisher reputation is established. Record observed behavior; do not
  claim SmartScreen trust without evidence.

## Linux Secrets

Linux public trust can remain checksum-only for tester/canary. For stronger
publisher trust, choose one of these:

| Secret | Required when used | Description |
|---|---:|---|
| `LINUX_GPG_PRIVATE_KEY` | Conditional | Base64 or armored GPG private key for detached signatures. |
| `LINUX_GPG_KEY_ID` | Conditional | Public signing key ID. |
| `LINUX_GPG_PASSPHRASE` | Conditional | GPG key passphrase. |
| `APPIMAGETOOL_SIGN_PASSPHRASE` | Conditional | AppImage embedded-signature passphrase. |
| `SIGN_KEY` | Conditional | AppImage signing key ID. |

Tauri/AppImage signing variables when using embedded AppImage signatures:

| Variable | Value |
|---|---|
| `SIGN` | `1` |
| `APPIMAGETOOL_FORCE_SIGN` | `1` for fail-closed CI behavior |
| `SIGN_KEY` | Selected GPG key ID |
| `APPIMAGETOOL_SIGN_PASSPHRASE` | Signing key password |

Proof required:

- published checksum asset;
- optional detached `.sig` or AppImage embedded signature output;
- clean Linux desktop AppImage launch;
- FUSE/dependency behavior recorded;
- 60-second liveness and `~/.daarion-edge/logs/boot.log` proof.

## Android Secrets

Current workflow already names these Android signing secrets:

| Secret | Required | Description |
|---|---:|---|
| `ANDROID_KEYSTORE_BASE64` | Yes if Android APK is built | Base64-encoded release keystore. |
| `ANDROID_KEYSTORE_PASSWORD` | Yes if Android APK is built | Keystore password. |
| `ANDROID_KEY_ALIAS` | Yes if Android APK is built | Release key alias. |
| `ANDROID_KEY_PASSWORD` | Yes if Android APK is built | Release key password. |

Proof required:

```bash
apksigner verify --verbose Daarion.Edge_0.2.2-4_android_universal_release.apk
adb install -r Daarion.Edge_0.2.2-4_android_universal_release.apk
adb shell monkey -p city.daarion.edge 1
adb logcat -d | tail -200
```

Policy decision:

- If Android is not part of immediate public onboarding, keep it tester-only or
  remove it from normal public install language.
- If Android remains public, missing APK proof blocks Android publication but
  should not automatically block desktop publication unless the public release
  is advertised as all-platform ready.

## Public Release Mode Gates

### Tester / Canary Mode

Allowed when:

- artifacts are built from current `main`;
- release notes clearly state untrusted or proof-pending status per platform;
- checksums are published;
- public download links are not switched to tester artifacts;
- Local Agent Runtime and Worker Node remain blocked.

### Signed Public Mode

Allowed only when:

- selected public platforms have signing assets wired into CI;
- public artifacts are signed where platform trust requires it;
- checksums and signature proof are attached;
- clean-machine install and first-run proof are recorded;
- `1.daarion.city` download links are updated only after proof passes;
- rollback target is documented.

## Minimum Next Implementation PRs

Recommended order:

1. `release: implement macOS Developer ID signing workflow`
2. `release: implement Windows installer signing workflow`
3. `release: validate Linux AppImage runtime and publisher trust`
4. `release: decide Android public scope`

Do not start product feature milestones until the selected public installer
surface is current, trusted, and validated.
