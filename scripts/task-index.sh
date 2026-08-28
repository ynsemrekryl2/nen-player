#!/usr/bin/env bash
# tasks/INDEX.md'yi task dosyalarının frontmatter'ından üretir.
#
#   bash scripts/task-index.sh           INDEX.md'yi yeniden yazar
#   bash scripts/task-index.sh --check   Bayatsa çıkış kodu 1 (CI için)
#
# INDEX.md ELLE DÜZENLENMEZ.

set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TASKS="$ROOT/tasks"
INDEX="$TASKS/INDEX.md"

CHECK=0
[ "${1:-}" = "--check" ] && CHECK=1

# frontmatter'dan tek alan oku:  fm_get <dosya> <anahtar>
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

# "[NEN-001, NEN-002]" -> "NEN-001 NEN-002"
list_items() {
  printf '%s' "$1" | tr -d '[]' | tr ',' ' ' | tr -s ' ' | sed 's/^ *//; s/ *$//'
}

milestone_name() {
  case "$1" in
    M0)  echo "Foundation" ;;
    M1)  echo "Core Technical Spike" ;;
    M2)  echo "Subtitle Core" ;;
    M3)  echo "macOS Vertical Slice" ;;
    M4)  echo "Stremio Handoff (macOS)" ;;
    M5)  echo "Translation Core" ;;
    M6)  echo "Real Providers" ;;
    M7)  echo "Manual Sync" ;;
    M8)  echo "AI Audio-Assisted Sync" ;;
    M9)  echo "Windows / Linux" ;;
    M10) echo "Android / Android TV" ;;
    M11) echo "iOS / tvOS" ;;
    *)   echo "?" ;;
  esac
}

state_icon() {
  case "$1" in
    done)    echo "✅" ;;
    active)  echo "🔵" ;;
    blocked) echo "⛔" ;;
    canceled) echo "🚫" ;;
    backlog) echo "⚪" ;;
    *)       echo "❓" ;;
  esac
}

# id'ye göre sıralı dosya listesi
collect() {
  for d in backlog active done canceled; do
    [ -d "$TASKS/$d" ] || continue
    for f in "$TASKS/$d"/NEN-*.md; do
      [ -e "$f" ] || continue
      echo "$(basename "$f")|$f"
    done
  done | sort | cut -d'|' -f2-
}

generate() {
  FILES="$(collect)"
  [ -z "$FILES" ] && { echo "HATA: hiç task dosyası bulunamadı" >&2; exit 1; }

  total=0; n_done=0; n_active=0; n_blocked=0; n_backlog=0; n_canceled=0
  while IFS= read -r f; do
    total=$((total + 1))
    case "$(fm_get "$f" state)" in
      done)    n_done=$((n_done + 1)) ;;
      active)  n_active=$((n_active + 1)) ;;
      blocked) n_blocked=$((n_blocked + 1)) ;;
      backlog) n_backlog=$((n_backlog + 1)) ;;
      canceled) n_canceled=$((n_canceled + 1)) ;;
    esac
  done <<EOF
$FILES
EOF

  echo "# Task Index"
  echo
  echo "<!-- ÜRETİLEN DOSYA — elle düzenlemeyin."
  echo "     Yenilemek için: bash scripts/task-index.sh -->"
  echo
  echo "Toplam **$total** task · ✅ done $n_done · 🔵 active $n_active · ⛔ blocked $n_blocked · 🚫 canceled $n_canceled · ⚪ backlog $n_backlog"
  echo
  echo "Format ve kurallar: [tasks/README.md](README.md) · Milestone planı: [docs/roadmap.md](../docs/roadmap.md)"

  for m in M0 M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11; do
    rows=""
    while IFS= read -r f; do
      [ "$(fm_get "$f" milestone)" = "$m" ] || continue
      id="$(fm_get "$f" id)"
      title="$(fm_get "$f" title)"
      size="$(fm_get "$f" size)"
      state="$(fm_get "$f" state)"
      deps="$(list_items "$(fm_get "$f" depends_on)")"
      [ -z "$deps" ] && deps="—"
      rel="${f#$ROOT/tasks/}"
      rows="${rows}| [$id]($rel) | $title | $size | $(state_icon "$state") $state | $deps |
"
    done <<EOF
$FILES
EOF
    [ -z "$rows" ] && continue
    echo
    echo "## $m — $(milestone_name "$m")"
    echo
    echo "| ID | Başlık | Boyut | Durum | Bağımlılık |"
    echo "|---|---|---|---|---|"
    printf '%s' "$rows"
  done

  echo
  echo "## Sıradaki uygun task'lar"
  echo
  echo "Bağımlılıkları tamamlanmış, henüz başlanmamış task'lar:"
  echo
  ready=""
  while IFS= read -r f; do
    [ "$(fm_get "$f" state)" = "backlog" ] || continue
    deps="$(list_items "$(fm_get "$f" depends_on)")"
    ok=1
    for d in $deps; do
      [ -z "$d" ] && continue
      if ! grep -qx "state: done" "$TASKS"/done/"$d"-*.md 2>/dev/null; then ok=0; break; fi
    done
    [ "$ok" = "1" ] && ready="${ready}- **$(fm_get "$f" id)** — $(fm_get "$f" title)
"
  done <<EOF
$FILES
EOF
  if [ -z "$ready" ]; then echo "_(yok — bloke bağımlılıklar için tabloya bakın)_"; else printf '%s' "$ready"; fi
}

if [ "$CHECK" = "1" ]; then
  tmp="$(mktemp)"
  generate > "$tmp"
  if [ ! -f "$INDEX" ]; then
    echo "HATA: $INDEX yok. Üretmek için: bash scripts/task-index.sh" >&2
    rm -f "$tmp"; exit 1
  fi
  if ! diff -q "$INDEX" "$tmp" >/dev/null; then
    echo "HATA: tasks/INDEX.md bayat. Yenilemek için: bash scripts/task-index.sh" >&2
    diff -u "$INDEX" "$tmp" | head -40 >&2
    rm -f "$tmp"; exit 1
  fi
  rm -f "$tmp"
  echo "OK: tasks/INDEX.md güncel."
else
  generate > "$INDEX"
  echo "Üretildi: tasks/INDEX.md"
fi
