---
id: NEN-072
title: Parse container metadata from remote byte windows
milestone: M5
size: M
state: backlog
depends_on: [NEN-036]
blocks: []
adr: []
---

# NEN-072 — Parse container metadata from remote byte windows

## Sonuç

Uzak medyanın bounded byte pencerelerinden Matroska/MP4 container metadata'sı
çıkarılır ve `MediaEvidence` içindeki container alanına taşınır.

## Kapsam

- İlk byte penceresinden Matroska/MP4 metadata ayrıştırma
- Boyut, derinlik ve malformed input sınırları
- Deterministic fixture ve negatif parser testleri

## YAPILMAYACAK

- Medyanın tamamını indirmek
- Playback demuxer'ını yeniden yazmak
- Torrent/debrid metadata'sı okumak

## Kanıt (DoD)

- [ ] Matroska ve MP4 title/year fixture'ları `ContainerMetadata`'ya dönüşüyor
- [ ] Bounded ve malformed byte input reddediliyor veya güvenle boş dönüyor
- [ ] Negatif: oversized/deep/malformed container input paniklemiyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
