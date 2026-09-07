#!/usr/bin/env bash
# NEN-043: libmpv ve dinamik bağımlılıklarını .app içine gömer.
#
#   bash scripts/bundle-macos.sh
#
# scripts/build-macos-app.sh'in ürettiği gelişim bundle'ını alır ve:
#   1. libmpv'nin geçişli Homebrew dylib kapanışını hesaplar (sabit sayı
#      değil — bu makinenin Homebrew kurulumundan okunur;
#      scripts/lib/rewrite_macho_deps.py)
#   2. her dylib'i Contents/Frameworks/ altına, kendi LC_ID_DYLIB'inin
#      basename'iyle kopyalar
#   3. install_name_tool ile tüm Homebrew yollarını @rpath/@loader_path'e
#      çevirir
#   4. LICENSE, mpv'nin kendi lisans metinleri ve üretilen bir
#      THIRD-PARTY.md'yi Contents/Resources/licenses/ altına koyar
#   5. her dylib'i ve .app'i ad-hoc imzalar (Developer ID + notarization
#      S11'dedir — ADR-0012 Karar 3, bkz. NEN-043 "YAPILMAYACAK")
#   6. bundle'ı bağımsızca tarayıp Homebrew'a işaret eden tek bir yol
#      kalmadığını doğrular — bulursa çıkış 1
#
# Yeniden çalıştırılabilir: her çalışma build-macos-app.sh'i baştan
# çağırdığı ve Frameworks/lisans dizinlerini temizlediği için önceki
# gömme kalıntısı taşınmaz.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG="$ROOT/platforms/macos"

command -v python3 >/dev/null 2>&1 || { echo "HATA: python3 bulunamadı." >&2; exit 1; }
command -v brew >/dev/null 2>&1 || { echo "HATA: brew bulunamadı." >&2; exit 1; }

echo "▶ 1/6  scripts/build-macos-app.sh"
bash "$ROOT/scripts/build-macos-app.sh" >&2

APP="$PKG/.build/NenPlayer.app"
CONTENTS="$APP/Contents"
FRAMEWORKS="$CONTENTS/Frameworks"
LICENSES="$CONTENTS/Resources/licenses"
BINARY="$CONTENTS/MacOS/NenPlayer"

[ -x "$BINARY" ] || { echo "HATA: $BINARY yok veya çalıştırılabilir değil" >&2; exit 1; }

rm -rf "$FRAMEWORKS" "$LICENSES"
mkdir -p "$FRAMEWORKS" "$LICENSES"

# --- 2+3+4. Kapanış hesapla, kopyala, yolları yeniden yaz ----------------
echo "▶ 2/6  bağımlılık kapanışı hesaplanıyor, kopyalanıyor, yollar yeniden yazılıyor"

MANIFEST="$(mktemp)"
trap 'rm -f "$MANIFEST"' EXIT

python3 "$ROOT/scripts/lib/rewrite_macho_deps.py" "$BINARY" "$FRAMEWORKS" > "$MANIFEST"

count="$(wc -l < "$MANIFEST" | tr -d ' ')"
echo "  $count Homebrew dylib gömüldü"
[ "$count" -gt 0 ] || { echo "HATA: hiç Homebrew dylib bulunamadı — libmpv bağlı mı?" >&2; exit 1; }

# --- 5. Lisanslar ----------------------------------------------------------
echo "▶ 3/6  lisans metinleri ve THIRD-PARTY bildirimi"

cp "$ROOT/LICENSE" "$LICENSES/LICENSE"

MPV_PREFIX="$(brew --prefix mpv 2>/dev/null || true)"
if [ -n "$MPV_PREFIX" ] && [ -d "$MPV_PREFIX" ]; then
  for f in Copyright LICENSE.GPL LICENSE.LGPL; do
    [ -f "$MPV_PREFIX/$f" ] && cp "$MPV_PREFIX/$f" "$LICENSES/mpv-$f"
  done
else
  echo "UYARI: mpv formülü prefix'i bulunamadı, mpv lisans metinleri kopyalanamadı" >&2
fi

THIRD_PARTY="$LICENSES/THIRD-PARTY.md"
{
  echo "# Üçüncü taraf bileşenler"
  echo
  echo "Bu \`.app\` aşağıdaki Homebrew formüllerinin ikili çıktısını gömer."
  echo "Liste \`scripts/bundle-macos.sh\` tarafından üretilmiştir — elle"
  echo "düzenlenmez."
  echo
  echo "| Dosya | Formül | Sürüm | Lisans |"
  echo "|---|---|---|---|"
  while IFS=$'\t' read -r real base; do
    formula="$(printf '%s' "$real" | sed -nE 's#.*/Cellar/([^/]+)/([^/]+)/.*#\1#p')"
    version="$(printf '%s' "$real" | sed -nE 's#.*/Cellar/([^/]+)/([^/]+)/.*#\2#p')"
    license="?"
    if [ -n "$formula" ]; then
      license="$(brew info --json=v2 "$formula" 2>/dev/null \
        | python3 -c "import json,sys
try:
    d = json.load(sys.stdin)
    print(d['formulae'][0].get('license') or '?')
except Exception:
    print('?')" 2>/dev/null || echo "?")"
    else
      formula="?"; version="?"
    fi
    echo "| \`$base\` | \`$formula\` | $version | $license |"
  done < "$MANIFEST" | sort
  echo
  echo "GNU GPL v3 §6 uyarınca: bu bundle'ın kaynak kodu depo kökündeki"
  echo "\`README.md\`'de belirtilen adreste GPL-3.0-or-later ile açıktır."
  echo "Tam lisans metni: \`LICENSE\`."
} > "$THIRD_PARTY"

# --- 6. İmzala --------------------------------------------------------------
echo "▶ 4/6  ad-hoc imza"

while IFS=$'\t' read -r real base; do
  codesign --force --sign - "$FRAMEWORKS/$base"
done < "$MANIFEST"

codesign --force --sign - \
  --entitlements "$PKG/NenPlayer.entitlements" \
  "$APP"

echo "▶ 5/6  codesign --verify"
codesign --verify --deep --strict --verbose=2 "$APP" >&2

# --- 7. Bağımsız doğrulama ---------------------------------------------------
echo "▶ 6/6  bundle taranıyor: kalan Homebrew referansı var mı?"

LEAK=0
while IFS= read -r macho; do
  while IFS= read -r rawdep; do
    dep="$(printf '%s' "$rawdep" | sed -E 's/^[[:space:]]+//; s/ \(compatibility.*$//')"
    case "$dep" in
      /opt/homebrew/*|/usr/local/*)
        echo "HATA: $macho hâlâ şuna işaret ediyor: $dep" >&2
        LEAK=1
        ;;
    esac
  done < <(otool -L "$macho" 2>/dev/null | tail -n +2)
done < <(find "$APP" -type f \( -perm -111 -o -name '*.dylib' \))

if [ "$LEAK" != "0" ]; then
  echo "HATA: bundle içinde Homebrew'a işaret eden referans kaldı." >&2
  exit 1
fi

echo "  temiz — hiçbir Mach-O /opt/homebrew veya /usr/local'a işaret etmiyor"
echo
echo "Uygulama: $APP"
echo "Gömülü dylib sayısı: $count"
echo "Çalıştır: open '$APP'"
