# Public, Private, Local and Cloud Boundaries

Status: **MANDATORY SECURITY AND DOCUMENTATION POLICY**

## Boundary principles

- Public repository architecture is sanitized design truth, not live operations truth.
- Local sovereign-agent data remains on the user's device by default.
- A safe projection is an explicit allowlist, never “everything except secrets.”
- Web cloud-assisted AI and Edge local inference are separate trust domains.
- Remote inference by Edge is disabled by default and cannot happen silently.
- Octelium, if used, is an external secure-access layer only. It neither replaces Reticulum/LXMF nor becomes repository-visible deployment truth.

## Data classification

| Class | Examples | Allowed destination |
| --- | --- | --- |
| Public architecture | Module boundaries, status vocabulary, sanitized ADRs | Public repositories |
| Product account data | Account/profile/MicroDAO membership | Authenticated web/backend under RLS and grants |
| Safe device projection | Versioned status enum, timestamps, non-sensitive capability summary | Authenticated web read model after signing/freshness design |
| Local private runtime state | Tasks, checkpoints, audit details, local queue | Edge local store only by default |
| Local private memory | Prompts, conversations, facts, embeddings, files, graph | Edge local store only by default |
| Cryptographic secret | Private keys, mnemonics, signing handles | Domain-specific secure storage/signer only |
| Tool-sensitive data | Paths, environment, commands, results, network targets | Local policy/executor; no broad projection |
| Transport-private data | Message bodies, mailbox, routes, peer metadata | Edge/mesh boundary under encryption and policy |
| Private operations truth | Live endpoints, network topology, credentials, incidents, operator access | Authorized private operations systems only |

## Repository rules

Public repositories may contain:

- source code and tests;
- sanitized contracts and schemas without live values;
- generic configuration examples;
- architecture, ADRs, threat models and roadmaps;
- public release evidence that contains no private infrastructure details.

They must not contain:

- credentials, tokens, private keys or mnemonics;
- live IP/DNS/firewall/Octelium/NODA configuration;
- private endpoint inventories or operator access;
- user data, raw local memory or private prompts;
- production incident evidence or non-public deployment topology;
- secret-derived hashes or copied environment files.

## Edge local-first rule

For the MVP:

```text
Edge inference policy = LocalOnly
remote provider = unavailable
remote fallback = unreachable
policy failure = fail closed
```

A future remote Edge mode requires a new ADR and must disclose provider, data categories, purpose, retention, cost, cancellation behavior and consent. Approval must be explicit, scoped, revocable and auditable.

## Web cloud-AI rule

`loval-echoes` may offer separate cloud-assisted features. Those features must:

- be labeled as cloud-assisted;
- enforce authenticated authorization server-side;
- disclose when user content is sent to a provider;
- avoid borrowing Edge “local,” “private,” “offline,” or readiness claims;
- remain unable to access Edge raw memory, keys, tools, checkpoints or signer;
- have their own retention/deletion and provider policy.

The existence of cloud web AI does not enable remote Edge inference.

## Supabase boundary

Supabase may hold product authentication, MicroDAO membership, pairing invitation state and safe readiness read models. It must not hold raw local agent memory, Edge private keys, wallet signing material, unrestricted tool results or mesh mailbox contents.

For exposed data:

- RLS and privileges/grants are both required controls;
- server functions must derive user identity from the authenticated context, not a browser-supplied user ID;
- MicroDAO membership and role must be checked for every privileged action;
- service-role capability remains server-only;
- migration source is not evidence that the intended policy is deployed.

## Projection allowlist

A future readiness projection may include only schema-approved fields. Candidate fields must be reviewed individually. Default-denied fields include:

- raw prompts/messages/memories;
- filenames or document content;
- local paths;
- model output;
- tool arguments/results;
- environment variables;
- wallet balances/addresses unless a separate product contract requires them;
- peer lists, routes, mailbox content or infrastructure topology;
- detailed logs and stack traces.

## Logging and completion reports

Reports may include relative repository paths, symbol names, safe commands, exit status and sanitized failure categories. They must redact values that look like credentials or private infrastructure. “Check passed” is not a substitute for recorded scope; “not run” must remain explicit.
