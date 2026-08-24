---
id: NEN-015
title: Encoding detection and sanitization
milestone: M2
size: M
state: backlog
depends_on: [NEN-013]
blocks: [NEN-025]
adr: [8]
---

# NEN-015 — Encoding detection and sanitization

## Sonuç

UTF-8 olmayan veya kontrol karakteri içeren altyazı dosyaları doğru okunur ve
zararlı içerikten arındırılır; okunamayanlar typed error ile reddedilir.

## Kapsam

- BOM tespiti (UTF-8, UTF-16LE/BE)
- Legacy code page tespiti (CP1254 Türkçe, CP1252) ve dönüşüm
- Sanitization: kontrol karakterleri, bidi override, sıfır genişlikli karakterler
- Aşırı boyut reddi
- `fixtures/subtitles/encodings/` korpusu

## YAPILMAYACAK

- Otomatik dil tahmini → NEN-020
- Dosya sistemi güvenlik kontrolleri (symlink, traversal) → NEN-025

## Kanıt (DoD)

- [ ] Her encoding fixture'ı doğru metne çözülüyor (golden)
- [ ] Negatif: bidi override / sıfır genişlikli karakter içeren girdi temizleniyor
- [ ] Negatif: boyut sınırını aşan dosya okunmuyor, typed error dönüyor
- [ ] Negatif: çözülemeyen encoding sessizce mojibake üretmiyor, hata veriyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
