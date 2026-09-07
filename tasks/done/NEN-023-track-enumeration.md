---
id: NEN-023
title: Embedded track enumeration and selection
milestone: M3
size: M
state: done
closed: 2026-08-26
depends_on: [NEN-022]
blocks: []
adr: [11, 32]
---

# NEN-023 — Embedded track enumeration and selection

## Sonuç

Medyadaki gömülü audio ve subtitle track'leri listelenir ve seçilebilir; subtitle
track'leri kataloğa girer ve menü projeksiyonunda görünür; bitmap altyazı
track'leri **çevrilemez** olarak işaretlenir.

## Kapsam

- Audio ve subtitle track enumeration (dil, başlık, codec, varsayılan bayrağı)
- Track seçimi
- Gömülü subtitle track'lerinin `SubtitleSourceCatalog`'a girmesi ve seçilen
  katalog girdisinin tekrar bir `TrackId`'ye çözülmesi (ADR-0031 Karar 4 menünün
  açıldığı anda gömülü track'leri göstermeyi şart koşuyor)
- Bitmap (PGS/VobSub/DVB) track tespiti ve `translatable = false` işareti —
  sınıflandırma codec üzerinden ve **Rust'ta**, her platform aynı listeyi kullanır
- Track başlığının FFI'yı geçmesi (menünün etiketi), K23 #8 guard'ıyla birlikte

## YAPILMAYACAK

- **Embedded text extraction → `NEN-044`.** Motorun bir subtitle track'inin tam
  metnini veren API'si yok; gerçek çıkarım konteyneri demux etmeyi gerektirir ve
  kendi ADR'sini ister. M3'te bu metni tüketen hiçbir şey yok (gömülü track
  seçilince motor kendisi çiziyor), şartname §7 de extraction'ı "yapılabilir"
  diyerek zorunlu kılmıyor. **Lazy kuralının kanıtı bu task'ta kalır:** katalog
  kurulurken `extract_text` çağrılmadığı negatif kontrolle gösterilir.
- Metnin katalogda gösterimi → NEN-026
- Bitmap OCR — kapsam dışı
- Track metinlerinin baştan çıkarılması — **lazy kuralı** (§7)

## Kanıt (DoD)

- [x] Çok track'li fixture'da doğru audio/subtitle track listesi
- [x] Track seçimi playback'e yansıyor
- [x] Bitmap track `translatable = false` olarak işaretleniyor
- [x] Katalog açılışında hiçbir track metni çıkarılmıyor (lazy kanıtı)

## Kanıt kaydı

### Özet

Gömülü track'ler artık ürün tarafına bağlı: `nen-app::embedded` bir medyanın
subtitle track'lerini `SubtitleSourceCatalog` girdilerine çeviriyor ve seçilen
girdiyi tekrar bir `TrackId`'ye çözüyor. Bitmap ayrımı Swift'ten alınıp
`nen-ports`'a taşındı, track başlığı FFI'yı geçti, `SubtitleSource` bir
`translatable` işareti kazandı.

**Test sayısı: Rust 406 → 438, Swift 9 → 16.** Yeni dış bağımlılık yok;
`core/deny.toml` ve hiçbir `Cargo.toml` değişmedi (`cargo deny check`:
`advisories ok, bans ok, licenses ok, sources ok`).

### DoD #1 — çok track'li fixture'da doğru liste

`TrackEnumerationTests.aMultiTrackContainerReportsEveryTrackOfEachKind`,
gerçek libmpv 2.5.0 ile `fixtures/media/contract-clip.mkv` üzerinde:

```
audio:    id=[1, 2]  lang=[eng, tur]  codec=opus    default=[true, false]
subtitle: id=[3, 4]  lang=[eng, tur]  codec=subrip  default=[true, false]
```

`theVideoTrackIsNeitherAudioNorSubtitle` ayrıca video track'inin hiçbir kind'a
sızmadığını sabitliyor: konteyner istediği track türünü uydurabilir, port'un
karşılığı olmayan her şey **görünmemelidir**.

Başlık iki yönde de kanıtlı: `bitmap-subs-clip.mkv`'nin 2 numaralı track'i
`title == "English"` veriyor, `contract-clip.mkv`'nin hiçbir track'i başlık
beyan etmiyor (`title == nil`).

### DoD #2 — seçim playback'e yansıyor

`everySubtitleTrackCanBeSelectedAndReadBack` her subtitle track'ini tek tek
seçip `selectedTrack`'ten geri okuyor — **bitmap track dahil**.
`selectingNothingIsWhatTheClosedEntryDoes` menünün `Kapalı` girdisinin
karşılığını (`nil` → seçim yok) kanıtlıyor.
`audioAndSubtitleSelectionsDoNotDisturbEachOther` id uzayının tek olmasından
doğan riski kapatıyor: audio 2 ve subtitle 3 aynı anda seçili kalıyor.

Rust tarafında `embedded_sources` ↔ `track_of` round-trip'i
`every_source_resolves_back_to_the_track_it_came_from` ile kilitli.

### DoD #3 — bitmap `translatable = false`

Sınıflandırma **codec'e göre ve Rust'ta**: `nen_ports::playback::
subtitle_carries_text`. Swift'teki `isTextCodec` **silindi** — adapter artık
hiçbir şey sınıflandırmıyor, yalnız konteynerin dediği codec'i raporluyor.
`FfiTrackDescriptor.is_text` alanı da kalktı: yanlış olabilecek bir alan değil,
çünkü gönderilen bir alan değil.

`aBitmapTrackIsReportedByItsCodec` gerçek motorda menteşeyi sabitliyor:
`codec == ["subrip", "hdmv_pgs_subtitle"]`. Rust tarafında dört bitmap codec'i
(`hdmv_pgs_subtitle` · `dvd_subtitle` · `dvb_subtitle` · `xsub`) ve beş metin
codec'i ayrı ayrı test ediliyor; **bilinmeyen codec metin sayılmıyor** ve audio
track'i hiçbir codec adıyla metin taşımıyor.

### DoD #4 — katalog açılışında metin çıkarılmıyor

`tests/embedded_lazy.rs`: `extract_text` çağrılınca **panikleyen** bir
`TripwireEngine` üzerinden `tracks()` → `embedded_sources` → `project` yolu
baştan sona koşuyor. Motor `EmbeddedTextExtraction` capability'sini **beyan
ediyor** — yani yol metni isteseydi alabilirdi, koruma capability'nin
yokluğundan gelmiyor.

Boşta dönmemesi üç şekilde güvenceye alındı: `track_queries == 1` (yol gerçekten
koştu), `catalog.len() == 2`, ve `the_tripwire_fires_when_something_does_decode`
tuzağın gerçekten patladığını gösteriyor.

**Dürüst sınır:** lazy garantisinin asıl kaynağı yapısaldır —
`embedded_sources` bir motor değil, bir `&[TrackDescriptor]` alıyor, yani
soracağı kimse yok. Test bu şekli koruyor: çıkarım NEN-044 ile geldiğinde bu
yolun onu çağırmadığı kırmızı testle anlaşılır.

### Ölçüm bir kusur buldu → ADR-0032

Track listesi ilk kez gerçek fixture'dan okununca çıktı: **Matroska ISO 639-2
yazıyor**, yani libmpv `eng` · `tur` · `fre` veriyor. `LanguageTag::parse`
2–3 harfli primary'yi geçerli saydığı için `eng` hatasız ayrışıyor ve `en`'den
**farklı** bir etiket oluyor.

Üç yerde birden kırılıyordu: (1) kullanıcının `Movie.en.srt`'si `en`, gömülü
track `eng` → menüde **iki ayrı İngilizce grubu**; (2) tercihi `en` olan
kullanıcı için hiçbir gömülü track eşleşmiyor, yani ADR-0010 Karar 9'un
otomatik seçimi gömülü katmanı hiç göremiyor; (3) ISO 639-2 bazı dilleri iki
kez yazıyor (`fre`/`fra`, `ger`/`deu`) — aynı dil, aynı konteyner ailesinde iki
yazım.

Bu ADR-0030'un çözdüğü problemin aynısı, başka bir eksende. Kural 4 gereği
[`ADR-0032`](../../docs/adr/0032-container-language-codes.md) yazıldı ve
kullanıcı onayıyla `accepted` oldu: ISO 639-2 → 639-1 indirgeme
`LanguageTag::parse`'ın **kendisinde** yapılıyor (konteyner sınırında değil —
aynı kod `.nfo`, dosya adı ve ileride OpenSubtitles kapılarından da giriyor).
Karşılığı olmayan kod (`fil`, `haw`, `nds`) **olduğu gibi kalıyor**; hiçbir şey
tahmin edilmiyor, hiçbir şey kesilmiyor.

Tablo 204 satır, 184 ISO 639-1 dili ve 20 /B–/T çifti kapsıyor.

### Fixture — `bitmap-subs-clip.mkv`

Bitmap track'i **ffmpeg üretemedi**: `Subtitle encoding currently only possible
from text to text or bitmap to bitmap`. Bu yüzden minimal bir HDMV PGS akışı
(PCS · WDS · PDS · ODS · END + temizleyen ikinci display set) elle üretilip
`-c:s copy` ile muxlandı. Üretici deterministik — sabit alanlar, rastgelelik
yok. Reçete `fixtures/media/bitmap-subs-clip.ffmpeg.txt`'de ve **çalıştığı
doğrulandı**: depodaki dosya o reçeteden üretildi.

İçerik: 15 s · 1 audio (eng) · 1 `subrip` (eng, başlık `English`) ·
1 `hdmv_pgs_subtitle` (fre).

### Negatif kontrol — beş mekanik bozma

Hepsi `cargo test --workspace --no-fail-fast` ile ölçüldü, hepsi geri alındı.

| Bozma | Kırmızıya dönen test |
|---|---|
| `subtitle_carries_text` her codec'e `true` | **5** |
| `canonical_primary` daima `None` (ADR-0032 devre dışı) | **6** |
| Elle yazılmış `Debug` yerine `#[derive(Debug)]` (FFI) | **3** |
| `embedded_sources` bitmap işaretini taşımıyor | **2** |
| `track_of` ters eşlemeyi çözmüyor | **1** |

Ölçümün kendisi bir kusur buldu: ilk turda `cargo test` **ilk kırmızı hedefte
duruyor**, yani sonraki crate'ler hiç koşmuyordu ve K1 yalnız 1 kırmızı test
raporlamıştı. `--no-fail-fast` ile gerçek sayı 5 çıktı. İkinci bir ölçüm hatası
da düzeltildi: kırmızı testleri toplayan grep'in deseni rakam içermiyordu, bu
yüzden `iso_639_2_codes_become_their_iso_639_1_equivalent` sayılmıyordu.

### Güvenlik (K23 #8)

Track başlığı FFI'yı geçen **ikinci** string oldu (ilki locator).
`FfiTrackDescriptor`'ın `#[derive(Debug)]`'ı kaldırıldı, elle `Debug` yazıldı
(`has_title: bool` basıyor, başlığı asla basmıyor) ve
`tests/guard_ffi_track_debug.rs` bunu kasıtlı `#[derive(Debug)]`'lı ikizle
kanıtlıyor — ikiz dört yasak parçanın **hepsini** sızdırıyor.

**Kapsanmayan taraf açıkça yazıldı:** Swift'te `FfiTrackDescriptor` üretilmiş
düz bir struct, yani `String(reflecting:)` başlığı **gerçekten** basar. Bunu
test yasaklayamaz. Önce buna zayıf bir test yazıldı, sonra kaldırıldı — boşta
dönen bir testle örtmek, sınırı yazmaktan kötü. `RedactionTests`'in doküman
yorumu şimdi neyin tuttuğunu (inbound-only, Rust tarafındaki guard) ve neyin
disiplin olduğunu (descriptor loglanmaz; adapter bugün hiçbir şey loglamıyor)
ayrı ayrı söylüyor.

### Kapsam kararları

1. **Extraction `NEN-044`'e taşındı** (kullanıcı onayıyla). libmpv bir subtitle
   track'inin tam metnini veren API sunmuyor; gerçek çıkarım konteyneri demux
   etmeyi gerektiriyor ve kendi ADR'sini istiyor. M3'te bu metni tüketen hiçbir
   şey yok — gömülü track seçilince motor kendisi çiziyor — ve şartname §7
   extraction'ı "yapılabilir" diyerek zorunlu kılmıyor. Lazy kuralının kanıtı
   bu task'ta kaldı.
2. **Eşleme `nen-catalog`'a değil `nen-app`'e kondu.** ADR-0006'nın crate
   tablosu `nen-catalog`'a `domain, subtitle, identity` veriyor — `ports` yok.
   `nen-app` ikisine de bağlı ve "motorun dediğini use-case değerine çevirmek"
   tam olarak o katmanın işi. ADR değişikliği gerekmedi.
3. **Başlıksız track'in etiketi boş.** ADR-0010 Karar 7 gereği dilin görünen
   adı UI'ın işi ve o dilin kendi adı; burada bir etiket uydurmak, onu tek bir
   dilde uydurup her dilde göstermek olurdu.
4. **Çevrilemez ≠ seçilemez.** §7 bitmap track'in gösterilebileceğini söylüyor.
   `a_bitmap_track_is_still_auto_selected` ve
   `a_translatable_track_does_not_outrank_a_bitmap_one_of_the_same_kind`
   `translatable`'ın otomatik seçime sızmadığını sabitliyor.
5. **Mevcut üç menü golden'ı byte-eşit kaldı.** Renderer bitmap işaretini
   yalnız istisnai duruma basıyor, sıradan kaynak hiçbir şey söylemiyor —
   ADR-0010 Karar 10'un sabitlediği çıktılar bu yüzden değişmedi.

### Doğrulama

```
cargo fmt --check           temiz
cargo clippy -D warnings    uyarı yok
cargo test --workspace      438 test, 0 başarısız
cargo deny check            advisories ok, bans ok, licenses ok, sources ok
bash scripts/test-macos.sh  16 test, 3 suite, geçti (libmpv 2.5.0)
bash scripts/test.sh        2 test dosyasının hepsi geçti
```
