import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, mkdirSync, copyFileSync, writeFileSync } from "node:fs";
import { dirname, resolve, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readJson = path => JSON.parse(readFileSync(join(root, path), "utf8"));
const pilot = readJson("src-tauri/tauri.local-agent.conf.json");
const windows = readJson("src-tauri/tauri.local-agent.windows.conf.json");
const config = {
  ...pilot, ...windows,
  build: { ...pilot.build, ...windows.build, devUrl: null },
  bundle: { ...pilot.bundle, ...windows.bundle },
};

// Never rename/package the normal Edge executable as if it were the pilot.
assert.equal(config.identifier, "city.daarion.edge.local-agent-pilot");
assert.equal(config.mainBinaryName, "edge-local-agent");
assert.equal(config.app.windows[0].url, "index.html#local-agent-pilot");
assert.deepEqual(config.app.security.capabilities, []);
assert.deepEqual(config.bundle.targets, ["nsis"]);
assert.equal(config.bundle.windows.nsis.installMode, "currentUser");
assert.equal(config.version, readJson("package.json").version);

if (process.argv.includes("--check")) {
  console.log("Windows pilot packaging configuration: PASS (no Windows build executed)");
  process.exit(0);
}
if (process.platform !== "win32" || process.arch !== "x64") {
  throw new Error("Run this build on Windows x64 with the repository Rust toolchain and MSVC build tools.");
}

const target = "x86_64-pc-windows-msvc";
const targetDir = join(root, "src-tauri", "target", "pilot-windows");
const releaseDir = join(targetDir, target, "release");
const output = join(root, "dist-local-agent", "windows-x64");
const env = { ...process.env, CARGO_TARGET_DIR: targetDir, TAURI_CONFIG: JSON.stringify(config) };
const run = (binary, args, cwd = root) => {
  const result = spawnSync(binary, args, { cwd, env, stdio: "inherit", shell: false });
  if (result.error || result.status !== 0) throw new Error(`Build step failed: ${binary}`);
};

run(process.execPath, ["node_modules/typescript/bin/tsc"]);
run(process.execPath, ["node_modules/vite/bin/vite.js", "build"]);
run("cargo", ["build", "--locked", "--release", "--target", target, "--bin", "edge-local-agent",
  "--features", "local-agent-pilot,tauri/custom-protocol"], join(root, "src-tauri"));

const executable = join(releaseDir, "edge-local-agent.exe");
const bytes = readFileSync(executable);
assert.equal(bytes.subarray(0, 2).toString(), "MZ", "Pilot must be a Windows executable");
const peOffset = bytes.readUInt32LE(0x3c);
assert.equal(bytes.subarray(peOffset, peOffset + 4).toString(), "PE\0\0");
assert.equal(bytes.readUInt16LE(peOffset + 4), 0x8664, "Expected Windows x64");

run(process.execPath, ["node_modules/@tauri-apps/cli/tauri.js", "bundle", "--config", JSON.stringify(config),
  "--features", "local-agent-pilot,tauri/custom-protocol", "--target", target, "--bundles", "nsis", "--ci", "--no-sign"]);
const bundleDir = join(releaseDir, "bundle", "nsis");
const installers = readdirSync(bundleDir).filter(name => name.endsWith("-setup.exe"));
assert.equal(installers.length, 1, "Expected exactly one pilot setup artifact");
mkdirSync(output, { recursive: true });
const installerName = "DAARION-Edge-Local-Pilot-windows-x64-setup.exe";
const installer = join(output, installerName);
copyFileSync(join(bundleDir, installers[0]), installer);
const sha256 = data => createHash("sha256").update(data).digest("hex");
const git = args => {
  const result = spawnSync("git", args, { cwd: root, encoding: "utf8", shell: false });
  if (result.status !== 0) throw new Error("Could not record source provenance");
  return result.stdout.trim();
};
const installerBytes = readFileSync(installer);
writeFileSync(join(output, "package-receipt.json"), JSON.stringify({
  schema_version: 1, artifact: "local-agent-pilot", platform: "windows-x64",
  app_identifier: config.identifier, version: config.version,
  source_commit: git(["rev-parse", "HEAD"]), source_dirty: git(["status", "--porcelain"]).length > 0,
  installer: installerName, bytes: installerBytes.length, sha256: sha256(installerBytes),
  executable: "edge-local-agent.exe", executable_sha256: sha256(bytes), signature: "unsigned",
  verification: "built; installation and user acceptance pending",
}, null, 2) + "\n");
console.log("Windows pilot installer and hash receipt created in dist-local-agent/windows-x64.");
