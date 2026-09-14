# NEN-112 — macOS Keychain credential adapter kanıtı

**Tarih:** 2026-09-14 · **Makine:** macOS 27.0 · Swift 6.3.3 · Xcode 26.6

## Uygulama

- `NenCredentialsKeychain` modülü gerçek login Keychain generic-password
  API'lerini kullanır: `SecItemAdd`, `SecItemCopyMatching`, `SecItemUpdate` ve
  `SecItemDelete`.
- Üretim service'i `player.nen.macos`; account değerleri üç kapalı
  `FfiCredentialKind` varyantının sabit slug'larıdır. Testler yalnız
  `player.nen.macos.test` namespace'ini kullanır.
- `errSecItemNotFound` sırasıyla `nil` / `false` / idempotent delete olur;
  auth, interaction ve user-cancel durumları `Denied`, diğer status'ler
  `Unavailable`, UTF-8 olmayan stored bytes `Corrupt` olarak döner.
- `CredentialCompositionRoot.makeProductionStore()` tek üretim composition
  hook'udur. `NenPlayerApp` bunu Rust-owned `FfiSecureCredentialStore` olarak
  `PlayerModel`'e enjekte eder; uygulama katmanı Keychain API'si çağırmaz.

## Gerçek Keychain contract ve negatifler

`KeychainCredentialStoreTests` (`@Suite(.serialized)`) gerçek login Keychain'de
4 test çalıştırdı:

1. Üç kind için başlangıçta yokluk, set→get, `contains`, kind izolasyonu,
   delete→get yokluğu ve eksik kaydı silmenin idempotent oluşu doğrulandı.
2. Test öncesi ve test namespace'i temizlendikten sonraki
   `SecItemCopyMatching` all-services generic-password sayısı eşit kaldı;
   adapterın test service'i dışına yazmadığı/silmediği doğrulandı.
3. `FfiSecureCredentialStore` üzerinden gerçek reverse-FFI set/contains/delete
   akışı çalıştı; whitespace Rust sınırında normalize edildi.
4. Bulunamayan kayıt yokluk olarak, geçersiz UTF-8 kayıt `Corrupt` olarak
   gözlendi. `errSecAuthFailed`, `errSecInteractionNotAllowed`,
   `errSecUserCanceled` → `Denied`; `errSecParam` → `Unavailable` testi geçti.

Test sırasında oluşturulan yalnız test-service kayıtları her testin başında ve
sonunda silindi; gerçek provider anahtarı veya kredi/kota kullanılmadı.

## K23 / statik guard

`bash scripts/tests/keychain-credential-guard.test.sh` geçti. Guard:

- adapter source'unda `print`/`NSLog`/`os_log`/`Logger` yolu olmadığını,
- cloud-sync ve data-protection Keychain alanlarının kullanılmadığını,
- yalnız namespaced generic-password `SecItem*` yüzeyinin bulunduğunu

kontrol ediyor. Swift testindeki payload-free FFI error guard'ı da geçti.

## Build ve uygulama kanıtı

- `bash scripts/test-macos.sh` → **275 test / 34 suite, 0 failure**.
- `swift test --package-path platforms/macos --filter KeychainCredentialStoreTests`
  → **4/4**.
- `bash scripts/build-macos-app.sh` → çıkış 0.
- `/usr/bin/codesign --verify --deep --strict --verbose=2
  platforms/macos/.build/NenPlayer.app` → **valid on disk**, designated
  requirement sağlandı; `codesign -dvv` → `Identifier=player.nen.macos`,
  `Signature=adhoc`.
- Ad-hoc bundle executable'ı cold-start edildi ve kontrollü sonlandırıldı;
  `otool -L` çıktısında `Security.framework` yer alıyor. Gerçek set/get
  denemesi, bu bundle'ın linklediği aynı `NenCredentialsKeychain` modülünün
  gerçek-Keychain contract suite'inde yapıldı; UI FFI yüzeyi yalnız set/
  contains/delete sunar ve stored secret'ı geri vermez (ADR-0020).

## Regresyon kapıları

- `cargo test --workspace --quiet` → geçti.
- `cargo fmt --all -- --check` → geçti.
- `cargo clippy --workspace --all-targets -- -D warnings` → geçti.
- `cargo deny check` → advisories/bans/licenses/sources geçti; mevcut duplicate
  `hashbrown` ve `syn` uyarıları bilgi seviyesinde.
- `bash scripts/test.sh` → **5/5** script suite geçti.
- `bash scripts/check-docs.sh` → tüm denetimler geçti; 0 aktif task, güncel
  INDEX/STATUS, en yeni done tarihi 2026-09-14.
- `git diff --check` → geçti.
