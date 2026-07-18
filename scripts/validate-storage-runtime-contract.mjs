import { readFileSync, readdirSync } from "node:fs";

const paths = {
  client: "src/lib/storageRuntimeClient.ts",
  card: "src/components/StorageRuntimeCard.tsx",
  app: "src/App.tsx",
  rustCommand: "src-tauri/src/runtime_store/commands.rs",
  rustTypes: "src-tauri/src/runtime_store/types.rs",
  rustRoot: "src-tauri/src/lib.rs",
  packageManifest: "package.json",
};

const sources = Object.fromEntries(
  Object.entries(paths).map(([key, path]) => [key, readFileSync(path, "utf8")]),
);
sources.runtimeStore = readdirSync("src-tauri/src/runtime_store")
  .filter((file) => file.endsWith(".rs"))
  .map((file) => readFileSync(`src-tauri/src/runtime_store/${file}`, "utf8"))
  .join("\n");

const command = "get_storage_runtime_status";

function snakeCase(value) {
  return value.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();
}

function rustEnum(source, name) {
  const body = source.match(new RegExp(`enum\\s+${name}\\s*\\{([\\s\\S]*?)\\}`))?.[1];
  if (!body) return null;
  return [...body.matchAll(/^\s*([A-Z][A-Za-z0-9]*)\s*,/gm)].map((match) =>
    snakeCase(match[1]),
  );
}

function tsUnion(source, name) {
  const body = source.match(new RegExp(`export\\s+type\\s+${name}\\s*=([\\s\\S]*?);`))?.[1];
  if (!body) return null;
  return [...body.matchAll(/"([a-z0-9_]+)"/g)].map((match) => match[1]);
}

function sameMembers(left, right) {
  return (
    left !== null &&
    right !== null &&
    left.length === right.length &&
    [...left].sort().every((value, index) => value === [...right].sort()[index])
  );
}

function commandArguments(source) {
  const match = source.match(
    new RegExp(`fn\\s+${command}\\s*\\(([\\s\\S]*?)\\)\\s*->`),
  );
  if (!match) return null;
  return match[1]
    .split(",")
    .map((argument) => argument.trim())
    .filter(Boolean);
}

function validate(candidate) {
  const errors = [];
  const require = (condition, message) => {
    if (!condition) errors.push(message);
  };

  require(
    (candidate.client.match(new RegExp(`"${command}"`, "g")) ?? []).length === 1,
    "typed client must declare the exact storage command once",
  );
  require(
    candidate.client.includes(
      "invoke<StorageRuntimeStatus>(storageRuntimeCommand)",
    ),
    "typed client must invoke the read-only command through its constant",
  );

  const argumentsList = commandArguments(candidate.rustCommand);
  require(argumentsList !== null, "Rust storage status command is missing");
  if (argumentsList) {
    const userArguments = argumentsList.filter(
      (argument) =>
        !/:\s*tauri::AppHandle(?:<[^>]+>)?$/.test(argument) &&
        !/:\s*tauri::State(?:<[^>]+>)?$/.test(argument),
    );
    require(
      userArguments.length === 0,
      "Rust storage status command must have no frontend-deserialized arguments",
    );
  }

  require(
    (
      candidate.rustRoot.match(
        /runtime_store::commands::get_storage_runtime_status/g,
      ) ?? []
    ).length === 1,
    "Rust storage status command must be registered exactly once",
  );

  for (const enumName of [
    "StorageRuntimeState",
    "StorageRuntimeErrorCode",
    "PersistenceState",
  ]) {
    require(
      sameMembers(
        rustEnum(candidate.rustTypes, enumName),
        tsUnion(candidate.client, enumName),
      ),
      `Rust and TypeScript ${enumName} variants must match exactly`,
    );
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
    require(
      candidate.client.includes(`${field}:`) && candidate.rustTypes.includes(field),
      `frontend/Rust status field is missing: ${field}`,
    );
  }

  for (const forbidden of [
    "database_path",
    "sql_text",
    "raw_error",
    "connection_string",
  ]) {
    require(
      !candidate.client.includes(forbidden) && !candidate.rustTypes.includes(forbidden),
      `public status contract leaks forbidden field: ${forbidden}`,
    );
  }

  for (const state of tsUnion(candidate.client, "StorageRuntimeState") ?? []) {
    require(candidate.card.includes(`${state}:`), `Dashboard does not render state ${state}`);
  }
  require(!candidate.card.includes("invoke("), "Dashboard card must not invoke Tauri directly");
  require(
    candidate.card.includes("getStorageRuntimeStatus"),
    "Dashboard card must use the typed storage client",
  );
  require(
    candidate.app.includes("<StorageRuntimeCard />") &&
      candidate.app.includes('from "./components/StorageRuntimeCard"'),
    "existing Dashboard must mount the storage runtime card",
  );

  for (const copy of ["Storage Runtime", "SQLite · Local device only", "No cloud sync"]) {
    require(candidate.card.includes(copy), `required local-only UI copy is missing: ${copy}`);
  }

  for (const forbiddenCommand of [
    "execute_sql",
    "query_sql",
    "run_sql",
    "create_conversation",
    "append_message",
    "create_task",
    "append_audit_event",
    "delete_conversation",
  ]) {
    require(
      !candidate.rustRoot.includes(forbiddenCommand) &&
        !candidate.rustCommand.includes(forbiddenCommand),
      `unauthorized storage command exists: ${forbiddenCommand}`,
    );
  }
  require(
    !/#\[tauri::command\][\s\S]{0,300}fn\s+\w+\s*\([^)]*\bpath\s*:/i.test(
      candidate.runtimeStore,
    ),
    "runtime_store must expose no Tauri command with a path argument",
  );
  require(
    !candidate.client.includes(": any") && !candidate.card.includes(": any"),
    "storage frontend contract must not use any",
  );
  require(
    candidate.packageManifest.includes('"test:storage-runtime-contract"'),
    "required storage runtime contract package script is missing",
  );
  return errors;
}

function mutationMustFail(label, mutate, expectedFragment) {
  const candidate = { ...sources };
  mutate(candidate);
  const errors = validate(candidate);
  if (!errors.some((error) => error.includes(expectedFragment))) {
    console.error(`FAIL: validator self-test did not reject ${label}`);
    process.exitCode = 1;
  }
}

for (const error of validate(sources)) {
  console.error(`FAIL: ${error}`);
  process.exitCode = 1;
}

mutationMustFail(
  "a missing TypeScript error variant",
  (candidate) => {
    candidate.client = candidate.client.replace('  | "internal";\n', ";\n");
  },
  "StorageRuntimeErrorCode variants",
);
mutationMustFail(
  "an extra TypeScript error variant",
  (candidate) => {
    candidate.client = candidate.client.replace(
      '  | "internal";\n',
      '  | "internal"\n  | "unexpected";\n',
    );
  },
  "StorageRuntimeErrorCode variants",
);
mutationMustFail(
  "a frontend-deserialized Rust path argument",
  (candidate) => {
    candidate.rustCommand = candidate.rustCommand.replace(
      "app: tauri::AppHandle",
      "path: String, app: tauri::AppHandle",
    );
  },
  "no frontend-deserialized arguments",
);
mutationMustFail(
  "a missing Tauri registration",
  (candidate) => {
    candidate.rustRoot = candidate.rustRoot.replace(
      "runtime_store::commands::get_storage_runtime_status,",
      "",
    );
  },
  "registered exactly once",
);

if (!process.exitCode) {
  console.log(
    "PASS: storage runtime command, enums, authority, UI, and validator self-tests are aligned",
  );
}
