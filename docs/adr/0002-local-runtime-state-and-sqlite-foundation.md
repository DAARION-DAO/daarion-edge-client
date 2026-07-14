# ADR 0002: Local Runtime State and SQLite Foundation

- Status: Accepted for baseline; implementation pending Phase 1B
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

See [Durable runtime state gate](../security/SECURITY_GATES.md). This ADR does not authorize a dependency change before the Phase 1B plan and human review.
