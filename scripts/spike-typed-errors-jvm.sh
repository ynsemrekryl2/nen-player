#!/usr/bin/env bash
# NEN-011 ölçümü: NEN-010'un (typed error mapping) Kotlin/JVM paritesi.
#
#   bash scripts/spike-typed-errors-jvm.sh                 # ölçüm + testler
#   bash scripts/spike-typed-errors-jvm.sh --test-only
#   bash scripts/spike-typed-errors-jvm.sh --measure-only
#
# Bu script ÜRÜN build'ine dokunmaz. Ölçülen crate core/spikes/ altındadır ve
# kendi FFI kapısını açar — buna izin veren karar ADR-0028'dir; NEN-011 aynı
# spike crate'inden Kotlin binding üretir (ADR-0028 "Sonuçlar").

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORE="$ROOT/core"
PKG="$ROOT/core/spikes/spike-typed-errors/jvm-harness"
OUT="$PKG/generated"

MODE="both"   # both | test-only | measure-only

while [ $# -gt 0 ]; do
  case "$1" in
    --test-only)      MODE="test-only"; shift ;;
    --measure-only)   MODE="measure-only"; shift ;;
    *) echo "Bilinmeyen argüman: $1 (--test-only | --measure-only)" >&2; exit 1 ;;
  esac
done

command -v cargo >/dev/null 2>&1 || {
  echo "HATA: cargo bulunamadı. Kurulum için: bash scripts/doctor.sh M1" >&2; exit 1; }
command -v java >/dev/null 2>&1 || {
  echo "HATA: JDK bulunamadı. Kurulum: brew install openjdk (veya --cask temurin). Bkz. bash scripts/doctor.sh" >&2; exit 1; }

echo "▶ 1/4  cargo build -p spike-typed-errors (release)"
cargo build --manifest-path "$CORE/Cargo.toml" -p spike-typed-errors --release

TARGET_DIR="$CORE/target/release"
DYLIB="$TARGET_DIR/libspike_typed_errors.dylib"
[ -f "$DYLIB" ] || { echo "HATA: beklenen çıktı yok: $DYLIB" >&2; exit 1; }

echo "▶ 2/4  spike-typed-errors-uniffi-bindgen generate --language kotlin"
rm -rf "$OUT"
mkdir -p "$OUT/lib"

( cd "$CORE" && cargo run -q -p spike-typed-errors --bin spike-typed-errors-uniffi-bindgen --release -- \
    generate --language kotlin --no-format --out-dir "$OUT" "$DYLIB" )
cp "$DYLIB" "$OUT/lib/libspike_typed_errors.dylib"

cat > "$OUT/README.md" <<'GEN'
# ÜRETİLEN DİZİN — elle düzenlemeyin, commit etmeyin

`bash scripts/spike-typed-errors-jvm.sh` üretir. `.gitignore` → `/core/spikes/**/generated/`.
GEN

cd "$PKG"
chmod +x ./gradlew

if [ "$MODE" != "measure-only" ]; then
  echo "▶ 3/4  ./gradlew test"
  ./gradlew --console=plain test
else
  echo "▶ 3/4  gradlew test atlandı (--measure-only)"
fi

if [ "$MODE" = "test-only" ]; then
  exit 0
fi

echo "▶ 4/4  ölçüm — eşleme maliyeti"
echo

echo "## Ölçüm bağlamı (build)"
echo
echo "- build: **release**"
echo "- cihaz: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo '?') · $(uname -m)"
echo "- OS: $(sw_vers -productName) $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
echo "- rustc: $(rustc --version)"
echo "- JVM: $(java -version 2>&1 | head -1)"
echo "- uniffi: $(grep -m1 '^uniffi = ' "$CORE/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
echo "- tarih: $(date '+%Y-%m-%d %H:%M %Z')"
echo

./gradlew --console=plain -q run
