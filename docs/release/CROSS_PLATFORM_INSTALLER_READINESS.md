# Cross-Platform Installer Readiness

Date: 2026-06-19

Branch: `codex/cross-platform-installer-readiness`

Milestone: `release: cross-platform installer readiness`

## Final Classification

```text
READY WITH MISSING SIGNING ASSETS
```

The Edge Client release pipeline is close enough to prepare implementation, but
public release remains blocked until platform signing assets, CI secrets, and
real-device install/first-run proof are available.

This is a release-engineering readiness milestone only. It does not publish a
release, create a tag, modify app runtime behavior, update public download
links, start Local Agent Runtime, or start Worker Node work.

## Current Product Release Status

| Area | Status |
|---|---|
| Foundation-aligned tester release | `READY WITH CONDITIONS` |
| Public signed release | `BLOCKED` |
| macOS public trust | `BLOCKED BY MISSING APPLE ASSETS` |
| Windows public trust | `BLOCKED BY SIGNING AND REAL-DEVICE PROOF` |
| Linux public trust | `BLOCKED BY RUNTIME VALIDATION AND PUBLISHER TRUST PROOF` |
| Android public scope | `TESTER-ONLY UNLESS EXPLICITLY KEPT IN PUBLIC RELEASE SCOPE` |
| Public onboarding | `BLOCKED` |
| Local Agent Runtime Foundation | `BLOCKED` |
| Worker Node onboarding | `BLOCKED` |

## Repository Evidence Reviewed

| Area | Evidence |
|---|---|
| App package version | `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json` are aligned at `0.2.2-4`. |
| Tauri config | `src-tauri/tauri.conf.json` has `bundle.targets = "all"` and no platform signing config. |
| Release workflow | `.github/workflows/release.yml` builds macOS ARM, macOS Intel, Linux x86_64, Windows x86_64, and Android arm64 matrix jobs. |
| Artifact staging | `scripts/prepare-release-artifacts.cjs` renames artifacts and writes `release-manifest-<platform>.json` plus `SHA256SUMS-<platform>.txt`. |
| Release status docs | `docs/release/EDGE_CLIENT_PUBLIC_RELEASE_REPORT.md`, `docs/release/MACOS_SIGNING_AND_NOTARIZATION_READINESS.md`, `docs/release/EDGE_CLIENT_RELEASE_CHECKLIST.md`. |
| Prior public artifact audit | `docs/audits/EDGE_CLIENT_INSTALLER_AND_FIRST_RUN_AUDIT.md`. |
| Canonical artifact names | `docs/operations/EDGE_CLIENT_RELEASE_TRUTH.md`. |

External reference material checked on 2026-06-19:

- Tauri macOS signing: <https://v2.tauri.app/distribute/sign/macos/>
- Tauri Windows signing: <https://v2.tauri.app/distribute/sign/windows/>
- Tauri AppImage: <https://v2.tauri.app/distribute/appimage/>
- Tauri Linux signing: <https://v2.tauri.app/distribute/sign/linux/>
- Tauri Debian packaging: <https://v2.tauri.app/distribute/debian/>
- Microsoft Smart App Control signing: <https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control>
- Microsoft Artifact Signing / Trusted Signing guide: <https://learn.microsoft.com/en-us/windows/msix/package/sign-msix-package-guide>
- Android APK signing: <https://developer.android.com/studio/publish/app-signing>
- Android `apksigner`: <https://developer.android.com/tools/apksigner>

## Current Release Workflow Map

Current matrix from `.github/workflows/release.yml`:

| Matrix label | Runner | Build args | Staged public artifact |
|---|---|---|---|
| `macos-arm64` | `macos-latest` | `--target aarch64-apple-darwin` | `Daarion.Edge_<version>_aarch64.dmg` |
| `macos-x86_64` | `macos-latest` | `--target x86_64-apple-darwin` | `Daarion.Edge_<version>_x64.dmg` |
| `linux-x86_64` | `ubuntu-22.04` | none | `Daarion.Edge_<version>_amd64.AppImage` |
| `windows-x86_64` | `windows-latest` | none | `Daarion.Edge_<version>_x64-setup.exe`, `Daarion.Edge_<version>_x64_en-US.msi` |
| `android-arm64` | `ubuntu-22.04` | `--target aarch64-linux-android` | `Daarion.Edge_<version>_android_universal_release.apk` |

Current workflow strengths:

- artifact naming is centralized in `scripts/prepare-release-artifacts.cjs`;
- per-platform checksum files are generated;
- release notes now distinguish build existence from install/boot/worker proof;
- Android has a fail-closed secret guard before release signing.

Current workflow gaps:

- macOS jobs have no Developer ID certificate import, notarization credentials,
  stapling, or Gatekeeper evidence upload;
- Windows jobs have no Authenticode signing step for NSIS or MSI artifacts;
- Windows SmartScreen behavior is not captured from a clean machine;
- Linux AppImage proof is not captured from a real Linux desktop;
- AppImage signature or separate GPG/checksum publisher trust is not configured;
- Android is present in the release matrix but remains tester-only until
  signature, install, launch, and log proof are attached;
- no workflow gate distinguishes tester publication from signed public release.

## macOS Readiness

Current state:

- macOS DMGs are produced by the release workflow.
- `src-tauri/tauri.conf.json` does not define macOS signing or entitlements.
- release workflow does not import Apple certificates or pass Apple
  notarization variables to the Tauri build.
- previous local proof shows ad-hoc/linker signing and failed `codesign`,
  `spctl`, and `stapler` checks.

Required assets:

- Apple Developer Program access;
- Apple Team ID;
- Developer ID Application certificate exported as protected `.p12`;
- App Store Connect API key for notarization, or Apple ID app-specific password
  fallback;
- GitHub Actions secrets documented in
  `docs/release/RELEASE_SECRETS_AND_SIGNING_MATRIX.md`.

Required implementation work:

1. Add macOS-only CI preflight for public signing mode.
2. Create and unlock a temporary keychain in macOS jobs.
3. Import the Developer ID Application `.p12`.
4. Pass `APPLE_SIGNING_IDENTITY` and notarization credentials to
   `tauri-apps/tauri-action`.
5. Build both Apple Silicon and Intel DMGs.
6. Run and preserve `codesign`, `spctl`, `stapler`, `hdiutil`, checksum, and
   first-run evidence.
7. Keep releases as prerelease/tester unless all proof gates pass.

Validation commands:

```bash
hdiutil verify Daarion.Edge_0.2.2-4_aarch64.dmg
codesign -dv --verbose=4 "/Applications/Daarion Edge.app"
codesign --verify --deep --strict --verbose=4 "/Applications/Daarion Edge.app"
spctl --assess --type execute --verbose=4 "/Applications/Daarion Edge.app"
spctl --assess --type open --verbose=4 Daarion.Edge_0.2.2-4_aarch64.dmg
xcrun stapler validate "/Applications/Daarion Edge.app"
xcrun stapler validate Daarion.Edge_0.2.2-4_aarch64.dmg
open "/Applications/Daarion Edge.app"
sleep 60
tail -120 "$HOME/.daarion-edge/logs/boot.log"
```

Public gate:

```text
macOS public release is blocked until Apple Silicon and Intel both pass
Developer ID signing, notarization, Gatekeeper, stapling, DMG verification, and
first-run proof from downloaded release artifacts.
```

## Windows Readiness

Current state:

- release workflow runs a `windows-latest` job and stages both NSIS
  `setup.exe` and WiX/MSI outputs;
- no Windows signing step is configured;
- no `bundle.windows.signCommand` is configured in `src-tauri/tauri.conf.json`;
- no Windows code signing secrets are referenced;
- previous audits covered static installer metadata but not real install,
  SmartScreen, WebView2, shortcut, uninstall, or first-run proof.

Signing options:

| Option | Use case | Repo impact |
|---|---|---|
| Azure Artifact Signing / Trusted Signing | Preferred for CI if available in the operator's region and account type. | Add Azure signing secrets, install signing tool/action, sign `.exe` and `.msi`, verify with SignTool. |
| OV/EV code signing certificate | Fallback if Azure Artifact Signing is unavailable. | Store certificate securely, sign with `signtool`, timestamp signatures, verify. |
| Microsoft Store | Future distribution path only. | Not in current GitHub release artifact flow. |

SmartScreen note:

- Code signing is required for a serious public Windows installer, but signing
  alone does not guarantee instant SmartScreen reputation for a new publisher.
  Release evidence must record the first-run prompt behavior on clean Windows
  machines.

Required implementation work:

1. Choose Windows signing path: Azure Artifact Signing or OV/EV certificate.
2. Add signing secrets and a Windows signing step after Tauri build and before
   artifact staging or upload.
3. Sign both `Daarion.Edge_<version>_x64-setup.exe` and
   `Daarion.Edge_<version>_x64_en-US.msi`.
4. Run `signtool verify` on both artifacts.
5. Record SmartScreen behavior, WebView2 bootstrapper behavior, shortcuts,
   uninstall entry, launch, liveness, and boot log proof.

Validation commands:

```powershell
Get-FileHash .\Daarion.Edge_0.2.2-4_x64-setup.exe -Algorithm SHA256
Get-FileHash .\Daarion.Edge_0.2.2-4_x64_en-US.msi -Algorithm SHA256
signtool verify /pa /v .\Daarion.Edge_0.2.2-4_x64-setup.exe
signtool verify /pa /v .\Daarion.Edge_0.2.2-4_x64_en-US.msi
Start-Process .\Daarion.Edge_0.2.2-4_x64-setup.exe -Wait
Start-Sleep -Seconds 60
Get-Content "$env:APPDATA\DAARION Edge\logs\boot.log" -Tail 120
```

Public gate:

```text
Windows public release is blocked until setup.exe and MSI are signed,
timestamped, verified, installed on clean Windows, launched, kept alive for
more than 60 seconds, and documented with SmartScreen/WebView2 evidence.
```

## Linux Readiness

Current state:

- release workflow runs `ubuntu-22.04` and installs WebKitGTK and related build
  dependencies;
- `scripts/prepare-release-artifacts.cjs` stages only the AppImage artifact;
- `src-tauri/tauri.conf.json` uses `bundle.targets = "all"`, but release docs
  and public artifact naming currently treat AppImage as the Linux public
  package;
- no AppImage signing or detached GPG signature is configured;
- no Linux desktop launch proof exists for `0.2.2-4`.

Policy recommendation:

- Keep the Linux public release scope to AppImage for now.
- Treat `.deb`/`.rpm` as out of public scope until artifact staging, docs,
  dependency testing, and install/remove proof are added.

Required implementation work:

1. Validate the AppImage on at least one clean Ubuntu/Debian desktop.
2. Decide whether Linux publisher trust uses:
   - checksums only for tester/canary;
   - detached GPG signatures;
   - AppImage embedded signature with documented manual validation.
3. If AppImage signing is used, configure `SIGN=1`,
   `APPIMAGETOOL_SIGN_PASSPHRASE`, and related signing variables in CI.
4. Record dependency/FUSE behavior and fallback instructions.
5. Record launch, nonblank UI, process liveness, and boot log proof.

Validation commands:

```bash
sha256sum Daarion.Edge_0.2.2-4_amd64.AppImage
chmod +x Daarion.Edge_0.2.2-4_amd64.AppImage
./Daarion.Edge_0.2.2-4_amd64.AppImage &
sleep 60
pgrep -fl "daarion|Daarion"
tail -120 "$HOME/.daarion-edge/logs/boot.log"
```

If AppImage signing is enabled:

```bash
./Daarion.Edge_0.2.2-4_amd64.AppImage --appimage-signature
```

Public gate:

```text
Linux public release is blocked until the AppImage launches on a clean Linux
desktop, stays alive, writes logs, and publisher trust evidence is documented.
```

## Android Readiness

Current state:

- Android is included in the release matrix as `android-arm64`.
- The workflow already guards for Android signing secrets:
  `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`,
  `ANDROID_KEY_ALIAS`, and `ANDROID_KEY_PASSWORD`.
- The workflow reconstructs a keystore and patches Gradle release signing
  config before `cargo tauri android build --apk`.
- Android remains tester/canary in release notes until signature and device
  proof are attached.

Policy recommendation:

- Android should remain out of normal public onboarding unless the owner
  explicitly keeps Android in public release scope.
- If Android remains in scope, it must not block desktop public release unless
  public pages expose Android as a normal user path.

Required proof if Android is kept:

```bash
apksigner verify --verbose Daarion.Edge_0.2.2-4_android_universal_release.apk
adb install -r Daarion.Edge_0.2.2-4_android_universal_release.apk
adb shell monkey -p city.daarion.edge 1
adb logcat -d | tail -200
```

Public gate:

```text
Android public release is blocked until APK signature verification, sideload
install, launch, and crash-log proof are attached. Otherwise keep Android
tester-only or remove it from normal public onboarding.
```

## Release Workflow Readiness Plan

Minimal implementation order:

1. Keep `0.2.2-4` prerelease/tester publication separate from public stable.
2. Add platform signing modes as explicit CI inputs or environment gates:
   - tester/canary: build unsigned or partially signed artifacts with truthful
     warning labels;
   - public: fail closed unless required signing secrets and validation steps
     pass.
3. Implement macOS signing and notarization first because it is the current
   highest-friction public onboarding blocker.
4. Implement Windows signing second, choosing Azure Artifact Signing or OV/EV
   certificate path.
5. Validate Linux AppImage execution and decide whether GPG/AppImage signature
   is required for public trust.
6. Decide Android public scope; keep tester-only if not needed for immediate
   onboarding.
7. Attach checksum files and proof logs to every release candidate.
8. Update `1.daarion.city` download links only after the selected public
   platform gates pass.

## Required Evidence Before Public Release

| Platform | Required evidence before public onboarding |
|---|---|
| macOS ARM | Signed app, notarization accepted, stapled ticket, Gatekeeper accepted, DMG verified, Finder launch, liveness, boot log, nonblank UI. |
| macOS Intel | Same as macOS ARM, recorded on Intel hardware or approved equivalent. |
| Windows setup.exe | Signed and timestamped installer, SignTool verification, SmartScreen observation, install, shortcuts, launch, liveness, boot log, uninstall. |
| Windows MSI | Signed and timestamped MSI, SignTool verification, install, Windows uninstall entry, launch, liveness, boot log, uninstall. |
| Linux AppImage | Checksum, executable bit, dependency/FUSE behavior, launch, liveness, boot log, nonblank UI, publisher trust policy. |
| Android APK | APK signature verification, `adb install`, launch, crash-log check, policy decision on public vs tester-only. |

## Not In Scope

- release publication;
- release tags;
- public download link changes;
- Local Agent Runtime Foundation;
- Worker Node onboarding;
- Genesis expansion;
- onboarding redesign;
- backend, pairing, or identity changes;
- real signing secret installation.

## Recommendation

Proceed with implementation only after the platform owners provide the missing
signing assets and confirm the public platform set.

Next narrow PR after this readiness document:

```text
release: implement macOS Developer ID signing workflow
```

That PR should still avoid public release publication. Its acceptance should be
a signed/notarized macOS candidate plus attached proof, not a new feature.
