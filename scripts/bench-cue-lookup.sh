#!/usr/bin/env bash
# NEN-017 ölçümü: 50k cue'luk bir dokümanda indeksli lookup'ın maliyeti ve
# product-spec §14'ün yasakladığı lineer taramayla karşılaştırması.
#
#   bash scripts/bench-cue-lookup.sh           # release (varsayılan)
#   bash scripts/bench-cue-lookup.sh --debug
#
# Ölçüm testi `#[ignore]`'lu olduğu için normal `cargo test` ve CI bundan
# etkilenmez — tablo yalnız bu script'le üretilir.
#
# Çıktı BASELINE'dır, eşik değil: test hiçbir süre iddia etmez, yalnız ölçer.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORE="$ROOT/core"

PROFILE="release"
CARGO_FLAG="--release"

while [ $# -gt 0 ]; do
  case "$1" in
    --debug)   PROFILE="debug"; CARGO_FLAG=""; shift ;;
    --release) PROFILE="release"; CARGO_FLAG="--release"; shift ;;
    *) echo "Bilinmeyen argüman: $1 (--debug | --release)" >&2; exit 1 ;;
  esac
done

command -v cargo >/dev/null 2>&1 || {
  echo "HATA: cargo bulunamadı. Kurulum için: bash scripts/doctor.sh M1" >&2; exit 1; }

echo "▶ nen-subtitle lookup baseline ($PROFILE)"
# shellcheck disable=SC2086 # boş CARGO_FLAG kelime olarak geçmemeli
cargo test --manifest-path "$CORE/Cargo.toml" -p nen-subtitle $CARGO_FLAG \
  --test lookup_bench -- --ignored --nocapture

echo "✔ Ölçüm tamam. Build tipi: $PROFILE — kanıt kaydına bu bilgiyle yazın."
