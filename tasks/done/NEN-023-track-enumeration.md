---
id: NEN-023
title: Embedded track enumeration and selection
milestone: M3
size: M
state: backlog
depends_on: [NEN-022]
blocks: []
adr: [11]
---

# NEN-023 — Embedded track enumeration and selection

## Sonuç

Medyadaki gömülü audio ve subtitle track'leri listelenir ve seçilebilir; bitmap
altyazı track'leri **çevrilemez** olarak işaretlenir.

## Kapsam

- Audio ve subtitle track enumeration (dil, başlık, codec, varsayılan bayrağı)
- Track seçimi
- Embedded text extraction capability — **lazy**: metin yalnız seçimde veya AI
  talebinde çıkarılır
- Bitmap (PGS/VobSub) track tespiti ve `translatable = false` işareti

## YAPILMAYACAK

- Metnin katalogda gösterimi → NEN-026
- Bitmap OCR — kapsam dışı
- Track metinlerinin baştan çıkarılması — **lazy kuralı** (§7)

## Kanıt (DoD)

- [ ] Çok track'li fixture'da doğru audio/subtitle track listesi
- [ ] Track seçimi playback'e yansıyor
- [ ] Bitmap track `translatable = false` olarak işaretleniyor
- [ ] Katalog açılışında hiçbir track metni çıkarılmıyor (lazy kanıtı)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
