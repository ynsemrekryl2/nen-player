#!/usr/bin/env bash
# Rust core'u derler ve Apple hedefleri için Swift binding'ini üretir.
#
#   bash scripts/build-apple.sh              # debug
#   bash scripts/build-apple.sh --release
#
# Üretilen her şey platforms/apple-shared/generated/ altındadır ve
# .gitignore'ludur — binding commit EDİLMEZ, bu script yeniden üretir
# (NEN-007 DoD: "binding üretimi tek komutla tekrarlanabilir").
#
# Binding teknolojisi (UniFFI) ADR-0003 kabul edilene kadar ADAYDIR.
# nen-ffi'ın tek dış kapı olması ise ADR-0006 ile kabul edilmiştir.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORE="$ROOT/core"
OUT="$ROOT/platforms/apple-shared/generated"

# Cargo profil adı ile çıktı dizini farklı: dev -> target/debug.
CARGO_PROFILE="dev"
PROFILE="debug"
case "${1:-}" in
  --release) CARGO_PROFILE="release"; PROFILE="release" ;;
  "")        ;;
  *)         echo "Bilinmeyen argüman: $1 (yalnız --release)" >&2; exit 1 ;;
esac

if ! command -v cargo >/dev/null 2>&1; then
  echo "HATA: cargo bulunamadı. Kurulum için: bash scripts/doctor.sh M1" >&2
  exit 1
fi

echo "▶ 1/3  cargo build -p nen-ffi ($PROFILE)"
cargo build --manifest-path "$CORE/Cargo.toml" -p nen-ffi --profile "$CARGO_PROFILE"

TARGET_DIR="$CORE/target/$PROFILE"
DYLIB="$TARGET_DIR/libnen_ffi.dylib"
STATICLIB="$TARGET_DIR/libnen_ffi.a"
for f in "$DYLIB" "$STATICLIB"; do
  [ -f "$f" ] || { echo "HATA: beklenen çıktı yok: $f" >&2; exit 1; }
done

echo "▶ 2/3  uniffi-bindgen generate --language swift"
rm -rf "$OUT"
mkdir -p "$OUT/nen_ffiFFI" "$OUT/NenCore" "$OUT/lib"

RAW="$OUT/.raw"
# uniffi-bindgen içeride `cargo metadata` çağırıyor; workspace kökünden
# çalıştırılmazsa manifest'i bulamıyor. Yollar mutlak.
( cd "$CORE" && cargo run -q -p nen-ffi --bin uniffi-bindgen -- \
    generate --language swift --no-format --out-dir "$RAW" "$DYLIB" )

echo "▶ 3/3  SwiftPM düzenine yerleştirme"
# systemLibrary hedefi modulemap'i tam olarak 'module.modulemap' adıyla ister.
mv "$RAW/nen_ffiFFI.h"         "$OUT/nen_ffiFFI/nen_ffiFFI.h"
mv "$RAW/nen_ffiFFI.modulemap" "$OUT/nen_ffiFFI/module.modulemap"
mv "$RAW/nen_ffi.swift"        "$OUT/NenCore/nen_ffi.swift"
cp "$STATICLIB"                "$OUT/lib/libnen_ffi.a"
rmdir "$RAW"

cat > "$OUT/README.md" <<'GEN'
# ÜRETİLEN DİZİN — elle düzenlemeyin, commit etmeyin

`bash scripts/build-apple.sh` üretir. `.gitignore` → `/platforms/**/generated/`.
GEN

echo
echo "Tamam. Üretilenler ($PROFILE):"
find "$OUT" -type f | sed "s|^$ROOT/|  |" | sort
echo
echo "Test: bash scripts/test-apple.sh"
