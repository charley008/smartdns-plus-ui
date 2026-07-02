#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_DIR="/data/smartdns"

echo "[1/4] build smartdns-plus-ui"
cd "$PROJECT_DIR"
make

echo "[2/4] copy plugin so to $RUNTIME_DIR"
cp "$PROJECT_DIR/target/smartdns_plus_ui.so" "$RUNTIME_DIR/smartdns_plus_ui.so"

echo "[3/4] restart docker compose in $RUNTIME_DIR"
cd "$RUNTIME_DIR"
sudo docker compose restart

echo "[4/4] done"
echo "WebUI: http://127.0.0.1:6081/"
