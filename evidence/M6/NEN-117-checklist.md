# NEN-117 kanıt kaydı — OpenRouter provider ve preflight

Tarih: 2026-09-14

## Kapsam

- OpenRouter Chat Completions adapter'ı approved `openrouter.ai` host kontrolü,
  OpenAI-only routing ve structured JSON schema gövdesiyle eklendi.
- NEN-116 ile istek, cevap, bounded body, retry, cancellation ve hata sınıflandırması
  ortak `translation_http` modülünde paylaşıldı.
- Model capability preflight'ı `supported_parameters` içindeki
  `structured_outputs` değerini kontrol ediyor; pozitif ve negatif sonuçlar
  `Clock` seam'i üzerinden deterministik 15 dakika cache'leniyor. Ağ/parse
  hataları cache'lenmiyor.
- Capability eksikliği provider tarafında `CapabilityMissing`, app/FFI yüzeyinde
  `ProviderCapabilityMissing` olarak taşınabilecek şekilde tanımlandı; task
  wiring'i NEN-118 kapsamındadır.

## DoD kanıtı

| Gereksinim | Kanıt |
|---|---|
| Contract kiti | `openrouter_passes_the_shared_contract_kit_with_preflight`; redakte fixture'lar |
| Capability eksikliği | `unsupported_model...` testi; preflight reddedildi ve translation POST sayacı 0 kaldı |
| Preflight transient hata | `preflight_transport_failure_is_transient_and_does_not_start_translation`; hiçbir artifact/progress üretilmedi |
| K23 redaction | `openrouter_debug_never_contains_secret_or_payload`; key, raw response ve subtitle metni Debug'a girmedi |
| Cache | Pozitif/negatif sonuçlarda ikinci GET yok; `capability_cache_expires_at_fifteen_minutes` exact expiry'de yeniliyor |
| Golden | `openrouter_request_matches_golden`; request gövdesi redakte golden ile birebir eşleşiyor |

## Çalıştırılan kapılar

```text
cargo test -p nen-providers                            PASS
cargo test -p nen-app -p nen-ffi -p nen-providers     PASS
cargo test --workspace                                PASS
cargo fmt --all -- --check                            PASS
cargo clippy --workspace --all-targets -- -D warnings PASS
cargo deny check                                      PASS
bash scripts/test.sh                                  PASS (6/6)
bash scripts/check-docs.sh                            PASS (10/10)
git diff --check                                      PASS
```

`cargo deny` yalnız mevcut duplicate `hashbrown` ve `syn` uyarılarını raporladı;
advisories, bans, licenses ve sources kontrolleri geçti. Testler fake HTTP client
ve redakte fixture'larla çalıştı; gerçek ağa çıkılmadı.
