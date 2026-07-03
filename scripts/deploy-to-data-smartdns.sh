#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_DIR="${1:-/data/smartdns}"
DIST_DIR="$RUNTIME_DIR/dist"
COMPOSE_FILE=""

detect_existing_compose_file() {
  local candidate
  for candidate in \
    "$RUNTIME_DIR/compose.yaml" \
    "$RUNTIME_DIR/docker-compose.yml" \
    "$RUNTIME_DIR/docker-compose.yaml"
  do
    if [[ -f "$candidate" ]]; then
      printf "%s" "$candidate"
      return 0
    fi
  done
  return 1
}

ensure_runtime_ready() {
  if [[ ! -d "$RUNTIME_DIR" ]]; then
    echo "error: runtime dir not found: $RUNTIME_DIR" >&2
    echo "hint: initialize it first with scripts/install-docker-runtime.sh" >&2
    exit 1
  fi

  if ! COMPOSE_FILE="$(detect_existing_compose_file)"; then
    echo "error: compose file not found in $RUNTIME_DIR" >&2
    echo "hint: initialize it first with scripts/install-docker-runtime.sh" >&2
    exit 1
  fi

  mkdir -p "$DIST_DIR/wwwroot"
}

echo "[1/5] check runtime dir"
ensure_runtime_ready

echo "[2/5] build smartdns-plus-ui"
cd "$PROJECT_DIR"
make

echo "[3/5] sync plugin and wwwroot into $RUNTIME_DIR"
cp "$PROJECT_DIR/target/smartdns_plus_ui.so" "$DIST_DIR/smartdns_plus_ui.so"
cp -a "$PROJECT_DIR/wwwroot/." "$DIST_DIR/wwwroot/"

echo "[4/5] restart docker compose in $RUNTIME_DIR"
cd "$RUNTIME_DIR"
docker compose restart

echo "[5/5] done"
echo "Runtime dir: $RUNTIME_DIR"
echo "Compose:     $COMPOSE_FILE"
echo "WebUI: http://127.0.0.1:6081/"
