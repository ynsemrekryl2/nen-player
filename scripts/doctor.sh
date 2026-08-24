#!/usr/bin/env bash
# Toolchain ön koşul kontrolü. HİÇBİR ŞEY KURMAZ — yalnız raporlar.
#
#   bash scripts/doctor.sh        Tüm durum. DAİMA çıkış 0 (bilgilendirici).
#   bash scripts/doctor.sh M1     Yalnız M1 kapısı. Blocker eksikse çıkış 1.
#   bash scripts/doctor.sh M3
#   bash scripts/doctor.sh M10
#
# Gereklilik seviyeleri:
#   blocker  bu milestone'a başlanamaz          → milestone modunda çıkış 1
#   soon     milestone İÇİNDE bir task'tan önce → raporlanır, çıkış kodunu etkilemez
#   info     bu milestone için gerekmez
#
# Milestone gereksinimleri KÜMÜLATİF: M3 core araçlarını da ister.

set -u

MILESTONES="M1 M3 M10"

# libmpv dylib arama yolları — testlerde geçersiz kılınabilsin diye env.
: "${NEN_DOCTOR_MPV_PATHS:=/opt/homebrew/lib/libmpv.dylib:/usr/local/lib/libmpv.dylib:/opt/homebrew/lib/libmpv.2.dylib:/usr/local/lib/libmpv.2.dylib}"

# Takılabilen komutlar için zaman sınırlı çalıştırma (macOS'ta GNU timeout yok).
run_timeout() { s="$1"; shift; perl -e 'alarm shift; exec @ARGV' "$s" "$@" 2>/dev/null; }

# ---------------------------------------------------------------- araç kayıtları

# Sıra bu listeye göre. Her araç için detect_<id> ve install_<id> tanımlı olmalı.
TOOLS="cargo rustc cargo_deny jdk gradle xcode swift libmpv android_sdk"

label() {
  case "$1" in
    cargo)       echo "cargo" ;;
    rustc)       echo "rustc" ;;
    cargo_deny)  echo "cargo-deny" ;;
    jdk)         echo "JDK" ;;
    gradle)      echo "Gradle" ;;
    xcode)       echo "Xcode" ;;
    swift)       echo "swift" ;;
    libmpv)      echo "libmpv" ;;
    android_sdk) echo "Android SDK" ;;
  esac
}

install_hint() {
  case "$1" in
    cargo)       echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" ;;
    rustc)       echo "rustup ile birlikte gelir (cargo satırına bakın)" ;;
    cargo_deny)  echo "cargo install cargo-deny" ;;
    jdk)         echo "brew install --cask temurin" ;;
    gradle)      echo "Gradle Wrapper tercih edilir; sistem kurulumu gerekmez" ;;
    xcode)       echo "App Store'dan Xcode, sonra: sudo xcode-select -s /Applications/Xcode.app" ;;
    swift)       echo "Xcode veya Command Line Tools ile gelir" ;;
    libmpv)      echo "brew install mpv   (libmpv dylib'i ile birlikte gelir)" ;;
    android_sdk) echo "Android Studio kurun veya ANDROID_HOME ayarlayın" ;;
  esac
}

# Bir aracın belirli bir milestone'daki gerekliliği.
# Kümülatif: M3 ve M10 de core araçlarını ister.
requirement() { # <tool> <milestone> -> blocker|soon|info
  case "$1:$2" in
    cargo:*|rustc:*)        echo blocker ;;
    cargo_deny:M1)          echo soon ;;
    cargo_deny:*)           echo info ;;
    jdk:M1)                 echo soon ;;
    jdk:M10)                echo blocker ;;
    jdk:*)                  echo info ;;
    gradle:*)               echo info ;;
    swift:M1|swift:M3)      echo blocker ;;
    xcode:M3|libmpv:M3)     echo blocker ;;
    xcode:*|swift:*|libmpv:*)    echo info ;;
    android_sdk:M10)        echo blocker ;;
    android_sdk:*)          echo info ;;
    *)                      echo info ;;
  esac
}

# Bir aracın hangi milestone'da neden gerektiğini anlatan kısa etiket.
need_note() {
  case "$1" in
    cargo|rustc) echo "M1" ;;
    cargo_deny)  echo "M1 · NEN-005 öncesi" ;;
    jdk)         echo "M1 · NEN-011 öncesi · M10" ;;
    gradle)      echo "Wrapper tercih edilir — blocker değil" ;;
    xcode)       echo "M3" ;;
    swift)       echo "M1" ;;
    libmpv)      echo "M3 · development/binding" ;;
    android_sdk) echo "M10" ;;
  esac
}

# ------------------------------------------------------------------- tespitler
# Her detect_*: bulunursa 0 döner ve tek satır detay basar; yoksa 1 döner.

detect_cargo() {
  command -v cargo >/dev/null 2>&1 || return 1
  run_timeout 10 cargo --version | head -1
}

detect_rustc() {
  command -v rustc >/dev/null 2>&1 || return 1
  run_timeout 10 rustc --version | head -1
}

detect_cargo_deny() {
  command -v cargo-deny >/dev/null 2>&1 || return 1
  run_timeout 10 cargo-deny --version | head -1
}

detect_jdk() {
  command -v java >/dev/null 2>&1 || return 1
  run_timeout 15 java -version >/dev/null 2>&1 || return 1
  run_timeout 15 java -version 2>&1 | head -1
}

detect_gradle() {
  command -v gradle >/dev/null 2>&1 || return 1
  run_timeout 20 gradle --version 2>/dev/null | grep -i '^Gradle' | head -1
}

# Tam Xcode mu, yoksa yalnız Command Line Tools mu?
# rc 0 = tam Xcode · rc 2 = var ama yanlış tür (detay basılır) · rc 1 = yok
detect_xcode() {
  p="$(xcode-select -p 2>/dev/null)"
  [ -n "$p" ] || return 1
  case "$p" in
    *CommandLineTools*)
      echo "yalnız CommandLineTools — TAM XCODE DEĞİL ($p)"
      return 2 ;;
  esac
  command -v xcodebuild >/dev/null 2>&1 || return 1
  run_timeout 20 xcodebuild -version | head -1
}

detect_swift() {
  command -v swift >/dev/null 2>&1 || return 1
  v="$(run_timeout 20 swift --version 2>&1 | grep -o 'Apple Swift version [0-9.]*' | head -1)"
  [ -n "$v" ] && echo "$v" || echo "swift (sürüm okunamadı)"
}

# mpv CLI'ı çalıştırmak takılabildiği için ÖNCE kütüphaneyi arıyoruz.
detect_libmpv() {
  if command -v pkg-config >/dev/null 2>&1; then
    v="$(run_timeout 10 pkg-config --modversion mpv)"
    [ -n "$v" ] && { echo "libmpv $v (pkg-config)"; return 0; }
  fi
  old_ifs="$IFS"; IFS=':'
  for p in $NEN_DOCTOR_MPV_PATHS; do
    [ -n "$p" ] && [ -e "$p" ] && { IFS="$old_ifs"; echo "$p"; return 0; }
  done
  IFS="$old_ifs"
  return 1
}

# sdkmanager ÇALIŞTIRILMAZ — yalnız env ve bilinen dizin kontrolü.
detect_android_sdk() {
  for d in "${ANDROID_HOME:-}" "${ANDROID_SDK_ROOT:-}" "$HOME/Library/Android/sdk"; do
    [ -n "$d" ] && [ -d "$d" ] && { echo "$d"; return 0; }
  done
  return 1
}

# ------------------------------------------------------------------- yazdırma

# NOT: işaretlerin hepsi 3 baytlık UTF-8 olmalı — printf %-3s bayt sayar.
row() { printf '  %-3s %-14s %s\n' "$1" "$2" "$3"; }

hint_line() { printf '        kurulum: %s\n' "$1"; }

header() {
  echo "Nen Player — toolchain doctor"
  os="$(sw_vers -productName 2>/dev/null) $(sw_vers -productVersion 2>/dev/null)"
  [ -n "$(echo "$os" | tr -d ' ')" ] && echo "$os"
  echo
}

# ------------------------------------------------------------------ mod: genel

report_all() {
  header
  printf '  %-3s %-14s %s\n' "" "ARAÇ" "DURUM"
  printf '  %s\n' "---------------------------------------------------------------------"
  missing=0
  for t in $TOOLS; do
    detail="$(detect_"$t")"; rc=$?
    if [ "$rc" -eq 0 ]; then
      row "✓" "$(label "$t")" "$detail"
      continue
    fi
    missing=$((missing + 1))
    [ "$rc" -eq 2 ] || detail="kurulu değil"
    row "○" "$(label "$t")" "$detail"
    printf '        gerekli: %s\n' "$(need_note "$t")"
    hint_line "$(install_hint "$t")"
  done
  echo
  echo "Bu script hiçbir şey KURMAZ — yalnız raporlar."
  echo "Eksikler burada blocker olarak sayılmaz; bir milestone'un kapısı için:"
  for m in $MILESTONES; do
    echo "    bash scripts/doctor.sh $m"
  done
  echo
  echo "SONUÇ: $missing araç kurulu değil (bilgilendirici — çıkış kodu 0)."
  return 0
}

# -------------------------------------------------------------- mod: milestone

report_milestone() {
  m="$1"
  header
  echo "Kapı: $m   (gereksinimler kümülatif)"
  echo

  blockers_missing=0
  blocker_out=""; soon_out=""; ok_out=""; ignored=""

  for t in $TOOLS; do
    lvl="$(requirement "$t" "$m")"
    if [ "$lvl" = "info" ]; then
      ignored="$ignored $(label "$t")"
      continue
    fi
    detail="$(detect_"$t")"; rc=$?
    if [ "$rc" -eq 0 ]; then
      ok_out="${ok_out}$(row "✓" "$(label "$t")" "$detail")
"
      continue
    fi
    [ "$rc" -eq 2 ] || detail="kurulu değil"
    hint="$(printf '        kurulum: %s' "$(install_hint "$t")")"
    if [ "$lvl" = "blocker" ]; then
      blockers_missing=$((blockers_missing + 1))
      blocker_out="${blocker_out}$(row "✗" "$(label "$t")" "$detail")
${hint}
"
    else
      soon_out="${soon_out}$(row "⚠" "$(label "$t")" "$detail — $(need_note "$t")")
${hint}
"
    fi
  done

  if [ -n "$blocker_out" ]; then
    echo "BLOCKER — $m başlayamaz:"
    printf '%s' "$blocker_out"
    echo
  fi
  if [ -n "$soon_out" ]; then
    echo "YAKINDA GEREKLİ — $m'e başlamayı engellemez:"
    printf '%s' "$soon_out"
    echo
  fi
  if [ -n "$ok_out" ]; then
    echo "HAZIR:"
    printf '%s' "$ok_out"
    echo
  fi
  [ -n "$ignored" ] && echo "$m için gerekmeyenler:$ignored"
  echo
  echo "Bu script hiçbir şey KURMAZ — yalnız raporlar."
  echo

  if [ "$blockers_missing" -gt 0 ]; then
    echo "SONUÇ: $m için $blockers_missing blocker eksik." >&2
    return 1
  fi
  echo "SONUÇ: $m için tüm blocker'lar hazır."
  return 0
}

# ------------------------------------------------------------------------ main

if [ "$#" -eq 0 ]; then
  report_all
  exit 0
fi

if [ "$#" -gt 1 ]; then
  echo "HATA: en fazla bir argüman beklenir." >&2
  echo "      Kullanım: bash scripts/doctor.sh [$(echo "$MILESTONES" | tr ' ' '|')]" >&2
  exit 1
fi

# Büyük/küçük harf duyarsız
arg="$(echo "$1" | tr '[:lower:]' '[:upper:]')"

case " $MILESTONES " in
  *" $arg "*) report_milestone "$arg"; exit $? ;;
esac

echo "HATA: bilinmeyen milestone '$1'." >&2
echo "      Kapısı tanımlı milestone'lar: $MILESTONES" >&2
echo "      Diğer milestone'lar için ek araç gerekmiyor; tüm durum için:" >&2
echo "          bash scripts/doctor.sh" >&2
exit 1
