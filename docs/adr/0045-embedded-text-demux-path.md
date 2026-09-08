---
adr: 0045
title: Gömülü altyazı metninin demux yolu
status: proposed
milestone: M5
tasks: [NEN-103, NEN-044]
date: —
---

# ADR-0045 — Gömülü altyazı metninin demux yolu

## Durum

`proposed`

## Bağlam

`NEN-023` gömülü track'leri kataloğa bağladı ama metin çıkarımını **kapsam
dışı bıraktı**: libmpv bir subtitle track'inin tam metnini veren bir API
sunmuyor (`sub-text` yalnız o anki cue'dur) ve M3'te bu metni tüketen hiçbir
şey yoktu.

Tüketici M5'te ortaya çıkıyor: çeviri pipeline'ı kaynak altyazının **tam**
metnini ister (`docs/product-spec.md` §9, §10). `NEN-027`'nin injection yolu
harici belgeler içindir, çıkarım için kullanılamaz.

`NEN-044` iki adayı sayıyor:

- **libavformat/libavcodec** — Homebrew mpv'nin zaten getirdiği kütüphaneler,
  Swift'te yeni bir `systemLibrary` hedefi. Bedeli: ikinci bir yerel bağımlılık
  ve `NEN-043`'ün 48 dylib'lik bundling kapanışının büyümesi.
- **Rust konteyner parser'ı** — `nen-identity/src/container.rs`'in beklediği
  MKV/MP4 demux'ü. Yeni yerel bağımlılık yok, her platformda aynı kod; en
  pahalı seçenek.

`NEN-072` bu dosyaya uzak byte penceresinden **dar** bir EBML/ISOBMFF okuması
ekledi (title/year) — tam demux değil, ve bilinçli olarak öyle kaldı.

Karar gerektiren noktalar:

1. Hangi yol seçilir ve neden?
2. `EmbeddedTrackExtractor` portunun sınırı nerededir — motor mu çıkarır, core
   mu parse eder (`docs/architecture.md`: "Embedded text extraction porttur,
   çünkü çıkarımı yapan zaten playback motorudur")?
3. Bitmap track'te çıkarımın reddi hangi katmanda olur?
4. Seçilen yol, ADR-0009 Karar 6'nın bugün boş duran I/O tarafını da karşılar mı
   (`NEN-044` bunu açıkça soruyor)?
5. `NEN-043`'ün bundling yüküne etkisi nedir — ölçülmüş mü, varsayım mı?

## Karar

<!-- Kullanıcı onayıyla doldurulacak. -->

## Gerekçe

<!-- … -->

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| … | … |

## Sonuçlar

**Olumlu:** …

**Olumsuz / kabul edilen maliyet:** …

**Geri dönüş maliyeti:** …

## İlgili task'lar

`NEN-103` (karar) · `NEN-044` (implementasyon)

## Notlar
