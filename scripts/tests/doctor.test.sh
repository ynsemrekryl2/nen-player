#!/usr/bin/env bash
# scripts/doctor.sh — deterministic shell testleri.
#
# Gerçek makinenin toolchain durumundan BAĞIMSIZ çalışır: PATH'in başına sahte
# executable'lar (shim) konur ve PATH yalnız /usr/bin:/bin ile sınırlanır.
# libmpv dosya varlığı NEN_DOCTOR_MPV_PATHS ile, Android SDK ANDROID_HOME ile
# kontrol edilir.

set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DOCTOR="$ROOT/scripts/doctor.sh"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

SENTINEL="$TMP/INSTALL_WAS_RUN"
FAILURES=0

pass() { printf '    ok   %s\n' "$1"; }
fail() { printf '    FAIL %s\n' "$1" >&2; FAILURES=$((FAILURES + 1)); }

# --------------------------------------------------------------------- shim'ler

mkshim() { # <ad> <gövde>
  printf '#!/bin/sh\n%s\n' "$2" > "$BIN/$1"
  chmod +x "$BIN/$1"
}

# Kurulum komutu çalıştırılırsa sentinel yazan tuzaklar.
mktraps() {
  for c in brew curl rustup sdkmanager; do
    mkshim "$c" "echo TRAP-$c >> \"$SENTINEL\"; exit 1"
  done
}

# Senaryo kurar: $BIN sıfırlanır, istenen araçlar shim'lenir.
# Kullanım: setup_scenario <rust|norust> <fullxcode|clt> <jdk|nojdk> <mpv|nompv>
setup_scenario() {
  BIN="$TMP/bin"; rm -rf "$BIN"; mkdir -p "$BIN"
  export HOME="$TMP/home"; rm -rf "$HOME"; mkdir -p "$HOME"
  mktraps

  # cargo shim: "install" alt komutu çağrılırsa bu bir KURULUM denemesidir.
  if [ "$1" = "rust" ]; then
    mkshim cargo 'if [ "${1:-}" = "install" ]; then echo TRAP-cargo-install >> "'"$SENTINEL"'"; exit 1; fi; echo "cargo 1.90.0 (shim)"'
    mkshim rustc 'echo "rustc 1.90.0 (shim)"'
    mkshim cargo-deny 'echo "cargo-deny 0.16.0 (shim)"'
  fi

  if [ "$2" = "fullxcode" ]; then
    mkshim xcode-select 'echo /Applications/Xcode.app/Contents/Developer'
    mkshim xcodebuild 'echo "Xcode 26.0 (shim)"'
  else
    mkshim xcode-select 'echo /Library/Developer/CommandLineTools'
  fi
  mkshim swift 'echo "Apple Swift version 6.4 (shim)" >&2; echo "Apple Swift version 6.4"'

  [ "$3" = "jdk" ] && mkshim java 'echo "openjdk version \"21.0.1\" (shim)" >&2'

  # pkg-config her senaryoda mpv bilmiyor; libmpv varlığı dylib yoluyla belirlenir.
  mkshim pkg-config 'exit 1'

  if [ "$4" = "mpv" ]; then
    : > "$TMP/libmpv.dylib"
    export NEN_DOCTOR_MPV_PATHS="$TMP/libmpv.dylib"
  else
    export NEN_DOCTOR_MPV_PATHS="$TMP/yok/libmpv.dylib"
  fi

  export PATH="$BIN:/usr/bin:/bin"
}

# doctor'ı çalıştırır; çıktı $OUT'a, çıkış kodu $RC'ye yazılır.
# NOT: komut ikamesi ($(...)) KULLANILMAZ — subshell $OUT'u kaybettirir.
OUT=""; RC=0
run_doctor() {
  OUT="$(bash "$DOCTOR" "$@" 2>&1)"
  RC=$?
}

expect_exit() { # <beklenen> <açıklama>   ($RC'yi denetler)
  if [ "$1" = "$RC" ]; then pass "$2 → exit $RC"
  else fail "$2 → beklenen exit $1, gelen $RC"; printf '%s\n' "$OUT" | sed 's/^/         | /' >&2; fi
}

expect_contains() { # <metin> <açıklama>
  case "$OUT" in
    *"$1"*) pass "$2" ;;
    *) fail "$2 — çıktıda '$1' yok"; printf '%s\n' "$OUT" | sed 's/^/         | /' >&2 ;;
  esac
}

expect_not_contains() { # <metin> <açıklama>
  case "$OUT" in
    *"$1"*) fail "$2 — çıktıda beklenmeyen '$1' var"; printf '%s\n' "$OUT" | sed 's/^/         | /' >&2 ;;
    *) pass "$2" ;;
  esac
}

# ------------------------------------------------------------------- senaryolar

echo "  S1: hepsi var"
setup_scenario rust fullxcode jdk mpv
export ANDROID_HOME="$TMP/android-sdk"; mkdir -p "$ANDROID_HOME"
run_doctor; expect_exit 0 "S1 parametresiz"
run_doctor M1; expect_exit 0 "S1 M1"
run_doctor M3; expect_exit 0 "S1 M3"
run_doctor M10; expect_exit 0 "S1 M10"

echo "  S2: Rust yok"
setup_scenario norust fullxcode jdk mpv
run_doctor; expect_exit 0 "S2 parametresiz (bilgilendirici)"
run_doctor M1; expect_exit 1 "S2 M1 blocker"
expect_contains "BLOCKER" "S2 M1 çıktısı BLOCKER bölümü içeriyor"
run_doctor M3; expect_exit 1 "S2 M3 blocker (kümülatif)"

echo "  S3: yalnız Xcode/libmpv yok"
setup_scenario rust clt jdk nompv
run_doctor; expect_exit 0 "S3 parametresiz"
run_doctor M1; expect_exit 0 "S3 M1 — Xcode/libmpv M1 blocker'ı DEĞİL"
expect_not_contains "BLOCKER" "S3 M1 çıktısında BLOCKER bölümü yok"
expect_contains "M1 için gerekmeyenler" "S3 M1 Xcode/libmpv 'gerekmeyenler' altında"
run_doctor M3; expect_exit 1 "S3 M3 blocker"
expect_contains "TAM XCODE DEĞİL" "S3 M3 CommandLineTools'u tam Xcode saymıyor"

echo "  S4: JDK yok (Rust var)"
setup_scenario rust fullxcode nojdk mpv
unset ANDROID_HOME
# GitHub Actions'ın macOS runner image'ında /usr/bin/java gerçekten çalışan
# bir JDK'dır (CLT-only bir Mac'teki boş stub'ın aksine) — S7'nin swift için
# yaptığı gibi, java hariç bir gölge /usr/bin bağlanıp PATH ona yönlendirilir.
JAVA_SHADOW="$TMP/usr-bin-no-java"; rm -rf "$JAVA_SHADOW"; mkdir -p "$JAVA_SHADOW"
for f in /usr/bin/*; do
  b="$(basename "$f")"
  [ "$b" = "java" ] && continue
  ln -s "$f" "$JAVA_SHADOW/$b" 2>/dev/null
done
export PATH="$BIN:$JAVA_SHADOW:/bin"
run_doctor; expect_exit 0 "S4 parametresiz"
run_doctor M1; expect_exit 0 "S4 M1 — JDK 'soon', blocker değil"
expect_contains "YAKINDA GEREKLİ" "S4 M1 çıktısı 'yakında gerekli' bölümü içeriyor"
run_doctor M10; expect_exit 1 "S4 M10 — JDK blocker"

echo "  S5: hatalı argüman"
setup_scenario rust fullxcode jdk mpv
run_doctor M99; expect_exit 1 "S5 bilinmeyen milestone"
expect_contains "bilinmeyen milestone" "S5 anlaşılır hata mesajı"
run_doctor M1 M3; expect_exit 1 "S5 fazla argüman"

echo "  S6: büyük/küçük harf duyarsızlık"
setup_scenario rust fullxcode jdk mpv
run_doctor m1; expect_exit 1 "S6 'm1' = 'M1' — INTENTIONAL DoD PROOF, expected 0"

echo "  S7: swift yok"
setup_scenario rust fullxcode jdk mpv
rm -f "$BIN/swift"
# CLT kurulu bir Mac'te /usr/bin/swift gerçek bir binary'dir — PATH'te
# /usr/bin durduğu sürece "command -v swift" onu bulur. Swift'in gerçekten
# yokmuş gibi görünmesi için /usr/bin'in geri kalanını (grep/head/perl/...)
# swift HARİÇ bir gölge dizine sembolik bağlayıp PATH'i ona yönlendiriyoruz.
SHADOW="$TMP/usr-bin-no-swift"; rm -rf "$SHADOW"; mkdir -p "$SHADOW"
for f in /usr/bin/*; do
  b="$(basename "$f")"
  [ "$b" = "swift" ] && continue
  ln -s "$f" "$SHADOW/$b" 2>/dev/null
done
export PATH="$BIN:$SHADOW:/bin"
run_doctor; expect_exit 0 "S7 parametresiz (bilgilendirici)"
run_doctor M1; expect_exit 1 "S7 M1 blocker (swift yok)"
expect_contains "BLOCKER" "S7 M1 çıktısı BLOCKER bölümü içeriyor"
run_doctor M3; expect_exit 1 "S7 M3 blocker (kümülatif)"

echo "  S8: hiçbir kurulum komutu çalıştırılmadı"
if [ -e "$SENTINEL" ]; then
  fail "KURULUM DENEMESİ TESPİT EDİLDİ:"; cat "$SENTINEL" >&2
else
  pass "brew/curl/rustup/sdkmanager/cargo-install hiç çağrılmadı"
fi

[ "$FAILURES" -eq 0 ] || { echo "  $FAILURES doğrulama başarısız" >&2; exit 1; }
exit 0
