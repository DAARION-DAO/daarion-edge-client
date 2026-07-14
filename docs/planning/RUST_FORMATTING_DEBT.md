# Rust Formatting Debt

Status: **OPEN / PRE-EXISTING / SEPARATE REMEDIATION**

This record isolates repository-wide `rustfmt` debt from security-sensitive
runtime phases. It is not authorization to mix a broad mechanical rewrite into
an application feature PR.

## Evidence

| Snapshot | Commit or context | Rust files reported by repository-wide `cargo fmt -- --check` |
| --- | --- | ---: |
| Phase 1A baseline | `a62626cab1fa1ede5a4990ef09fde940f8634c67` | 101 |
| Phase 1A initial candidate | `51194cf7e6c0d4b87b6b0fe7fa4c9d027aee88f2` | 97 |
| Phase 1A focused-review candidate | same PR branch after changed-file formatting | 94 |

Phase 1A therefore introduces no formatting regression. Every Rust file added
or modified by that phase passes a changed-scope `rustfmt --check`. The
remaining 94 files are outside its runtime scope.

## Required remediation workflow

1. Use a dedicated formatting-only branch and PR from a fresh accepted `main`.
2. Do not combine dependency, behavior, migration, generated-file or
   architecture changes with the formatting diff.
3. Record the exact baseline and final file counts.
4. Run full Rust tests, `cargo check`, `cargo clippy --all-targets`, frontend
   build and `git diff --check`.
5. Review non-whitespace diff separately so no semantic change is hidden.
6. Require human review before merge.

## Closure criteria

- repository-wide `cargo fmt -- --check` exits successfully;
- the PR is formatting-only or every exception is explicitly justified;
- full required checks pass;
- no warning suppression, dependency update or runtime behavior change is
  hidden in the cleanup.
