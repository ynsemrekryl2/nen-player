#!/usr/bin/env bash
# scripts/check-docs.sh denetim 8 — STATUS.md ↔ INDEX ready listesi.
#
# Test kendi task fixture'ını kurar: canlı tasks/ ve INDEX.md OKUNMAZ. Sonuç
# bu yüzden repo'nun o anki durumundan — aktif task var mı, ready listesi boş
# mu — bağımsızdır. Denetim 8'in HER İKİ dalı ayrı ayrı doğrulanır.
#
# Gerçek repo dosyalarına dokunulmaz; T7 bunu içerik parmak iziyle kanıtlar.

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
#
# task-index.sh sandbox içinde $ROOT/tasks/INDEX.md'ye YAZAR — ROOT çözümlemesi
# kaçarsa gerçek INDEX.md ezilir. Parmak izi bu yüzden task dosyalarını ve
# üreticinin kendisini de kapsar.
real_fingerprint() {
  shasum "$ROOT/docs/STATUS.md" "$ROOT/tasks/INDEX.md" \
         "$ROOT/scripts/check-docs.sh" "$ROOT/scripts/task-index.sh" \
         "$ROOT"/tasks/*/NEN-*.md
}
FINGERPRINT_BEFORE="$(real_fingerprint)"

REPO="$TMP/repo"
CHECK="$REPO/scripts/check-docs.sh"
STATUS="$REPO/docs/STATUS.md"

# --------------------------------------------------------------------- fixture

# Geçerli frontmatter taşıyan fixture task'ı yazar. Denetim 1-7 de geçmeli,
# yoksa çıkış kodu denetim 8 dışında bir sebeple 1 olur.
# Kullanım: write_task <dizin> <id> <state> <depends_on içeriği>
write_task() {
  cat > "$REPO/tasks/$1/$2-fixture.md" <<EOF
---
id: $2
title: Fixture task $2
milestone: M0
size: S
state: $3
depends_on: [$4]
blocks: []
adr: []
---

# $2 — Fixture task

## Kanıt kaydı

Fixture — scripts/tests/check-docs.test.sh tarafından üretildi.
EOF
}

# Yalnız denetim 8'in ihtiyaç duyduğu satırı taşıyan minimal STATUS.md.
write_status() {
  cat > "$STATUS" <<'EOF'
# Durum (fixture)

| | |
|---|---|
| **Mevcut milestone** | M0 |
| **Sıradaki READY** | henüz belirlenmedi |
EOF
}

# Sandbox'ı sıfırdan kurar.
#   ready → ready listesi DOLU  (NEN-902 backlog, bağımlılığı done)
#   empty → ready listesi BOŞ   (NEN-902 active, hiç backlog task'ı yok)
# core/ ve platforms/ kopyalanmaz; check-docs.sh'ın ihtiyacı olan tek şey
# scripts/ + docs/STATUS.md + tasks/.
make_fixture() {
  rm -rf "$REPO"
  mkdir -p "$REPO/scripts" "$REPO/docs" \
           "$REPO/tasks/backlog" "$REPO/tasks/active" "$REPO/tasks/done"
  cp "$ROOT/scripts/check-docs.sh" "$ROOT/scripts/task-index.sh" "$REPO/scripts/"

  write_task done NEN-901 done ""
  case "$1" in
    ready) write_task backlog NEN-902 backlog "NEN-901" ;;
    empty) write_task active  NEN-902 active  "NEN-901" ;;
    *) echo "make_fixture: bilinmeyen mod '$1'" >&2; exit 1 ;;
  esac

  # INDEX elle yazılmaz — üreticinin gerçek çıktısı kullanılır (CLAUDE.md kural
  # 9). Denetim 7 böylece kendiliğinden geçer ve ready listesi testin varsayımı
  # değil, task-index.sh'ın hesabı olur.
  bash "$REPO/scripts/task-index.sh" >/dev/null

  write_status
}

# Fixture'ın gerçekten kurulmak istenen dalda olduğunu doğrular. Bu olmadan
# boş-ready testleri sessizce boş yere geçebilir.
expect_index_ready() { # <beklenen liste — boş olabilir>
  got="$(awk '/^## Sıradaki uygun task/{f=1;next} f' "$REPO/tasks/INDEX.md" \
        | grep -o 'NEN-[0-9][0-9][0-9]' | sort -u | tr '\n' ' ' | sed 's/ *$//')"
  if [ "$got" = "$1" ]; then
    pass "fixture ready listesi = '${1:-<boş>}'"
  else
    fail "fixture ready listesi '$got', beklenen '${1:-<boş>}'"
  fi
}

# ------------------------------------------------------------------- yardımcılar

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

# =============================================== A. ready listesi DOLU olan dal

echo "  Fixture: ready listesi dolu (NEN-902 backlog)"
make_fixture ready
expect_index_ready "NEN-902"

echo "  T1: STATUS ready listesi INDEX ile uyumlu"
set_status_row '`NEN-902`'
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
write_status
set_status_row '`NEN-902`'
run_check
expect 0 "STATUS.md ready listesi INDEX ile uyumlu" "T6 düzeltme sonrası geçiyor"

# =============================================== B. ready listesi BOŞ olan dal
#
# Sistemin normal hâli: her backlog task'ı bir bağımlılık bekliyor ya da tek
# kalan task active'e alınmış. Eski fixture bu dalı hiç test edemiyordu.

echo "  Fixture: ready listesi boş (NEN-902 active, backlog boş)"
make_fixture empty
expect_index_ready ""

echo "  T8: ready listesi boş, STATUS de task listelemiyor"
set_status_row 'henüz belirlenmedi'
run_check
expect 0 "ready listesi boş, STATUS.md de task listelemiyor" "T8 iki taraf da boş → geçiyor"

echo "  T9: ready listesi boş ama STATUS task listeliyor (negatif)"
set_status_row '`NEN-902`'
run_check
expect 1 "INDEX ile uyuşmuyor" "T9 boş INDEX + dolu STATUS → hata"

# =============================================================== C. yan etkisizlik

echo "  T7: gerçek repo dosyaları değişmedi"
if [ "$(real_fingerprint)" = "$FINGERPRINT_BEFORE" ]; then
  pass "T7 gerçek STATUS.md / INDEX.md / task dosyaları / script'ler değişmedi"
else
  fail "T7 — test gerçek repo dosyalarını değiştirmiş!"
  diff <(printf '%s\n' "$FINGERPRINT_BEFORE") <(real_fingerprint) >&2
fi

[ "$FAILURES" -eq 0 ] || { echo "  $FAILURES doğrulama başarısız" >&2; exit 1; }
exit 0
