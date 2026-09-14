#!/usr/bin/env bash
# K23 guard for the macOS credential settings surface.

set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VIEW="$ROOT/platforms/macos/Sources/NenPlayerShell/SubtitlePreferencesSettingsView.swift"
MODEL="$ROOT/platforms/macos/Sources/NenPlayerShell/CredentialSettings.swift"

for source in "$VIEW" "$MODEL"; do
  if grep -En 'UserDefaults|FileManager|applicationSupportDirectory|SecItem|Security|KeychainCredentialStore|\.get[[:space:]]*\(' "$source" >/dev/null; then
    echo "HATA: credential settings surface plaintext storage veya doğrudan Keychain erişimi içeriyor: $source" >&2
    exit 1
  fi
done

rows="$(grep -Ec 'credentialRow\(\.(openSubtitles|openAi|openRouter),' "$VIEW" || true)"
if [ "$rows" -ne 3 ]; then
  echo "HATA: üç provider credential satırı bulunamadı (bulunan: $rows)." >&2
  exit 1
fi

ids="$(grep -Ec 'accessibilityIdentifier\(' "$VIEW" || true)"
if [ "$ids" -ne 4 ]; then
  echo "HATA: credential field/status/save/delete accessibility yüzeyi eksik (bulunan: $ids)." >&2
  exit 1
fi

if ! grep -En 'SecureField|Kayıtlı|Değiştir|Sil' "$VIEW" >/dev/null; then
  echo "HATA: maskeli credential alanı veya durum eylemleri bulunamadı." >&2
  exit 1
fi

if ! grep -En 'FfiSecureCredentialStore' "$MODEL" >/dev/null; then
  echo "HATA: ayar modelinin FFI credential yüzeyi yok." >&2
  exit 1
fi

if ! grep -En 'contains|\.set\(|\.delete\(' "$MODEL" >/dev/null; then
  echo "HATA: ayar modelinin yalnız presence/set/delete yolu doğrulanamadı." >&2
  exit 1
fi

echo "OK: credential settings yalnız Rust FFI store üzerinden çalışıyor; üç satır ve plaintext guard geçildi."
