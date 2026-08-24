#!/usr/bin/env bash
# Toolchain ön koşul kontrolü. HİÇBİR ŞEY KURMAZ — yalnız raporlar.
#
#   bash scripts/doctor.sh
#
# Eksik ZORUNLU araç varsa çıkış kodu 1.

set -u

MISSING_REQUIRED=0
MISSING_OPTIONAL=0

# Takılabilen komutlar için zaman sınırlı çalıştırma (macOS'ta GNU timeout yok).
run_timeout() { s="$1"; shift; perl -e 'alarm shift; exec @ARGV' "$s" "$@" 2>/dev/null; }

row() { # <durum> <araç> <detay> <milestone>
  printf '  %-4s %-14s %-40s %s\n' "$1" "$2" "$3" "$4"
}

fail() { # <araç> <milestone> <kurulum>
  MISSING_REQUIRED=$((MISSING_REQUIRED + 1))
  row "✗" "$1" "EKSİK" "$2"
  printf '       kurulum: %s\n' "$3"
}

warn() { # <araç> <milestone> <kurulum>
  MISSING_OPTIONAL=$((MISSING_OPTIONAL + 1))
  row "⚠" "$1" "eksik (opsiyonel)" "$2"
  printf '       kurulum: %s\n' "$3"
}

echo "Nen Player — toolchain doctor"
echo "$(sw_vers -productName 2>/dev/null) $(sw_vers -productVersion 2>/dev/null)"
echo
printf '  %-4s %-14s %-40s %s\n' "" "ARAÇ" "DURUM" "GEREKLİ"
printf '  %s\n' "------------------------------------------------------------------------------"

# --- Rust (M1) ---
if command -v cargo >/dev/null 2>&1; then
  row "✓" "cargo" "$(run_timeout 10 cargo --version | head -1)" "M1"
else
  fail "cargo" "M1" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi

if command -v rustc >/dev/null 2>&1; then
  row "✓" "rustc" "$(run_timeout 10 rustc --version | head -1)" "M1"
else
  fail "rustc" "M1" "rustup ile birlikte gelir (yukarı bakın)"
fi

# --- cargo-deny (M1, CI) ---
if command -v cargo-deny >/dev/null 2>&1; then
  row "✓" "cargo-deny" "$(run_timeout 10 cargo-deny --version | head -1)" "M1 (CI)"
else
  fail "cargo-deny" "M1 (CI)" "cargo install cargo-deny"
fi

# --- Xcode (M3) ---
XCODE_PATH="$(xcode-select -p 2>/dev/null)"
if [ -z "$XCODE_PATH" ]; then
  fail "Xcode" "M3" "App Store'dan Xcode kurun, sonra: sudo xcode-select -s /Applications/Xcode.app"
elif echo "$XCODE_PATH" | grep -q "CommandLineTools"; then
  MISSING_REQUIRED=$((MISSING_REQUIRED + 1))
  row "✗" "Xcode" "yalnız CommandLineTools — TAM XCODE DEĞİL" "M3"
  printf '       aktif yol: %s\n' "$XCODE_PATH"
  printf '       kurulum: App Store'"'"'dan Xcode, sonra: sudo xcode-select -s /Applications/Xcode.app\n'
elif command -v xcodebuild >/dev/null 2>&1; then
  row "✓" "Xcode" "$(run_timeout 20 xcodebuild -version | head -1)" "M3"
else
  fail "Xcode" "M3" "sudo xcode-select -s /Applications/Xcode.app"
fi

# --- Swift (M3) ---
if command -v swift >/dev/null 2>&1; then
  row "✓" "swift" "$(run_timeout 20 swift --version 2>&1 | grep -o 'Apple Swift version [0-9.]*' | head -1)" "M3"
else
  fail "swift" "M3" "Xcode veya Command Line Tools ile gelir"
fi

# --- libmpv (M3) ---
# mpv CLI'ı çalıştırmak takılabildiği için önce kütüphaneyi arıyoruz.
MPV_FOUND=""
if command -v pkg-config >/dev/null 2>&1; then
  MPV_VER="$(run_timeout 10 pkg-config --modversion mpv)"
  [ -n "$MPV_VER" ] && MPV_FOUND="libmpv $MPV_VER (pkg-config)"
fi
if [ -z "$MPV_FOUND" ]; then
  for p in /opt/homebrew/lib/libmpv.dylib /usr/local/lib/libmpv.dylib /opt/homebrew/lib/libmpv.2.dylib /usr/local/lib/libmpv.2.dylib; do
    [ -e "$p" ] && { MPV_FOUND="$p"; break; }
  done
fi
if [ -n "$MPV_FOUND" ]; then
  row "✓" "libmpv" "$MPV_FOUND" "M3"
else
  fail "libmpv" "M3" "brew install mpv   (libmpv dylib'i ile birlikte gelir)"
fi

# --- JDK + Gradle (M10, opsiyonel) ---
if run_timeout 15 java -version >/dev/null 2>&1; then
  row "✓" "java" "$(run_timeout 15 java -version 2>&1 | head -1)" "M10"
else
  warn "java" "M10" "brew install --cask temurin"
fi

if command -v gradle >/dev/null 2>&1; then
  row "✓" "gradle" "$(run_timeout 20 gradle --version 2>/dev/null | grep -i '^Gradle' | head -1)" "M10"
else
  warn "gradle" "M10" "brew install gradle   (veya Gradle wrapper kullanın)"
fi

echo
echo "Not: Bu script hiçbir şey kurmaz. M0 hiçbir araç gerektirmez;"
echo "     M1 için Rust, M3 için tam Xcode + libmpv gerekir."
echo

if [ "$MISSING_REQUIRED" -gt 0 ]; then
  echo "SONUÇ: $MISSING_REQUIRED zorunlu araç eksik, $MISSING_OPTIONAL opsiyonel eksik." >&2
  exit 1
fi
echo "SONUÇ: tüm zorunlu araçlar mevcut ($MISSING_OPTIONAL opsiyonel eksik)."
