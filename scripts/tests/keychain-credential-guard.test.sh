#!/usr/bin/env bash
# K23/static guard for the macOS Keychain adapter (NEN-112).

set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SOURCE="$ROOT/platforms/macos/Sources/NenCredentialsKeychain/KeychainCredentialStore.swift"

[ -f "$SOURCE" ] || {
  echo "HATA: Keychain adapterı yok: $SOURCE" >&2
  exit 1
}

# The adapter has no logging surface at all. Keeping this as a source guard
# prevents a future diagnostic from accidentally interpolating a credential.
if rg -n '\b(print|NSLog|os_log|Logger)\b' "$SOURCE"; then
  echo "HATA: Keychain adapterında log yüzeyi bulundu." >&2
  exit 1
fi

# These APIs would either enable cloud sync or require the data-protection
# entitlement rejected by ADR-0020. The adapter must not mention or use them.
if rg -n 'kSecAttrSynchronizable|kSecUseDataProtectionKeychain' "$SOURCE"; then
  echo "HATA: Keychain adapterı ADR-0020 dışı bir Keychain alanı kullanıyor." >&2
  exit 1
fi

rg -n 'kSecClassGenericPassword|kSecAttrService|kSecAttrAccount|SecItem(Add|CopyMatching|Update|Delete)' "$SOURCE" >/dev/null
echo "OK: Keychain adapterı logsuz, namespaced ve izin verilen SecItem API'lerini kullanıyor."
