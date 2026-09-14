# NEN-121 kanıt kaydı — OpenSubtitles aday kataloğu

Tarih: 2026-09-14

## DoD kanıtı

| Gereksinim | Kanıt |
|---|---|
| Fixture adayları kataloglanıyor | `nen-providers` fixture testi bounded adayları, normalize dili, release metadata'sını ve iki kapalı rozeti doğruluyor; `nen-app` testi tercih gruplarını, token upsert'ini ve `document() == None` durumunu doğruluyor. |
| Arama indirme yapmıyor | Adapter testinde tek istek `GET` metadata araması; download endpoint'i için sayaç sıfır. `SubtitleLibrary::add_opensubtitles` yalnız metadata kataloğu güncelliyor. |
| Credential ve kimlik negatifleri | Anahtar yokken app orchestration provider'a hiç istek göndermiyor; hash ve verified identity yokken credential/HTTP kapısından önce `NoIdentity` dönüyor. Exact hash boş sonucu verified identity aramasına deterministik sırayla düşüyor. |
| K23 redaction | App ve FFI guard'ları `SubtitleSource`, `MenuEntryView`, `FfiMenuEntry`, hash ve typed candidate error Debug yüzeylerinde hassas değer olmadığını; derived twin'ın sızıntıyı yakabildiğini doğruluyor. Private file id yalnız app içi eşlemede tutuluyor. |
| Untrusted response | JSON gövde sınırı, 16 aday sınırı, duplicate/invalid kayıt filtreleme, LanguageTag normalizasyonu, release sanitizasyonu ve approved-host redirect kapısı testli. Hatalar payload-free typed error olarak dönüyor. |
| Otomatik seçim | Mevcut `opensubtitles_is_never_selected_automatically` testi değişmeden yeşil; `AUTO_SELECTABLE_KINDS` yalnız Embedded/User olarak kaldı. |
| Contract kit | Port fake contract testi ve gerçek OpenSubtitles adapter fixture contract testi geçti. |

## Çalıştırılan kapılar

```text
cargo test --workspace                                      PASS
cargo fmt --all -- --check                                  PASS
cargo clippy --workspace --all-targets -- -D warnings       PASS
cargo deny check                                            PASS
bash scripts/build-macos-app.sh                             PASS
bash scripts/test-macos.sh                                  PASS (295 tests / 36 suites)
bash scripts/test.sh                                        PASS (6/6)
bash scripts/check-docs.sh                                  PASS (10/10)
git diff --check                                            PASS
```

Tüm provider çağrıları deterministic fake HTTP/credential fixture'larıyla
çalıştı; canlı provider ağı kullanılmadı.
