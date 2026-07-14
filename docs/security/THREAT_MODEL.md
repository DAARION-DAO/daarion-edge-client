# Sovereign Agent Threat Model

Status: **BASELINE / PRE-IMPLEMENTATION**

## Assets

- local prompts, conversations, memory and files;
- device, agent, pairing, transport, session and wallet keys;
- task/loop state, checkpoints and audit records;
- tool and worker authority;
- MicroDAO membership and device binding;
- model artifacts and inference outputs;
- readiness projections and capability announcements;
- mesh messages/mailbox;
- update/package integrity.

## Trust boundaries

1. Browser ↔ Supabase/web backend.
2. Web/backend ↔ pairing invitation.
3. Pairing invitation ↔ Edge parser/local state.
4. Frontend ↔ Tauri commands.
5. Deterministic Edge core ↔ local model provider.
6. Model output ↔ policy/tool executor.
7. Edge ↔ local runtime database/OS secure storage.
8. Edge ↔ future mesh daemon IPC.
9. Mesh daemon ↔ untrusted remote agents.
10. Wallet proposal ↔ isolated signer/user approval.
11. Build/update pipeline ↔ installed application.

## Threats and required controls

| Threat | Current evidence | Severity | Required controls | Blocking gate |
| --- | --- | --- | --- | --- |
| Malicious/replayed pairing invitation | Edge parses URL/base64 JSON without signature/expiry/replay/revocation checks | HIGH | Signed purpose-bound envelope, trusted issuer, nonce, expiry, single use, replay cache, revocation | Pairing |
| Compromised web account | Product session may create invitations within backend policy | HIGH | Step-up/role policy as selected, short lifetime, device confirmation, revocation, audit | Pairing |
| Compromised backend | Could mint/alter product data | HIGH | Minimize trust, signed evidence, key rotation/revocation, Edge confirmation, no raw local secrets | Pairing/projection |
| Silent remote Edge inference | Simulated remote branch exists; no LocalOnly invariant | HIGH | Provider boundary, fail-closed policy, egress test, explicit future consent | Phase 1A |
| Malicious local model/prompt injection | Model output can contain hostile instructions | HIGH | Structured schema, treat output as proposal, policy broker, tool allowlists, no secrets | Supervisor/tools |
| Malicious remote agent | Future mesh input could request privileged work | HIGH | Authenticated identity, signed/expiring envelope, capability policy, bounded delegation | Mesh/multi-agent |
| Replay/duplicate message | No mesh envelope exists | HIGH | Message IDs, nonce/expiry, replay cache, idempotent task IDs and receipts | Mesh |
| Forged capability/readiness projection | No signed Edge projection exists | HIGH | Versioned signed document, freshness, revocation, minimal fields | Readiness |
| Poisoned memory | Durable memory not implemented | HIGH | Source references, trust labels, validation, dedup/contradiction workflow, approval for destructive merge | Memory |
| Tool escalation/command injection | Shell/network/worker surfaces lack unified broker | CRITICAL if enabled | No unrestricted shell, typed tools, argument/path/egress allowlists, approvals, audit | Tools |
| Wallet drain/key exposure | Mnemonic returned in serializable command result | CRITICAL if wallet enabled | Isolated signer, keys outside frontend/model, transaction preview, explicit approval, limits/replay | Wallet |
| Worker sandbox escape/untrusted lease | Mock lease/dummy key/unbounded loop | CRITICAL if enabled | Hard-disable, signed leases, strong sandbox, resource/time limits, no secret environment, kill switch | Worker |
| IPC spoofing/sidecar replacement | Future boundary only | HIGH | Mutual authentication, executable/package verification, per-install keys, peer authorization | Mesh |
| Local database theft | SQLite not yet implemented | HIGH | Platform storage decision, least privilege, optional encryption/OS protection, redacted data, deletion | Phase 1B/memory |
| Model supply-chain attack | Artifact verification is simulated | HIGH | Approved manifest, digest/size/signature, atomic safe path, quarantine, provenance | Model download |
| Unsafe auto-update | Release docs exist; not verified here | HIGH | Signed updates, pinned trust root, rollback, staged release, no secret logging | Hardening |
| SSRF/network egress | Backend URLs and HTTP clients exist | HIGH | URL normalization, scheme/host policy, loopback restrictions, no redirect abuse, egress policy | Pairing/inference/tools |
| Path traversal | Future downloads/tools/database export | HIGH | Canonicalization, roots, no symlink escape, atomic writes, adversarial tests | Model/tools/memory |
| Insecure deep link | Web handoff carries a bearer-like payload | HIGH | Purpose-specific signed envelope, expiry, single use, minimal data, safe parsing and UI confirmation | Pairing |
| Debug logging of secrets | Security-sensitive flows exist | HIGH | Redaction, structured safe logging, tests, no prompts/keys/tokens/URLs | All sensitive phases |
| Unbounded loop/resource exhaustion | Worker loop is unbounded; bounded Loop Runtime absent | HIGH | Iteration/time/tool/token/retry/cost limits, cancellation, checkpoints, backoff | Loop/worker |

“CRITICAL if enabled” means the source is not proof of a current production exploit; it means enablement is forbidden until the control exists.

## Security review rules

- Distinguish confirmed source behavior, deployed proof, design risk and hypothesis.
- Any unresolved Critical or High finding in the phase scope blocks `PASS` unless a named human authority explicitly accepts it in a reviewable record.
- Acceptance never changes a capability status to verified.
- Tests must cover denial and failure paths, not only mocked success.
- Every sensitive phase updates this threat model and its gate in [SECURITY_GATES.md](SECURITY_GATES.md).
