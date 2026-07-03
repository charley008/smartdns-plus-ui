#!/usr/bin/env bash
set -euo pipefail

BUNDLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

SRC_DIST_DIR="$BUNDLE_DIR/dist"
SRC_ETC_DIR="$BUNDLE_DIR/etc"
SRC_CONFD_DIR="$SRC_ETC_DIR/conf.d"
SRC_RULES_DIR="$SRC_ETC_DIR/rules"
SRC_SMARTDNS_CONF="$SRC_ETC_DIR/smartdns.conf"
SRC_PLUGIN_SO="$SRC_DIST_DIR/smartdns_plus_ui.so"
SRC_WWWROOT_DIR="$SRC_DIST_DIR/wwwroot"

SMARTDNS_CONF_DIR="${SMARTDNS_CONF_DIR:-/etc/smartdns}"
SMARTDNS_CONF_FILE="$SMARTDNS_CONF_DIR/smartdns.conf"
SMARTDNS_CONFD_DIR="$SMARTDNS_CONF_DIR/conf.d"
SMARTDNS_RULES_DIR="$SMARTDNS_CONF_DIR/rules"
SMARTDNS_PLUGIN_DIR="${SMARTDNS_PLUGIN_DIR:-/usr/lib/smartdns}"
SMARTDNS_PLUGIN_DST="$SMARTDNS_PLUGIN_DIR/smartdns_plus_ui.so"
SMARTDNS_WWWROOT_DST="${SMARTDNS_WWWROOT_DST:-/usr/share/smartdns-plus/wwwroot}"
SMARTDNS_PREINSTALL_BACKUP="$SMARTDNS_CONF_FILE.pre-smartdns-plus-ui.backup"

PLUGIN_BEGIN="# BEGIN smartdns-plus-ui plugin"
PLUGIN_END="# END smartdns-plus-ui plugin"
MANAGED_BEGIN="# BEGIN smartdns-plus-ui managed"
MANAGED_END="# END smartdns-plus-ui managed"

ensure_dir() {
  mkdir -p "$1"
}

copy_file_if_missing() {
  local src="$1"
  local dst="$2"
  if [[ ! -f "$dst" ]]; then
    cp "$src" "$dst"
  fi
}

copy_conf_templates_if_missing() {
  local src_dir="$1"
  local dst_dir="$2"
  local src_file
  for src_file in "$src_dir"/*.conf; do
    [[ -f "$src_file" ]] || continue
    copy_file_if_missing "$src_file" "$dst_dir/$(basename "$src_file")"
  done
}

copy_rule_templates_if_missing() {
  local src_dir="$1"
  local dst_dir="$2"
  local src_file
  for src_file in "$src_dir"/*.txt; do
    [[ -f "$src_file" ]] || continue
    copy_file_if_missing "$src_file" "$dst_dir/$(basename "$src_file")"
  done
}

require_writable_parent() {
  local target="$1"
  local parent
  parent="$(dirname "$target")"
  if [[ ! -d "$parent" ]]; then
    mkdir -p "$parent"
  fi
  if [[ ! -w "$parent" ]]; then
    echo "error: $parent is not writable. Please run as root or with sudo." >&2
    exit 1
  fi
}

require_bundle_files() {
  local missing=0
  for path in "$SRC_PLUGIN_SO" "$SRC_SMARTDNS_CONF"; do
    if [[ ! -e "$path" ]]; then
      echo "error: required bundle file missing: $path" >&2
      missing=1
    fi
  done
  if [[ ! -d "$SRC_WWWROOT_DIR" ]]; then
    echo "error: required bundle directory missing: $SRC_WWWROOT_DIR" >&2
    missing=1
  fi
  if [[ ! -d "$SRC_CONFD_DIR" ]]; then
    echo "error: required bundle directory missing: $SRC_CONFD_DIR" >&2
    missing=1
  fi
  if [[ ! -d "$SRC_RULES_DIR" ]]; then
    echo "error: required bundle directory missing: $SRC_RULES_DIR" >&2
    missing=1
  fi
  if [[ "$missing" -ne 0 ]]; then
    echo "hint: please run this script from the extracted smartdns-plus-ui-runtime package directory." >&2
    exit 1
  fi
}

backup_existing_conf_once() {
  if [[ -f "$SMARTDNS_CONF_FILE" && ! -f "$SMARTDNS_PREINSTALL_BACKUP" ]]; then
    cp "$SMARTDNS_CONF_FILE" "$SMARTDNS_PREINSTALL_BACKUP"
    echo "  - backup existing smartdns.conf -> $SMARTDNS_PREINSTALL_BACKUP"
  fi
}

upsert_block() {
  local file="$1"
  local begin="$2"
  local end="$3"
  local content="$4"

  python3 - "$file" "$begin" "$end" "$content" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
begin = sys.argv[2]
end = sys.argv[3]
content = sys.argv[4]

text = path.read_text(encoding="utf-8") if path.exists() else ""
block = f"{begin}\n{content.rstrip()}\n{end}\n"

if begin in text and end in text:
    start = text.index(begin)
    finish = text.index(end, start) + len(end)
    suffix = text[finish:]
    if suffix.startswith("\n"):
        suffix = suffix[1:]
    text = text[:start].rstrip("\n") + "\n\n" + block
    if suffix:
        text += "\n" + suffix.lstrip("\n")
else:
    text = text.rstrip("\n")
    if text:
        text += "\n\n"
    text += block

path.write_text(text.rstrip("\n") + "\n", encoding="utf-8")
PY
}

prepare_target_dirs() {
  echo "[1/6] validate bundle and prepare target directories"
  require_bundle_files
  require_writable_parent "$SMARTDNS_PLUGIN_DST"
  require_writable_parent "$SMARTDNS_CONF_FILE"
  require_writable_parent "$SMARTDNS_WWWROOT_DST/.touch"

  ensure_dir "$SMARTDNS_PLUGIN_DIR"
  ensure_dir "$SMARTDNS_WWWROOT_DST"
  ensure_dir "$SMARTDNS_CONF_DIR"
  ensure_dir "$SMARTDNS_CONFD_DIR"
  ensure_dir "$SMARTDNS_RULES_DIR"
}

install_runtime_files() {
  echo "[2/6] install plugin and wwwroot"
  cp "$SRC_PLUGIN_SO" "$SMARTDNS_PLUGIN_DST"
  cp -a "$SRC_WWWROOT_DIR/." "$SMARTDNS_WWWROOT_DST/"
}

install_templates() {
  echo "[3/6] install default conf.d and rules templates"
  copy_conf_templates_if_missing "$SRC_CONFD_DIR" "$SMARTDNS_CONFD_DIR"
  copy_rule_templates_if_missing "$SRC_RULES_DIR" "$SMARTDNS_RULES_DIR"
}

prepare_smartdns_conf() {
  echo "[4/6] ensure smartdns.conf and managed include blocks"
  if [[ -f "$SMARTDNS_CONF_FILE" ]]; then
    echo "  - existing smartdns.conf detected, keep user config and only inject managed blocks"
    backup_existing_conf_once
  else
    echo "  - smartdns.conf not found, initialize from bundled etc/smartdns.conf"
    copy_file_if_missing "$SRC_SMARTDNS_CONF" "$SMARTDNS_CONF_FILE"
  fi

  local plugin_block
  plugin_block="$(cat <<'EOF'
plugin smartdns_plus_ui.so
smartdns-plus-ui.www-root /usr/share/smartdns-plus/wwwroot
smartdns-plus-ui.ip http://0.0.0.0:6081
smartdns-plus-ui.config-file /etc/smartdns/smartdns.conf
smartdns-plus-ui.rules-dir /etc/smartdns/rules
smartdns-plus-ui.token-expire 600
smartdns-plus-ui.user admin
smartdns-plus-ui.password password
smartdns-plus-ui.enable-terminal yes
EOF
)"

  local managed_block
  managed_block="$(cat <<'EOF'
conf-file /etc/smartdns/conf.d/10-basic.conf
conf-file /etc/smartdns/conf.d/20-upstreams.conf
conf-file /etc/smartdns/conf.d/30-cache.conf
conf-file /etc/smartdns/conf.d/60-sets.conf
conf-file /etc/smartdns/conf.d/40-nameserver.conf
conf-file /etc/smartdns/conf.d/50-rules.conf
conf-file /etc/smartdns/conf.d/70-logging.conf
conf-file /etc/smartdns/conf.d/80-network.conf
EOF
)"

  upsert_block "$SMARTDNS_CONF_FILE" "$PLUGIN_BEGIN" "$PLUGIN_END" "$plugin_block"
  upsert_block "$SMARTDNS_CONF_FILE" "$MANAGED_BEGIN" "$MANAGED_END" "$managed_block"
}

restart_smartdns_service() {
  echo "[5/6] restart smartdns service"

  if command -v systemctl >/dev/null 2>&1; then
    if systemctl restart smartdns >/dev/null 2>&1; then
      echo "  - restarted via systemctl restart smartdns"
      return 0
    fi
  fi

  if command -v service >/dev/null 2>&1; then
    if service smartdns restart >/dev/null 2>&1; then
      echo "  - restarted via service smartdns restart"
      return 0
    fi
  fi

  echo "  - could not restart smartdns automatically"
  echo "  - please restart SmartDNS manually on your system"
}

final_tips() {
  echo "[6/6] done"
  echo
  echo "Plugin:   $SMARTDNS_PLUGIN_DST"
  echo "WWW root: $SMARTDNS_WWWROOT_DST"
  echo "Config:   $SMARTDNS_CONF_FILE"
  if [[ -f "$SMARTDNS_PREINSTALL_BACKUP" ]]; then
    echo "Backup:   $SMARTDNS_PREINSTALL_BACKUP"
  fi
  echo
  echo "Open: http://127.0.0.1:6081/"
}

prepare_target_dirs
install_runtime_files
install_templates
prepare_smartdns_conf
restart_smartdns_service
final_tips
