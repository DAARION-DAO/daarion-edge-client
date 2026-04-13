#!/usr/bin/env bash
set -euo pipefail
rg -n "MATRIX_SHARED_SECRET|MATRIX_BRIDGE_TOKEN|GENESIS_GRANT|ACCESS_TOKEN|PRIVATE_KEY|messaging_token" src src-tauri && {
  echo "Forbidden secret-like pattern found"
  exit 1
}
echo "No forbidden patterns found"
