#!/usr/bin/env bash
# scripts/lib/rewrite_macho_deps.py — deterministic testler.
#
# Gerçek Homebrew'a veya libmpv'ye dokunmaz: sentetik bir vendor prefix'i
# ($TMP/vendor) altında Homebrew'un kendi düzenini (Cellar/<formül>/<sürüm>/
# ile opt/<formül> sembolik bağı, sürümlü gerçek dosya + sürümsüz sembolik
# bağ) taklit eden iki-katmanlı bir dylib zinciri (mainbin -> liba -> libb)
# clang ile derlenir. `NEN_BUNDLE_VENDOR_PREFIXES` script'e bu sentetik
# prefix'i "vendor" olarak öğretir.
#
# Asıl iddia çalıştırılarak sınanır: yeniden yazılmış bundle, vendor prefix'i
# diskten kaldırıldıktan SONRA hâlâ doğru çalışıyor (main -> a_func() ->
# b_func() zinciri gerçekten yürüyor, yalnız statik otool okunmuyor).

set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
REWRITE="$ROOT/scripts/lib/rewrite_macho_deps.py"

command -v clang >/dev/null 2>&1 || { echo "ATLA: clang yok" >&2; exit 0; }
command -v otool >/dev/null 2>&1 || { echo "ATLA: otool yok" >&2; exit 0; }
command -v install_name_tool >/dev/null 2>&1 || { echo "ATLA: install_name_tool yok" >&2; exit 0; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

FAILURES=0
pass() { printf '    ok   %s\n' "$1"; }
fail() { printf '    FAIL %s\n' "$1" >&2; FAILURES=$((FAILURES + 1)); }

VENDOR="$TMP/vendor"
mkdir -p "$VENDOR/Cellar/liba/1.0.0/lib" "$VENDOR/Cellar/libb/2.0.0/lib" "$VENDOR/opt"

cat > "$TMP/libb.c" <<'EOF'
int b_value = 7;
int b_func(void) { return b_value; }
EOF
cat > "$TMP/liba.c" <<'EOF'
extern int b_func(void);
int a_func(void) { return b_func() + 1; }
EOF
cat > "$TMP/main.c" <<'EOF'
extern int a_func(void);
int main(void) { return a_func() == 8 ? 0 : 1; }
EOF

# --- Homebrew'un kendi düzeni: sürümlü gerçek dosya + sürümsüz sembolik
# bağ (Cellar içinde) + opt/<formül> -> Cellar/<formül>/<sürüm> ------------

clang -dynamiclib -o "$VENDOR/Cellar/libb/2.0.0/lib/libb.2.0.0.dylib" \
  -install_name "$VENDOR/opt/libb/lib/libb.2.dylib" "$TMP/libb.c" \
  || { echo "HATA: libb derlenemedi" >&2; exit 1; }
ln -s libb.2.0.0.dylib "$VENDOR/Cellar/libb/2.0.0/lib/libb.2.dylib"
ln -s ../Cellar/libb/2.0.0 "$VENDOR/opt/libb"

clang -dynamiclib -o "$VENDOR/Cellar/liba/1.0.0/lib/liba.1.0.0.dylib" \
  -install_name "$VENDOR/opt/liba/lib/liba.1.dylib" \
  "$TMP/liba.c" "$VENDOR/opt/libb/lib/libb.2.dylib" \
  || { echo "HATA: liba derlenemedi" >&2; exit 1; }
ln -s liba.1.0.0.dylib "$VENDOR/Cellar/liba/1.0.0/lib/liba.1.dylib"
ln -s ../Cellar/liba/1.0.0 "$VENDOR/opt/liba"

# --- İki paralel bundle: biri yeniden yazılacak, biri (negatif kontrol
# için) ham kopya olarak kalacak -------------------------------------------

REWRITTEN="$TMP/rewritten"
RAW="$TMP/raw"
mkdir -p "$REWRITTEN/Contents/MacOS" "$REWRITTEN/Contents/Frameworks"
mkdir -p "$RAW/Contents/MacOS" "$RAW/Contents/Frameworks"

clang -o "$REWRITTEN/Contents/MacOS/mainbin" "$TMP/main.c" \
  "$VENDOR/opt/liba/lib/liba.1.dylib" \
  || { echo "HATA: mainbin derlenemedi" >&2; exit 1; }
cp "$REWRITTEN/Contents/MacOS/mainbin" "$RAW/Contents/MacOS/mainbin"
cp "$VENDOR/opt/liba/lib/liba.1.0.0.dylib" "$RAW/Contents/Frameworks/liba.1.dylib" 2>/dev/null \
  || cp "$VENDOR/Cellar/liba/1.0.0/lib/liba.1.0.0.dylib" "$RAW/Contents/Frameworks/liba.1.dylib"
cp "$VENDOR/Cellar/libb/2.0.0/lib/libb.2.0.0.dylib" "$RAW/Contents/Frameworks/libb.2.dylib"

echo "▶ pozitif: yeniden yazma"
MANIFEST="$TMP/manifest.txt"
if ! NEN_BUNDLE_VENDOR_PREFIXES="$VENDOR" python3 "$REWRITE" \
     "$REWRITTEN/Contents/MacOS/mainbin" "$REWRITTEN/Contents/Frameworks" > "$MANIFEST" 2>"$TMP/rewrite.err"; then
  fail "rewrite_macho_deps.py çıkış 0 vermedi: $(cat "$TMP/rewrite.err")"
else
  pass "rewrite_macho_deps.py çıkış 0"
fi

manifest_lines="$(wc -l < "$MANIFEST" | tr -d ' ')"
[ "$manifest_lines" = "2" ] && pass "manifest 2 satır (liba + libb)" \
  || fail "manifest $manifest_lines satır, 2 bekleniyordu"

# --- basename seçimi: LC_ID_DYLIB'in basename'i, realpath'in değil --------
if [ -f "$REWRITTEN/Contents/Frameworks/liba.1.dylib" ] && [ -f "$REWRITTEN/Contents/Frameworks/libb.2.dylib" ]; then
  pass "kopyalanan dosyalar install-name basename'iyle adlandı (liba.1.dylib, libb.2.dylib — liba.1.0.0.dylib değil)"
else
  fail "beklenen bundle basename'leri yok: $(ls "$REWRITTEN/Contents/Frameworks" 2>/dev/null)"
fi

# --- id ve -change: hiçbir Mach-O vendor prefix'i taşımıyor ---------------
leaked=0
for f in "$REWRITTEN/Contents/MacOS/mainbin" "$REWRITTEN/Contents/Frameworks"/*; do
  [ -f "$f" ] || continue
  if otool -L "$f" 2>/dev/null | tail -n +2 | grep -qF "$VENDOR"; then
    leaked=1
  fi
done
[ "$leaked" = "0" ] && pass "yeniden yazılmış bundle'da vendor yolu kalmamış" \
  || fail "yeniden yazılmış bundle hâlâ vendor yolu taşıyor"

otool -L "$REWRITTEN/Contents/MacOS/mainbin" | grep -q '@rpath/liba\.1\.dylib' \
  && pass "mainbin @rpath/liba.1.dylib'e bağlı" \
  || fail "mainbin beklenen @rpath referansını taşımıyor"

otool -L "$REWRITTEN/Contents/Frameworks/liba.1.dylib" | grep -q '@rpath/libb\.2\.dylib' \
  && pass "liba.1.dylib @rpath/libb.2.dylib'e bağlı (geçişli bağımlılık takip edildi)" \
  || fail "liba.1.dylib beklenen @rpath referansını taşımıyor"

otool -D "$REWRITTEN/Contents/Frameworks/liba.1.dylib" | tail -1 | grep -q '^@rpath/liba\.1\.dylib$' \
  && pass "liba.1.dylib'in kendi LC_ID_DYLIB'i @rpath'e çevrildi" \
  || fail "liba.1.dylib'in LC_ID_DYLIB'i beklenmedik"

# --- rpath'ler: yürütülebilir @executable_path/../Frameworks, dylib
# @loader_path taşıyor ------------------------------------------------------
otool -l "$REWRITTEN/Contents/MacOS/mainbin" | grep -A2 LC_RPATH | grep -q '@executable_path/\.\./Frameworks' \
  && pass "mainbin LC_RPATH = @executable_path/../Frameworks" \
  || fail "mainbin'de beklenen LC_RPATH yok"

otool -l "$REWRITTEN/Contents/Frameworks/liba.1.dylib" | grep -A2 LC_RPATH | grep -q '@loader_path' \
  && pass "liba.1.dylib LC_RPATH = @loader_path" \
  || fail "liba.1.dylib'de beklenen LC_RPATH yok"

# --- asıl iddia: vendor diskten kaldırılınca da çalışıyor ------------------
echo "▶ pozitif: vendor kaldırıldıktan sonra çalışma zamanı"
"$REWRITTEN/Contents/MacOS/mainbin" >/dev/null 2>&1
rewritten_before=$?
[ "$rewritten_before" = "0" ] && pass "yeniden yazılmış binary, vendor hâlâ diskteyken çalışıyor (sağlık kontrolü)" \
  || fail "yeniden yazılmış binary vendor diskteyken bile çalışmadı (exit $rewritten_before)"

mv "$VENDOR" "$VENDOR.hidden"

"$REWRITTEN/Contents/MacOS/mainbin" >/dev/null 2>&1
rewritten_after=$?
[ "$rewritten_after" = "0" ] && pass "yeniden yazılmış binary, vendor GİZLİYKEN de çalışıyor (asıl iddia)" \
  || fail "yeniden yazılmış binary vendor gizliyken çalışmadı (exit $rewritten_after) — bundle hâlâ vendor'a bağımlı"

# --- negatif kontrol: kontrol sağır değil ----------------------------------
echo "▶ negatif: yeniden yazma atlanmış ham kopya"

"$RAW/Contents/MacOS/mainbin" >/dev/null 2>&1
raw_after=$?
[ "$raw_after" != "0" ] && pass "ham (yeniden yazılmamış) kopya vendor gizliyken çöküyor (exit $raw_after) — kontrol sağır değil" \
  || fail "ham kopya vendor gizliyken de çalıştı — negatif kontrol hiçbir şey ayırt etmiyor"

# scripts/bundle-macos.sh'in kendi tarama adımıyla aynı desen: ham kopyada
# hâlâ vendor yolu bulunmalı.
raw_leak=0
for f in "$RAW/Contents/MacOS/mainbin" "$RAW/Contents/Frameworks"/*; do
  [ -f "$f" ] || continue
  if otool -L "$f" 2>/dev/null | tail -n +2 | grep -qF "$VENDOR"; then
    raw_leak=1
  fi
done
[ "$raw_leak" = "1" ] && pass "bundle-macos.sh'in tarama deseni ham kopyada vendor yolunu buluyor" \
  || fail "tarama deseni ham kopyadaki vendor yolunu KAÇIRDI — tarama sağır"

echo
if [ "$FAILURES" -gt 0 ]; then
  echo "SONUÇ: $FAILURES doğrulama başarısız." >&2
  exit 1
fi
echo "SONUÇ: tüm doğrulamalar geçti."
