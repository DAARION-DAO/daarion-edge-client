# DAARION Edge Client Release Checklist

Use this checklist for the next foundation-aligned Edge Client release.

Target default:

```text
Version: 0.2.2-4
Tag: v0.2.2-4
Mode: prerelease/tester first
Public onboarding: blocked until trust and full product smoke pass
```

## 1. Preflight

- [ ] Confirm `main` is up to date.
- [ ] Confirm no newer tag supersedes `v0.2.2-3`.
- [ ] Confirm Local Agent Runtime and Worker Node UX work are out of scope.
- [ ] Confirm `docs/audits/EDGE_CLIENT_INSTALLER_AND_FIRST_RUN_AUDIT.md` is reviewed.
- [ ] Confirm release notes will not claim `Release-Signed` unless signing and notarization are proven.

## 2. Version Truth

- [ ] Bump `package.json` to `0.2.2-4`.
- [ ] Bump root version in `package-lock.json` to `0.2.2-4`.
- [ ] Bump `src-tauri/Cargo.toml` to `0.2.2-4`.
- [ ] Bump `src-tauri/Cargo.lock` package entry to `0.2.2-4`.
- [ ] Bump `src-tauri/tauri.conf.json` to `0.2.2-4`.
- [ ] Verify app UI/version reporting shows `0.2.2-4`.
- [ ] Verify artifact names use `Daarion.Edge_0.2.2-4_*`.

## 3. Pre-Release Validation

- [ ] `npm run build`
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] `git diff --check`
- [ ] `bash scripts/check-no-secrets.sh`
- [ ] `bash scripts/check-rust-touched-warnings.sh`
- [ ] Record warning count and confirm protected modules remain clean.

## 4. Release Workflow Readiness

- [ ] Release workflow builds macOS Apple Silicon DMG.
- [ ] Release workflow builds macOS Intel DMG.
- [ ] Release workflow builds Windows setup.exe.
- [ ] Release workflow builds Windows MSI.
- [ ] Release workflow builds Linux AppImage.
- [ ] Release workflow builds Android APK if Android remains in this release.
- [ ] Release workflow publishes official SHA-256 checksum asset.
- [ ] Release workflow publishes platform release manifests.
- [ ] Release notes are proof-based and do not overstate validation.

## 5. Trust And Signing

### macOS

- [ ] Apple Silicon app is Developer ID signed or explicitly marked unsigned/tester-only.
- [ ] Intel app is Developer ID signed or explicitly marked unsigned/tester-only.
- [ ] `codesign --verify --deep --strict` passes for each public macOS app.
- [ ] `spctl` accepts each public macOS app/DMG.
- [ ] `stapler validate` passes for each public macOS app/DMG.
- [ ] Gatekeeper behavior is recorded from Finder launch.

### Windows

- [ ] SmartScreen behavior is recorded for setup.exe.
- [ ] SmartScreen behavior is recorded for MSI.
- [ ] WebView2 bootstrapper behavior is recorded on a clean machine.

### Android

- [ ] APK signature is verified with Android tooling.
- [ ] Android signing key identity is recorded without exposing secrets.

## 6. Artifact Inventory

For each artifact, record URL, size, content type, checksum, and verdict.

- [ ] `Daarion.Edge_0.2.2-4_aarch64.dmg`
- [ ] `Daarion.Edge_0.2.2-4_x64.dmg`
- [ ] `Daarion.Edge_0.2.2-4_x64-setup.exe`
- [ ] `Daarion.Edge_0.2.2-4_x64_en-US.msi`
- [ ] `Daarion.Edge_0.2.2-4_amd64.AppImage`
- [ ] `Daarion.Edge_0.2.2-4_android_universal_release.apk`
- [ ] `SHA256SUMS` or equivalent checksum asset
- [ ] platform release manifests

## 7. Platform Install Proof

### macOS Apple Silicon

- [ ] Download from GitHub release link.
- [ ] Verify checksum.
- [ ] `hdiutil verify` passes.
- [ ] Mount DMG.
- [ ] Copy app to Applications on clean test account or machine.
- [ ] Launch via Finder.
- [ ] Record Gatekeeper behavior.
- [ ] App window opens and is not blank.
- [ ] App stays alive for more than 60 seconds.
- [ ] Boot log exists and ends with successful setup.
- [ ] Clean uninstall/removal notes recorded.

### macOS Intel

- [ ] Repeat Apple Silicon checklist on real Intel Mac or equivalent approved runner.

### Windows setup.exe

- [ ] Download from GitHub release link.
- [ ] Verify checksum.
- [ ] Launch installer.
- [ ] Record SmartScreen behavior.
- [ ] Installation completes.
- [ ] Desktop shortcut exists if expected.
- [ ] Start Menu shortcut exists.
- [ ] App launches and is not blank.
- [ ] App stays alive for more than 60 seconds.
- [ ] `%APPDATA%\DAARION Edge\logs\boot.log` exists and ends with successful setup.
- [ ] Uninstall entry exists and uninstall succeeds.

### Windows MSI

- [ ] Repeat Windows proof for MSI installer.

### Linux AppImage

- [ ] Download from GitHub release link.
- [ ] Verify checksum.
- [ ] `chmod +x` succeeds.
- [ ] Launch on Ubuntu/Debian desktop.
- [ ] Record FUSE/dependency behavior.
- [ ] App window opens and is not blank.
- [ ] App stays alive for more than 60 seconds.
- [ ] `~/.daarion-edge/logs/boot.log` exists and ends with successful setup.

### Android APK

- [ ] Download from GitHub release link.
- [ ] Verify checksum.
- [ ] Verify APK signature.
- [ ] `adb install` succeeds on target Android version.
- [ ] App launches.
- [ ] No immediate crash in logcat.

## 8. Product Smoke From Public Links

Run after artifacts are available from GitHub release links.

- [ ] Fresh install opens first-run flow.
- [ ] No pairing shows `pairing_required`.
- [ ] Valid paired backend health `ok` shows `online`.
- [ ] Valid paired backend health `degraded` shows `degraded`.
- [ ] Valid paired backend health `maintenance` shows `maintenance`.
- [ ] Bad backend shows `offline`.
- [ ] Invalid health contract shows `contract_invalid`.
- [ ] Unsupported schema/protocol/client version shows `version_mismatch`.
- [ ] Real invitation code pairs the device.
- [ ] Genesis can complete with real provisioning backend.
- [ ] Dashboard is reachable.
- [ ] User-facing flow does not require knowledge of backend, registry, localhost, repo split, or runtime internals.

## 9. Public Link Update Gate

Do not update `1.daarion.city` direct download links until all selected public
platform gates pass.

- [ ] GitHub release is foundation-aligned.
- [ ] Release notes are truthful.
- [ ] Checksums are published.
- [ ] Public platform artifacts have install and boot proof.
- [ ] Product smoke passes from public links.
- [ ] Rollback target is documented.

## 10. Final Verdict

Choose exactly one:

- [ ] `READY`: signed/trusted artifacts and full product smoke passed.
- [ ] `READY WITH CONDITIONS`: tester/canary only; public onboarding remains blocked.
- [ ] `NOT READY`: blockers prevent even tester/canary publication.

Current expected verdict before the next release execution:

```text
Official public release: NOT READY
Unsigned tester/canary release: READY WITH CONDITIONS
Local Agent Runtime Foundation: BLOCKED
```
