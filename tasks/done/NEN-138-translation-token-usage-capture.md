---
id: NEN-138
title: Capture provider token usage in translation pipeline
milestone: M6
size: L
state: done
closed: 2026-09-17
depends_on: [NEN-116, NEN-117]
blocks: [NEN-139]
adr: []
---

# NEN-138 — Capture provider token usage in translation pipeline

## Sonuç

Her çeviri sağlayıcı çağrısından (belge analizi, blok çevirisi, repair) dönen
input/cache/output token sayıları yakalanır, tüm çeviri işi için toplanır ve
`nen-app`'ten FFI özetine kadar taşınır.

## Bağlam

Bu iş M6'nın altı çıkış kriterinden hiçbirini kapsamıyor; `NEN-126` kabulünü
**beklemez ve onu bloklamaz**. Kullanıcı isteği: çeviri sırasında tüketilen
input/cache/output token sayısının ve maliyetin Olaylar penceresinden
görülebilmesi (`NEN-139`).

Maliyet kaynağı sağlayıcıya göre asimetrik: OpenRouter, `usage: {include:
true}` isteğiyle gerçek faturalanan `usage.cost`'u döndürür — bakım
gerektirmez, kullanıcının seçtiği her model için doğrudur. Doğrudan OpenAI
API'si yalnız token sayısı döndürür, dolar tutarı vermez. Model adı serbest
metin olduğu için (ADR-0019 belirli bir model listesiyle sınırlamıyor) elle
bakımı gereken bir fiyat tablosu hızla bayatlar; yanlış rakam göstermek K23'ün
uyardığı sınıfa yakın bir risk. Bu yüzden **OpenAI-direct yolda yalnız token
sayıları gösterilir, `$` tutarı üretilmez** — M6'nın varsayılan gerçek çeviri
yolu zaten OpenRouter/Luna olduğu için maliyet asıl kullanılan yolda görünür
kalır. Bu varsayım task'ın kapsamıdır; farklı istenirse task açılmadan önce
değiştirilebilir.

## Kapsam

- OpenAI Responses API yanıtındaki `usage` alanını (`input_tokens`,
  `input_tokens_details.cached_tokens`, `output_tokens`) `nen-providers`
  içinde (`translation_http.rs` / `openai.rs`) ayrıştırmak; alan adları
  implementasyon sırasında güncel OpenAI dokümantasyonuyla doğrulanır.
- OpenRouter yanıtındaki eşdeğer `usage` alanını (`prompt_tokens`,
  `prompt_tokens_details.cached_tokens`, `completion_tokens`) `openrouter.rs`
  içinde ayrıştırmak; isteğe `usage: {include: true}` eklemek ve dönen gerçek
  `usage.cost`'u taşımak.
- `nen-ports::translation` içine sağlayıcıdan bağımsız bir `TokenUsage` tipi
  eklemek (input, cached_input, output; opsiyonel `provider_reported_cost_usd`)
  ve bunu `TranslationResponse` ile `DocumentAnalysis` sonucuna eklemek.
- `nen-app::translation` içinde belge analizi + her blok çağrısı + varsa
  repair çağrılarının usage'ını tüm çeviri işi için toplamak; **iptal veya
  hata durumunda da o ana kadar tüketilen usage kaybolmaz** — kullanıcı o
  kadarını zaten ödedi.
- `nen-ffi::translation::FfiTranslationSummary`'e toplam input/cached-input/
  output token sayısını ve (varsa) `usd_cost`'u eklemek. K23 kapsamındaki
  hiçbir alan (ham yanıt, prompt, cue metni) taşınmaz — yalnız sayılar.

## YAPILMAYACAK

- OpenAI-direct yol için maliyeti tahmin eden, elle bakımı gereken bir fiyat
  tablosu — bkz. Bağlam. Gerekirse ayrı bir task.
- Ham HTTP yanıt gövdesini loglamak, artifact'e yazmak veya `Debug` çıktısına
  taşımak.
- macOS Olaylar penceresinde gösterim — `NEN-139`.
- Prompt/şema/versiyon sözleşmesini (ADR-0048) değiştirmek.
- `NEN-126` kabul checklist'ine yeni madde eklemek.

## Kanıt (DoD)

- [x] OpenAI ve OpenRouter fixture yanıtlarıyla usage ayrıştırma testleri:
      cache'siz, cache'li ve `usage` alanı eksik yanıt için negatif test —
      eksik `usage` çeviriyi **başarısız etmez**, `TokenUsage::default()`
      döner (her sayaç sıfır, `cost_usd` `None`); bu tipli ayrım zaten
      `cost_usd: Option<f64>` ile kurulu.
- [x] Belge-geneli toplama testi: analiz + N blok + repair çağrısının usage'ı
      doğru toplanıyor.
- [x] İptal/hata negatif testi: iptal edilen veya kısmen başarısız bir çeviri
      işinde o ana kadarki usage sıfırlanmıyor.
- [x] `FfiTranslationSummary` alanlarının doğru taşındığını gösteren
      `nen-ffi` testi.
- [x] Negatif: ham response body'nin hiçbir `Debug`/log/artifact yüzeyine
      sızmadığını gösteren test (mevcut K23 guard testleriyle aynı desen).
- [x] `cargo test --workspace`, `cargo fmt --all --check`, `cargo clippy
      --workspace --all-targets --all-features -- -D warnings`, `cargo deny
      check`, `bash scripts/check-docs.sh`, `bash scripts/task-index.sh
      --check` ve `git diff --check` çıkış 0.

## Kanıt kaydı

2026-09-17 doğrulama kaydı:

- **Usage ayrıştırma (OpenAI/OpenRouter, cache + malformed):**
  `cargo test -p nen-providers` içinde yeni testler geçti —
  `translate_and_analyze_report_token_usage_from_the_response` (OpenAI,
  `input_tokens_details.cached_tokens: 50` doğru okundu, `cost_usd: None`),
  `translate_and_analyze_report_token_usage_including_billed_cost`
  (OpenRouter, `prompt_tokens_details.cached_tokens: 30`, gerçek
  `usage.cost: 0.0038` → `cost_usd: Some(0.0038)`),
  `request_asks_openrouter_to_include_billed_usage` (istek gövdesi
  `usage.include: true` taşıyor — dört golden fixture da güncellendi) ve
  her iki dosyada `missing_usage_defaults_to_zero_without_failing_the_call`
  (usage alanı yok → `TokenUsage::default()`, çeviri yine `Ok`).
- **Belge-geneli + repair toplama:** `repair_attempts_accumulate_usage_across_every_provider_call`
  (nen-providers, `call.fork()` üzerinden iki `translate()` çağrısının
  input/output'u toplanıyor) ve `a_completed_job_reports_usage_summed_across_analysis_and_every_block`
  (nen-app, gerçek `translation::start`/`join` akışı: analiz + tek blok
  usage'ı `6*10+6*5=90` input, `20+6*8=68` output olarak doğru toplanıyor).
- **İptal/hata sonrası usage kaybolmuyor:** `a_failed_job_still_reports_the_usage_already_billed`
  (nen-app, 70 cue/2 blok, `FailAfterOneProvider` ikinci bloğu reddediyor;
  `TranslationCancelHandle::total_usage()` — `join()` `Err` döndükten sonra
  bile — analiz ve ilk bloğun tükettiği token'ları sıfırlamıyor).
- **FFI taşınması:** `the_summary_and_total_usage_carry_the_jobs_real_token_total`
  (nen-ffi, gerçek `FfiTranslationEngine`/`FfiTranslationJob` gate'i:
  `summary.usage` ve bağımsız `job.total_usage()` aynı gerçek toplamı
  taşıyor, ikisi birbiriyle tutarlı).
- **Ham yanıt sızıntısı yok:** yeni `TokenUsage`/`FfiTokenUsage` tipleri
  yalnız `u64`/`Option<f64>` alanlar taşıyor — bir string/byte alanı yok,
  dolayısıyla sızacak bir şey yok; mevcut `sensitive_provider_surfaces_are_shape_only`
  (openai/openrouter) ve `guard_ffi_translation_debug.rs`'nin gerçek uçtan
  uca koşusu (özet artık `usage` alanını da taşıyor) bunu değişiklik
  gerektirmeden doğruluyor — hepsi bu koşuda yeşil.
- **Kapılar:** `cargo test --workspace` (115 test binary'si, tümü `ok`, 0
  başarısız), `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets --all-features -- -D warnings`, `cargo deny check`
  (advisories/bans/licenses/sources ok), `bash scripts/check-docs.sh` (10
  denetim de ok), `bash scripts/task-index.sh --check` ve `git diff --check`
  hepsi çıkış 0 verdi.
