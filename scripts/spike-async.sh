#!/usr/bin/env bash
# NEN-009 ölçümü: FFI sınırından geçen bir işin cooperative cancellation'ı.
#
#   bash scripts/spike-async.sh                 # ölçüm + testler, release
#   bash scripts/spike-async.sh --debug
#   bash scripts/spike-async.sh --test-only      # yalnız invariant testleri (swift test)
#   bash scripts/spike-async.sh --measure-only   # yalnız baseline ölçümü (CLI)
#
# Bu script ÜRÜN build'ine dokunmaz. Ölçülen crate core/spikes/ altındadır ve
# kendi FFI kapısını açar — buna izin veren karar ADR-0028'dir.
#
# swift-testing kullanır (XCTest DEĞİL): bu makinede tam Xcode yoksa
# CommandLineTools'ta eksik olan plugin-path / rpath bayrakları aşağıda VARSA
# eklenir — aynı desen scripts/test-apple.sh'te.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORE="$ROOT/core"
PKG="$ROOT/core/spikes/spike-async-cancel/apple-harness"
OUT="$PKG/generated"

CARGO_PROFILE="release"
PROFILE="release"
MODE="both"   # both | test-only | measure-only

while [ $# -gt 0 ]; do
  case "$1" in
    --debug)         CARGO_PROFILE="dev"; PROFILE="debug"; shift ;;
    --release)        CARGO_PROFILE="release"; PROFILE="release"; shift ;;
    --test-only)      MODE="test-only"; shift ;;
    --measure-only)   MODE="measure-only"; shift ;;
    *) echo "Bilinmeyen argüman: $1 (--debug | --release | --test-only | --measure-only)" >&2; exit 1 ;;
  esac
done

command -v cargo >/dev/null 2>&1 || {
  echo "HATA: cargo bulunamadı. Kurulum için: bash scripts/doctor.sh M1" >&2; exit 1; }
command -v swift >/dev/null 2>&1 || {
  echo "HATA: swift bulunamadı. Bkz. bash scripts/doctor.sh" >&2; exit 1; }

echo "▶ 1/4  cargo build -p spike-async-cancel ($PROFILE)"
cargo build --manifest-path "$CORE/Cargo.toml" -p spike-async-cancel --profile "$CARGO_PROFILE"

TARGET_DIR="$CORE/target/$PROFILE"
DYLIB="$TARGET_DIR/libspike_async_cancel.dylib"
STATICLIB="$TARGET_DIR/libspike_async_cancel.a"
for f in "$DYLIB" "$STATICLIB"; do
  [ -f "$f" ] || { echo "HATA: beklenen çıktı yok: $f" >&2; exit 1; }
done

echo "▶ 2/4  spike-async-uniffi-bindgen generate --language swift"
rm -rf "$OUT"
mkdir -p "$OUT/spike_async_cancelFFI" "$OUT/SpikeCore" "$OUT/lib"

RAW="$OUT/.raw"
( cd "$CORE" && cargo run -q -p spike-async-cancel --bin spike-async-uniffi-bindgen --profile "$CARGO_PROFILE" -- \
    generate --language swift --no-format --out-dir "$RAW" "$DYLIB" )

mv "$RAW/spike_async_cancelFFI.h"         "$OUT/spike_async_cancelFFI/spike_async_cancelFFI.h"
mv "$RAW/spike_async_cancelFFI.modulemap" "$OUT/spike_async_cancelFFI/module.modulemap"
mv "$RAW/spike_async_cancel.swift"        "$OUT/SpikeCore/spike_async_cancel.swift"
cp "$STATICLIB"                           "$OUT/lib/libspike_async_cancel.a"
rmdir "$RAW"

cat > "$OUT/README.md" <<'GEN'
# ÜRETİLEN DİZİN — elle düzenlemeyin, commit etmeyin

`bash scripts/spike-async.sh` üretir. `.gitignore` → `/core/spikes/**/generated/`.
GEN

SWIFT_CONFIG="release"
[ "$PROFILE" = "debug" ] && SWIFT_CONFIG="debug"

DEVDIR="$(xcode-select -p 2>/dev/null || echo /Library/Developer/CommandLineTools)"
FLAGS=""
add_plugin_path() { [ -d "$1" ] && FLAGS="$FLAGS -Xswiftc -plugin-path -Xswiftc $1"; return 0; }
add_rpath()       { [ -d "$1" ] && FLAGS="$FLAGS -Xlinker -rpath -Xlinker $1";      return 0; }
add_plugin_path "$DEVDIR/usr/lib/swift/host/plugins/testing"
add_rpath "$DEVDIR/Library/Developer/Frameworks"
add_rpath "$DEVDIR/Library/Developer/usr/lib"

if [ "$MODE" != "measure-only" ]; then
  echo "▶ 3/4  swift test ($PROFILE)${FLAGS:+  (ek bayraklar: tam Xcode yok)}"
  # shellcheck disable=SC2086  # FLAGS bilinçli olarak kelimelere ayrılıyor
  swift test --package-path "$PKG" -c "$SWIFT_CONFIG" $FLAGS
else
  echo "▶ 3/4  swift test atlandı (--measure-only)"
fi

if [ "$MODE" = "test-only" ]; then
  exit 0
fi

echo "▶ 4/4  swift build + ölçüm ($PROFILE)"
swift build --package-path "$PKG" -c "$SWIFT_CONFIG" >/dev/null
BIN="$(swift build --package-path "$PKG" -c "$SWIFT_CONFIG" --show-bin-path)/SpikeAsyncCancel"

echo
echo "## Ölçüm bağlamı (build)"
echo
echo "- build: **$PROFILE** (rust profile: $CARGO_PROFILE · swift: -c $SWIFT_CONFIG)"
echo "- cihaz: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo '?') · $(uname -m)"
echo "- OS: $(sw_vers -productName) $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
echo "- rustc: $(rustc --version)"
echo "- swift: $(swift --version 2>/dev/null | head -1)"
echo "- uniffi: $(grep -m1 '^uniffi = ' "$CORE/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
echo "- tarih: $(date '+%Y-%m-%d %H:%M %Z')"
echo

"$BIN"
