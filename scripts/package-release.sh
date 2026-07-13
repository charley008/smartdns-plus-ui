#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_DIR="$PROJECT_DIR/release"

echo "[1/5] build smartdns-plus-ui"
cd "$PROJECT_DIR"
make

echo "[2/5] prepare release directories"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR/runtime/dist"
mkdir -p "$RELEASE_DIR/runtime/etc"
mkdir -p "$RELEASE_DIR/runtime/db"
mkdir -p "$RELEASE_DIR/runtime/log"
mkdir -p "$RELEASE_DIR/docker/dist"
mkdir -p "$RELEASE_DIR/docker/etc"
mkdir -p "$RELEASE_DIR/docker/db"
mkdir -p "$RELEASE_DIR/docker/log"

echo "[3/5] copy runtime assets"
cp "$PROJECT_DIR/target/smartdns_plus_ui.so" "$RELEASE_DIR/runtime/dist/smartdns_plus_ui.so"
cp -a "$PROJECT_DIR/wwwroot" "$RELEASE_DIR/runtime/dist/wwwroot"
cp "$PROJECT_DIR/docker/etc/smartdns.conf" "$RELEASE_DIR/runtime/etc/smartdns.conf"
cp -a "$PROJECT_DIR/docker/etc/conf.d" "$RELEASE_DIR/runtime/etc/conf.d"
cp -a "$PROJECT_DIR/docker/etc/rules" "$RELEASE_DIR/runtime/etc/rules"
cp "$PROJECT_DIR/scripts/install-runtime-package.sh" "$RELEASE_DIR/runtime/install.sh"

echo "[4/5] copy docker runtime assets"
cp "$PROJECT_DIR/docker/compose.yaml" "$RELEASE_DIR/docker/compose.yaml"
cp "$PROJECT_DIR/docker/Dockerfile" "$RELEASE_DIR/docker/Dockerfile"
cp "$PROJECT_DIR/target/smartdns_plus_ui.so" "$RELEASE_DIR/docker/dist/smartdns_plus_ui.so"
cp -a "$PROJECT_DIR/wwwroot" "$RELEASE_DIR/docker/dist/wwwroot"
cp "$PROJECT_DIR/docker/etc/smartdns.conf" "$RELEASE_DIR/docker/etc/smartdns.conf"
cp -a "$PROJECT_DIR/docker/etc/conf.d" "$RELEASE_DIR/docker/etc/conf.d"
cp -a "$PROJECT_DIR/docker/etc/rules" "$RELEASE_DIR/docker/etc/rules"
cp "$PROJECT_DIR/README.md" "$RELEASE_DIR/README.md"

echo "[5/5] create tarballs"
tar -C "$RELEASE_DIR" -czf "$PROJECT_DIR/smartdns-plus-ui-runtime.tar.gz" runtime README.md
tar -C "$RELEASE_DIR" -czf "$PROJECT_DIR/smartdns-plus-ui-docker-runtime.tar.gz" docker README.md

echo "Done"
echo "Release dir: $RELEASE_DIR"
echo "Artifacts:"
echo "  - $PROJECT_DIR/smartdns-plus-ui-runtime.tar.gz"
echo "  - $PROJECT_DIR/smartdns-plus-ui-docker-runtime.tar.gz"
