---
id: NEN-103
title: Decide the demux path for embedded subtitle text
milestone: M5
size: S
state: backlog
closed:
depends_on: [NEN-089]
blocks: [NEN-044]
adr: [0045]
---

# NEN-103 — Decide the demux path for embedded subtitle text

## Sonuç

Gömülü bir metin altyazı track'inin tam metnine hangi yoldan ulaşılacağı
`accepted` bir ADR ile sabitlendi.

## Bağlam

`NEN-044` gövdesi bu kararı kendi ön koşulu olarak yazıyor ve iki adayı
sayıyor: **libavformat/libavcodec** (Homebrew mpv'nin zaten getirdiği
kütüphaneler; bedeli ikinci bir yerel bağımlılık ve `NEN-043`'ün bundling
yükünün büyümesi) ile **Rust konteyner parser'ı** (yeni yerel bağımlılık yok,
her platformda aynı kod; en pahalı seçenek).

Karar `NEN-089`'dan sonra verilir: çıkarılan metnin tüketicisi çeviri
pipeline'ıdır, ve bloklara nasıl girdiği bilinmeden çıkarım yüzeyi seçilemez.

`nen-identity/src/container.rs` (`NEN-072`) uzak byte penceresinden **dar** bir
EBML/ISOBMFF okuması yapıyor — tam demux değil. ADR, ADR-0009 Karar 6'nın
bugün boş duran I/O tarafını aynı yolun karşılayıp karşılamayacağını da
değerlendirmelidir (`NEN-044` bunu açıkça istiyor).

## Kapsam

- ADR-0045: demux yolu, `EmbeddedTrackExtractor` portunun sınırı, bitmap
  track'in reddi, `NEN-043`'ün bundling yüküne etkisi
- `docs/architecture.md` → `EmbeddedTrackExtractor` satırının karara göre
  netleştirilmesi
- En az bir reddedilen alternatifin gerekçesi

## YAPILMAYACAK

- Implementasyon — `NEN-044`
- Bitmap OCR — kapsam dışı (`NEN-044`'ün YAPILMAYACAK'ı)
- `nen-identity/src/container.rs`'in tam demux'e genişletilmesi — kararın konusu,
  sonucu değil

## Kanıt (DoD)

- [ ] ADR-0045 `accepted` (kullanıcı onayı alınmış)
- [ ] `NEN-044`'ün `adr:` alanı ve "Ön koşul — ADR" bölümü karara bağlandı
- [ ] `bash scripts/check-docs.sh` çıkış 0

## Kanıt kaydı

<!-- done olurken doldurulacak -->
