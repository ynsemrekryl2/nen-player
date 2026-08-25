---
id: NEN-020
title: Subtitle language detection with confidence threshold
milestone: M2
size: S
state: active
depends_on: [NEN-013]
blocks: []
adr: [10, 29]
---

# NEN-020 — Subtitle language detection with confidence threshold

## Sonuç

Altyazı metninden dil tespit edilir; güven eşiğini geçemeyen kaynak yanlış bir
dile atanmak yerine `Dil Belirsiz` grubuna düşer.

## Kapsam

- Metin tabanlı dil tespiti
- Güven eşiği ve eşik altı davranışı
- Metadata'da dil etiketi varsa onun önceliği ve çelişki durumu
- Çok dilli fixture seti

## YAPILMAYACAK

- Otomatik çeviri tetikleme — dil tespiti çeviri **başlatmaz**
- Ağ tabanlı dil servisi

## Kanıt (DoD)

- [ ] Çok dilli fixture'larda doğru dil tespiti (golden)
- [ ] Kısa/karışık metinde eşik altı → `Dil Belirsiz`
- [ ] Metadata etiketi ile tespit çeliştiğinde tanımlı davranış test edildi

## Kanıt kaydı

<!-- done olurken doldurulacak -->
