---
id: NEN-011
title: Spike - Kotlin binding parity
milestone: M1
size: M
state: backlog
depends_on: [NEN-008, NEN-009, NEN-010]
blocks: [NEN-012]
adr: [3]
---

# NEN-011 — Spike: Kotlin binding parity

## Sonuç

NEN-008/009/010'daki üç ölçümün Kotlin/JVM binding'indeki sonuçları bilinir ve
Swift'ten sapmalar belgelidir.

## Kapsam

- Aynı üç spike'ın Kotlin/JVM karşılığı
- `suspend` fonksiyon davranışı, coroutine iptali ile `JobHandle.cancel()` uyumu
- Sapmaların tablo halinde kaydı

## YAPILMAYACAK

- Android cihazda çalıştırma → M10 (bu spike JVM üzerinde)
- Media3 / Android UI → M10

## Kanıt (DoD)

- [ ] Üç testin JVM sonuçları Swift sonuçlarıyla tablo halinde karşılaştırıldı
- [ ] Coroutine iptali sonrası **late callback yok**
- [ ] Sapma varsa sebebi ve M10'a etkisi yazıldı

## Kanıt kaydı

<!-- done olurken doldurulacak -->
