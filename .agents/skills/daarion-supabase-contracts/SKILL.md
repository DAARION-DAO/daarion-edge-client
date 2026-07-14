---
name: daarion-supabase-contracts
description: "Use for DAARION Supabase Auth, schemas, RLS, RPCs, Edge Functions, pairing invitations, membership authorization, revocation, or readiness projections."
---

# DAARION Supabase Contracts

Follow the mandatory lifecycle in the active `AGENTS.md`: require a reviewed baseline audit and written phase plan with an allowed verdict before implementation, then require tests, security review, full diff review, documentation, a completion report, and the release gate. Never claim completion when a required check failed.

Before implementation, inspect the repository client/migrations/types and current official Supabase documentation and changelog relevant to the feature. Do not assume an API or security behavior from memory.

- Authenticate the user and authorize current MicroDAO membership at the trusted boundary. Client-supplied IDs, URLs, metadata, or UI state are not authorization.
- Enable and review RLS for exposed tables; review grants, Data API exposure, function execution privileges, and every policy path. Do not use user-editable metadata as authorization evidence.
- Keep service-role and other privileged credentials out of the browser. Security-definer functions require minimal scope, controlled `search_path`, explicit grants, and negative tests.
- Pairing invitations must be short-lived, single-purpose, single-use, bound to the intended user/MicroDAO/device purpose, revocable, replay-resistant, and safely consumed.
- A backend URL is not a credential or authorization proof. Return only minimal data and generic public errors.
- Keep raw prompts, private local memory, private keys, wallet material, and private operational evidence out of Supabase. Dashboard readiness is a safe, freshness-aware projection.

Use repository migration conventions, reversible changes, generated/verified types, RLS/RPC contract tests, and readback against the intended project only when the phase explicitly authorizes it. Never apply a migration or deploy an Edge Function automatically.
