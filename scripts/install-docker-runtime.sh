#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_DIR="${1:-/data/smartdns}"

SMARTDNS_CONF_DIR="$RUNTIME_DIR/etc"
SMARTDNS_CONF_FILE="$SMARTDNS_CONF_DIR/smartdns.conf"
SMARTDNS_CONFD_DIR="$SMARTDNS_CONF_DIR/conf.d"
SMARTDNS_RULES_DIR="$SMARTDNS_CONF_DIR/rules"
SMARTDNS_DB_DIR="$RUNTIME_DIR/db"
SMARTDNS_LOG_DIR="$RUNTIME_DIR/log"
SMARTDNS_DIST_DIR="$RUNTIME_DIR/dist"
SMARTDNS_PLUGIN_DST="$SMARTDNS_DIST_DIR/smartdns_plus_ui.so"
SMARTDNS_WWWROOT_DST="$SMARTDNS_DIST_DIR/wwwroot"
SMARTDNS_COMPOSE_FILE="$RUNTIME_DIR/compose.yaml"
SMARTDNS_COMPOSE_TEMPLATE="$PROJECT_DIR/docker/compose.yaml"
SMARTDNS_CONF_TEMPLATE="$PROJECT_DIR/docker/etc/smartdns.conf"
SMARTDNS_CONFD_TEMPLATE_DIR="$PROJECT_DIR/docker/etc/conf.d"
SMARTDNS_RULES_TEMPLATE_DIR="$PROJECT_DIR/docker/etc/rules"

PLUGIN_BEGIN="# BEGIN smartdns-plus-ui plugin"
PLUGIN_END="# END smartdns-plus-ui plugin"
MANAGED_BEGIN="# BEGIN smartdns-plus-ui managed"
MANAGED_END="# END smartdns-plus-ui managed"
SMARTDNS_PREINSTALL_BACKUP="$SMARTDNS_CONF_FILE.pre-smartdns-plus-ui.backup"

ensure_dir() {
  mkdir -p "$1"
}

ensure_file() {
  local file="$1"
  local content="$2"
  if [[ ! -f "$file" ]]; then
    printf "%s" "$content" > "$file"
  fi
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

sync_wwwroot() {
  ensure_dir "$SMARTDNS_WWWROOT_DST"
  cp -a "$PROJECT_DIR/wwwroot/." "$SMARTDNS_WWWROOT_DST/"
}

build_plugin() {
  echo "[1/6] build smartdns-plus-ui"
  cd "$PROJECT_DIR"
  make
}

copy_plugin() {
  echo "[2/6] copy plugin and wwwroot"
  cp "$PROJECT_DIR/target/smartdns_plus_ui.so" "$SMARTDNS_PLUGIN_DST"
  sync_wwwroot
}

prepare_runtime_tree() {
  echo "[3/6] prepare runtime tree in $RUNTIME_DIR"
  ensure_dir "$SMARTDNS_CONF_DIR"
  ensure_dir "$SMARTDNS_CONFD_DIR"
  ensure_dir "$SMARTDNS_RULES_DIR"
  ensure_dir "$SMARTDNS_DB_DIR"
  ensure_dir "$SMARTDNS_LOG_DIR"
  ensure_dir "$SMARTDNS_DIST_DIR"

  copy_conf_templates_if_missing "$SMARTDNS_CONFD_TEMPLATE_DIR" "$SMARTDNS_CONFD_DIR"
  copy_rule_templates_if_missing "$SMARTDNS_RULES_TEMPLATE_DIR" "$SMARTDNS_RULES_DIR"
}

prepare_smartdns_conf() {
  echo "[4/6] ensure smartdns.conf and managed include blocks"
  if [[ -f "$SMARTDNS_CONF_FILE" ]]; then
    echo "  - existing smartdns.conf detected, keep user config and only inject managed blocks"
    backup_existing_conf_once
  else
    echo "  - smartdns.conf not found, initialize from docker/etc/smartdns.conf"
    copy_file_if_missing "$SMARTDNS_CONF_TEMPLATE" "$SMARTDNS_CONF_FILE"
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

prepare_compose() {
  echo "[5/6] ensure compose.yaml"
  local existing_compose=""
  if existing_compose="$(detect_existing_compose_file)"; then
    echo "  - existing compose file detected: $existing_compose"
    if [[ "$existing_compose" != "$SMARTDNS_COMPOSE_FILE" ]]; then
      echo "  - keep existing file name, skip generating $SMARTDNS_COMPOSE_FILE"
    else
      echo "  - existing compose.yaml detected, skip overwrite"
    fi
    return
  fi

  cp "$SMARTDNS_COMPOSE_TEMPLATE" "$SMARTDNS_COMPOSE_FILE"
}

final_tips() {
  echo "[6/6] done"
  echo
  echo "Runtime dir: $RUNTIME_DIR"
  echo "Dist dir:     $SMARTDNS_DIST_DIR"
  echo "Plugin:      $SMARTDNS_PLUGIN_DST"
  echo "WWW root:    $SMARTDNS_WWWROOT_DST"
  echo "Config:      $SMARTDNS_CONF_FILE"
  if [[ -f "$SMARTDNS_PREINSTALL_BACKUP" ]]; then
    echo "Backup:      $SMARTDNS_PREINSTALL_BACKUP"
  fi
  local existing_compose=""
  if existing_compose="$(detect_existing_compose_file)"; then
    echo "Compose:     $existing_compose"
  else
    echo "Compose:     $SMARTDNS_COMPOSE_FILE"
  fi
  echo
  echo "Next steps:"
  echo "  1. Review $SMARTDNS_CONF_FILE"
  echo "  2. If your old smartdns.conf already contains directives that now also exist in conf.d/, manually remove duplicates later"
  echo "  3. Add your upstream DNS in $SMARTDNS_CONFD_DIR/20-upstreams.conf or through WebUI"
  echo "  4. Start service:"
  echo "     cd $RUNTIME_DIR && docker compose up -d"
  echo "  5. Open: http://127.0.0.1:6081/"
}

build_plugin
copy_plugin
prepare_runtime_tree
prepare_smartdns_conf
prepare_compose
final_tips
