---
name: daarion-local-llm
description: "Use for DAARION model discovery, download, verification, lifecycle, local inference, streaming, structured output, embeddings, or remote-execution policy."
---

# DAARION Local LLM

Follow the mandatory lifecycle in the active `AGENTS.md`: require a reviewed baseline audit and written phase plan with an allowed verdict before implementation, then require tests, security review, full diff review, documentation, a completion report, and the release gate. Never claim completion when a required check failed.

- Local inference is the default. Prohibit silent remote fallback, implicit provider switching, and undocumented prompt egress.
- Define an explicit `ExecutionPolicy` for local-only, any separately approved remote mode, data classification, and user consent. A provider must fail closed when policy cannot be satisfied.
- Put inference behind an `InferenceProvider` abstraction. Ollama may be the first adapter but must not become the permanent domain boundary.
- Keep registry identifiers, provider identifiers, model files, and claimed capabilities distinct and validated. Verify download integrity and compatibility before load where downloads are in scope.
- Do not claim embedded `llama.cpp` support unless an executable path and tests prove it.
- Bound concurrency and context size; implement timeouts, cancellation, lifecycle state, robust streaming framing, partial/error termination, and resource cleanup.
- Validate structured model output against a schema before use. Treat all model output as untrusted data.
- Private prompts, retrieved memory, and tool results may not leave the device without explicit policy plus visible approval.

Test local-only rejection of remote providers, provider selection, identifier mapping, timeouts, cancellation, concurrent requests, malformed streams, structured-output failures, and cleanup. Record whether each path uses a real provider, controlled fixture, or mock.
