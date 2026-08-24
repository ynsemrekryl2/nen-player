#!/usr/bin/env bash
# Task / ADR / index tutarlılık denetimleri.
#
#   bash scripts/check-docs.sh
#
# Her hata: dosya + sorun + ne yapılacağı. Hata varsa çıkış kodu 1.

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TASKS="$ROOT/tasks"
ADRS="$ROOT/docs/adr"
ERRORS=0

err() { echo "HATA  $*" >&2; ERRORS=$((ERRORS + 1)); }
ok()  { echo "  ok  $*"; }

fm_get() {
  awk -v key="$2" '
    NR == 1 && $0 == "---" { inside = 1; next }
    inside && $0 == "---"  { exit }
    inside {
      i = index($0, ":")
      if (i > 0) {
        k = substr($0, 1, i - 1); v = substr($0, i + 1)
        gsub(/^[ \t]+|[ \t]+$/, "", k)
        gsub(/^[ \t]+|[ \t]+$/, "", v)
        if (k == key) { print v; exit }
      }
    }
  ' "$1"
}

list_items() {
  printf '%s' "$1" | tr -d '[]' | tr ',' ' ' | tr -s ' ' | sed 's/^ *//; s/ *$//'
}

collect() {
  for d in backlog active done; do
    [ -d "$TASKS/$d" ] || continue
    for f in "$TASKS/$d"/NEN-*.md; do
      [ -e "$f" ] || continue
      echo "$(basename "$f")|$f"
    done
  done | sort | cut -d'|' -f2-
}

FILES="$(collect)"
[ -z "$FILES" ] && { err "hiç task dosyası bulunamadı ($TASKS)"; exit 1; }

echo "== 1. Aktif task sayısı =="
n_active=$(ls "$TASKS"/active/NEN-*.md 2>/dev/null | wc -l | tr -d ' ')
if [ "$n_active" -gt 1 ]; then
  err "tasks/active/ içinde $n_active task var, en fazla 1 olabilir (CLAUDE.md kural 2)."
  ls "$TASKS"/active/NEN-*.md >&2
  echo "      Yapılacak: fazla task'ı tasks/backlog/ altına geri taşıyın ve state: backlog yapın." >&2
else
  ok "aktif task: $n_active"
fi

echo "== 2. state alanı ↔ dizin uyumu =="
bad=0
while IFS= read -r f; do
  dir="$(basename "$(dirname "$f")")"
  st="$(fm_get "$f" state)"
  case "$dir:$st" in
    backlog:backlog|backlog:blocked|active:active|done:done) ;;
    *)
      err "${f#$ROOT/}: state '$st' ile dizin '$dir' uyumsuz."
      echo "      Yapılacak: state'i düzeltin veya dosyayı doğru dizine taşıyın." >&2
      bad=1 ;;
  esac
done <<EOF
$FILES
EOF
[ "$bad" = "0" ] && ok "tüm state alanları dizinleriyle uyumlu"

echo "== 3. done task'larda kanıt kaydı =="
bad=0
while IFS= read -r f; do
  [ "$(fm_get "$f" state)" = "done" ] || continue
  body="$(awk '/^## Kanıt kaydı/{flag=1; next} flag' "$f" | sed 's/<!--.*-->//' | tr -d '[:space:]')"
  if [ -z "$body" ] || [ "$body" = "TBD" ]; then
    err "${f#$ROOT/}: done ama 'Kanıt kaydı' bölümü boş."
    echo "      Yapılacak: gerçek test çıktısı / benchmark sayısı / kayıt yolu yazın (CLAUDE.md kural 3)." >&2
    bad=1
  fi
done <<EOF
$FILES
EOF
[ "$bad" = "0" ] && ok "tüm done task'ların kanıt kaydı dolu"

echo "== 4. depends_on / blocks hedefleri =="
ALL_IDS="$(while IFS= read -r f; do fm_get "$f" id; done <<EOF
$FILES
EOF
)"
bad=0
while IFS= read -r f; do
  id="$(fm_get "$f" id)"
  for field in depends_on blocks; do
    for d in $(list_items "$(fm_get "$f" $field)"); do
      [ -z "$d" ] && continue
      if ! printf '%s\n' "$ALL_IDS" | grep -qx "$d"; then
        err "${f#$ROOT/}: $field içinde '$d' var ama böyle bir task yok."
        echo "      Yapılacak: ID'yi düzeltin veya eksik task dosyasını oluşturun." >&2
        bad=1
      fi
    done
  done
done <<EOF
$FILES
EOF
[ "$bad" = "0" ] && ok "tüm depends_on/blocks hedefleri mevcut"

echo "== 5. Bağımlılık döngüsü =="
EDGES="$(while IFS= read -r f; do
  id="$(fm_get "$f" id)"
  for d in $(list_items "$(fm_get "$f" depends_on)"); do
    [ -n "$d" ] && echo "$d $id"
  done
done <<EOF
$FILES
EOF
)"
if [ -n "$EDGES" ]; then
  cyc="$(printf '%s\n' "$EDGES" | tsort 2>&1 >/dev/null)"
  if [ -n "$cyc" ]; then
    err "bağımlılık döngüsü var:"
    printf '%s\n' "$cyc" >&2
    echo "      Yapılacak: döngüdeki task'lardan birinin depends_on'unu kaldırın." >&2
  else
    ok "döngü yok"
  fi
else
  ok "bağımlılık kenarı yok"
fi

echo "== 6. done task'ların ADR durumu =="
bad=0
while IFS= read -r f; do
  [ "$(fm_get "$f" state)" = "done" ] || continue
  for a in $(list_items "$(fm_get "$f" adr)"); do
    [ -z "$a" ] && continue
    num="$(printf '%04d' "$a" 2>/dev/null || echo "$a")"
    adr_file="$(ls "$ADRS"/${num}-*.md 2>/dev/null | head -1)"
    if [ -z "$adr_file" ]; then
      err "${f#$ROOT/}: ADR-$num referans ediliyor ama docs/adr/ altında yok."
      bad=1; continue
    fi
    st="$(fm_get "$adr_file" status)"
    if [ "$st" != "accepted" ]; then
      err "${f#$ROOT/}: done, ama ADR-$num durumu '$st' (accepted olmalı)."
      echo "      Yapılacak: ADR'yi onaylatın veya task'ı done'dan geri alın (ADR-0001)." >&2
      bad=1
    fi
  done
done <<EOF
$FILES
EOF
[ "$bad" = "0" ] && ok "done task'ların ADR'leri accepted"

echo "== 7. INDEX.md güncelliği =="
if bash "$ROOT/scripts/task-index.sh" --check >/dev/null 2>&1; then
  ok "tasks/INDEX.md güncel"
else
  err "tasks/INDEX.md bayat."
  echo "      Yapılacak: bash scripts/task-index.sh" >&2
fi

echo
if [ "$ERRORS" -gt 0 ]; then
  echo "SONUÇ: $ERRORS hata." >&2
  exit 1
fi
echo "SONUÇ: tüm denetimler geçti."
