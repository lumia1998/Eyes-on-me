#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export AGENT_SERVER_API_BASE_URL="${AGENT_SERVER_API_BASE_URL:-http://127.0.0.1:8787}"
exec cargo run -p client-desktop --manifest-path "$ROOT_DIR/Cargo.toml"
