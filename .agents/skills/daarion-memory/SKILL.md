---
name: daarion-memory
description: "Use for DAARION local memory models, SQLite persistence, retrieval, consolidation, retention, deletion, export, encryption, or schema migrations."
---

# DAARION Memory

Follow the mandatory lifecycle in the active `AGENTS.md`: require a reviewed baseline audit and written phase plan with an allowed verdict before implementation, then require tests, security review, full diff review, documentation, a completion report, and the release gate. Never claim completion when a required check failed.

Prefer SQLite for the first durable local implementation unless an ADR, measured requirement, and migration plan justify another service.

Define explicit boundaries for:

- working memory;
- conversation memory;
- episodic memory;
- semantic memory;
- procedural memory;
- graph memory.

For each type define owner, schema, provenance, evidence references, confidence, lifecycle, retention, deletion, export, backup, and privacy class. Model-generated or extracted memories are untrusted until validated; preserve source references and never let consolidation erase contradictory evidence silently.

Use versioned migrations, transactions, foreign-key/integrity checks, deterministic IDs, bounded queries, and crash-safe writes. Define encryption and key-storage boundaries without exposing keys to the LLM or frontend. Keep raw local memory out of Supabase and public projections.

Test migrations, rollback/compatibility where supported, transactions, corruption/error handling, retention, deletion, export, retrieval isolation, duplicate/idempotent writes, and restart persistence. External vector/graph databases require an ADR and must not be introduced merely because the long-term model names semantic or graph memory.
