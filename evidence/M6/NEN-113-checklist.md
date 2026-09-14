# NEN-113 kanıt checklist

Tarih: 2026-09-14

## DoD

- [x] Swift lifecycle testleri: fake `ForeignSecureCredentialStore` ile üç provider için save → kayıtlı göstergesi → delete → boş durum ve whitespace no-op doğrulandı. `CredentialSettingsTests`: 6 test / 1 suite geçti.
- [x] Plaintext persistence negatif testi: test sentinel'ı `UserDefaults.standard`, gerçek preferences plist'i ve uygulama Application Support taramasında bulunmadı. Checklist ölçümü: `grep -r -a -F` → `grep-sentinel=no-match`.
- [x] UI plaintext negatif testi: view model'in dışa verdiği presentation string'lerinde sentinel yok; SwiftUI view yalnız `SecureField` kullanıyor ve UI katmanında secret getter yok.
- [x] Gerçek `.app` checklist: ad-hoc imzalı uygulamada Ayarlar penceresi açıldı; üç provider satırı ve üç maskeli alan görüntülendi. Sentetik fixture akışı `enabled=truetrue...; saved=3; reopened-saved=3; deleted-empty=3` sonucu verdi. Smoke sonunda OpenSubtitles, OpenAI ve OpenRouter production Keychain hesaplarının her biri `44` (bulunamadı) idi.
- [x] Baseline + task testleri: `bash scripts/test-macos.sh` → 281 test / 35 suite geçti.

## Ek doğrulamalar

- `bash scripts/tests/credential-settings-guard.test.sh` geçti; doğrudan `UserDefaults`, dosya sistemi, `SecItem`, Keychain adapter ve secret `get` erişimi guard tarafından reddediliyor.
- `PATH=/usr/bin:/bin bash scripts/test.sh` → 6/6 shell test dosyası geçti.
- `cargo test --workspace --quiet`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` ve `cargo deny check` geçti. `cargo deny` yalnız mevcut duplicate `hashbrown` ve `syn` uyarılarını raporladı.
- `bash scripts/build-macos-app.sh` geçti. `/usr/bin/codesign --verify --deep --strict` → `verify=0`; bundle identifier `player.nen.macos`, `Security.framework` linki mevcut.
- `ApiKey` validation kontrol karakterini trim öncesi reddediyor; baş/son newline vakası için Rust regresyon testi eklendi.

## Değişen yüzey

- `CredentialSettings.swift`: üç provider'ın durum, save/delete lifecycle'ı ve typed FFI hata mesajı.
- `SubtitlePreferencesSettingsView.swift`: üç Türkçe API key satırı, masked fields, kayıtlı/değiştir/sil durumları ve başarısız denemede draft temizliği.
- `CredentialSettingsTests.swift`: lifecycle, typed failures, missing composition, accessibility ve plaintext negatif testleri.
- `credential-settings-guard.test.sh`: UI katmanının yalnız izin verilen FFI surface'ini kullandığını doğrulayan statik guard.

Gerçek provider anahtarı, ağ çağrısı veya kullanıcı verisi kullanılmadı; smoke fixture'ları aynı akış içinde silindi.
