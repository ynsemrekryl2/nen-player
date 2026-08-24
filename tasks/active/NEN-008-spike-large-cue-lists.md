---
id: NEN-008
title: Spike - large cue list across FFI
milestone: M1
size: M
state: active
depends_on: [NEN-007]
blocks: [NEN-011]
adr: [28]
---

# NEN-008 — Spike: large cue list across FFI

## Sonuç

50.000 cue'luk bir dokümanın UI'a hangi modelle (tam liste vs. pencereli erişim)
sunulacağı ölçümle bilinir.

## Kapsam

- Sahte 50k cue üretimi
- İki yaklaşımın karşılaştırması:
  1. Tam listeyi FFI'dan geçirme
  2. Pencere/handle: `cues(range:)`, `activeCue(at:)`
**Baseline olarak raporlanacaklar** (pass/fail eşiği **değil** — bütçe ADR-0027
ile sonradan kabul edilir):

| Ölçüm | Not |
|---|---|
| Toplam geçiş süresi | her iki yaklaşım için |
| p50 / p95 erişim süresi | pencereli erişimde |
| Peak memory / RSS | her iki yaklaşım için |
| UI thread bloklanma süresi | gözlemlenen en uzun blok |

Her ölçümle birlikte **bağlam** kaydedilir: fixture cue sayısı ve veri boyutu ·
cihaz · OS/toolchain sürümü · debug mi release mi build. Bağlamsız sayı kanıt
sayılmaz.

## YAPILMAYACAK

- Gerçek SRT parse → M2
- Renderer entegrasyonu → NEN-027
- Spike kodunun ürüne terfisi — `core/spikes/` altında kalır

## Kanıt (DoD)

- [ ] Baseline tablosu: her iki yaklaşım için süre, p50/p95, peak RSS
- [ ] Ölçüm bağlamı (fixture boyutu, cihaz, OS/toolchain, build tipi) kayıtlı
- [ ] Pencereli erişimin UI thread'i ne kadar blokladığı ölçüldü
- [ ] Hangi yaklaşımın önerildiği ve **neden** yazıldı (sayılarla)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
