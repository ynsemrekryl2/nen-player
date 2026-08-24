---
id: NEN-009
title: Spike - async progress and cancellation
milestone: M1
size: M
state: backlog
depends_on: [NEN-007]
blocks: [NEN-011]
adr: [4]
---

# NEN-009 — Spike: async progress and cancellation

## Sonuç

FFI sınırından geçen uzun bir iş iptal edilebilir, iptalden sonra **hiçbir
callback gelmez** ve kaynak sızmaz.

## Kapsam

- Sahte uzun iş (bloklara bölünmüş, checkpoint'li)
- `onProgress(jobId, phase, done, total)` callback akışı — **cue içeriği yok**
- `JobHandle.cancel()` ile kooperatif iptal
**Baseline olarak raporlanacaklar** (pass/fail eşiği **değil**):

- Cancellation latency (p50 / p95), ölçüm bağlamıyla birlikte
- Checkpoint aralığı ile latency arasındaki ilişki
- Fixture boyutu · cihaz · OS/toolchain · debug/release build

**Invariant'lar (pass/fail — gevşetilemez, K15'in doğrudan karşılığı):**

| # | Invariant |
|---|---|
| I1 | İptalden sonra **hiçbir callback gelmez** (sayı = 0) |
| I2 | İptalden sonra **late commit yok** |
| I4 | Tekrarlı iptal ve shutdown sonrası **thread/bellek sızıntısı yok** |

## YAPILMAYACAK

- Gerçek çeviri pipeline'ı → M5
- Progress'in UI'da gösterimi → M3+

## Kanıt (DoD)

- [ ] **I1** — Swift testi: 5 sn'lik işi 1. sn'de iptal ediyor, late callback yok
- [ ] **I2** — Negatif: iptal sonrası commit denemesi engelleniyor
- [ ] **I4** — Tekrarlı iptal (100 kez) sonrası thread/bellek sızıntısı yok
- [ ] Baseline: cancellation latency p50/p95 + ölçüm bağlamı kayıtlı

## Kanıt kaydı

<!-- done olurken doldurulacak -->
