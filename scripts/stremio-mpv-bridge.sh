#!/usr/bin/env bash
# NEN-087: install and remove the reversible Stremio MPV launcher bridge.
#
# The installed wrapper is deliberately small and POSIX-only. It never
# evaluates caller-provided text: every handoff and fallback call uses a
# quoted argv array. The test-root hook is used only by the deterministic
# shell suite; production installs always target /usr/local.

set -euo pipefail

readonly MARKER='# nen-player-stremio-mpv-bridge:v2'
readonly LEGACY_MARKER='# nen-player-stremio-mpv-bridge:v1'
readonly DEFAULT_TARGET='/usr/local/bin/mpv'
readonly DEFAULT_STATE='/usr/local/libexec/nen-player/stremio-mpv-bridge'

TEST_ROOT="${NEN_BRIDGE_TEST_ROOT:-}"
if [ -n "$TEST_ROOT" ]; then
  TARGET="$TEST_ROOT$DEFAULT_TARGET"
  STATE_DIR="$TEST_ROOT$DEFAULT_STATE"
  OPEN_COMMAND="${NEN_BRIDGE_TEST_OPEN:-$TEST_ROOT/usr/bin/open}"
  RUNNING_COMMAND="${NEN_BRIDGE_TEST_RUNNING:-$TEST_ROOT/usr/bin/pgrep}"
else
  TARGET="$DEFAULT_TARGET"
  STATE_DIR="$DEFAULT_STATE"
  OPEN_COMMAND='/usr/bin/open'
  RUNNING_COMMAND='/usr/bin/pgrep'
fi

BACKUP="$STATE_DIR/previous-mpv"
MANIFEST="$STATE_DIR/manifest"
TARGET_DIR="${TARGET%/*}"

die() {
  printf '%s\n' "$1" >&2
  exit 1
}

usage() {
  cat >&2 <<'EOF'
Kullanım:
  sudo bash scripts/stremio-mpv-bridge.sh install --app <mutlak-app-yolu> [--replace] [--fallback-mpv <mutlak-yol>]
  sudo bash scripts/stremio-mpv-bridge.sh uninstall
  bash scripts/stremio-mpv-bridge.sh status
EOF
  exit 2
}

require_root() {
  [ -n "$TEST_ROOT" ] || [ "$(id -u)" -eq 0 ] || die 'root-required'
}

sha256() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    die 'sha256-unavailable'
  fi
}

same_device() {
  local left right
  if stat -f '%d' "$1" >/dev/null 2>&1; then
    left="$(stat -f '%d' "$1")"
    right="$(stat -f '%d' "$2")"
  else
    left="$(stat -c '%d' "$1")"
    right="$(stat -c '%d' "$2")"
  fi
  [ "$left" = "$right" ]
}

is_regular_file() {
  [ -f "$1" ] && [ ! -L "$1" ]
}

is_managed_marker() {
  is_regular_file "$TARGET" || return 1
  case "$(sed -n '2p' "$TARGET")" in
    "$MARKER"|"$LEGACY_MARKER") return 0 ;;
    *) return 1 ;;
  esac
}

is_current_wrapper() {
  is_regular_file "$TARGET" || return 1
  sed -n '2p' "$TARGET" | grep -qxF "$MARKER"
}

manifest_hash() {
  [ -f "$MANIFEST" ] || return 1
  awk -F= '$1 == "sha256" { print $2; exit }' "$MANIFEST"
}

managed_and_intact() {
  is_managed_marker || return 1
  expected="$(manifest_hash 2>/dev/null || true)"
  [ -n "$expected" ] || return 1
  actual="$(sha256 "$TARGET")"
  [ "$actual" = "$expected" ]
}

shell_quote() {
  local value="$1"
  if printf '%s' "$value" | LC_ALL=C grep -q '[[:cntrl:]]'; then
    die 'invalid-path'
  fi
  # The generated wrapper contains literals only; no eval or text execution.
  printf "'%s'" "$(printf '%s' "$value" | sed "s/'/'\\''/g")"
}

validate_app() {
  local app="$1"
  case "$app" in /*.app) ;; *) die 'invalid-app' ;; esac
  is_regular_file "$app/Contents/MacOS/NenPlayer" || die 'app-unavailable'
  [ -x "$app/Contents/MacOS/NenPlayer" ] || die 'app-unavailable'
}

validate_delegate() {
  local delegate="$1"
  [ -z "$delegate" ] && return 0
  # Homebrew and MacPorts expose mpv through a symlink in their bin
  # directory; delegation may follow that link, unlike the managed target.
  [ -f "$delegate" ] || die 'delegate-unavailable'
  [ -x "$delegate" ] || die 'delegate-unavailable'
}

discover_delegate() {
  local candidate
  for candidate in /opt/homebrew/bin/mpv /opt/local/bin/mpv /sw/bin/mpv; do
    if [ -f "$candidate" ] && [ -x "$candidate" ] && [ "$candidate" != "$TARGET" ]; then
      printf '%s' "$candidate"
      return 0
    fi
  done
  return 0
}

write_wrapper() {
  local temp="$1" app_bundle="$2" app_exec="$3" delegate="$4"
  local q_bundle q_exec q_delegate q_open q_running
  q_bundle="$(shell_quote "$app_bundle")"
  q_exec="$(shell_quote "$app_exec")"
  q_delegate="$(shell_quote "$delegate")"
  q_open="$(shell_quote "$OPEN_COMMAND")"
  q_running="$(shell_quote "$RUNNING_COMMAND")"

  {
    printf '%s\n' '#!/bin/sh'
    printf '%s\n' "$MARKER"
    printf '%s\n' 'set -eu'
    printf 'APP_BUNDLE=%s\n' "$q_bundle"
    printf 'APP_EXEC=%s\n' "$q_exec"
    printf 'DELEGATE=%s\n' "$q_delegate"
    printf 'OPEN_COMMAND=%s\n' "$q_open"
    printf 'RUNNING_COMMAND=%s\n' "$q_running"
    cat <<'EOF'

is_stremio_argv() {
  [ "$#" -eq 3 ] || return 1
  case "$1" in
    --start=*) ;;
    *) return 1 ;;
  esac
  [ "$2" = '--no-terminal' ] || return 1
  case "$3" in
    -*) return 1 ;;
  esac
}

is_running() {
  "$RUNNING_COMMAND" -x NenPlayer >/dev/null 2>&1
}

if is_stremio_argv "$@"; then
  start_seconds=${1#--start=}
  locator=$3
  if is_running; then
    # The open callback reaches the already-running process. The raw
    # locator is carried as data in one quoted argument; it is never eval'd.
    handoff="nenplayer://$locator#t=$start_seconds"
    exec "$OPEN_COMMAND" -a "$APP_BUNDLE" "$handoff" >/dev/null 2>&1
  fi
  # Cold launch: preserve Stremio's measured argv byte-for-byte.
  exec "$APP_EXEC" "$@"
fi

if [ -n "$DELEGATE" ] && [ -x "$DELEGATE" ]; then
  exec "$DELEGATE" "$@"
fi

printf '%s\n' 'delegate-unavailable' >&2
exit 127
EOF
  } > "$temp"
  chmod 755 "$temp"
}

write_manifest() {
  local temp="$1" digest="$2"
  {
    printf '%s\n' 'version=1'
    printf 'sha256=%s\n' "$digest"
  } > "$temp"
  chmod 644 "$temp"
}

status() {
  if ! [ -e "$TARGET" ] && ! [ -L "$TARGET" ]; then
    printf '%s\n' 'absent'
    return 1
  fi
  if managed_and_intact; then
    printf '%s\n' 'installed'
    return 0
  fi
  if is_managed_marker; then
    printf '%s\n' 'modified'
  else
    printf '%s\n' 'foreign'
  fi
  return 2
}

install_bridge() {
  local app='' fallback='' replace=0 option
  while [ "$#" -gt 0 ]; do
    option="$1"
    shift
    case "$option" in
      --app)
        [ "$#" -gt 0 ] || usage
        app="$1"
        shift
        ;;
      --replace)
        replace=1
        ;;
      --fallback-mpv)
        [ "$#" -gt 0 ] || usage
        fallback="$1"
        shift
        ;;
      *) usage ;;
    esac
  done
  [ -n "$app" ] || usage
  require_root
  validate_app "$app"

  if [ -L "$STATE_DIR" ]; then die 'state-conflict'; fi
  mkdir -p "$STATE_DIR" "$TARGET_DIR"
  if ! same_device "$STATE_DIR" "$TARGET_DIR"; then die 'different-device'; fi

  local managed_upgrade=0
  if [ -e "$TARGET" ] || [ -L "$TARGET" ]; then
    if managed_and_intact; then
      if is_current_wrapper; then
        printf '%s\n' 'already-installed'
        return 0
      fi
      [ "$replace" -eq 1 ] || die 'modified-target'
      managed_upgrade=1
    fi
    if is_managed_marker; then
      [ "$replace" -eq 1 ] || die 'modified-target'
      managed_upgrade=1
    else
      [ "$replace" -eq 1 ] || die 'foreign-target'
      [ ! -e "$BACKUP" ] || die 'backup-conflict'
      [ ! -L "$BACKUP" ] || die 'backup-conflict'
      [ ! -L "$TARGET" ] || die 'symlink-target'
    fi
  else
    [ ! -e "$BACKUP" ] || die 'backup-conflict'
    [ ! -L "$BACKUP" ] || die 'backup-conflict'
    [ ! -e "$MANIFEST" ] || die 'state-conflict'
  fi

  local had_target=0 delegate="$fallback"
  local moved_backup=0 moved_old=0 created_target=0 committed=0
  local old_target='' temp manifest_temp digest
  if [ -n "$delegate" ]; then
    case "$delegate" in /*) ;; *) die 'invalid-delegate' ;; esac
  fi

  if [ "$managed_upgrade" -eq 1 ]; then
    had_target=0
    if [ -z "$delegate" ] && [ -e "$BACKUP" ]; then delegate="$BACKUP"; fi
    if [ -z "$delegate" ]; then delegate="$(discover_delegate)"; fi
  elif [ -e "$TARGET" ] || [ -L "$TARGET" ]; then
    had_target=1
    [ -n "$delegate" ] || delegate="$BACKUP"
  elif [ -z "$delegate" ]; then
    delegate="$(discover_delegate)"
  fi
  [ "$delegate" = "$BACKUP" ] || validate_delegate "$delegate"

  temp="$(mktemp "$TARGET_DIR/.mpv.nen-player.XXXXXX")"
  manifest_temp="$(mktemp "$STATE_DIR/.manifest.XXXXXX")"
  rollback() {
    if [ "$committed" -eq 0 ]; then
      [ "$created_target" -eq 0 ] || rm -f "$TARGET"
      if [ "$moved_old" -eq 1 ] && [ -e "$old_target" ] && [ ! -e "$TARGET" ]; then
        mv "$old_target" "$TARGET" || true
      fi
      if [ "$moved_backup" -eq 1 ] && [ -e "$BACKUP" ] && [ ! -e "$TARGET" ]; then
        mv "$BACKUP" "$TARGET" || true
      fi
    fi
    rm -f "$temp" "$manifest_temp"
  }
  trap rollback EXIT HUP INT TERM

  if [ "$had_target" -eq 1 ]; then
    mv "$TARGET" "$BACKUP"
    moved_backup=1
    validate_delegate "$delegate"
  fi
  if [ "$managed_upgrade" -eq 1 ]; then
    old_target="$TARGET_DIR/.mpv.nen-player.old.$$"
    [ ! -e "$old_target" ] || die 'state-conflict'
    mv "$TARGET" "$old_target"
    moved_old=1
    validate_delegate "$delegate"
  fi
  write_wrapper "$temp" "$app" "$app/Contents/MacOS/NenPlayer" "$delegate"
  mv "$temp" "$TARGET"
  created_target=1
  digest="$(sha256 "$TARGET")"
  write_manifest "$manifest_temp" "$digest"
  mv "$manifest_temp" "$MANIFEST"
  committed=1
  [ "$moved_old" -eq 0 ] || rm -f "$old_target"
  trap - EXIT HUP INT TERM
  printf '%s\n' 'installed'
}

uninstall_bridge() {
  require_root
  managed_and_intact || die 'modified'
  [ -L "$STATE_DIR" ] && die 'state-conflict'
  if [ -e "$BACKUP" ] || [ -L "$BACKUP" ]; then
    is_regular_file "$BACKUP" || die 'backup-invalid'
    [ -d "$STATE_DIR" ] || die 'state-conflict'
    local removed="$STATE_DIR/.removed-wrapper.$$"
    mv "$TARGET" "$removed"
    if mv "$BACKUP" "$TARGET"; then
      rm -f "$removed" "$MANIFEST"
    else
      mv "$removed" "$TARGET" || true
      die 'restore-failed'
    fi
  else
    rm -f "$TARGET" "$MANIFEST"
  fi
  rmdir "$STATE_DIR" 2>/dev/null || true
  printf '%s\n' 'uninstalled'
}

[ "$#" -gt 0 ] || usage
command="$1"
shift
case "$command" in
  install) install_bridge "$@" ;;
  uninstall) [ "$#" -eq 0 ] || usage; uninstall_bridge ;;
  status) [ "$#" -eq 0 ] || usage; status ;;
  *) usage ;;
esac
