# Edge to existing City surface — local joining pilot

EXECUTION_MODE = RESULT_FIRST_PROTOTYPE_FIRST
PRIMARY_OUTCOME = The same existing local agent accepts a City conversation through the existing AG-UI interface and preserves its result.
CORE_USER_PATH = Open Edge -> choose an installed local model -> connect existing City panel -> see personal agent -> send one message -> receive a saved local answer.
THIS_TASK_MUST_SHIP = A native AG-UI bridge, a one-button registration path using existing OpenBot APIs, executable protocol evidence, and an exact account of any final runtime/identity gate.
SHORTEST_SAFE_PATH = Reuse Edge inference and OpenBot external-agent profiles. Do not add an auth framework or duplicate the panel.
RELEASE_BLOCKERS = P0 and real P1 only.
DEFERRED_BACKLOG = Multi-device transport, shared compute, production packaging, general agent tools.
WORKSTREAM_NAME = edge-city-join-pilot
WORKSTREAM_BOUNDARY = Edge pilot bridge and its UI; existing City APIs only.
FILES_IN_SCOPE = src-tauri/src/local_agent/{city,mod,store}.rs, LocalAgentPilotPage.tsx, Cargo manifest/lock, bounded docs and tests.
FILES_OUT_OF_SCOPE = MicroDAO production, City source/overlay, identity provider settings, City memory/runtimes, previous working copies.
ACTIVE_PR_OVERLAP = Edge PR 21 changes only release documents. City memory is active in a separate task; no City source is changed.
OPERATOR_AUTHORIZATION = YES for local prototype execution and task-requested personal-agent application records; NO for production auth changes, public exposure, permission changes, or shared City restart without the final rollout decision.

## Objective and current state

Edge canonical main is eafb30d8548e2b1e3bc36fc34b2c8f0b36f633b7. The adopted audit, ownership and roadmap were read. The prior 22 reviewed, uncommitted local-pilot files are carried into an isolated task branch; the prior copy remains intact. This is a continuation of that isolated prototype, not promotion of Phase 1B.4/1C.

Current City Control Plane main is 5cd0061c4a58a6fbe27854f0f89429de13e756cd; the running adopted OpenBot v0.0.5 uses its existing external AG-UI agent API. Read-only registration preflight refused the proposed additional loopback endpoint under its current endpoint allowlist. The current panel also uses its explicit local single-user operator mode. Neither fact is employee authentication.

MicroDAO already has the managed OAuth consent UI and discovery. No configured MicroDAO-to-OpenBot identity-provider client is established in the inspected source/runtime. Membership/device-pairing invitations are not substitutes for that client or for employee entitlements. Actual MicroDAO login remains an explicit unresolved full-path gate; no copied session token or simulated employee identity will stand in for it.

## Scope, ownership and contracts

The Edge-owned bridge is a private, session-only loopback listener using the existing AG-UI RunAgentInput/SSE protocol. It has a random bearer credential scoped to this app session; no key enters prompts, frontend state, URLs, logs or artifacts. OpenBot's existing write-only encrypted endpoint-credential API is the consumer. A City profile is only a UI projection pointing to the exact local agent UUID route; it does not create another sovereign identity or register hardware into the Observer registry.

Registration uses existing /api/me, /api/agents, /api/agents/test-connection and private profile CRUD. The native prototype accepts only the currently verified local operator posture, reports it explicitly, and refuses a different/missing auth context. It does not bypass the server's endpoint policy. A denied preflight performs no profile write. After restart the listener and credential are gone; reconnect rotates the credential on the matching owned profile without duplicating it.

Only explicit chat messages from the City conversation reach local inference. Folder contents, saved private tasks, other conversations and tools are excluded. The profile description and bounded message context guide the local model. The model cannot execute tools, choose a path or choose a provider. The existing one-task limit, local-only model checks, cancel and timeout behavior remain enforced. The City result is saved locally with request correlation; duplicate runs return the same persisted result, conflicting replay is rejected.

## Security, migration and compatibility

Use the already locked hyper/http-body-util/hyper-util primitives directly instead of inventing an HTTP parser. Exact versions and server-only feature additions are reviewed in the lockfile. Request body, identifiers, message count, concurrency and lifetime are bounded. Only the pilot-owned listener/connections are stopped. Revocation cancels active work. New optional task metadata is backward-compatible with the existing private pilot JSON rows; no existing or production schema migration.

No MicroDAO Supabase write or auth client registration, City allowlist edit/restart, public service, callback tool token, memory grant or employee role change is included in automatic execution. Prepare any necessary final rollout change concretely, then request the required operator decision.

## Implementation and tests

Implement the authenticated local AG-UI endpoint and idempotent task execution, then native registration/readback and a visible local-only connection card. Test request validation, bearer refusal, body bounds, conflicting replay, SSE shape, terminal outcomes and cancellation/persistence. Build and exercise the native app against a real installed local model. Compare requests and events with the pinned OpenBot consumer contract; prove the native producer with a real HTTP client. Actual OpenBot consumption remains required after its allowlist gate is resolved. Attempt only existing-server policy preflight before registration; classify refusal truthfully.

## Acceptance, rollback and documentation

Required local evidence: real request/response through AG-UI, same local agent identity, saved result, missing-token rejection, replay behavior, shutdown. Full City UI acceptance requires an allowed endpoint plus successful profile creation/readback and a visible conversation. Full employee acceptance additionally requires actual MicroDAO authentication/entitlements and another-device transport; those cannot be inferred from this local operator proof.

Rollback: close the pilot; keep saved results. A newly created City profile, if eventually authorized, can be disconnected/hidden through existing UI. No automatic deletion of City data. Write a completion report and a concrete rollout/auth-client handoff if that gate remains.

## Verdict

CONDITIONAL_GO for the isolated Edge bridge and its local protocol proof. NO_GO for claiming authenticated employee onboarding or silently changing the active City allowlist. This named runtime gate does not block the independent implementation that makes a final decision concrete.
