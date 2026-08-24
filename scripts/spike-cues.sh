#!/usr/bin/env bash
# NEN-008 ölçümü: 50k cue'luk bir dokümanın FFI'dan geçiş maliyeti.
#
#   bash scripts/spike-cues.sh                 # release (varsayılan)
#   bash scripts/spike-cues.sh --debug
#   bash scripts/spike-cues.sh --count 50000
#
# Bu script ÜRÜN build'ine dokunmaz. Ölçülen crate core/spikes/ altındadır ve
# kendi FFI kapısını açar — buna izin veren karar ADR-0028'dir.
#
# İki yaklaşım AYRI PROCESS'lerde koşar: peak RSS ölçümü tek process'te
# birbirini kirletirdi.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORE="$ROOT/core"
PKG="$ROOT/core/spikes/spike-cue-transfer/apple-harness"
OUT="$PKG/generated"

CARGO_PROFILE="release"
PROFILE="release"
COUNT=50000

while [ $# -gt 0 ]; do
  case "$1" in
    --debug)   CARGO_PROFILE="dev"; PROFILE="debug"; shift ;;
    --release) CARGO_PROFILE="release"; PROFILE="release"; shift ;;
    --count)   COUNT="${2:?--count bir sayı ister}"; shift 2 ;;
    *) echo "Bilinmeyen argüman: $1 (--debug | --release | --count N)" >&2; exit 1 ;;
  esac
done

command -v cargo >/dev/null 2>&1 || {
  echo "HATA: cargo bulunamadı. Kurulum için: bash scripts/doctor.sh M1" >&2; exit 1; }
command -v swift >/dev/null 2>&1 || {
  echo "HATA: swift bulunamadı. Bkz. bash scripts/doctor.sh" >&2; exit 1; }

echo "▶ 1/4  cargo build -p spike-cue-transfer ($PROFILE)"
cargo build --manifest-path "$CORE/Cargo.toml" -p spike-cue-transfer --profile "$CARGO_PROFILE"

TARGET_DIR="$CORE/target/$PROFILE"
DYLIB="$TARGET_DIR/libspike_cue_transfer.dylib"
STATICLIB="$TARGET_DIR/libspike_cue_transfer.a"
for f in "$DYLIB" "$STATICLIB"; do
  [ -f "$f" ] || { echo "HATA: beklenen çıktı yok: $f" >&2; exit 1; }
done

echo "▶ 2/4  spike-uniffi-bindgen generate --language swift"
rm -rf "$OUT"
mkdir -p "$OUT/spike_cue_transferFFI" "$OUT/SpikeCore" "$OUT/lib"

RAW="$OUT/.raw"
( cd "$CORE" && cargo run -q -p spike-cue-transfer --bin spike-uniffi-bindgen --profile "$CARGO_PROFILE" -- \
    generate --language swift --no-format --out-dir "$RAW" "$DYLIB" )

mv "$RAW/spike_cue_transferFFI.h"         "$OUT/spike_cue_transferFFI/spike_cue_transferFFI.h"
mv "$RAW/spike_cue_transferFFI.modulemap" "$OUT/spike_cue_transferFFI/module.modulemap"
mv "$RAW/spike_cue_transfer.swift"        "$OUT/SpikeCore/spike_cue_transfer.swift"
cp "$STATICLIB"                           "$OUT/lib/libspike_cue_transfer.a"
rmdir "$RAW"

cat > "$OUT/README.md" <<'GEN'
# ÜRETİLEN DİZİN — elle düzenlemeyin, commit etmeyin

`bash scripts/spike-cues.sh` üretir. `.gitignore` → `/core/spikes/**/generated/`.
GEN

echo "▶ 3/4  swift build ($PROFILE)"
SWIFT_CONFIG="release"
[ "$PROFILE" = "debug" ] && SWIFT_CONFIG="debug"
swift build --package-path "$PKG" -c "$SWIFT_CONFIG" >/dev/null
BIN="$(swift build --package-path "$PKG" -c "$SWIFT_CONFIG" --show-bin-path)/SpikeCueTransfer"

echo "▶ 4/4  ölçüm — her yaklaşım ayrı process"
echo

# --- Ölçüm bağlamı: bağlamsız sayı kanıt sayılmaz (M1 kuralı) ---
echo "## Bağlam"
echo
echo "- fixture: $COUNT cue"
echo "- build: **$PROFILE** (rust profile: $CARGO_PROFILE · swift: -c $SWIFT_CONFIG)"
echo "- cihaz: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo '?') · $(uname -m)"
echo "- OS: $(sw_vers -productName) $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
echo "- rustc: $(rustc --version)"
echo "- swift: $(swift --version 2>/dev/null | head -1)"
echo "- uniffi: $(grep -m1 '^uniffi = ' "$CORE/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
echo "- tarih: $(date '+%Y-%m-%d %H:%M %Z')"
echo

"$BIN" full "$COUNT"
echo
"$BIN" windowed "$COUNT"
