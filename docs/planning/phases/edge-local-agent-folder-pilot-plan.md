# Local agent and folder pilot

EXECUTION_MODE = RESULT_FIRST_PROTOTYPE_FIRST
PRIMARY_OUTCOME = A native local agent profile with a distinct node reference, explicit folder access, a completed bounded local task, and a result that survives reopening.
CORE_USER_PATH = Install/open Edge pilot -> create personal agent -> choose folder -> run one bounded task -> view saved result.
THIS_TASK_MUST_SHIP = A reviewable native macOS package and executable acceptance evidence; Windows source/build readiness is separate from Windows execution proof.
RELEASE_BLOCKERS = P0 and real P1 in this bounded path.
DEFER = DAGI enrollment, external access, cloud inference, arbitrary tools, recursive loops, wallet/voice/token flows, general Supervisor, worker resource contribution.
WORKSTREAM_NAME = edge-local-agent-folder-pilot
WORKSTREAM_BOUNDARY = Isolated native pilot profile; no main Edge enrollment/heartbeat or shared canonical-state mutation.
OPERATOR_AUTHORIZATION = YES for the requested local prototype, native folder selection and synthetic acceptance; NO for production, public rollout, accounts or permissions in City.

## Objective

Implement the local-first first half of the user's agreed employee path. The same future local agent can be connected to the existing City panel later; this phase must not invent network registration or label a stored local profile as a sovereign cryptographic identity.

## Current State

Fresh canonical Edge main is eafb30d8548e2b1e3bc36fc34b2c8f0b36f633b7. The adopted human-reviewed baseline and ownership docs were read. Phase 1A local inference and Phase 1B private storage primitives exist; a complete personal-agent folder path does not. The existing document pilot remains intact in another working copy. Active PR #21 changes release documentation only; no overlap. City Control Plane/OpenBot are an active separate workstream and will not be edited.

The installed local Ollama lists existing artifacts but reports cloud disabled=false. Its shared configuration must not be relaxed or changed. The pilot may start one temporary private process of the already-installed Ollama executable with cloud disabled and CPU-only requests, using existing model artifacts. No new model download or persistent service registration is authorized. A missing executable/artifact remains visibly unavailable. Deterministic folder inspection is useful independently of optional model narration.

## Scope and Ownership

Edge owns local UI, profile, folder capability, fixed bounded task, local persistence and process lifecycle. MicroDAO retains membership. City retains its existing agent adapters. No network/AG-UI/SSO endpoint is introduced here. The pilot uses its own app identifier and local database. This does not promote Phase 1B.4/1C or change the canonical runtime-store commands.

## Files and Modules Expected to Change

- src/main.tsx; new src/pages/LocalAgentPilotPage.tsx.
- src-tauri/src/lib.rs: separate pilot entrypoint only.
- new src-tauri/src/local_agent/ modules, src-tauri/src/bin/edge-local-agent.rs and tauri.local-agent.conf.json.
- narrow internal inference factory/options reuse in src-tauri/src/inference/; default inference options unchanged; accept documented bare SHA-256 Ollama digests after a real compatibility failure.
- src-tauri/Cargo.toml and Cargo.lock: pinned cap-std 4.0.3 for directory-relative capability access, preventing traversal and symlink escape; reuse existing SQLite, UUID, HTTP and model code.
- focused tests, package helper, README and phase completion.

## Contracts and Security

Stable local UUID references distinguish agent and node; they are not network certificates or canonical DAGI membership. No private-key creation, wallet, signing or identity authority is added. One local agent profile belongs to this isolated app. Native commands accept bounded names/instructions, not arbitrary command strings, filesystem paths or provider URLs. The OS folder picker grants a held directory capability only for this process. Closing/revoking drops it; reopening requires fresh selection.

Folder operation is READ_ONLY: one directory, at most 256 entries, no recursion, hidden entries and symlinks excluded, bounded text-file excerpts only, no workspace writes. Results go only to the pilot-owned SQLite store. The fixed local task is manually invoked, bounded, cancellable, and records actual terminal outcomes. Model output remains text and cannot choose tools or paths. File contents and names are untrusted data. The user controls whether local model narration is requested; no remote fallback. Local model process arguments and environment are native-owned; only the child this app created may be stopped.

## Migration and Compatibility

No production migration and no main-profile write. A private pilot SQLite database stores profile, task/result and bounded audit rows. It is not synchronized to the existing runtime store, MicroDAO or City. Normal app entry remains unchanged. The native folder picker supports macOS/Windows; real Windows execution requires a Windows host. Unsupported platforms fail visibly.

## Implementation Steps

Create the profile and capability-backed folder action; persist actual outputs; expose a simple native screen; reuse the existing inference service for optional CPU-only local narration; package; run focused boundary tests and native acceptance with synthetic files; one scoped review/remediation; report exact delivered versus missing City path.

## Tests and Acceptance

Test profile persistence/distinct IDs, folder denial/revocation, bounds, traversal/symlink exclusion, restart revocation, terminal persistence and duplicate handling, local-only inference rejection, child lifecycle. Native acceptance uses a synthetic folder: create agent, select folder through native picker, execute, inspect actual result, reopen and verify same profile/result. Test the optional real model only if its isolated local policy is proven. No fake model output may be reported as real inference.

## Rollback and Documentation

Close the isolated app and use the original application. No shared-state rollback is required. Do not delete user data automatically. Write one completion report and README link; retain unproven Windows/DAGI items explicitly.

## Open Questions / Verdict

CONDITIONAL_GO for the isolated bounded local pilot authorized by the user's current request. No security blocker is accepted silently. Full cryptographic identity, folder mutation, general agent loop and City connection remain outside this first functional step and are not claimed complete.
