#!/usr/bin/env bash
# scripts/check-docs.sh denetim 8 — STATUS.md ↔ INDEX ready listesi.
#
# Repo'nun geçici bir KOPYASINDA çalışır; gerçek dosyalara dokunmaz.

set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
FAILURES=0

pass() { printf '    ok   %s\n' "$1"; }
fail() { printf '    FAIL %s\n' "$1" >&2; FAILURES=$((FAILURES + 1)); }

# Testin gerçek repo dosyalarına dokunmadığını git durumundan BAĞIMSIZ ölçmek
# için, test başında içerik parmak izi alınır. (git status'a bakmak yanıltıcı:
# dosya başka bir sebeple de commit'lenmemiş olabilir.)
REAL_FILES="$ROOT/docs/STATUS.md $ROOT/tasks/INDEX.md $ROOT/scripts/check-docs.sh"
FINGERPRINT_BEFORE="$(shasum $REAL_FILES)"

REPO="$TMP/repo"
mkdir -p "$REPO"
# .git hariç repo kopyası
(cd "$ROOT" && tar --exclude=.git -cf - .) | (cd "$REPO" && tar -xf -)

CHECK="$REPO/scripts/check-docs.sh"
STATUS="$REPO/docs/STATUS.md"
INDEX="$REPO/tasks/INDEX.md"

# INDEX'in hesapladığı gerçek ready listesi
READY="$(awk '/^## Sıradaki uygun task/{f=1;next} f' "$INDEX" \
        | grep -o 'NEN-[0-9][0-9][0-9]' | sort -u | tr '\n' ' ')"

# STATUS'taki READY satırını verilen içerikle değiştirir.
set_status_row() { # <satır içeriği>
  python3 - "$STATUS" "$1" <<'PY'
import re, sys
p, content = sys.argv[1], sys.argv[2]
s = open(p, encoding='utf-8').read()
s = re.sub(r'^\| \*\*Sıradaki READY\*\* \|.*\|$',
           f'| **Sıradaki READY** | {content} |', s, count=1, flags=re.M)
open(p, 'w', encoding='utf-8').write(s)
PY
}

remove_status_row() {
  python3 - "$STATUS" <<'PY'
import re, sys
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
s = re.sub(r'^\| \*\*Sıradaki READY\*\* \|.*\|\n', '', s, count=1, flags=re.M)
open(p, 'w', encoding='utf-8').write(s)
PY
}

run_check() {
  OUT="$(bash "$CHECK" 2>&1)"
  RC=$?
}

expect() { # <beklenen rc> <çıktıda aranan> <açıklama>
  if [ "$RC" != "$1" ]; then
    fail "$3 → beklenen exit $1, gelen $RC"
    printf '%s\n' "$OUT" | sed 's/^/         | /' >&2
    return
  fi
  case "$OUT" in
    *"$2"*) pass "$3" ;;
    *) fail "$3 — çıktıda '$2' yok"; printf '%s\n' "$OUT" | sed 's/^/         | /' >&2 ;;
  esac
}

echo "  T1: STATUS ready listesi INDEX ile uyumlu"
set_status_row "$(for t in $READY; do printf '`%s` ' "$t"; done)"
run_check
expect 0 "STATUS.md ready listesi INDEX ile uyumlu" "T1 uyumlu → geçiyor"

echo "  T2: STATUS'ta olmayan bir task listelenmiş"
set_status_row '`NEN-999` (uydurma)'
run_check
expect 1 "INDEX ile uyuşmuyor" "T2 uyumsuz → hata"

echo "  T3: STATUS eksik task listeliyor (INDEX'te olan yok)"
set_status_row 'henüz belirlenmedi'
run_check
expect 1 "INDEX ile uyuşmuyor" "T3 boş liste → hata"

echo "  T4: READY satırı tamamen silinmiş"
remove_status_row
run_check
expect 1 "'**Sıradaki READY**' satırı yok" "T4 satır yok → hata"

echo "  T5: STATUS.md hiç yok"
rm -f "$STATUS"
run_check
expect 1 "docs/STATUS.md yok" "T5 dosya yok → hata"

echo "  T6: düzeltilince tekrar geçiyor"
cp "$ROOT/docs/STATUS.md" "$STATUS"
set_status_row "$(for t in $READY; do printf '`%s` ' "$t"; done)"
run_check
expect 0 "STATUS.md ready listesi INDEX ile uyumlu" "T6 düzeltme sonrası geçiyor"

echo "  T7: gerçek repo dosyaları değişmedi"
if [ "$(shasum $REAL_FILES)" = "$FINGERPRINT_BEFORE" ]; then
  pass "T7 gerçek STATUS.md / INDEX.md / check-docs.sh değişmedi"
else
  fail "T7 — test gerçek repo dosyalarını değiştirmiş!"
  diff <(printf '%s\n' "$FINGERPRINT_BEFORE") <(shasum $REAL_FILES) >&2
fi

[ "$FAILURES" -eq 0 ] || { echo "  $FAILURES doğrulama başarısız" >&2; exit 1; }
exit 0
