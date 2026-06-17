#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_FILE="$(mktemp)"
trap 'rm -f "$OUTPUT_FILE"' EXIT

cd "$ROOT_DIR/src-tauri"

if ! cargo check >"$OUTPUT_FILE" 2>&1; then
  cat "$OUTPUT_FILE"
  exit 1
fi

PROTECTED_MODULES=(
  "src/config.rs"
  "src/enrollment.rs"
  "src/heartbeat.rs"
  "src/lib.rs"
  "src/messaging.rs"
  "src/provisioning.rs"
  "src/registry_client.rs"
  "src/worker/mod.rs"
  "src/worker/onboarding.rs"
)

found=0
for module in "${PROTECTED_MODULES[@]}"; do
  if grep -Eq "^ +--> ${module}:" "$OUTPUT_FILE"; then
    found=1
  fi
done

if [[ "$found" -ne 0 ]]; then
  echo "cargo check reported warnings in protected hardening modules:"
  for module in "${PROTECTED_MODULES[@]}"; do
    grep -En "^ +--> ${module}:" "$OUTPUT_FILE" || true
  done
  echo
  echo "Run cargo check and remove or explicitly justify warnings in those modules."
  exit 1
fi

echo "No Rust warnings reported in protected hardening modules."
