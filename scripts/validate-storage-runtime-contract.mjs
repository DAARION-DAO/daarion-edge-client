import { readFileSync } from "node:fs";

const client = readFileSync("src/lib/storageRuntimeClient.ts", "utf8");
const card = readFileSync("src/components/StorageRuntimeCard.tsx", "utf8");
const app = readFileSync("src/App.tsx", "utf8");
const rustCommand = readFileSync("src-tauri/src/runtime_store/commands.rs", "utf8");
const rustTypes = readFileSync("src-tauri/src/runtime_store/types.rs", "utf8");
const rustRoot = readFileSync("src-tauri/src/lib.rs", "utf8");
const packageManifest = readFileSync("package.json", "utf8");

function fail(message) {
  console.error(`FAIL: ${message}`);
  process.exitCode = 1;
}

const command = "get_storage_runtime_status";
if ((client.match(new RegExp(command, "g")) ?? []).length !== 1) {
  fail("typed client must declare the storage command exactly once");
}
if (!rustCommand.includes(`fn ${command}`) || !rustRoot.includes(`runtime_store::commands::${command}`)) {
  fail("Rust command must be implemented and registered");
}
if (!client.includes(`invoke<StorageRuntimeStatus>(storageRuntimeCommand)`)) {
  fail("typed client must invoke the read-only command through its constant");
}
if (card.includes("invoke(")) {
  fail("Dashboard card must use the typed client, not invoke directly");
}
if (!app.includes("<StorageRuntimeCard />") || !app.includes('from "./components/StorageRuntimeCard"')) {
  fail("Dashboard must mount the storage runtime card");
}

for (const state of [
  "initializing",
  "healthy",
  "warning",
  "unavailable",
  "migration_failed",
  "integrity_failed",
  "locked",
  "permission_denied",
  "resource_limited",
]) {
  if (!client.includes(`"${state}"`) || !card.includes(`${state}:`)) {
    fail(`frontend contract does not render state ${state}`);
  }
}

for (const field of [
  "state",
  "initialized",
  "schema_version",
  "database_health",
  "database_size_bytes",
  "storage_backend",
  "sqlite_version",
  "persistence_state",
  "last_start_time_ms",
  "error_code",
]) {
  if (!client.includes(`${field}:`) || !rustTypes.includes(field)) {
    fail(`frontend/Rust status field is missing: ${field}`);
  }
}

for (const forbidden of ["database_path", "sql_text", "raw_error", "connection_string"]) {
  if (client.includes(forbidden) || rustTypes.includes(forbidden)) {
    fail(`public status contract leaks forbidden field: ${forbidden}`);
  }
}

for (const copy of ["Storage Runtime", "SQLite · Local device only", "No cloud sync"]) {
  if (!card.includes(copy)) fail(`required local-only UI copy is missing: ${copy}`);
}

for (const forbiddenCommand of [
  "execute_sql",
  "query_sql",
  "create_conversation",
  "append_message",
  "create_task",
  "append_audit_event",
]) {
  if (rustRoot.includes(forbiddenCommand) || rustCommand.includes(forbiddenCommand)) {
    fail(`unauthorized storage command exists: ${forbiddenCommand}`);
  }
}
if (client.includes(": any") || card.includes(": any")) {
  fail("storage frontend contract must not use any");
}
if (!packageManifest.includes('"test:storage-runtime-contract"')) {
  fail("required storage runtime contract package script is missing");
}

if (!process.exitCode) {
  console.log("PASS: storage runtime frontend/Rust contract is aligned");
}
