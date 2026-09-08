---
adr: 0045
title: Gömülü altyazı metninin demux yolu
status: accepted
milestone: M5
tasks: [NEN-103, NEN-044]
date: 2026-09-08
---

# ADR-0045 — Gömülü altyazı metninin demux yolu

## Durum

`accepted`

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
  Swift'te yeni bir `systemLibrary` hedefi. Mevcut `libmpv` dylib'inin geçişli
  kapanışında bu iki kütüphane zaten bulunuyor; doğrudan linkleme yeni bir
  bağımlılık ailesi eklemiyor.
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

1. **Demux yolu:** `libavformat` ve `libavcodec` seçildi. NEN-044'te macOS
   `PlaybackEngine` adapter'ı seçili medyanın container'ını açacak, ilgili
   subtitle stream'ini bulacak ve cue metnini çıkaracaktır. Adapter için
   Swift'te ayrı bir `systemLibrary` yüzeyi açılabilir; mevcut libmpv
   linkleme ve bundle kapanışı yeniden kullanılacaktır.
2. **Port sınırı:** `EmbeddedTrackExtractor`, `PlaybackEngine` capability-gated
   operasyonu olarak kalır. Motor/container I/O'su adapter'a aittir; core
   yalnız dönen metni `SubtitleDocument`'a parse eder. Application katmanı
   motor adına göre dallanmaz.
3. **Bitmap reddi:** Codec sınıflandırmasının kanonik kaynağı
   `nen-ports::TrackDescriptor::is_text`'tir. Bitmap track katalogda görünür
   fakat `translatable = false` olur ve çeviri talebi oluşturulmaz. Adapter,
   doğrudan gelen hatalı bir çıkarım çağrısını da metin olmayan track için
   tipli bir red olarak sonlandırır; boş metin başarı sayılmaz.
4. **ADR-0009 Karar 6'nın I/O tarafı:** Seçilen adapter yolu yerel medya için
   container okumasının I/O'sunu karşılayabilir; bu I/O `nen-identity`'ye
   taşınmaz. `nen-identity` saf kalır ve kendisine verilen byte/metadata'yı
   işler. Uzak medya için bounded byte-window evidence yolu (`NEN-072`) aynı
   kalır; tam demuxer onu genişletmez.
5. **Bundling etkisi:** Ölçülen mevcut `libmpv` bağımlılık kapanışı hem
   `libavformat` hem `libavcodec` içeriyor. Bu nedenle NEN-044'te doğrudan
   linkleme, NEN-043'ün kapanışına yeni bir dylib ailesi eklememelidir. Son
   bundle manifesti ve `otool -L` taraması implementasyon task'ının kanıtıdır;
   bu ADR ölçülmemiş bir sabit dylib sayısı vaat etmez.

## Gerekçe

`libmpv` zaten FFmpeg demux/decode katmanını kullanıyor ve sistemdeki
`pkg-config --libs libavformat libavcodec` çıktısı ile `otool -L
/opt/homebrew/lib/libmpv.dylib` ölçümü bunu doğruluyor. Aynı container ailesi
üzerinde ikinci bir tam parser yazmak yerine motor adapter'ının hazır codec
adlandırmasını ve format desteğini kullanmak, mevcut port sınırını koruyor ve
`NEN-044`ün gerçek işi olan lazy text extraction'a odaklanıyor.

Bu seçim platform adapter'ını değiştirilebilir bırakır: ileride başka bir
platform aynı `EmbeddedTrackExtractor` kontratını kendi decoder'ıyla
uygulayabilir. Core'un I/O'suzluğu ve metin parse sorumluluğu değişmez.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Rust konteyner parser'ı | MKV/MP4 için tam demux, codec decode ve stream/cue ayrıştırmasının yeni ve platformlar arası bir bakım yüzeyi açması; mevcut libmpv'nin zaten taşıdığı yeteneği yeniden üretmesi |
| `libmpv`'nin anlık `sub-text` değerini biriktirmek | Yalnız o anda çizilen cue'yu verir; tam dokümanı, seçilmemiş track'i ve çeviri için gerekli tüm zaman aralıklarını güvenilir biçimde çıkaramaz |
| `nen-identity` içinde tam demux ve I/O | ADR-0009 Karar 2'nin saf/I/O'suz crate sınırını ihlal eder ve uzak evidence yolunu container parser'ıyla karıştırır |

## Sonuçlar

**Olumlu:** Tam metin çıkarımı mevcut playback adapter'ının gerçek container
desteğiyle yapılır; core portu ve lazy katalog davranışı korunur; bitmap
track'ler sessizce boş belgeye dönüşmez.

**Olumsuz / kabul edilen maliyet:** macOS build'i FFmpeg header/modulemap ve
system-library ayarlarını taşır; libav sürüm farkları adapter contract/golden
testleriyle izlenir. Bundle kapanışı her implementasyon değişikliğinde yeniden
ölçülür.

**Geri dönüş maliyeti:** `EmbeddedTrackExtractor` portu ve core parse yüzeyi
değişmeden adapter içindeki demux implementasyonu başka bir decoder'a
değiştirilebilir; ancak Swift binding ve fixture testleri yeniden yazılır.

## İlgili task'lar

`NEN-103` (karar) · `NEN-044` (implementasyon)

## Notlar
