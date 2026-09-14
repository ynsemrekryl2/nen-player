# NEN-038 — Kanıt kaydı

Tarih: 2026-09-14  
Task: `NEN-038`  
ADR: [`ADR-0047`](../../docs/adr/0047-preferred-language-auto-download.md)

## Uygulama kanıtı

- `nen-catalog` OpenSubtitles basamağını varsayılan yerel seçimden ayrı,
  açıkça parametrelenen saf bir seçim yolu olarak sunuyor; `ai` otomatik
  seçime girmiyor.
- macOS ayarı varsayılan kapalı ve `UserDefaults` ile kalıcı. Açıkken hazır
  oynatma, sidecar taraması, aday araması ve kesin doğrulanmış hash eşleşmesi
  birlikte sağlanmadan otomatik indirme başlamıyor.
- Birinci tercih dili bulunamazsa ikinci tercih dili deneniyor; iki dilde aday
  yoksa sonuç `Closed`. Medya ve tam takvim günü için yalnızca özetlenmiş
  deneme anahtarı tutuluyor; başarısız indirme aynı gün tekrarlanmıyor.
- İndirme sürerken kullanıcının altyazıyı kapatması veya başka bir kaynak
  seçmesi korunuyor; başarılı provider sonucu kullanıcı seçimini sessizce
  değiştirmiyor.

## DoD kanıtı

- `unqualifiedIdentityDoesNotDownload`: otomatik kapının altındaki `.noMatch`
  sonucu provider satırı görünse bile indirme yapmıyor.
- `disabledAutomaticDownloadDoesNotDownload`: ayar kapalıyken otomatik
  indirme isteği çıkmıyor.
- `automaticDownloadUsesSecondaryPreference`: birinci tercih `fr` yoksa
  ikinci tercih `en` seçiliyor ve deterministic fake provider indirmesi
  tamamlanıyor.
- `automaticDownloadClosesWithoutCandidate`: her iki tercih için boş fake
  provider sonucu `Closed` kalıyor.
- `automaticDownloadPreservesUserChoice` ve günlük bütçe testi, kullanıcı
  seçiminin korunmasını ve aynı medya/günde tek denemeyi kanıtlıyor.

## Doğrulama

- `swift test --package-path platforms/macos --filter OpenSubtitlesSelectionTests`
  — **11/11 geçti**.
- `swift test --package-path platforms/macos --filter SubtitlePreferenceStoreTests`
  — **8/8 geçti**.
- `cargo test --workspace --quiet`, `cargo fmt --all -- --check` ve
  `cargo clippy --workspace --all-targets -- -D warnings` — geçti.
- `cargo deny check` — advisories, bans, licenses ve sources geçti; yalnız
  mevcut duplicate dependency uyarıları var.
- `bash scripts/test.sh` — **6/6 geçti**.
- `bash scripts/build-macos-app.sh` — macOS uygulama derleme/link aşaması
  geçti; yalnız mevcut deployment-target linker uyarıları var.

K23 kapsamında özel URL, credential, tam yol, provider payload'ı, altyazı
diyaloğu veya dosya metadata'sı bu kayda alınmamıştır.
