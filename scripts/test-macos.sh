#!/usr/bin/env bash
# macOS platform testlerini çalıştırır (libmpv adapter'ı — NEN-022).
#
#   bash scripts/test-macos.sh
#
# test-apple.sh ile aynı swift-testing bayrak mantığını kullanır; farkı
# paketin platforms/macos olması ve libmpv gerektirmesi.
#
# libmpv'ye DİNAMİK linklenir (ADR-0012 Karar 3): geliştirmede Homebrew'un
# dylib'i kullanılır, .app içine gömme NEN-043'ün işidir.

set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG="$ROOT/platforms/macos"

command -v swift >/dev/null 2>&1 || {
  echo "HATA: swift bulunamadı. Bkz. bash scripts/doctor.sh M3" >&2; exit 1; }

if ! pkg-config --exists mpv 2>/dev/null; then
  echo "HATA: libmpv bulunamadı (pkg-config --exists mpv)." >&2
  echo "      Kurulum: brew install mpv" >&2
  exit 1
fi

echo "▶ libmpv $(pkg-config --modversion mpv)"

# Gömülü altyazı metin çıkarımı (NEN-044, ADR-0045) — libmpv'nin zaten
# getirdiği ffmpeg formülü.
if ! pkg-config --exists libavformat libavcodec libavutil 2>/dev/null; then
  echo "HATA: libavformat/libavcodec/libavutil bulunamadı." >&2
  echo "      Kurulum: brew install ffmpeg" >&2
  exit 1
fi

echo "▶ libavformat $(pkg-config --modversion libavformat) · libavcodec $(pkg-config --modversion libavcodec) · libavutil $(pkg-config --modversion libavutil)"

echo "▶ binding üretimi"
bash "$ROOT/scripts/build-apple.sh"

DEVDIR="$(xcode-select -p 2>/dev/null || echo /Library/Developer/CommandLineTools)"

FLAGS=""
add_plugin_path() { [ -d "$1" ] && FLAGS="$FLAGS -Xswiftc -plugin-path -Xswiftc $1"; return 0; }
add_rpath()       { [ -d "$1" ] && FLAGS="$FLAGS -Xlinker -rpath -Xlinker $1";      return 0; }

# swift-testing macro plugin (@Test / @Suite / #expect makroları)
add_plugin_path "$DEVDIR/usr/lib/swift/host/plugins/testing"
# Testing.framework ve bağımlılığı lib_TestingInterop.dylib
add_rpath "$DEVDIR/Library/Developer/Frameworks"
add_rpath "$DEVDIR/Library/Developer/usr/lib"

echo
echo "▶ swift test --package-path platforms/macos${FLAGS:+  (ek bayraklar: tam Xcode yok)}"
# shellcheck disable=SC2086  # FLAGS bilinçli olarak kelimelere ayrılıyor
exec swift test --package-path "$PKG" $FLAGS
