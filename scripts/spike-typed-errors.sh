#!/usr/bin/env bash
# NEN-010 ölçümü: typed error mapping FFI sınırını nasıl geçiyor.
#
#   bash scripts/spike-typed-errors.sh                  # her şey
#   bash scripts/spike-typed-errors.sh --rust-only       # yalnız cargo test
#   bash scripts/spike-typed-errors.sh --swift-only      # bindgen + swift test
#   bash scripts/spike-typed-errors.sh --negative-only   # yalnız DoD #3 kanıtı
#
# Bu script ÜRÜN build'ine dokunmaz. Ölçülen crate core/spikes/ altındadır ve
# kendi FFI kapısını açar — buna izin veren karar ADR-0028'dir.
#
# swift-testing kullanır (XCTest DEĞİL): bu makinede tam Xcode yoksa
# CommandLineTools'ta eksik olan plugin-path / rpath bayrakları aşağıda VARSA
# eklenir — aynı desen scripts/spike-async.sh'te.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORE="$ROOT/core"
PKG="$ROOT/core/spikes/spike-typed-errors/apple-harness"
OUT="$PKG/generated"

MODE="all"   # all | rust-only | swift-only | negative-only

while [ $# -gt 0 ]; do
  case "$1" in
    --rust-only)      MODE="rust-only"; shift ;;
    --swift-only)     MODE="swift-only"; shift ;;
    --negative-only)  MODE="negative-only"; shift ;;
    *) echo "Bilinmeyen argüman: $1 (--rust-only | --swift-only | --negative-only)" >&2; exit 1 ;;
  esac
done

command -v cargo >/dev/null 2>&1 || {
  echo "HATA: cargo bulunamadı. Kurulum için: bash scripts/doctor.sh M1" >&2; exit 1; }
command -v swift >/dev/null 2>&1 || {
  echo "HATA: swift bulunamadı. Bkz. bash scripts/doctor.sh" >&2; exit 1; }

DEVDIR="$(xcode-select -p 2>/dev/null || echo /Library/Developer/CommandLineTools)"
FLAGS=""
add_plugin_path() { [ -d "$1" ] && FLAGS="$FLAGS -Xswiftc -plugin-path -Xswiftc $1"; return 0; }
add_rpath()       { [ -d "$1" ] && FLAGS="$FLAGS -Xlinker -rpath -Xlinker $1";      return 0; }
add_plugin_path "$DEVDIR/usr/lib/swift/host/plugins/testing"
add_rpath "$DEVDIR/Library/Developer/Frameworks"
add_rpath "$DEVDIR/Library/Developer/usr/lib"

run_rust_tests() {
  echo "▶ cargo test -p spike-typed-errors"
  cargo test --manifest-path "$CORE/Cargo.toml" -p spike-typed-errors
}

build_and_test_swift() {
  echo "▶ cargo build -p spike-typed-errors (release)"
  cargo build --manifest-path "$CORE/Cargo.toml" -p spike-typed-errors --release

  local target_dir="$CORE/target/release"
  local dylib="$target_dir/libspike_typed_errors.dylib"
  local staticlib="$target_dir/libspike_typed_errors.a"
  for f in "$dylib" "$staticlib"; do
    [ -f "$f" ] || { echo "HATA: beklenen çıktı yok: $f" >&2; exit 1; }
  done

  echo "▶ spike-typed-errors-uniffi-bindgen generate --language swift"
  rm -rf "$OUT"
  mkdir -p "$OUT/spike_typed_errorsFFI" "$OUT/SpikeCore" "$OUT/lib"

  local raw="$OUT/.raw"
  ( cd "$CORE" && cargo run -q -p spike-typed-errors --bin spike-typed-errors-uniffi-bindgen --release -- \
      generate --language swift --no-format --out-dir "$raw" "$dylib" )

  mv "$raw/spike_typed_errorsFFI.h"         "$OUT/spike_typed_errorsFFI/spike_typed_errorsFFI.h"
  mv "$raw/spike_typed_errorsFFI.modulemap" "$OUT/spike_typed_errorsFFI/module.modulemap"
  mv "$raw/spike_typed_errors.swift"        "$OUT/SpikeCore/spike_typed_errors.swift"
  cp "$staticlib"                           "$OUT/lib/libspike_typed_errors.a"
  rmdir "$raw"

  cat > "$OUT/README.md" <<'GEN'
# ÜRETİLEN DİZİN — elle düzenlemeyin, commit etmeyin

`bash scripts/spike-typed-errors.sh` üretir. `.gitignore` → `/core/spikes/**/generated/`.
GEN

  echo "▶ swift test${FLAGS:+  (ek bayraklar: tam Xcode yok)}"
  # shellcheck disable=SC2086  # FLAGS bilinçli olarak kelimelere ayrılıyor
  swift test --package-path "$PKG" -c release $FLAGS

  echo "▶ swift build + ölçüm (varyant sayısı · eşleme maliyeti)"
  swift build --package-path "$PKG" -c release $FLAGS >/dev/null
  local bin
  bin="$(swift build --package-path "$PKG" -c release --show-bin-path)/SpikeTypedErrors"
  "$bin"
}

# DoD #3 — "bilinmeyen varyant sessizce yutulmuyor". Gerçek crate'e hiç
# dokunmadan, tamamen ayrı bir scratch cargo+swift projesinde AYNI
# uniffi::Error makine mekanizmasını kullanarak mekanik olarak kanıtlıyoruz:
# (1) 3 varyantlı bir hata + üstünde exhaustive (default'suz) bir Swift
#     switch → derlenir.
# (2) AYNI switch koduna dokunmadan Rust tarafına 4. bir varyant eklenir →
#     Swift artık exhaustive değildir, `swift build` KIRILIR.
# Böylece "yeni bir varyant eklenirse Swift tarafı sessizce onu yutar mı"
# sorusunun cevabı çalışma zamanı davranışı değil, derleme zamanı garantisi
# olarak kanıtlanmış olur. scripts/tests/doctor.test.sh'ın shadow-PATH
# tekniğiyle aynı ruh: geçici durum, mekanik kanıt, iz bırakmadan temizlik.
run_negative_check() {
  echo "▶ DoD #3 negatif kontrol: bilinmeyen varyant Swift derlemesini kırıyor mu"

  local scratch
  scratch="$(mktemp -d)"
  trap 'rm -rf "$scratch"' RETURN

  mkdir -p "$scratch/rustcrate/src" "$scratch/swiftpkg/Sources/NegCheck" "$scratch/swiftpkg/Tests/NegCheckTests"

  cat > "$scratch/rustcrate/Cargo.toml" <<'EOF'
[package]
name = "negcheck"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
crate-type = ["staticlib", "cdylib"]
name = "negcheck"

[[bin]]
name = "negcheck-uniffi-bindgen"
path = "src/bin/uniffi-bindgen.rs"

[dependencies]
uniffi = { version = "0.32.0", features = ["cli"] }
EOF

  mkdir -p "$scratch/rustcrate/src/bin"
  cat > "$scratch/rustcrate/src/bin/uniffi-bindgen.rs" <<'EOF'
fn main() { uniffi::uniffi_bindgen_main() }
EOF

  write_lib_rs() { # <variant_count>
    if [ "$1" -eq 3 ]; then
      cat > "$scratch/rustcrate/src/lib.rs" <<'EOF'
uniffi::setup_scaffolding!();
#[derive(Debug, uniffi::Error)]
pub enum NegError { A, B, C }
impl std::fmt::Display for NegError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for NegError {}
#[uniffi::export]
pub fn trigger(which: u8) -> Result<(), NegError> {
    Err(match which { 0 => NegError::A, 1 => NegError::B, _ => NegError::C })
}
EOF
    else
      cat > "$scratch/rustcrate/src/lib.rs" <<'EOF'
uniffi::setup_scaffolding!();
#[derive(Debug, uniffi::Error)]
pub enum NegError { A, B, C, D }
impl std::fmt::Display for NegError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for NegError {}
#[uniffi::export]
pub fn trigger(which: u8) -> Result<(), NegError> {
    Err(match which { 0 => NegError::A, 1 => NegError::B, 2 => NegError::C, _ => NegError::D })
}
EOF
    fi
  }

  cat > "$scratch/swiftpkg/Package.swift" <<EOF
// swift-tools-version: 6.0
import Foundation
import PackageDescription
let dir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let package = Package(
    name: "NegCheck",
    platforms: [.macOS(.v14)],
    targets: [
        .systemLibrary(name: "negcheckFFI", path: "generated/negcheckFFI"),
        .target(name: "SpikeCore", dependencies: ["negcheckFFI"], path: "generated/SpikeCore",
                linkerSettings: [.unsafeFlags(["-L\(dir.appendingPathComponent("generated/lib").path)", "-lnegcheck"])]),
        .testTarget(name: "NegCheckTests", dependencies: ["SpikeCore"], path: "Tests/NegCheckTests")
    ]
)
EOF

  # Exhaustive (default'suz) switch — 3 varyantlı enum'a göre yazıldı ve
  # 4. varyant eklendiğinde DEĞİŞTİRİLMEYECEK: kanıtlanan tam olarak bu.
  # Not: uniffi::Error varyant adları PascalCase (spike-typed-errors ile aynı
  # gözlem — bkz. TypedErrorTests.swift).
  cat > "$scratch/swiftpkg/Tests/NegCheckTests/ExhaustiveSwitchTests.swift" <<'EOF'
import Testing
import SpikeCore

@Test func exhaustiveSwitchOverThreeVariants() throws {
    func classify(_ e: NegError) -> String {
        switch e {
        case .A: return "a"
        case .B: return "b"
        case .C: return "c"
        }
    }
    do { try trigger(which: 0) } catch let e as NegError { #expect(classify(e) == "a") }
}
EOF

  generate_and_build() { # <profile: dev|release swift: debug|release>
    local dylib_ext="dylib"
    rm -rf "$scratch/rustcrate/target"
    ( cd "$scratch/rustcrate" && cargo build --release ) >"$scratch/cargo-build.log" 2>&1
    local tgt="$scratch/rustcrate/target/release"
    rm -rf "$scratch/swiftpkg/generated"
    mkdir -p "$scratch/swiftpkg/generated/negcheckFFI" "$scratch/swiftpkg/generated/SpikeCore" "$scratch/swiftpkg/generated/lib"
    ( cd "$scratch/rustcrate" && cargo run -q --bin negcheck-uniffi-bindgen --release -- \
        generate --language swift --no-format --out-dir "$scratch/raw" "$tgt/libnegcheck.$dylib_ext" )
    mv "$scratch/raw/negcheckFFI.h"         "$scratch/swiftpkg/generated/negcheckFFI/negcheckFFI.h"
    mv "$scratch/raw/negcheckFFI.modulemap" "$scratch/swiftpkg/generated/negcheckFFI/module.modulemap"
    mv "$scratch/raw/negcheck.swift"        "$scratch/swiftpkg/generated/SpikeCore/negcheck.swift"
    cp "$tgt/libnegcheck.a"                 "$scratch/swiftpkg/generated/lib/libnegcheck.a"
    rmdir "$scratch/raw"
  }

  write_lib_rs 3
  generate_and_build
  echo "  1/2  3 varyant + exhaustive switch → swift build (beklenen: başarı)"
  # shellcheck disable=SC2086
  if ! swift build --package-path "$scratch/swiftpkg" --build-tests -c release $FLAGS >"$scratch/swift-baseline.log" 2>&1; then
    echo "HATA: baseline (3 varyant) derlemesi beklenmedik şekilde kırıldı:" >&2
    tail -40 "$scratch/swift-baseline.log" >&2
    exit 1
  fi
  echo "       ok — baseline yeşil"

  write_lib_rs 4
  generate_and_build
  echo "  2/2  4. varyant eklendi, switch AYNI kaldı → swift build (beklenen: kırılma)"
  set +e
  # shellcheck disable=SC2086
  swift build --package-path "$scratch/swiftpkg" --build-tests -c release $FLAGS >"$scratch/swift-broken.log" 2>&1
  BUILD_EXIT=$?
  set -e
  if [ "$BUILD_EXIT" -eq 0 ]; then
    echo "HATA: 4. varyant eklendiğinde swift build YİNE BAŞARILI oldu — exhaustiveness kanıtlanamadı." >&2
    exit 1
  fi
  if ! grep -qi "exhaustive" "$scratch/swift-broken.log"; then
    echo "UYARI: build kırıldı ama 'exhaustive' ifadesi bulunamadı — çıktı:" >&2
    tail -40 "$scratch/swift-broken.log" >&2
    exit 1
  fi
  echo "       ok — swift build çıkış $BUILD_EXIT, derleyici exhaustiveness ihlalini bildirdi:"
  grep -i "exhaustive" "$scratch/swift-broken.log" | head -3 | sed 's/^/         /'
  echo "▶ DoD #3 kanıtlandı: yeni bir varyant, switch güncellenmeden Swift derlemesini kırıyor — sessizce yutulmuyor."
}

case "$MODE" in
  rust-only)     run_rust_tests ;;
  swift-only)    build_and_test_swift ;;
  negative-only) run_negative_check ;;
  all)
    run_rust_tests
    build_and_test_swift
    run_negative_check
    ;;
esac
