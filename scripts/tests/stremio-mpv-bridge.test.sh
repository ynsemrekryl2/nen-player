#!/usr/bin/env bash
# NEN-087 deterministic bridge-manager tests. No real /usr/local path or
# application is touched; the manager's test root is a temporary directory.

set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BRIDGE="$ROOT/scripts/stremio-mpv-bridge.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

FAILURES=0
pass() { printf '    ok   %s\n' "$1"; }
fail() { printf '    FAIL %s\n' "$1" >&2; FAILURES=$((FAILURES + 1)); }

TEST_ROOT="$TMP/root"
APP="$TMP/Nen Player.app"
APP_EXEC="$APP/Contents/MacOS/NenPlayer"
BIN="$TEST_ROOT/usr/bin"
TARGET="$TEST_ROOT/usr/local/bin/mpv"
STATE="$TEST_ROOT/usr/local/libexec/nen-player/stremio-mpv-bridge"
BACKUP="$STATE/previous-mpv"
CAPTURE="$TMP/capture"
OPEN_CAPTURE="$TMP/open-capture"
RUNNING_STATE="$TMP/running-state"

mkdir -p "$APP/Contents/MacOS" "$BIN" "$(dirname "$TARGET")"

cat > "$APP_EXEC" <<'EOF'
#!/bin/sh
printf '%s\n' 'APP_EXEC_CALLED' >> "$CAPTURE"
printf '<%s>\n' "$@" >> "$CAPTURE"
EOF
chmod 755 "$APP_EXEC"

REAL_MPV="$TMP/real-mpv"
cat > "$REAL_MPV" <<'EOF'
#!/bin/sh
printf '%s\n' 'REAL_MPV_CALLED' >> "$CAPTURE"
printf '<%s>\n' "$@" >> "$CAPTURE"
EOF
chmod 755 "$REAL_MPV"
REAL_MPV_LINK="$TMP/real-mpv-link"
ln -s "$REAL_MPV" "$REAL_MPV_LINK"

cat > "$BIN/open" <<'EOF'
#!/bin/sh
printf '%s\n' "$@" > "$OPEN_CAPTURE"
EOF
chmod 755 "$BIN/open"

cat > "$BIN/pgrep" <<'EOF'
#!/bin/sh
if [ -f "$RUNNING_STATE" ]; then exit 0; else exit 1; fi
EOF
chmod 755 "$BIN/pgrep"

export CAPTURE OPEN_CAPTURE RUNNING_STATE
export NEN_BRIDGE_TEST_ROOT="$TEST_ROOT"
export NEN_BRIDGE_TEST_OPEN="$BIN/open"
export NEN_BRIDGE_TEST_RUNNING="$BIN/pgrep"

run_bridge() { bash "$BRIDGE" "$@"; }
expect_rc() {
  expected="$1"; shift
  output=''; rc=0
  output="$(run_bridge "$@" 2>&1)" || rc=$?
  if [ "$rc" -eq "$expected" ]; then pass "bridge command → exit $rc"; else fail "bridge command → exit $rc (beklenen $expected): $output"; fi
}
expect_output() {
  expected="$1"; shift
  output="$(run_bridge "$@" 2>&1)" || true
  if [ "$output" = "$expected" ]; then pass "bridge command → $expected"; else fail "bridge command → beklenen '$expected', gelen '$output'"; fi
}

echo '  S1: temiz kurulum ve idempotency'
expect_output installed install --app "$APP" --fallback-mpv "$REAL_MPV_LINK"
expect_output installed status
expect_output already-installed install --app "$APP" --fallback-mpv "$REAL_MPV_LINK"

echo '  S2: cold Stremio argv and unknown-call delegation'
: > "$CAPTURE"
"$TARGET" '--start=12.5' '--no-terminal' 'fixture://opaque' >/dev/null 2>&1
if grep -qx 'APP_EXEC_CALLED' "$CAPTURE" && grep -qx '<--start=12.5>' "$CAPTURE" \
  && grep -qx '<--no-terminal>' "$CAPTURE" && grep -qx '<fixture://opaque>' "$CAPTURE"; then
  pass 'Stremio argv cold handoff''ı değişmeden app executable''ına ulaştı'
else
  fail 'Stremio argv cold handoff''ı app executable''ına değişmeden ulaşmadı'
fi
: > "$CAPTURE"
"$TARGET" '--volume=0.4' 'ordinary-call' >/dev/null 2>&1
if grep -qx 'REAL_MPV_CALLED' "$CAPTURE" && grep -qx '<--volume=0.4>' "$CAPTURE" \
  && grep -qx '<ordinary-call>' "$CAPTURE"; then
  pass 'tanınmayan çağrı gerçek MPV''ye argv ile devredildi'
else
  fail 'tanınmayan çağrı gerçek MPV''ye devredilmedi'
fi

echo '  S3: warm handoff and negative output leak'
: > "$RUNNING_STATE"
: > "$OPEN_CAPTURE"
"$TARGET" '--start=0' '--no-terminal' 'https://example.test/opaque?token=sentinel' >"$TMP/stdout" 2>"$TMP/stderr"
if grep -qx -- '-a' "$OPEN_CAPTURE" \
  && grep -Fqx -- "$APP" "$OPEN_CAPTURE" \
  && grep -Fqx -- 'nenplayer://https://example.test/opaque?token=sentinel#t=0' "$OPEN_CAPTURE"; then
  pass 'sıcak handoff application open yoluna ulaştı'
else
  fail 'sıcak handoff application open yoluna ulaşmadı'
fi
if [ ! -s "$TMP/stdout" ] && [ ! -s "$TMP/stderr" ] \
  && ! grep -R -qF 'token=sentinel' "$TMP/stdout" "$TMP/stderr"; then
  pass 'wrapper stdout/stderr locator ve argv sızdırmadı'
else
  fail 'wrapper stdout/stderr sızıntı içeriyor'
fi

echo '  S4: yabancı hedef ret, replace ve birebir restore'
rm -f "$TARGET" "$STATE/manifest" "$BACKUP"
mkdir -p "$(dirname "$TARGET")"
cat > "$TARGET" <<'EOF'
#!/bin/sh
foreign-wrapper
EOF
chmod 751 "$TARGET"
original_hash="$(shasum -a 256 "$TARGET" | awk '{print $1}')"
original_mode="$(stat -f '%Mp%Lp' "$TARGET" 2>/dev/null || stat -c '%a' "$TARGET")"
expect_rc 1 install --app "$APP" --fallback-mpv "$REAL_MPV"
if [ ! -e "$BACKUP" ]; then pass 'yabancı hedef varsayılan olarak korunup reddedildi'; else fail 'ret sırasında yabancı hedef taşındı'; fi
expect_output installed install --app "$APP" --replace
if [ -f "$BACKUP" ] && [ "$(shasum -a 256 "$BACKUP" | awk '{print $1}')" = "$original_hash" ]; then
  pass 'replace atomik yedek içeriğini korudu'
else
  fail 'replace yedek içeriğini korumadı'
fi
expect_output uninstalled uninstall
restored_hash="$(shasum -a 256 "$TARGET" | awk '{print $1}')"
restored_mode="$(stat -f '%Mp%Lp' "$TARGET" 2>/dev/null || stat -c '%a' "$TARGET")"
if [ "$restored_hash" = "$original_hash" ] && [ "$restored_mode" = "$original_mode" ]; then
  pass 'uninstall önceki wrapper''ı içerik ve izinleriyle geri yükledi'
else
  fail 'uninstall önceki wrapper''ı birebir geri yüklemedi'
fi

echo '  S5: bütünlük kapıları'
rm -f "$TARGET"
expect_output installed install --app "$APP" --fallback-mpv "$REAL_MPV"
printf '%s\n' '# modified' >> "$TARGET"
expect_output modified status
expect_rc 1 uninstall
if [ -f "$TARGET" ]; then pass 'değiştirilmiş wrapper sessizce silinmedi'; else fail 'değiştirilmiş wrapper silindi'; fi
expect_output installed install --app "$APP" --replace
expect_output installed status

echo '  S6: eski Nen wrapper yükseltmesi'
legacy_target="$TMP/legacy-wrapper"
sed 's/nen-player-stremio-mpv-bridge:v2/nen-player-stremio-mpv-bridge:v1/' "$TARGET" > "$legacy_target"
mv "$legacy_target" "$TARGET"
legacy_hash="$(shasum -a 256 "$TARGET" | awk '{print $1}')"
{
  printf '%s\n' 'version=1'
  printf 'sha256=%s\n' "$legacy_hash"
} > "$STATE/manifest"
expect_output installed install --app "$APP" --replace
expect_output installed status
if sed -n '1,2p' "$TARGET" | grep -qxF '# nen-player-stremio-mpv-bridge:v2'; then
  pass 'eski yönetilen wrapper yedeği koruyarak güncellendi'
else
  fail 'eski yönetilen wrapper güncellenmedi'
fi

if [ "$FAILURES" -gt 0 ]; then
  printf 'SONUÇ: %s doğrulama başarısız.\n' "$FAILURES" >&2
  exit 1
fi
printf '%s\n' 'SONUÇ: tüm doğrulamalar geçti.'
