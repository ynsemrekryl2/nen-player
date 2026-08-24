#!/usr/bin/env bash
# Shell testlerini çalıştırır.
#
#   bash scripts/test.sh
#
# scripts/tests/*.test.sh dosyalarını sırayla koşar. Biri bile başarısızsa çıkış 1.

set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FAILED=0
RUN=0

for t in "$ROOT"/scripts/tests/*.test.sh; do
  [ -e "$t" ] || continue
  RUN=$((RUN + 1))
  name="$(basename "$t")"
  echo "▶ $name"
  if bash "$t"; then
    echo "  ✓ $name geçti"
  else
    echo "  ✗ $name BAŞARISIZ" >&2
    FAILED=$((FAILED + 1))
  fi
  echo
done

if [ "$RUN" -eq 0 ]; then
  echo "HATA: scripts/tests/ altında test bulunamadı." >&2
  exit 1
fi

if [ "$FAILED" -gt 0 ]; then
  echo "SONUÇ: $RUN test dosyasından $FAILED tanesi başarısız." >&2
  exit 1
fi
echo "SONUÇ: $RUN test dosyasının hepsi geçti."
