#!/usr/bin/env bash
# NEN-011 ölçümü: NEN-008'in (50k cue FFI geçişi) Kotlin/JVM paritesi.
#
#   bash scripts/spike-cues-jvm.sh                 # release (varsayılan)
#   bash scripts/spike-cues-jvm.sh --debug
#   bash scripts/spike-cues-jvm.sh --count 50000
#   bash scripts/spike-cues-jvm.sh --test-only
#
# Bu script ÜRÜN build'ine dokunmaz. Ölçülen crate core/spikes/ altındadır ve
# kendi FFI kapısını açar — buna izin veren karar ADR-0028'dir; NEN-011 aynı
# spike crate'inden Kotlin binding üretir (ADR-0028 "Sonuçlar").
#
# scripts/spike-cues.sh (Swift) ile aynı 4 adım, farkla: adım 2 --language
# kotlin üretir, adım 3 swift build yerine ./gradlew.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORE="$ROOT/core"
PKG="$ROOT/core/spikes/spike-cue-transfer/jvm-harness"
OUT="$PKG/generated"

CARGO_PROFILE="release"
PROFILE="release"
COUNT=50000
MODE="both"   # both | test-only | measure-only

while [ $# -gt 0 ]; do
  case "$1" in
    --debug)         CARGO_PROFILE="dev"; PROFILE="debug"; shift ;;
    --release)        CARGO_PROFILE="release"; PROFILE="release"; shift ;;
    --count)          COUNT="${2:?--count bir sayı ister}"; shift 2 ;;
    --test-only)      MODE="test-only"; shift ;;
    --measure-only)   MODE="measure-only"; shift ;;
    *) echo "Bilinmeyen argüman: $1 (--debug | --release | --count N | --test-only | --measure-only)" >&2; exit 1 ;;
  esac
done

command -v cargo >/dev/null 2>&1 || {
  echo "HATA: cargo bulunamadı. Kurulum için: bash scripts/doctor.sh M1" >&2; exit 1; }
command -v java >/dev/null 2>&1 || {
  echo "HATA: JDK bulunamadı. Kurulum: brew install openjdk (veya --cask temurin). Bkz. bash scripts/doctor.sh" >&2; exit 1; }

echo "▶ 1/4  cargo build -p spike-cue-transfer ($PROFILE)"
cargo build --manifest-path "$CORE/Cargo.toml" -p spike-cue-transfer --profile "$CARGO_PROFILE"

TARGET_DIR="$CORE/target/$PROFILE"
DYLIB="$TARGET_DIR/libspike_cue_transfer.dylib"
[ -f "$DYLIB" ] || { echo "HATA: beklenen çıktı yok: $DYLIB" >&2; exit 1; }

echo "▶ 2/4  spike-uniffi-bindgen generate --language kotlin"
rm -rf "$OUT"
mkdir -p "$OUT/lib"

( cd "$CORE" && cargo run -q -p spike-cue-transfer --bin spike-uniffi-bindgen --profile "$CARGO_PROFILE" -- \
    generate --language kotlin --no-format --out-dir "$OUT" "$DYLIB" )
cp "$DYLIB" "$OUT/lib/libspike_cue_transfer.dylib"

cat > "$OUT/README.md" <<'GEN'
# ÜRETİLEN DİZİN — elle düzenlemeyin, commit etmeyin

`bash scripts/spike-cues-jvm.sh` üretir. `.gitignore` → `/core/spikes/**/generated/`.
GEN

cd "$PKG"
chmod +x ./gradlew

if [ "$MODE" != "measure-only" ]; then
  echo "▶ 3/4  ./gradlew test ($PROFILE)"
  ./gradlew --console=plain test
else
  echo "▶ 3/4  gradlew test atlandı (--measure-only)"
fi

if [ "$MODE" = "test-only" ]; then
  exit 0
fi

echo "▶ 4/4  ölçüm — her yaklaşım ayrı process"
echo

# --- Ölçüm bağlamı: bağlamsız sayı kanıt sayılmaz (M1 kuralı) ---
echo "## Bağlam"
echo
echo "- fixture: $COUNT cue"
echo "- build: **$PROFILE** (rust profile: $CARGO_PROFILE)"
echo "- cihaz: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo '?') · $(uname -m)"
echo "- OS: $(sw_vers -productName) $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
echo "- rustc: $(rustc --version)"
echo "- JVM: $(java -version 2>&1 | head -1)"
echo "- uniffi: $(grep -m1 '^uniffi = ' "$CORE/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
echo "- tarih: $(date '+%Y-%m-%d %H:%M %Z')"
echo

./gradlew --console=plain -q run --args="full $COUNT"
echo
./gradlew --console=plain -q run --args="windowed $COUNT"
