---
id: NEN-016
title: SubtitleDocument and timeline fingerprint
milestone: M2
size: M
state: backlog
depends_on: [NEN-013]
blocks: [NEN-017, NEN-019]
adr: [7]
---

# NEN-016 — SubtitleDocument and timeline fingerprint

## Sonuç

Bir altyazının **zamanlaması** ile **içeriği** ayrı ayrı parmak izlenir; aynı
timeline'dan türeyen çeviriler aynı timeline fingerprint'ini paylaşır.

## Kapsam

- `SubtitleDocument` modeli
- `TimelineFingerprint` — yalnız cue zamanlarından
- `SourceFingerprint` — içerik (metin dahil) dahil
- Fingerprint'lerin kararlılık ve duyarlılık testleri

## YAPILMAYACAK

- Cache identity'nin tamamı → M5 / ADR-0018
- SyncProfile → M7 (yalnız fingerprint burada üretilir)
- Media fingerprint → NEN-018

## Kanıt (DoD)

- [ ] Aynı timeline → aynı `TimelineFingerprint`
- [ ] Tek cue 1 ms kayınca `TimelineFingerprint` **değişiyor**
- [ ] Metin değişip zamanlar aynı kalınca `TimelineFingerprint` **değişmiyor**,
      `SourceFingerprint` **değişiyor**
- [ ] Cue sırası değişince fingerprint değişiyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
