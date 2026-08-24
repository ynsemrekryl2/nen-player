---
id: NEN-014
title: WebVTT writer
milestone: M2
size: S
state: backlog
depends_on: [NEN-013]
blocks: []
adr: [7]
---

# NEN-014 — WebVTT writer

## Sonuç

`SubtitleDocument`, byte düzeyinde kararlı UTF-8 WebVTT olarak yazılır.

## Kapsam

- UTF-8 WebVTT çıktısı (şartname §10 gereği final format)
- Zaman formatı, cue ayırıcı, metin kaçışları
- SRT → doc → WebVTT round-trip golden snapshot

## YAPILMAYACAK

- WebVTT **okuma** (parse) — şu an ihtiyaç yok, gerekirse ayrı task
- Stil/konum (cue settings) — kaynakta yoksa üretilmez

## Kanıt (DoD)

- [ ] Round-trip golden: SRT → WebVTT snapshot byte-eşit
- [ ] Çıktı UTF-8 ve BOM'suz
- [ ] Cue ID/sıra/zamanlar girdiyle **aynen** aynı

## Kanıt kaydı

<!-- done olurken doldurulacak -->
