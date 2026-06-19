# macOS Signing Checklist

Use this checklist only for the DAARION Edge Client macOS public release gate.
It does not authorize Local Agent Runtime, Worker Node, Genesis, onboarding, or
backend work.

## 1. Apple Account And Certificates

- [ ] Apple Developer Program membership is active.
- [ ] Apple Team ID is recorded in the release operator vault.
- [ ] Developer ID Application certificate exists and is valid.
- [ ] Developer ID Application certificate is exported as password-protected
      `.p12`.
- [ ] `.p12` is base64-encoded for CI secret storage.
- [ ] `.p12` password is stored as a protected CI secret.
- [ ] Developer ID Installer certificate is created only if DAARION ships a
      signed `.pkg` installer.
- [ ] No certificate, private key, or `.p12` file is committed to the repo.

## 2. Notarization Credentials

Preferred App Store Connect API path:

- [ ] App Store Connect API key exists.
- [ ] API issuer ID is stored as `APPLE_API_ISSUER`.
- [ ] API key ID is stored as `APPLE_API_KEY`.
- [ ] API private key content is stored as `APPLE_API_PRIVATE_KEY`.
- [ ] Workflow writes API private key to a temporary `$RUNNER_TEMP/*.p8` file.
- [ ] Workflow sets `APPLE_API_KEY_PATH` to that temporary file path.

Alternative Apple ID path:

- [ ] `APPLE_ID` is stored only if API key path is unavailable.
- [ ] `APPLE_PASSWORD` is an app-specific password.
- [ ] `APPLE_TEAM_ID` is stored.

## 3. GitHub Actions Wiring

- [ ] macOS jobs fail closed for public release when Apple signing secrets are
      missing.
- [ ] macOS jobs create a temporary keychain.
- [ ] macOS jobs import the Developer ID Application certificate.
- [ ] macOS jobs unlock the temporary keychain for codesign.
- [ ] macOS jobs pass `APPLE_CERTIFICATE`.
- [ ] macOS jobs pass `APPLE_CERTIFICATE_PASSWORD`.
- [ ] macOS jobs pass `APPLE_SIGNING_IDENTITY`.
- [ ] macOS jobs pass notarization credentials.
- [ ] Temporary keychain and private key files are deleted in `always()` cleanup.
- [ ] Logs never print certificate contents, passwords, or private key material.

## 4. Build Outputs

- [ ] Apple Silicon DMG is produced from the release workflow.
- [ ] Intel DMG is produced from the release workflow.
- [ ] Artifact names match canonical release naming.
- [ ] `SHA256SUMS-macos-arm64.txt` is uploaded.
- [ ] `SHA256SUMS-macos-x86_64.txt` is uploaded.
- [ ] Release manifests are uploaded if generated.

## 5. Signature Proof

Run for each downloaded release artifact:

```bash
codesign -dv --verbose=4 "/Applications/Daarion Edge.app"
codesign --verify --deep --strict --verbose=4 "/Applications/Daarion Edge.app"
```

- [ ] Signature is not ad-hoc.
- [ ] Authority chain includes Developer ID Application.
- [ ] TeamIdentifier matches the DAARION Apple team.
- [ ] Strict verification exits `0`.
- [ ] Output is saved as release evidence.

## 6. Notarization And Stapling Proof

```bash
xcrun stapler validate "/Applications/Daarion Edge.app"
xcrun stapler validate "Daarion.Edge_0.2.2-4_aarch64.dmg"
```

- [ ] App stapler validation passes.
- [ ] DMG stapler validation passes or an Apple-supported alternate validation
      path is documented.
- [ ] Notarization submission logs show accepted status.
- [ ] Notarization proof is attached to the release report.

## 7. Gatekeeper Proof

```bash
spctl --assess --type execute --verbose=4 "/Applications/Daarion Edge.app"
spctl --assess --type open --verbose=4 "Daarion.Edge_0.2.2-4_aarch64.dmg"
```

- [ ] App assessment exits `0`.
- [ ] DMG assessment exits `0` or accepted platform behavior is documented.
- [ ] Finder launch does not require Privacy & Security override.
- [ ] No "unidentified developer" bypass is required.

## 8. First-Run Proof

- [ ] Fresh clean account or clean machine is used.
- [ ] DMG verifies with `hdiutil verify`.
- [ ] App installs into `/Applications`.
- [ ] App launches via Finder.
- [ ] App remains alive for more than 60 seconds.
- [ ] UI is not blank.
- [ ] Pairing gate is reachable.
- [ ] Backend health diagnostics are reachable.
- [ ] `~/.daarion-edge/logs/boot.log` exists.
- [ ] Boot log shows setup completed successfully.
- [ ] Apple Silicon proof is recorded.
- [ ] Intel proof is recorded.

## 9. Release Classification

Choose exactly one:

- [ ] `READY FOR SIGNED PUBLIC RELEASE`
- [ ] `READY FOR TESTER RELEASE ONLY`
- [ ] `NOT READY FOR RELEASE`

Do not choose `READY FOR SIGNED PUBLIC RELEASE` unless every signature,
notarization, Gatekeeper, installer, and first-run item above is complete.
