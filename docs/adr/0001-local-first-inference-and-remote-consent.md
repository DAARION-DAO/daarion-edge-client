# ADR 0001: Local-First Inference and Remote Consent

- Status: Accepted for baseline; implementation pending Phase 1A
- Date: 2026-07-04
- Scope: `daarion-edge-client` inference policy and its product disclosure boundary

## Context

Edge source contains a real loopback Ollama chat path and a simulated remote-offload branch. It also labels Ollama HTTP behavior as `llama.cpp`. Separately, `loval-echoes` has cloud-assisted web AI functions. A single statement that “DAARION never uses cloud inference” would be inaccurate and would conflate trust domains.

## Decision

1. `InferencePolicy::LocalOnly` is the only enabled Edge policy for the MVP.
2. The simulated remote fallback is removed or made unreachable before Phase 1A is accepted.
3. Policy failure, unknown provider, invalid model ID, deadline and cancellation fail closed.
4. A provider-neutral `InferenceProvider` boundary precedes additional backends.
5. Ollama is the first provider, not the permanent domain API.
6. Provider/runtime/model identifiers shown in logs/UI are truthful.
7. Remote Edge inference is a future feature requiring a new ADR, separately enabled policy, explicit user consent, egress disclosure, cancellation and audit.
8. Web cloud-assisted AI remains a separate trust domain and must be clearly labeled. It cannot be represented as Edge local inference or access Edge private state by default.

## Consequences

- Phase 1A can be narrow and testable.
- Remote provider implementation is out of scope.
- No “automatic best provider” or silent fallback is allowed.
- Tests require a fake provider and proof that LocalOnly never invokes a remote provider.
- Loopback/redirect/proxy behavior must be included in the network policy analysis.

## Alternatives rejected

- Keep simulated remote fallback: rejected because it normalizes an unsafe control path.
- Couple the entire runtime directly to Ollama: rejected because model/runtime policy must remain provider-neutral.
- Ban all cloud AI across the web product: rejected because it contradicts current web source and is not required for Edge sovereignty.

## Verification gate

See [Local-only inference gate](../security/SECURITY_GATES.md). This ADR does not itself prove the gate closed.
