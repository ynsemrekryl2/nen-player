#!/usr/bin/env bash
# Apple tarafındaki Swift testlerini çalıştırır (binding'i önce üretir).
#
#   bash scripts/test-apple.sh
#
# Neden düz `swift test` değil: bu depo swift-testing kullanıyor (XCTest
# DEĞİL — CommandLineTools XCTest.framework'ü getirmiyor). Tam Xcode kurulu
# olmayan bir makinede swift-testing'in macro plugin'i ve runtime kütüphaneleri
# SwiftPM'in varsayılan arama yollarında değil; aşağıda VARSA eklenirler.
# Tam Xcode kuruluysa hiçbir ek bayrak eklenmez ve komut düz `swift test`e eşittir.

set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG="$ROOT/platforms/apple-shared"

command -v swift >/dev/null 2>&1 || {
  echo "HATA: swift bulunamadı. Bkz. bash scripts/doctor.sh" >&2; exit 1; }

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
echo "▶ swift test${FLAGS:+  (ek bayraklar: tam Xcode yok)}"
# shellcheck disable=SC2086  # FLAGS bilinçli olarak kelimelere ayrılıyor
exec swift test --package-path "$PKG" $FLAGS
