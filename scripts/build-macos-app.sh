#!/usr/bin/env bash
# NEN-024 development app bundle. libmpv remains a Homebrew dynamic dependency
# and the app is signed ad-hoc here — copying dependencies into the bundle,
# rewriting their load paths, and Developer ID / notarization are
# scripts/bundle-macos.sh's job (NEN-043).

set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG="$ROOT/platforms/macos"

command -v swift >/dev/null 2>&1 || {
  echo "HATA: swift bulunamadı. Bkz. bash scripts/doctor.sh M3" >&2; exit 1; }

pkg-config --exists mpv 2>/dev/null || {
  echo "HATA: libmpv bulunamadı (pkg-config --exists mpv)." >&2; exit 1; }

bash "$ROOT/scripts/build-apple.sh"
swift build --package-path "$PKG" --product NenPlayer

BIN_DIR="$(swift build --package-path "$PKG" --show-bin-path)"
APP="$PKG/.build/NenPlayer.app"
CONTENTS="$APP/Contents"

mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources"
install -m 755 "$BIN_DIR/NenPlayer" "$CONTENTS/MacOS/NenPlayer"
install -m 644 "$PKG/Resources/Info.plist" "$CONTENTS/Info.plist"

/usr/bin/codesign --force --sign - \
  --entitlements "$PKG/NenPlayer.entitlements" \
  "$APP"

echo "Uygulama: $APP"
echo "Çalıştır: open '$APP'"
