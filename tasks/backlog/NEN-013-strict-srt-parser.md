---
id: NEN-013
title: Strict SRT parser
milestone: M2
size: M
state: backlog
depends_on: [NEN-012]
blocks: [NEN-014, NEN-015, NEN-016, NEN-020, NEN-025]
adr: [7]
---

# NEN-013 — Strict SRT parser

## Sonuç

Geçerli SRT dosyaları doğru `SubtitleDocument`'a dönüşür; bozuk her dosya
**typed error** ile reddedilir, hiçbiri sessizce kabul edilmez.

## Kapsam

- Strict SRT parse: index, zaman satırı, çok satırlı metin, blok ayrımı
- Her bozukluk için ayrı typed error varyantı
- `fixtures/subtitles/valid/` ve `malformed/` korpusu (her dosya **tek** bozukluk)
- Fuzz smoke testi (panik yok, `unwrap` yok)

## YAPILMAYACAK

- Encoding tespiti → NEN-015 (bu task UTF-8 girdi varsayar)
- WebVTT yazımı → NEN-014
- Toleranslı/kurtarıcı parse — strict, bilinçli tercih

## Kanıt (DoD)

- [ ] Geçerli korpus golden testleri geçiyor
- [ ] ≥20 malformed vaka, her biri **beklenen typed error** varyantını döndürüyor
- [ ] Fuzz smoke: rastgele/kesik girdide panik yok
- [ ] Kod içinde `unwrap`/`expect` yok (lint kanıtı)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
