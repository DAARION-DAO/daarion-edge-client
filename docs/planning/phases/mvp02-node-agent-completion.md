# MVP02 Node Agent completion

## Implemented
A macOS one-shot `daarion-node init` / `observe CONFIG` binary reuses the existing
identity service, UUID/public-key metadata and capability scanner. `observe` does
not initialize identity implicitly. Native Keychain backend is enabled for the
existing keyring crate; lockfile adds its Apple security-framework dependency.
No GUI, worker, inference, wallet or membership flow is activated.

## Verification
- Rust 1.95 build of the new binary: PASS.
- Existing identity unit tests: 5 PASS, including missing/mismatched key and signatures.
- Rust 1.95 clippy for the binary: PASS with existing library warnings; no unrelated fixes.
- New binary formatted with Rust 1.95 rustfmt.
- Real identity initialization and separate-process reopen: same public metadata.
- Real signed HTTPS observation accepted by isolated City registry and projected
  CURRENT with hardware/runtime facts and explicit no-inference limitations.
- City verification tests cover wrong node/device/audience, forged signature,
  stale/future time, digest mismatch, replay/order, revocation and restart.

The local shell initially selected an older compiler despite rustup; running the
installed 1.95 toolchain explicitly resolved it. No toolchain/source workaround.

## Security and compatibility
Private key never leaves existing OS key store. No unrestricted signing command,
no redirects, no inference, fixed loopback read-only probes, bounded timeouts.
The new City domain is operator-approved and separate from legacy worker registry.
One headless entry is added; existing Tauri commands and metadata remain compatible.
macOS only; other platforms fail explicitly. No frontend changes, so frontend
checks are outside this native-only slice. No native GUI package is replaced.

## Gate
Source checks and authenticated transport PASS. Final independent exact-head
review, merge and production Office acceptance are separate delivery gates, recorded
in the task receipt. Do not infer production completion from this document.

## Rollback
Stop the managed observation invocation and restore previous City binding.
Preserve device identity and enrollment evidence; never delete keys on rollback.
