# ADR 0002: Local Runtime State and SQLite Foundation

- Status: Accepted; implementation partial; Phase 1B.1 merged and fresh-main verified
- Date: 2026-07-04
- Scope: first durable Edge runtime-state store

## Context

The audited Edge snapshot has no durable task/conversation/audit store. Messaging uses in-memory vectors and agent/loop recovery is not possible. Introducing a full six-level memory system at the same time as inference and Supervisor foundations would create an oversized, difficult-to-review change.

## Decision

1. SQLite is the initial durable local runtime store.
2. Phase 1B is separate from inference and Supervisor work.
3. The first schema contains only:
   - schema migrations;
   - conversations;
   - messages;
   - tasks;
   - audit events.
4. Transactions, versioned migrations, restart recovery, retention hooks, deletion, export and migration/corruption tests are mandatory.
5. Storage is accessed through repository interfaces so later memory layers do not leak SQL across runtime modules.
6. Six-level memory is a later phase. Episodes, semantic facts, procedures, entities, relations and embeddings are excluded from the foundation.
7. No external vector database, graph database, Supabase memory store or cloud database is introduced in Phase 1B.
8. Platform at-rest protection/encryption requirements must be resolved in the Phase 1B plan.

## Consequences

- The first durable slice remains reversible and testable.
- Agent Supervisor recovery can rely on a stable task/audit contract in Phase 1C.
- “SQLite foundation” must not be marketed as complete agent memory.
- Schema evolution and deletion/export behavior become release-gated.

## Alternatives rejected

- RAM-only state: rejected because restart recovery and auditability are required.
- Full six-level schema immediately: rejected due scope and truthfulness risk.
- External vector/graph service first: rejected because local-first ownership and MVP complexity favor SQLite.

## Verification gate

See [Durable runtime state gate](../security/SECURITY_GATES.md). The separately
authorized Phase 1B.1 vertical slice is merged and fresh-main verified. It
implements only bootstrap and a safe read-only status projection: exact
`rusqlite` 0.40.1, bundled SQLite 3.53.2, the five-table empty schema, one
bounded Rust owner, explicit bounded application shutdown, deadline-interrupt
and hard-link-aware path controls, strict UUID-v4 schema constraints, and
restart/reopen tests.

```text
PHASE_1B_1 = MERGED / FRESH_MAIN_VERIFIED
MERGED_REVIEWED_HEAD = 5d894f42a967c9360d86382c1aab9e603472e0c8
MERGE_COMMIT = cd903fb18d1618bbe0787d2397948622849ef9d4
MERGED_AT = 2026-07-24T11:44:00Z
STORAGE_BOOTSTRAP = IMPLEMENTED_AND_VERIFIED
STORAGE_RUNTIME_PROJECTION = IMPLEMENTED_AND_VERIFIED_IN_REPOSITORY
DURABLE_RUNTIME_STATE = PARTIALLY_IMPLEMENTED
PHASE_1B = NOT COMPLETE
PHASE_1B_2 = NOT AUTHORIZED
REAL_DESKTOP_RESTART_FLOW = NOT VERIFIED
CROSS_PLATFORM_RUNTIME = NOT VERIFIED
REMOTE_CI = NOT PRESENT / NOT CLAIMED
REMOTE_PRODUCTION_WRITES = 0
REAL_USER_PROFILE_WRITES = 0
DEPLOYMENTS = 0
```

Fresh-main verification passed 64/64 storage, 67/67 inference and 180/180
full Rust tests with Rust 1.95.0, plus Cargo check/Clippy, 29/29 primary
boundary fixtures, 13/13 defense-in-depth fixtures, 46 structural checks,
production build over 1,763 modules, and zero production npm vulnerabilities.
Runtime-store warning locations were 0. Dev-inclusive npm audit retained 11
inherited advisories outside the production dependency set. Inherited RustSec,
warning and rustfmt debt were unchanged. No remote CI was present.

The verified schema invariants are:

```text
MIGRATION_SHA = 62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d
STRUCTURAL_FINGERPRINT = 37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77
TABLES = 5
EXPLICIT_INDEXES = 7
SQLITE_AUTOINDEXES = 7
SQLITE_SEQUENCE = 0
MIGRATION_2 = ABSENT
```

This first slice does not implement public runtime-state CRUD, backup/export,
retention/deletion services, six-level memory, Phase 1B.2, or
production/platform acceptance. The full ADR implementation therefore remains
partial.
