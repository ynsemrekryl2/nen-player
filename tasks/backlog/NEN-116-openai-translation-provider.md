---
id: NEN-116
title: OpenAI Responses API translation provider
milestone: M6
size: M
state: backlog
closed:
depends_on: [NEN-115, NEN-111]
blocks: [NEN-117, NEN-118, NEN-034]
adr: [19]
---

# NEN-116 — OpenAI Responses API translation provider

## Sonuç

`nen-providers::openai::OpenAiTranslationProvider`, `TranslationProvider`
portunu OpenAI Responses API üzerinden, ADR-0019'un structured-output
şemasıyla dolduruyor; redakte edilmiş kayıtlı fixture'larla contract kitini
geçiyor ve raw cevap, API anahtarı veya diyalog hiçbir log/`Debug` yüzeyine
düşmüyor.

## Kapsam

- İstek: ADR-0019'un prompt'u (`nen-translate::context` belge bağlamı +
  blok cue'ları), `text.format = json_schema` (strict), model kimliği
  ayardan; `Authorization: Bearer` header'ı `NEN-111`'in newtype'ından
  (adapter saklamaz — ADR-0040 emsali)
- Cevap: **untrusted** parse — `output_text`/JSON gövdesi `serde_json` ile
  bounded (1 MiB) çözülür, `TranslationResponse` (`CueId + text`) üretilir;
  doğrulama `NEN-091`'e bırakılır, adapter kendi doğrulamasını yapmaz
- Hata sınıfları → `TranslationProviderError`: 401/403 → `Permanent`
  (credential), 408/429/5xx/transport → `Transient`, bozuk JSON/şema dışı →
  `Permanent`; `refusal` alanı → `Permanent`
- Bounded transient retry (ADR-0019'un sayısı) adapter içinde mi, pipeline'da
  mı — ADR'nin kararı; `TranslationCall` iptal geçidi her denemede kontrol
- Progress: blok başına `TranslationCall::progress` (`NEN-106`'nın fork
  deseniyle uyumlu)
- `fixtures/providers/openai/` — redakte kayıtlı cevaplar (başarılı blok ·
  eksik cue · tekrar ID · 429 · 500 · bozuk JSON · refusal)
- Approved host: yalnız `https://api.openai.com`

## YAPILMAYACAK

- Gerçek ağa çıkan test — Kural 8 **yasak**; tek gerçek koşu `NEN-126`
- OpenRouter — `NEN-117`
- Ortam kurulumu / ayar seçimi — `NEN-118`
- Glossary gönderimi — M6 dışı (ADR-0019)
- Streaming / SSE

## Kanıt (DoD)

- [ ] `nen-ports` translation contract kiti (`NEN-090`) `FakeHttpClient` +
      fixture ile geçiyor
- [ ] Golden: gönderilen istek gövdesi (`fixtures/providers/openai/request-*.golden`)
      — prompt/schema versiyonu değişirse golden değişir
- [ ] Negatif (zorunlu, K23): sentinel API anahtarı, sentinel diyalog ve raw
      cevap gövdesi adapter tiplerinin/hatalarının `Debug`/`Display`'inde yok;
      kasıtlı `#[derive(Debug)]` ikizi sızdırıyor
- [ ] Negatif: approved host dışı bir URL'e (test ile enjekte) istek çıkmıyor
      (`send` sayacı 0, `Permanent`)
- [ ] Negatif: 429/5xx → `Transient`, bozuk JSON → `Permanent`; retry
      doğrulama hatasında **tetiklenmiyor** (sayaç)
- [ ] Negatif: iptal edilmiş `TranslationCall` ile retry denemesi yapılmıyor
- [ ] `cargo deny check` — yeni dış bağımlılık yoksa "yok", varsa gerekçe

## Kanıt kaydı

<!-- done olurken doldurulacak -->
