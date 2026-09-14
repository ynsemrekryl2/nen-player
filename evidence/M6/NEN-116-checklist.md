# NEN-116 kanıt kaydı — OpenAI Responses API translation provider

Tarih: 2026-09-14

## Uygulanan sınır

- `nen-providers::openai::OpenAiTranslationProvider`, ortak `TranslationProvider`
  portunu doğrudan `https://api.openai.com/v1/responses` adresine bounded JSON
  `POST` ile uygular.
- İstek `store: false`, model kimliği, `text.format.type=json_schema`,
  `subtitle_translation` adı ve `strict: true` içerir. Prompt ve şema sürümleri
  `1` olarak korunmuştur.
- `ApiKey` yalnız Authorization header üretiminde açılır; request, response,
  provider ve typed error Debug/Display yüzeyleri payload içermez.
- 1 MiB request/response sınırı, 60 saniye timeout, 408/429/5xx ve transport
  için en fazla iki retry, bounded `Retry-After` ve her denemede cancellation
  checkpoint uygulanmıştır.
- Eksik/tekrar cue response'ları provider tarafından onarılmaz; ortak local
  validator'a bırakılır.

## Kanıtlar

| Kontrol | Sonuç |
|---|---|
| `cargo test -p nen-providers` | ✅ geçti: 4 unit + 13 OpenAI + 9 OpenSubtitles + 5 translation-provider integration test |
| OpenAI shared contract kit | ✅ `FakeHttpClient` ve başarı fixture'ı ile geçti |
| Request golden | ✅ `fixtures/providers/openai/request-success.golden` semantic JSON karşılaştırması |
| Response fixtures | ✅ başarı, eksik cue, tekrar ID, 429, 500, bozuk JSON ve refusal fixture'ları |
| HTTP/error sınıfları | ✅ 401/403/diğer kalıcı durumlar, 429/500/transport retry ve retry exhaustion |
| Retry/cancellation | ✅ fallback 500 ms/2 s, `Retry-After` 10 s clamp, backoff sırasında cancel sonrası yeni send yok |
| Approved-host negatif | ✅ yabancı endpoint `Permanent`, `send` sayacı 0 |
| K23 negatif | ✅ sentinel API key/diyalog/raw response provider, request, response ve key Debug yüzeylerinde yok; derived twin guard'ı sızdırıyor |
| Bounded body | ✅ request ve response 1 MiB aşımında `Permanent`; oversized request send öncesi reddediliyor |
| `cargo test --workspace` | ✅ geçti |
| `cargo fmt --all -- --check` | ✅ geçti |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ geçti |
| `cargo deny check` | ✅ geçti; mevcut duplicate `hashbrown` ve `syn` uyarıları dışında advisory/bans/licenses/sources temiz |
| `bash scripts/test.sh` | ✅ 6/6 shell test dosyası geçti |
| `bash scripts/check-docs.sh` | ✅ tüm 10 dokümantasyon denetimi geçti |

Canlı OpenAI ağına çıkılmadı; tüm provider testleri deterministic fake HTTP ile
çalıştırıldı.
