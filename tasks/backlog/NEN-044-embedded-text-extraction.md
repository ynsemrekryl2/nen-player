---
id: NEN-044
title: Embedded subtitle text extraction
milestone: M5
size: M
state: backlog
depends_on: [NEN-023, NEN-103]
blocks: [NEN-104]
adr: [0045]
---

# NEN-044 — Embedded subtitle text extraction

## Sonuç

Gömülü bir metin altyazı track'inin **tam** metni, yalnız seçimde veya AI
talebinde, bir `SubtitleDocument` olarak çıkarılabilir; adapter
`EmbeddedTextExtraction` capability'sini beyan eder.

## Bağlam

`NEN-023` gömülü track'leri kataloğa bağladı fakat metin çıkarımını **kapsam
dışı bıraktı**: libmpv bir subtitle track'inin tam metnini veren bir API
sunmuyor (`sub-text` yalnız o anki cue'dur), gerçek çıkarım konteyneri demux
etmeyi gerektiriyor ve M3'te bu metni tüketen hiçbir şey yoktu — gömülü track
seçilince motor kendisi çiziyor.

Tüketici M5'te ortaya çıkıyor: çeviri, kaynak altyazının metnini ister
(şartname §9, §10). `NEN-027`'nin injection yolu harici belgeler içindir.

## Ön koşul — ADR-0045

Demux yolu **kararlaştırılmamıştır** ve kod yazılmadan önce bir ADR ister.
Karar `NEN-103`'ün konusudur (ADR-0045); bu task o ADR `accepted` olmadan
başlatılmaz. Bilinen adaylar:

- **libavformat/libavcodec** — Homebrew mpv'nin zaten getirdiği kütüphaneler,
  Swift'te yeni bir `systemLibrary` hedefi. Bedeli: ikinci bir yerel bağımlılık
  ve `NEN-043`'ün bundling yükünün büyümesi.
- **Rust konteyner parser'ı** — `nen-identity/src/container.rs`'in zaten
  beklediği MKV/MP4 demux'ü. Yeni yerel bağımlılık yok, her platformda aynı kod;
  en pahalı seçenek.

ADR ayrıca ADR-0009 Karar 6'nın (container metadata katmanı) bugün boş duran
I/O tarafını aynı demux'ün karşılayıp karşılamayacağını değerlendirmelidir.

## Kapsam

- Seçilen demux yolunun implementasyonu
- `PlaybackEngine::extract_text` — macOS adapter'ında gerçek implementasyon
- `Capability::EmbeddedTextExtraction`'ın beyan edilmesi; contract kitinin
  capability'li kolunun gerçek adapter'da da koşması
- Bitmap track'te çıkarımın **reddi** (`translatable = false` olan track)

## YAPILMAYACAK

- Bitmap OCR — kapsam dışı
- Katalog kurulurken eager çıkarım — `NEN-023`'ün lazy negatif kontrolü geçerli
  kalır
- Çıkarılan metnin loglanması — K23 #4, subtitle diyaloğu

## Kanıt (DoD)

- [ ] ADR-0045 `accepted` (`NEN-103`)
- [ ] Fixture'ın metin track'inin çıkarılan metni golden ile eşleşiyor
- [ ] Bitmap track'te çıkarım tipli hata ile reddediliyor
- [ ] `NEN-023`'ün lazy negatif kontrolü hâlâ yeşil — çıkarım yalnız açıkça
      istendiğinde koşuyor
- [ ] Guard: çıkarılan metin hiçbir log yüzeyine düşmüyor (negatif kontrolle)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
