# Phase 1B.1 Rust Toolchain Unblock — Completion Evidence

Status: **CONDITIONAL_PASS / DRAFT REVIEW PENDING / PHASE 1B.1 NO_GO**

## Result

```text
RUST_TOOLCHAIN_FLOOR =
1.95.0 / IMPLEMENTED / TESTED / REVIEW_PENDING

RUSQLITE_0_40_1_COMPATIBILITY =
PROBE_PASS

RUSQLITE_REPOSITORY_DEPENDENCY =
NOT_ADDED

PHASE_1B_1 =
BLOCKED_PENDING_TOOLCHAIN_PR_MERGE

PHASE_1B_2 =
NOT_AUTHORIZED

PRODUCTION_WRITES = 0
DEPLOYMENTS = 0
```

This change establishes only the repository-wide Rust toolchain contract. It
does not implement SQLite, migrations, durable runtime state, memory, or a
later Phase 1B slice.

## Starting state and preflight

- Repository: `DAARION-DAO/daarion-edge-client`
- Starting `origin/main`: `0e6ff6ada0dd967b6543f3a534f756787c916c42`
- PR #25: merged as the starting main commit and reachable from `origin/main`
- Branch: `build/rust-1.95-toolchain-floor`
- Conflicting local/remote branch or PR before work: none
- Starting toolchain on the host: Homebrew `rustc 1.94.1` and
  `cargo 1.94.1`

The source worktree was created cleanly from the exact starting main. The
separate blocked Phase 1B.1 worktree remained on
`phase-01b1/storage-bootstrap-migrations` at the same starting commit.

## Original dependency blocker

The authorized Phase 1B plan selects exact `rusqlite 0.40.1` with
`default-features = false` and only `bundled`, `limits`, and `backup`.
That dependency resolves `libsqlite3-sys 0.38.1`, whose build script uses
`cfg_select!`. Rust 1.94.1 reports `E0658` for that macro. The Rust 1.95.0
release stabilizes `cfg_select!`, so the accepted dependency cannot be
evaluated under the former implicit host toolchain.

The binding human decision is:

```text
RUST_TOOLCHAIN_CHANNEL = 1.95.0
RUST_MINIMUM_SUPPORTED_VERSION = 1.95
RUST_TOOLCHAIN_PROFILE = minimal
RUST_COMPONENTS = rustfmt, clippy

RUSQLITE_VERSION = 0.40.1 RETAINED
RUSQLITE_FEATURES = bundled, limits, backup
RUSQLITE_DEPENDENCY_INSTALLATION = NOT_AUTHORIZED_BY_THIS_TASK
```

Primary public evidence:

- Rust 1.95.0 release announcement:
  <https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/>
- Repository toolchain-file behavior:
  <https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file>

## Files changed

- `rust-toolchain.toml`: exact 1.95.0 channel, minimal profile, `rustfmt` and
  `clippy` components.
- `src-tauri/Cargo.toml`: declares `rust-version = "1.95"`; edition `2021`
  and package version `0.2.2-4` are unchanged.
- `.github/workflows/release.yml`: removes the moving
  `dtolnay/rust-toolchain@stable` selection, activates the repository toolchain,
  prints exact Rust/Cargo evidence, and adds matrix targets to the active pinned
  toolchain. The duplicate Android target installation was removed.
- `README.md`: replaces the moving `Rust stable` prerequisite with the exact
  repository selection and verification commands.
- This completion report.

No Rust/TypeScript runtime source, dependency version, Tauri capability,
migration, schema, release signing behavior, or deployment configuration was
changed.

## Canonical toolchain configuration

`rust-toolchain.toml` is the single version source:

```toml
[toolchain]
channel = "1.95.0"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

`src-tauri/Cargo.toml` independently declares only the compatible package
floor:

```toml
rust-version = "1.95"
```

Under the rustup proxies, repository commands selected:

```text
rustc 1.95.0 (59807616e 2026-04-14)
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
1.95.0-aarch64-apple-darwin (overridden by rust-toolchain.toml)
```

An explicit negative check with standalone Homebrew Cargo 1.94.1 failed closed
before compilation because the package requires Rust 1.95.

## Workflow audit

The repository contains two workflows. The Pages workflow is Node-only and
does not select Rust. The release workflow previously installed moving
`stable`, which could bypass or drift from the repository decision. It now:

1. lets rustup discover `rust-toolchain.toml` after checkout;
2. reports the active toolchain and exact compiler/package-manager versions;
3. adds only the matrix target to that active toolchain;
4. contains no explicit `stable`, `nightly`, alternate Rust version, directory
   override, or second toolchain channel.

The updated release workflow parses as YAML. It was not dispatched because
that workflow can publish release artifacts; this task authorizes no deployment
or production write.

## Isolated rusqlite compatibility probe

A temporary crate outside the repository and all repository worktrees used
only:

```toml
rusqlite = { version = "=0.40.1", default-features = false, features = ["bundled", "limits", "backup"] }
```

The probe ran explicitly with Rust/Cargo 1.95.0 and resolved:

| Package | Resolved version |
| --- | --- |
| `rusqlite` | `0.40.1` |
| `libsqlite3-sys` | `0.38.1` |
| bundled SQLite | `3.53.2` |

`cargo check` passed. One test passed after opening only an in-memory database,
checking that `SQLITE_OPEN_NOFOLLOW` is exposed, reading and restoring an
SQLite limit through the `limits` API, copying one in-memory row with the
`backup` API, and reading it from the destination. The probe never accessed
DAARION application data. Its manifest, lockfile, source, build output, and the
downloaded rustup installer were deleted after evidence was recorded.

## Repository validation under Rust 1.95.0

| Command or gate | Result |
| --- | --- |
| `rustc --version --verbose` | PASS — exact `1.95.0`, Apple Silicon host |
| `cargo --version --verbose` | PASS — exact `1.95.0` |
| `rustup show active-toolchain` | PASS — repository override `1.95.0-aarch64-apple-darwin` |
| `git diff --check` | PASS |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --all --check` | PRE-EXISTING DEBT — exit 1 on the candidate and exact starting main; both report the same 94 files and byte-identical normalized output |
| `cargo check --manifest-path src-tauri/Cargo.toml --all-targets --locked` | PASS — 312 existing warnings; exact-main comparison under 1.95.0 also reports 312 |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked` | PASS — 116/116 |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked inference:: --lib` | PASS — 67/67 Phase 1A tests |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked` | PASS command exit — legacy warnings remain; no Rust source changed |
| `bash scripts/check-rust-touched-warnings.sh` | PASS — protected hardening modules clean |
| `bash scripts/check-no-secrets.sh` | PASS |
| `node scripts/validate-inference-contract.mjs` | PASS |
| `npm ci` | PASS; package lock unchanged |
| `npm run build` | PASS — TypeScript and production Vite/PWA build |
| `npm audit --omit=dev` | PASS — 0 production vulnerabilities |
| release-workflow YAML parse | PASS |

The repository-wide `rustfmt` debt is already tracked in
`docs/planning/RUST_FORMATTING_DEBT.md`. This PR changes no Rust source and
reproduces the exact-main 94-file result, so it introduces no formatting
regression. The raw required command still exits non-zero; therefore this
report does not claim a clean release-gate `PASS`.

The full npm install audit also reports 11 pre-existing development-only
advisories (2 low, 4 moderate, 5 high); production dependencies report zero.
No JavaScript dependency changed, applicability was not expanded by this diff,
and remediation belongs in a separate dependency-review PR.

## Lockfile and dependency proof

- `src-tauri/Cargo.lock` SHA-256 before and after:
  `1c6eefec5a292882163309ba10a15927a3827bd9adf84e11a14ea43c93cba401`
- `package-lock.json` remained unchanged.
- `rusqlite` and `libsqlite3-sys` do not appear in the repository manifest or
  lockfile.
- No dependency was added, removed, or re-resolved.

## Security and supply-chain review

| Finding | Severity | Disposition |
| --- | --- | --- |
| Moving release `stable` could drift from the accepted floor | MEDIUM | Closed in this diff by repository-owned exact pin plus version evidence |
| Standalone Rust earlier on local `PATH` can bypass rustup discovery | MEDIUM | Manifest MSRV fails closed; README requires rustup proxies first and exact-version verification |
| Toolchain provenance | INFO | Rust 1.95.0 is an official stable release; rustup installer matched the official SHA-256 and rustup verifies downloaded components |
| Repository dependency/lock drift | INFO | None; both lockfiles are unchanged and application `rusqlite` remains absent |
| Runtime/SQLite scope expansion | INFO | None; no runtime source, schema, migration, or application dependency exists in this diff |
| Secret, private endpoint, cache, or generated binary leakage | INFO | None found; temporary probe and installer removed |
| Development-only npm advisories | OUT_OF_SCOPE / UNASSESSED | Pre-existing and dependency-unchanged; production audit is zero; separate review required before any dependency update |

No confirmed Critical or High finding was introduced by the toolchain diff.

## Platform evidence and gaps

- Verified locally: macOS Apple Silicon host, Rust/Cargo 1.95.0, complete Rust
  suite, Phase 1A inference suite, frontend contract and production build.
- Configured but not yet executed for this head: macOS x86_64, Windows x86_64,
  Linux x86_64, and Android arm64 release jobs.
- Android remains a separately authorized validation gate.
- iOS remains unsupported and unclaimed.

No cross-platform PASS is claimed from the macOS host. Focused review and
repository CI evidence must be evaluated before ready/merge.

## Blocked Phase 1B.1 worktree preservation

The separate blocked branch remains uncommitted at the starting main with:

```text
 M src-tauri/Cargo.lock
 M src-tauri/Cargo.toml
 M src-tauri/src/lib.rs
?? docs/planning/phases/phase-01b1-storage-bootstrap-plan.md
?? src-tauri/src/runtime_store/
```

Its tracked diff remains 86 insertions and 3 deletions across the three tracked
files; the untracked runtime-store directory contains its actor, connection,
error, migration, module, path-policy, and initial SQL work. The tracked binary
diff fingerprint remained
`8e069857177bf7174981d48e3b053d1ab8f0e7020b88ac938e3fec738e41fbea`
when this report was prepared. Nothing from that worktree is staged, committed,
pushed, reset, cleaned, copied, or mixed into this branch.

## Rollback

Before Phase 1B.1 or production use, rollback is a narrow revert of
`rust-toolchain.toml`, the manifest MSRV line, the release-workflow selection,
the README setup note, and this report. No persisted data or runtime migration
exists, and neither lockfile needs regeneration. A downgrade of `rusqlite` is
not part of this rollback and would require a separate human dependency
decision.

## Release gate and next action

The toolchain compatibility change and local probe are implemented and tested.
The gate is `CONDITIONAL_PASS`, not clean `PASS`, because the required full
formatter check reproduces the accepted 94-file repository debt and real
cross-platform jobs have not run for this head.

The only authorized next action is a draft PR and exactly one focused Codex
review covering toolchain correctness, workflow consistency, MSRV,
reproducibility, regression risk, and scope. Do not mark ready or merge. Even
after this PR merges, Phase 1B.1 must restart from a new clean worktree and
requires separate authorization; the partial blocked worktree is not a source
to transplant blindly.
