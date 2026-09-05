# Edge City joining pilot — completion

TASK = edge-city-join-pilot
PRIMARY_OUTCOME = The existing local agent connects to the existing City panel and preserves a conversation result.
RESULT = PARTIAL
IMPLEMENTED = Authenticated session-only AG-UI bridge, native existing-panel registration/readback path, explicit local-only connection card, idempotent saved tasks and revocation.
FILES_CHANGED = 10 in this step; reviewed prior local-agent pilot changes carried forward in isolation.
TESTS_RUN = Four focused Rust tests, one real-model HTTP integration test, frontend typecheck/build, native build/signature verification, inference contract, diff check. Storage boundary validator fails identically in the prior pilot.
CORE_USER_PATH_SMOKE = Local protocol PASS; actual native button reaches the existing panel and displays its endpoint-policy refusal. City conversation and MicroDAO employee joining NOT PROVEN.
P0 = None found in the scoped review; not a general security certification.
P1 = Existing City endpoint allowlist rejects Edge. Full employee flow additionally lacks configured MicroDAO identity/entitlement linkage and other-device transport.
P2_BACKLOG = Existing pilot fails the fixed Tauri importer inventory in the storage contract validator; no regression from this connector. Remote profile presence after local disconnect is not updated.
P3_BACKLOG = Distribution signing, installer packaging and shared compute after the first connected-user demonstration.
PRODUCTION_WRITES = NO
MIGRATIONS_APPLIED = NO
DEPLOYMENT_PERFORMED = NO
CURRENT_BRANCH = codex/edge-city-join-pilot
STARTING_MAIN_SHA = eafb30d8548e2b1e3bc36fc34b2c8f0b36f633b7
FINAL_HEAD_SHA = eafb30d8548e2b1e3bc36fc34b2c8f0b36f633b7; implementation remains uncommitted.
PR = NONE
NEXT_SINGLE_ACTION = Approve and apply the exact existing-City endpoint allowance, then verify this same agent in its real chat.

## Implemented boundary

`src-tauri/src/local_agent/city.rs` reuses the installed local inference provider and the existing profile. It serves bounded AG-UI requests with a session-only bearer key and no browser CORS access. It does not access the folder grant. Only bounded conversation text reaches the model; external tool/state/system instructions receive no execution authority. Results use deterministic run IDs and are saved before a success terminal event. Replays return the saved result; conflicting content with the same run ID is refused.

The native registration path is restricted to the verified local OpenBot admin actor. It calls the existing policy preflight before any profile write. A successful, completed preflight is required, then an owned matching endpoint is updated or a private profile created, followed by authenticated endpoint readback. The random credential stays out of frontend state, prompts, URLs and evidence; the existing City encrypted credential field is its intended consumer. No profile was created during this task.

One scoped post-implementation review identified a revocation race during asynchronous registration. The single remediation added cancellation notification and a session-token check, so revocation cannot restore a stale connected status or admit a previous session request into a new session. It also requires the exact observed local admin role. Final tests and the native build passed after that change.

Dependency review: Hyper 1.8.1, hyper-util 0.1.20 and http-body-util 0.1.3 were already locked transitively. Direct use is optional under the pilot feature; enabling the HTTP server adds only httpdate 1.0.3 to this step's lock delta. Capability filesystem dependencies belong to the carried-forward local folder pilot. No production database or existing schema changes were made; optional City metadata reads legacy task JSON.

## Verification and limitations

- `npm run build`: PASS (TypeScript and Vite); native package includes this frontend.
- `cargo test --offline --manifest-path src-tauri/Cargo.toml --lib --features local-agent-pilot local_agent -- --nocapture`: PASS, 4 tests; explicit real-model test ignored in this command.
- `cargo test --offline --manifest-path src-tauri/Cargo.toml --lib --features local-agent-pilot real_local_ag_ui_round_trip_replay_and_shutdown -- --ignored --nocapture`: PASS on final code, one real HTTP integration test, 9.49 seconds. It uses an already-installed qwen3.5:9b model, a separate local-only process and synthetic private store. No downloads or external model calls. Real answer describes local execution and no file access. It verifies saved completion, identical replay, conflicting replay refusal, missing-key refusal, Origin refusal and listener shutdown.
- `cargo build --offline --manifest-path src-tauri/Cargo.toml --bin edge-local-agent --features local-agent-pilot,tauri/custom-protocol`: PASS with the pilot TAURI_CONFIG and repository-supported Rust 1.95. Existing compiler warnings are retained.
- macOS packaging and `codesign --verify --deep --strict`: PASS. Signed binary SHA-256: `cea2a5863fed7cffd785a1eb8dfdce22186ccf5c942100eb9eee539e43dcc707`. Ad-hoc signature, not notarized. Windows execution is not tested.
- `node scripts/validate-inference-contract.mjs`: PASS.
- `node scripts/validate-storage-runtime-contract.mjs`: FAIL with `PRIMARY_MODULE_BOUNDARY_GATE: Tauri core importer baseline mismatch`. Reproduced against the untouched previous local-agent pilot; the importer inventory is unchanged by this step. No claim of a green complete repository release gate.
- `git diff --check`: PASS. Scoped source scan reports no credential/private-path literals introduced by this step.
- Actual native UI: existing personal profile, same agent/node references and old saved results survived. Starting the installed model enabled the connect button. Clicking it returned the real existing panel's policy refusal and showed **Edge готовий · панель очікує**. No profile creation was claimed. Quitting/reopening removed the session connection and retained profile/results. A screenshot was viewed in the task.
- Real OpenBot chat consumption remains blocked. Its producer/consumer source contract was inspected; the isolated protocol proof is not a substitute for City UI acceptance.
- City canonical checkout remained clean at `5cd0061c4a58a6fbe27854f0f89429de13e756cd`. No City runtime/configuration/source restart or mutation. Previous worktrees and MicroDAO production are untouched.

Evidence JSON and local package receipts remain in ignored `.pilot-evidence/` and `dist-local-agent/`; private operational evidence is not committed. An initial UI automation attempt could not access the window; the dedicated native UI tool subsequently inspected and exercised the exact new application successfully.

## Concrete final City change — prepared, not applied

The existing API's host-and-port allowlist must include exactly the new Edge bridge entry `127.0.0.1:19436`, preserving every current entry in `AGENT_ENDPOINT_ALLOWED_HOSTS`. Do not set a broad private-host bypass, alter bind addresses, expose a public port or change single-user authentication. The active launch configuration has not yet been identified; the inspected checkout has no `.env`, so the generic upstream start script must not be substituted for the real launcher.

After operator approval, identify the actual current server launcher without printing its secrets, retain its environment and rollback configuration, and restart only that API server at an agreed point in the parallel City work. Start the Edge model, reconnect, require a completed preflight plus private-profile readback, then use the existing City interface to send one synthetic message to that personal agent. Check both the visible City reply and the saved Edge result. If acceptance fails, restore the former allowlist using that same launcher. Never reset the City checkout or replace its database/runtime overlay.

This permission applies only to the local operator demonstration. MicroDAO consent/discovery already exists, but an actual registered OIDC/OAuth client, claim/entitlement mapping and reachable employee-device transport are still explicit gaps. Do not reuse a browser session token or a device invitation as proof of employee authentication. Any managed MicroDAO auth/production write requires its own concrete, authorized rollout. OpenFabric remains a candidate for the later network/compute boundary, not an implemented dependency of this AG-UI pilot.

## Release decision

Local protocol and native preflight behavior: verified. Overall employee/City milestone: PARTIAL. Repository release gate: FAIL for promotion while required City acceptance is absent and the inherited storage validator is red. These facts do not erase the working local prototype or justify expanding this task into an infrastructure audit. Close the pilot to revoke its listener and owned model process; keep saved results.
