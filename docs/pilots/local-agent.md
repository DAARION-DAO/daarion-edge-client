# Local agent pilot

This isolated native Edge pilot demonstrates the first half of employee onboarding: create a personal local profile and node reference, choose one folder, run a bounded read-only task, and reopen its saved result. City membership, cryptographic enrollment and shared compute are not implemented here.

Open the packaged **DAARION Edge Local Pilot.app** on the tested Apple Silicon Mac. Create an agent, choose a folder, and select **Виконати огляд папки**. The exact file listing works without a model. To add an AI overview, select **Підключити локальну модель**, choose an already-installed model, then select **Зробити AI-огляд папки**. Model output is advisory text and cannot execute tools.

The pilot reads one directory level, at most 256 entries, skipping hidden entries and symlinks. It reads at most eight small `.txt`/`.md` excerpts. A held directory capability confines file access. Closing or revoking removes the folder grant. The profile and results remain in the pilot's private app data; source files are unchanged. The network card distinguishes a local operator connection from DAGI employee enrollment.

Optional inference reuses Edge's existing local-only service. It starts a temporary loopback Ollama process with cloud disabled, using installed artifacts; there is no automatic model download. CPU requests use a bounded context and token limit, and unload after completion. The existing Ollama service and City runtime settings are not changed. Only the process tree created by the pilot is stopped at exit. This pilot expects Ollama in a standard installation location.

## Build the isolated native profile

Use the repository's supported Rust and Node toolchains. From the repository root:

```sh
npm ci
npm run build
TAURI_CONFIG="$(cat src-tauri/tauri.local-agent.conf.json)" cargo build \
  --manifest-path src-tauri/Cargo.toml --bin edge-local-agent \
  --features local-agent-pilot,tauri/custom-protocol
python3 scripts/package-local-agent-pilot.py \
  --binary src-tauri/target/debug/edge-local-agent
```

The helper creates `dist-local-agent/DAARION Edge Local Pilot.app` and a receipt with the final signed binary hash. The Mac package is ad-hoc signed for local testing; it is not notarized for general distribution.

For a Windows x64 installer, use Windows with MSVC build tools, the repository Rust toolchain and Node 22:

```powershell
npm ci
node scripts/package-local-agent-windows.mjs
```

This reuses the existing Tauri/NSIS installer technology. It explicitly builds `edge-local-agent`, packages the isolated pilot identity as a current-user install, and writes the installer and SHA-256 receipt under `dist-local-agent/windows-x64/`. The normal Edge executable is not renamed into a pilot. The package is unsigned; Windows trust prompts and employee-device acceptance remain separate checks. WebView2 may be downloaded by the installer if missing. Models are not bundled or downloaded.

The `local-agent-windows.yml` workflow is prepared to build on Windows and check installation, native device scanning, agent creation, restart persistence and uninstall on a disposable runner. It uploads an Actions artifact with a short retention period; it has no release/deployment step or write permission. It does not test a folder task through Windows UI, install models or call City. The first authorized run was blocked before any step by GitHub's account billing lock, so there is still no Windows installer artifact. Do not dispatch the normal `release.yml`, which publishes releases.

The previously published v0.2.2-3 setup/MSI remains a separate older Edge release. It predates the uncommitted local-agent pilot and must not be offered as this pilot's Windows download.

## Device scanning and installation advice

Select **Перевірити пристрій** in the native pilot. It reuses Edge's existing capability measurements for OS, architecture, CPU and memory, and checks whether Ollama exists at a supported installation location. This is a basic local check, not a complete compatibility advisor. It does not expose the legacy heuristic GPU/model catalogue, install software, start a model, upload hardware details or enroll a node. Finding an executable does not prove a compatible runtime or an installed model; the separate model action verifies that path.

The intended employee path is MicroDAO download -> small platform client -> native scan -> explained installation recommendation -> user confirmation -> local agent. Browser platform detection only chooses the initial client. Full GPU/driver/available-volume checks and an automatically selected model installation are still missing.

Previously discussed OSS components remain relevant:

- [Local-LLM-Hardware-Checker](https://huggingface.co/spaces/0xSojalSec/Local-LLM-Hardware-Checker/blob/main/README.md): currently a static hardware/model explorer, useful for the comparison UI; not a full native scanner or installer.
- [Jan hardware plugin](https://github.com/janhq/jan/tree/main/src-tauri/plugins/tauri-plugin-hardware): native hardware integration reference for Tauri.
- [Lemonade](https://github.com/lemonade-sdk/lemonade): candidate runtime/device/model installation component; not adopted by this packaging step.
- [local-ai-registry](https://github.com/0xSero/local-ai-registry): compatibility/recipe/evidence data; candidate recipes must not be presented as verified installations.
- [LocalScore](https://github.com/cjpais/LocalScore): performance measurements, separate from initial compatibility detection.
- [llmfit](https://github.com/AlexsJones/llmfit): native hardware/model recommendations in JSON. Using its scanner would not require adopting Hermes; neither is newly integrated here.

See [Windows preparation and verification](../planning/phases/edge-windows-installer-pilot-completion.md) for the current evidence boundary.

The normal Edge application remains the default Cargo target. The pilot has its own feature, entrypoint and app identifier. Rollback is closing the pilot and opening the original application; do not delete saved results automatically.

See [the completion report](../planning/phases/edge-local-agent-folder-pilot-completion.md) for actual acceptance results and the next integration boundary.

## Connect the existing local City panel

Start the local model and choose an installed model. Select **Підключити до міської панелі**. The native bridge starts with a fresh session credential and asks the existing OpenBot API to verify the agent. Only a successful completed preflight allows a private City profile to be created or updated. The profile points to the existing Edge agent; it is not a second local identity or a device registration.

The current panel rejects the extra loopback endpoint. The verified native UI therefore shows **Edge готовий · панель очікує**, and no City profile is created. This condition needs an operator-approved server allowlist update before the existing City chat can be verified. The connector never edits that policy itself.

Once allowed, open the existing panel using **Відкрити міську панель** and select the personal agent. Conversation text goes to the selected local model. Its answer is saved locally and returned through AG-UI. Only this conversation’s recent text is supplied; folder grants, private task history and City tools are not shared. The actual City UI round trip is still unverified.

**Від’єднати міський чат** revokes the session listener and cancels active work. Closing the app also stops the pilot-owned model process. Saved results remain. Reconnecting updates the matching owned City profile with a fresh credential. A City profile may remain visible while its agent is offline; this prototype does not change City presence handling.

This connector is restricted to the existing local single-user operator panel. It does not authenticate employees through MicroDAO, reach other computers, grant access to core City agents or contribute compute. OpenFabric remains a candidate for the later network/compute boundary, and is not the transport used by this local AG-UI pilot.

See [the joining report](../planning/phases/edge-city-join-pilot-completion.md) for the exact verified scope and remaining gates.

### Contract for the parallel City workstream

The operator relayed confirmation that Edge belongs in Agents as a separate `remote-ag-ui` agent, outside Core. The existing connector follows that separation. Its protocol contract is:

- Method/route: `POST /agents/<agent_id>/ag-ui`, served by the pilot-owned loopback listener. The running session constructs the endpoint; it is not a fixed employee identity or a public service address.
- Request: AG-UI `threadId`, `runId` and text messages. Response: `text/event-stream` with run/message events and a terminal success/error. Replayed task IDs are idempotent; conflicting replays are refused.
- Authentication: `Authorization: Bearer <session credential>`. The app creates a fresh random session credential and supplies it directly to the existing encrypted endpoint-auth API after preflight. It is never displayed in chat, URLs, frontend state or build artifacts. Closing/disconnecting revokes the listener; reconnecting rotates the credential.
- Verified boundary: native protocol tests and the earlier real local AG-UI/model exchange. Actual City registration remains unverified under its current endpoint policy.
- Reachability gap: the listener accepts loopback connections only. A City server on another employee computer/node cannot reach it as currently implemented. A permitted transport/access path plus employee authentication is still required; selecting `remote-ag-ui` alone does not create that connectivity.

This Windows packaging task does not change City policies, Core agents, credentials, server processes or network exposure.
