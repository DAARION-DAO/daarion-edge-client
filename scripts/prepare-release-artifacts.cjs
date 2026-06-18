const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// 1. Read version from package.json
const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, '../package.json'), 'utf8'));
const version = packageJson.version;

// 2. Read environment variables
const label = process.env.ARTIFACT_LABEL; // e.g. "macos-arm64", "macos-x86_64", "windows-x86_64", "linux-x86_64", "android-arm64"
const rustTarget = process.env.RUST_TARGET || ""; // e.g. "aarch64-apple-darwin", etc.

console.log(`Preparing artifacts for version: ${version}, label: ${label}, target: ${rustTarget}`);

// Create output directory
const outputDir = path.join(__dirname, '../dist/artifacts');
fs.mkdirSync(outputDir, { recursive: true });

// Define paths to search
const baseDir = path.join(__dirname, '..');
const searchPaths = [];
const artifactsMeta = [];

if (label === 'android-arm64') {
  const filename = `Daarion.Edge_${version}_android_universal_release.apk`;
  searchPaths.push({
    dir: 'src-tauri/gen/android/app/build/outputs/apk/universal/release',
    ext: '.apk',
    renameFn: () => filename
  });
  artifactsMeta.push({
    platform: "android",
    arch: "arm64",
    kind: "apk",
    filename: filename,
    status: "beta"
  });
} else if (label === 'macos-arm64') {
  const targetSub = rustTarget ? `target/${rustTarget}` : 'target';
  const filename = `Daarion.Edge_${version}_aarch64.dmg`;
  searchPaths.push({
    dir: `src-tauri/${targetSub}/release/bundle/dmg`,
    ext: '.dmg',
    renameFn: () => filename
  });
  artifactsMeta.push({
    platform: "macos",
    arch: "arm64",
    kind: "dmg",
    filename: filename,
    status: "beta"
  });
} else if (label === 'macos-x86_64') {
  const targetSub = rustTarget ? `target/${rustTarget}` : 'target';
  const filename = `Daarion.Edge_${version}_x64.dmg`;
  searchPaths.push({
    dir: `src-tauri/${targetSub}/release/bundle/dmg`,
    ext: '.dmg',
    renameFn: () => filename
  });
  artifactsMeta.push({
    platform: "macos",
    arch: "x64",
    kind: "dmg",
    filename: filename,
    status: "beta"
  });
} else if (label === 'linux-x86_64') {
  const filename = `Daarion.Edge_${version}_amd64.AppImage`;
  searchPaths.push({
    dir: 'src-tauri/target/release/bundle/appimage',
    ext: '.AppImage',
    renameFn: () => filename
  });
  artifactsMeta.push({
    platform: "linux",
    arch: "x64",
    kind: "appimage",
    filename: filename,
    status: "beta"
  });
} else if (label === 'windows-x86_64') {
  const filenameExe = `Daarion.Edge_${version}_x64-setup.exe`;
  const filenameMsi = `Daarion.Edge_${version}_x64_en-US.msi`;
  searchPaths.push({
    dir: 'src-tauri/target/release/bundle/nsis',
    ext: '.exe',
    renameFn: () => filenameExe
  });
  searchPaths.push({
    dir: 'src-tauri/target/release/bundle/msi',
    ext: '.msi',
    renameFn: () => filenameMsi
  });
  artifactsMeta.push({
    platform: "windows",
    arch: "x64",
    kind: "exe",
    filename: filenameExe,
    status: "beta"
  }, {
    platform: "windows",
    arch: "x64",
    kind: "msi",
    filename: filenameMsi,
    status: "beta"
  });
} else {
  console.warn(`Unknown or missing ARTIFACT_LABEL: "${label}". Skipping rename.`);
}

// Perform search and copy
let found = 0;
for (const item of searchPaths) {
  const fullDir = path.join(baseDir, item.dir);
  if (!fs.existsSync(fullDir)) {
    console.warn(`Directory does not exist: ${fullDir}`);
    continue;
  }
  
  const files = fs.readdirSync(fullDir);
  for (const file of files) {
    if (file.endsWith(item.ext)) {
      const srcPath = path.join(fullDir, file);
      const newName = item.renameFn();
      const destPath = path.join(outputDir, newName);
      
      console.log(`Found artifact: ${srcPath} -> Copying to: ${destPath}`);
      fs.copyFileSync(srcPath, destPath);
      found++;
    }
  }
}

// Create platform-specific release manifest
if (found > 0 && artifactsMeta.length > 0) {
  const manifest = {
    project: "DAARION Edge Client",
    channel: "beta",
    version: version,
    manual_update_only: true,
    platform_label: label,
    artifacts: artifactsMeta
  };
  
  const manifestFilename = `release-manifest-${label || 'unknown'}.json`;
  const manifestPath = path.join(outputDir, manifestFilename);
  fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
  console.log(`Generated release manifest at: ${manifestPath}`);

  const checksumFilename = `SHA256SUMS-${label || 'unknown'}.txt`;
  const checksumPath = path.join(outputDir, checksumFilename);
  const checksumLines = fs.readdirSync(outputDir)
    .filter((file) => !file.startsWith('SHA256SUMS-'))
    .sort()
    .map((file) => {
      const filePath = path.join(outputDir, file);
      const digest = crypto
        .createHash('sha256')
        .update(fs.readFileSync(filePath))
        .digest('hex');
      return `${digest}  ${file}`;
    });
  fs.writeFileSync(checksumPath, `${checksumLines.join('\n')}\n`);
  console.log(`Generated SHA-256 checksums at: ${checksumPath}`);
}

if (found === 0) {
  console.warn("No artifacts found to prepare. This might be normal if running locally outside CI.");
} else {
  console.log(`Prepared ${found} artifacts successfully.`);
}
