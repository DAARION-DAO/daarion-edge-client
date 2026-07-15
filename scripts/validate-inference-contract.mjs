import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = process.cwd();
const read = (path) => readFileSync(resolve(root, path), "utf8");
const fail = (message) => {
  console.error(`FAIL: ${message}`);
  process.exitCode = 1;
};

const adapter = read("src/lib/inferenceClient.ts");
const panel = read("src/components/LocalInferencePanel.tsx");
const app = read("src/App.tsx");
const rustRoot = read("src-tauri/src/lib.rs");
const capability = read("src-tauri/capabilities/default.json");

const commands = [...adapter.matchAll(/^\s+\w+: "([a-z0-9_]+)",$/gm)].map(
  (match) => match[1],
);

if (commands.length !== 7) fail(`expected 7 typed inference commands, found ${commands.length}`);
for (const command of commands) {
  if (!rustRoot.includes(`inference::commands::${command}`)) {
    fail(`frontend command ${command} is not registered by Tauri`);
  }
}

if (panel.includes("invoke(")) fail("mounted inference panel bypasses the typed adapter");
for (const forbidden of ["RoutingToNetwork", "RemoteExecution", "run_chat", "DAARION Network"] ) {
  if (adapter.includes(forbidden) || panel.includes(forbidden)) {
    fail(`mounted inference surface contains forbidden legacy state: ${forbidden}`);
  }
}
for (const falseClaim of ["llama.cpp", "Zero-latency", "100% locally"]) {
  if (panel.includes(falseClaim)) fail(`mounted UI contains an unverified runtime claim: ${falseClaim}`);
}

if (/shell:(default|allow-execute)/.test(capability)) {
  fail("main-window capability still exposes shell execution");
}
if (rustRoot.includes("tauri_plugin_shell::init")) {
  fail("application still initializes the browser-facing shell plugin");
}
if (app.includes("LocalModelsPanel")) {
  fail("placeholder LocalModelsPanel became reachable from the application shell");
}

for (const legacyFile of [
  "src-tauri/src/models/local_inference.rs",
  "src-tauri/src/models/inference_arbitrator.rs",
  "src-tauri/src/models/ollama.rs",
]) {
  if (existsSync(resolve(root, legacyFile))) fail(`legacy inference path remains: ${legacyFile}`);
}

for (const inferenceFile of [
  "src-tauri/src/inference/commands.rs",
  "src-tauri/src/inference/model_resolver.rs",
  "src-tauri/src/inference/ollama_provider.rs",
  "src-tauri/src/inference/policy.rs",
  "src-tauri/src/inference/provider.rs",
  "src-tauri/src/inference/service.rs",
  "src-tauri/src/inference/types.rs",
]) {
  const source = read(inferenceFile);
  if (source.includes("println!") || source.includes("eprintln!")) {
    fail(`inference code logs data directly: ${inferenceFile}`);
  }
}

if (!adapter.includes('"local-inference-event"')) {
  fail("typed adapter does not subscribe to the canonical inference event");
}
if (!adapter.includes("export interface PrepareLocalModelRequest")) {
  fail("typed adapter does not define the model-preparation request contract");
}
if (!adapter.includes('cancelPreparation: "cancel_local_model_preparation"')) {
  fail("typed adapter does not expose the dedicated preparation cancellation command");
}
if (!adapter.includes("local_only_verified: boolean")) {
  fail("typed adapter does not expose explicit LocalOnly verification state");
}
if (!panel.includes('policy_unverified: "Ollama reachable · LocalOnly not verified"')) {
  fail("mounted inference panel does not distinguish unverified LocalOnly policy");
}
if (!panel.includes("providerStatus.local_only_verified")) {
  fail("mounted inference panel does not gate preparation on LocalOnly verification");
}
if (!panel.includes("!providerStatus?.local_only_verified")) {
  fail("mounted inference panel does not gate Run on LocalOnly verification");
}
if (!panel.includes("DAARION did not change Ollama settings")) {
  fail("mounted inference panel lacks truthful host-configuration remediation text");
}
if (panel.includes('" · installed"')) {
  fail("mounted inference panel still labels unverified inventory as generically installed");
}
if (!panel.includes('aria-label="Cancel local model preparation"')) {
  fail("mounted inference panel does not expose model-preparation cancellation");
}
if (!panel.includes('transition("preparing")') || !panel.includes('transition("cancelled")')) {
  fail("mounted inference panel lacks truthful preparation terminal states");
}
const prepareCall = panel.indexOf("await prepareLocalModel({");
const postPrepareRefresh = panel.indexOf("await refresh();", prepareCall);
const postPrepareCompletion = panel.indexOf('transition("completed_locally")', postPrepareRefresh);
if (
  prepareCall < 0 ||
  postPrepareRefresh < prepareCall ||
  postPrepareCompletion < postPrepareRefresh
) {
  fail("mounted inference panel can report local preparation completion before postflight verification");
}
if (panel.includes('aria-label="Preparing local model"')) {
  fail("mounted inference panel still exposes a non-cancellable preparation spinner");
}

if (!process.exitCode) console.log("PASS: local inference frontend/Rust contract is aligned");
