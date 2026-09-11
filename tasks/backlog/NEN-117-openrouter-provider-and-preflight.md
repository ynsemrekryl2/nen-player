---
id: NEN-117
title: OpenRouter translation provider with structured-output preflight
milestone: M6
size: M
state: backlog
closed:
depends_on: [NEN-116]
blocks: [NEN-118]
adr: [19]
---

# NEN-117 — OpenRouter translation provider with structured-output preflight

## Sonuç

`nen-providers::openrouter::OpenRouterTranslationProvider` aynı contract kitini
geçiyor ve seçilen model structured output desteklemiyorsa **preflight** bunu
iş başlamadan tespit ediyor — hiçbir çeviri isteği çıkmıyor.

## Kapsam

- OpenRouter chat-completions uyumlu istek (`response_format: json_schema`),
  `Authorization: Bearer` + ADR-0019'un istediği tanımlayıcı header'lar;
  approved host yalnız `https://openrouter.ai`
- `NEN-116`'nın istek/cevap/hata kodunu **paylaşan** ortak modül (iki adapter
  iki kopya değil) — ADR-0019'un tarif ettiği yerde
- Capability preflight: model listesi/parametre endpoint'inden
  structured-output desteği; sonuç `ProviderCapabilities` (kapalı struct);
  desteklemiyorsa `StartRefusal::ProviderCapabilityMissing` (adı ADR'de)
- Preflight sonucu iş süresince önbellek — aynı iş içinde tekrar sorulmaz;
  önbellek süresi ADR'nin kararı (`Clock` portu, deterministik test)
- `fixtures/providers/openrouter/` — redakte fixture'lar (destekleyen model ·
  desteklemeyen model · başarılı blok · 429 · 502 · bozuk JSON)

## YAPILMAYACAK

- Gerçek ağ — Kural 8
- Model kataloğu UI'ı / model listesi seçici — `NEN-118` yalnız ayarı
  bağlar; tam liste tarayıcı M6 dışı
- Fiyat/kredi tahmini
- OpenRouter'a özgü fallback/routing parametreleri — tek model, ADR-0019

## Kanıt (DoD)

- [ ] Contract kiti fixture ile geçiyor
- [ ] Negatif (zorunlu): desteklemeyen model → preflight reddi, çeviri
      endpoint'ine `send` sayacı **0**
- [ ] Negatif: preflight'ın kendi ağ hatası → `Transient`, iş başlamıyor,
      yarım artifact yok
- [ ] Negatif (K23): anahtar/raw cevap/diyalog `Debug`'da yok (`NEN-116`
      guard'ının OpenRouter kopyası)
- [ ] Unit: preflight önbelleği — aynı iş içinde ikinci `send` yok
- [ ] Golden: istek gövdesi

## Kanıt kaydı

<!-- done olurken doldurulacak -->
