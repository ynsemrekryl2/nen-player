---
id: NEN-114
title: Decide the real translation provider boundary
milestone: M6
size: S
state: backlog
closed:
depends_on: [NEN-090]
blocks: [NEN-115]
adr: [19]
---

# NEN-114 — Decide the real translation provider boundary

## Sonuç

ADR-0019 `accepted`: gerçek bir çeviri sağlayıcısının (OpenAI Responses API ·
OpenRouter) `TranslationProvider` portunu nasıl doldurduğu — HTTP taşıması,
istek/cevap şeması, prompt/schema versiyonları, retry sınırı, capability
preflight, model seçimi — ve M6'ya bırakılan **S3** (kalite çıtası) ile
**S9** (çoklu artifact sunumu) sorularının cevabı karara bağlanmış.

## Bağlam

`nen-ports::translation::TranslationProvider` (`NEN-090`) senkron, object-safe
ve mock ile kanıtlı; `nen-translate::versions` beş versiyon sabitini
(`prompt` · `schema` · ...) placeholder değerle tutuyor (`NEN-097`).
`HttpClient` portu yalnız `HEAD`/`GET` taşıyor — bir LLM çağrısı POST + JSON
gövde ister. Roadmap S3 ("anlaşılır" mı "yayın kalitesi" mi) ve S9 (aynı
medya için birden fazla AI çevirisinin sunumu) açıkça "gerçek sağlayıcıyla
M6'da kapanır" diyor. Kullanıcı kararı (2026-09-11): kalite çıtası **ölçümle**
bu ADR'de kararlaştırılır; varsayılan "anlaşılır".

## Kapsam

ADR-0019 en az şunları kararlaştırır:

- `HttpClient` portunun POST + JSON gövde + timeout ile genişlemesi (ADR-0039'a
  Notlar girdisi; politika core'da, transport adapter'da kuralı korunur)
- Sağlayıcı isteğinin şekli: sistem/kullanıcı prompt'u, belge bağlamı (ADR-0015),
  cue listesi `CueId + text`; **structured output** JSON şeması (`cue_id`,
  `text` dizisi) — cevap yine untrusted, yerel doğrulama authoritative
  (ADR-0016)
- `versions::PROMPT_VERSION` / `SCHEMA_VERSION`'ın gerçek ilk değeri ve
  bump kuralı (ADR-0018 ile tutarlı)
- Retry: yalnız bounded transient (ağ hatası, 408/429/5xx), doğrulama hatası
  retry sebebi değil (`security-policy.md` §3); `NEN-092`'nin repair bütçesiyle
  ilişkisi
- OpenRouter structured-output **capability preflight**: hangi endpoint,
  sonuç negatifse iş **başlamaz** (`StartRefusal` varyantı)
- Approved host listesi (`api.openai.com` · `openrouter.ai`), yalnız HTTPS
- Provider/model ayarı: kullanıcı seçer; varsayılan sağlayıcı ve model ADR'de
  yazılır; model kimliği `TranslationProviderIdentity`'ye (cache identity'ye)
  girer
- **S3:** kalite çıtası "anlaşılır" varsayılan; ölçüm yöntemi (kabul
  checklist'inde `NEN-126`'nın dilsel inceleme maddesi) ve "yayın kalitesi"
  iddiasının ne zaman/nasıl yazılabileceği
- **S9:** hedef dil grubunda birden fazla AI artifact'i olduğunda sunum —
  en yeni (mevcut `NEN-098` projeksiyonu) mı, provider/model etiketli seçim mi
- Anahtarın adapter'a enjeksiyonu: `NEN-111` credential portundan, adapter
  saklamaz (ADR-0040 emsali)

## YAPILMAYACAK

- HTTP port genişlemesinin kodu — `NEN-115`
- Adapter'lar — `NEN-116`, `NEN-117`; ortam kurulumu — `NEN-118`
- Glossary (§10 user glossary) — M6 kapsamı dışı, `GlossaryIdentity::none()`
  kalır; ADR bunu açıkça erteler
- Belirli bir model adının "yayın kalitesi" verdiğini ölçmeden iddia etmek

## Kanıt (DoD)

- [ ] `docs/adr/0019-*.md` yazıldı, kullanıcı onayıyla `accepted`
- [ ] `docs/roadmap.md` S3 ve S9 satırları ve `docs/DECISIONS.md` §3 karara
      bağlandı (tarihli); `docs/adr/README.md`, ADR sayacı tutarlı
- [ ] ADR-0039'a Notlar girdisi (POST genişlemesi)
- [ ] `docs/architecture.md` → Provider portları tablosuna OpenAI/OpenRouter
      adapter satırları (aday değil, karar)
- [ ] `bash scripts/check-docs.sh` çıkış 0

## Kanıt kaydı

<!-- done olurken doldurulacak -->
