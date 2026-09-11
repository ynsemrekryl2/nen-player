---
id: NEN-044
title: Embedded subtitle text extraction
milestone: M5
size: M
state: done
closed: 2026-09-11
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

Demux yolu ADR-0045 ile kararlaştırıldı; bu task o ADR `accepted` olmadan
başlatılmaz. Kabul edilen yol:

- **libavformat/libavcodec** — Homebrew mpv'nin zaten getirdiği kütüphaneler,
  Swift'te adapter'a bağlanır; mevcut libmpv kapanışında zaten bulunan
  bağımlılıklar yeniden kullanılır.

Rust konteyner parser'ı, tam demux/decode bakım yüzeyi açtığı için ADR-0045'te
reddedildi.

ADR ayrıca ADR-0009 Karar 6'nın yerel container I/O tarafının adapter ile
karşılanabileceğini, fakat `nen-identity`'nin I/O'suz kalacağını sabitledi.

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

- [x] ADR-0045 `accepted` (`NEN-103`)
- [x] Fixture'ın metin track'inin çıkarılan metni golden ile eşleşiyor
- [x] Bitmap track'te çıkarım tipli hata ile reddediliyor
- [x] `NEN-023`'ün lazy negatif kontrolü hâlâ yeşil — çıkarım yalnız açıkça
      istendiğinde koşuyor
- [x] Guard: çıkarılan metin hiçbir log yüzeyine düşmüyor (negatif kontrolle)

## Kanıt kaydı

### Özet

Gömülü bir metin track'inin tam metni artık `libavformat`/`libavcodec`
üzerinden (ADR-0045) çıkarılabiliyor: yeni `EmbeddedTextExtractor.swift`
(macOS adapter), seçili medyanın container'ını **ikinci kez, mpv'nin
handle'ından bağımsız** açıp ilgili subtitle stream'ini demux/decode ediyor
ve kanonik SRT olarak `PlaybackSession::prepare_embedded_document`
(`nen-app`, yeni) → `nen_subtitle::srt::parse` (NEN-013) yoluna veriyor.
`docs/STATUS.md`'nin NEN-102 kapanışında kaydettiği canlı kusur — gömülü
İngilizce track'in `NoDocument` ile kalıcı reddi — bu yolla kapandı:
`SubtitleLibrary::attach_embedded_document` idempotent biçimde belgeyi
takar, `translation::prepare` artık aynı token için `NoDocument` vermiyor.

**Kullanıcı kararıyla üç uygulama noktası kapandı** (plan onayı sırasında):
tüm metin codec'leri libavcodec decode yoluyla (ASS event → düz metin dahil,
yalnız subrip değil); bitmap/metin-taşımayan track için yeni, payload'suz
`PlaybackError::TrackCarriesNoText` varyantı (port + FFI + Swift, mevcut
`UnknownTrack`'ten farklı — "id yok" ile "id'de metin yok" ayrı anlam
taşıyor); yalnız yerel dosya — uzak (Stremio HTTP) locator `Unsupported`
ile döner, iptal edilemez sınırsız indirme riski `NEN-109`'a (backlog, M6)
ayrıldı (Kural 5). ADR-0045 → Notlar bu üç kararı ve ek olarak taşıma
biçiminin **kanonik SRT** olduğunu ve çıkarımın session kilidinin **dışında**
çalıştığını kayda geçirdi.

**Test sayısı:** Rust workspace 832 → **846** (+14: `nen-ports` guard'ı
genişletildi + `nen-app::embedded_extraction.rs` 7 + `nen-ffi::
embedded_document.rs` 5 + `nen-ffi::guard_ffi_embedded_document_debug.rs` 2).
Swift `bash scripts/test-macos.sh` 256/31 suite → **264 test / 33 suite**
(+8: `EmbeddedTextExtractionTests` 6 + "Embedded document preparation
(NEN-044)" 2). Yeni dış Rust bağımlılığı yok (`cargo deny check`:
`advisories ok, bans ok, licenses ok, sources ok`, `Cargo.lock` diff'i boş).

### Katmanlar

**`nen-ports`:** `PlaybackError::TrackCarriesNoText` (payload'suz) +
`ErrorKind` eşlemesi; fake engine'in `extract_text`'i artık `TrackDescriptor.
is_text()`'e bakıyor (önceden codec'e bakmadan sabit metin dönüyordu — bu,
kit fixture'ının `FAKE_SUBTITLE_TRACK`'ının aslında bitmap track olduğunu
ortaya çıkardı; sabit `TrackId(0)`'a — gerçek metin track'ine — çekildi,
bitmap için ayrı `FAKE_BITMAP_SUBTITLE_TRACK` kondu. Kit fixture'ında bitmap
senaryosu yok — ADR-0045 Karar 3'ün reddi yalnız adapter/nen-app seviyesinde
kanıtlanıyor, kullanıcı kararıyla).

**`nen-app`:** `SubtitleLibrary::attach_embedded_document` (idempotent,
yalnız embedded satır); `PlaybackSession::prepare_embedded_document` — tek
yeni giriş noktası: token zaten belge taşıyorsa motora hiç gidilmez
(kullanıcı dosyası/AI satırı/önceden çıkarılmış embedded), embedded değilse
veya `translatable=false` ise motora gidilmeden reddedilir, aksi halde
`ShellEngineBridge::engine()` ile alınan `Arc<dyn ShellEngine>` **session
kilidinin dışında** çağrılır (gerçek demux saniyeler sürebilir; kilit altında
`position_ms`/pump donardı). `embedded_lazy.rs` (NEN-023) değişmeden yeşil —
bu yeni yol o testin tripwire'ını hiç çağırmıyor.

**`nen-ffi`:** `FfiPlaybackError::TrackCarriesNoText`; `FfiPlaybackSession::
prepare_embedded_document`; `FfiEmbeddedDocumentError` — `FfiTranslationStartError`
emsaliyle **düz ve payload'suz** (nested `PlaybackError` taşımıyor, sekiz
varyanta düzleştirildi — plandaki ilk taslak bir `Playback{error:...}` iç içe
varyantı öneriyordu, uygulama sırasında codebase'in kendi "flat error"
kuralına uydurmak için bu şekilde revize edildi).

**macOS adapter:** `Package.swift`'e `Cavformat` `systemLibrary`'si
(`pkgConfig: "libavformat libavcodec libavutil"` — tek isimle `-I`/`-lavformat`
yeterli olmuyordu, üç `.pc` adı birlikte verilince hem include hem tüm
`-l` bayrakları doğru geldi, ölçüldü). `EmbeddedTextExtractor.swift`
`avformat_open_input` → `avcodec_decode_subtitle2` döngüsü; zamanlama
`AVPacket.duration`'dan (birincil, gerçek fixture'da ölçüldü —
`AVSubtitle.end_display_time` yalnız `<=0` ise yedek). `MPVPlaybackEngine.
extractText` locator'ı `/` ile başlamıyorsa (`url.absoluteString` — uzak
medya) `Unsupported` ile döner, gerçek demux'e hiç girmez.

### Golden ve negatif kanıt — gerçek libmpv + libavformat'ta ölçüldü

`EmbeddedTextExtractionTests.swift`, gerçek `contract-clip.mkv` üzerinde:
çıkarılan İngilizce (track 3) ve Türkçe (track 4) metin, yeni
`fixtures/media/contract-clip.sub-{eng,tur}.golden` ile **birebir** eşleşiyor
— zamanlamalar (`00:00:01,000 --> 00:00:04,000` vb.) fixture'ın kendi
üretim recipe'sindeki (`contract-clip.ffmpeg.txt`) SRT kaynağıyla bit bit
aynı; ilk denemede, ayar değişikliği gerekmeden ölçüldü.

**Zorunlu negatif** (`bitmap-subs-clip.mkv`): `hdmv_pgs_subtitle` track (id 3)
`TrackCarriesNoText` ile reddediliyor; **aynı dosyanın** metin track'i (id 2,
subrip) sorunsuz çıkarılıyor — red, container veya codec-lookup'ta genel bir
kusur değil, doğrudan bitmap'e özel. Ayrıca: bilinmeyen id → `UnknownTrack`;
medya yüklenmeden → `NotLoaded`; `currentLocator` elle uzak bir URL'e
çevrilince (gerçek ağ erişimi olmadan, deterministik) → `Unsupported`, hiçbir
`libavformat` çağrısı yapılmadan.

**İki mutasyon elle uygulanıp geri alındı, ikisinde de tam olarak beklenen
test(ler) kırmızıya döndü** (kontrol sağır değil): (a) bitmap rect'lerin de
metne eklenmesi → yalnız `aBitmapTrackIsATypedRefusalNotEmptyText` kırmızı;
(b) süre hesabının sabit `1ms`'e sabitlenmesi → yalnız iki golden test
kırmızı, bitmap/unknown/remote testleri etkilenmedi.

### `nen-app`/`nen-ffi` uçtan uca kanıt

`embedded_extraction.rs` (`nen-app`, 7 test): sahte `ShellEngine` ile
`prepare_embedded_document` → `Ready`, ardından `translation::prepare`
**artık `NoDocument` vermiyor** (uçtan uca — bu task'ın kapattığı gerçek
kusurun doğrudan ölçümü); ikinci çağrıda motor sayacı artmıyor (idempotent);
bitmap girdi → `TrackCarriesNoText`, motor sayacı 0 (motora hiç gidilmedi);
parse edilemeyen metin → `Unparseable`; kullanıcı dosyası motora hiç
gitmiyor; `show_source` (seçim) çıkarım tetiklemiyor. `embedded_document.rs`
(`nen-ffi`, 5 test) aynı senaryoları FFI sınırında, `guard_ffi_embedded_
document_debug.rs` (2 test) sekiz varyantın yalnız kendi adını bastığını ve
sentinel diyalogla koşan gerçek bir çıkarımın hatasının diyalogu
sızdırmadığını kanıtlıyor.

### Swift wiring — `PlayerModel.translateSelectedSubtitle()`

Detached task içinde, `FfiTranslationEngine` kurulmadan **önce**
`session.prepareEmbeddedDocument(library:token:)` çağrılıyor —
`PlaybackSessionClient` bu nedenle `Sendable` oldu (`FfiPlaybackSession`
zaten `@unchecked Sendable`; `FakeSession` aynı şekilde işaretlendi).
Reddi yeni `TranslationJoinOutcome.prepareFailed` taşıyor, kendi Türkçe
mesajıyla (`PlaybackPresentation.prepareEmbeddedDocumentMessage`).
`EmbeddedDocumentPreparationTests.swift` (2 test, kullanıcı dosyasıyla —
`FakeSession` gerçek `FfiSubtitleLibrary`'ye belge ekleyemez, o
materialization prod `FfiPlaybackSession`'ın işi, yukarıda kanıtlı):
`prepareEmbeddedDocument` seçimde değil yalnız çeviri komutunda ve tam bir
kez çağrılıyor; bir red kendi mesajını gösteriyor ve hiçbir `artifacts/`
dizini oluşmuyor. **Bir mutasyon** (çağrının kaldırılması) elle uygulandı —
yalnız bu iki test kırmızıya döndü, `TranslationCommandTests`'in (NEN-101)
hiçbiri etkilenmedi.

### Bundle kapanışı (ADR-0045 Karar 5)

`bash scripts/build-macos-app.sh` + `bash scripts/bundle-macos.sh`:
**Gömülü dylib sayısı: 48** — `NEN-043`'ün orijinal kapanışıyla birebir aynı;
`otool -L` ana ikilinin `libavformat`/`libavcodec`/`libavutil`'i artık
**doğrudan** (yalnız libmpv üzerinden transitif değil) bağladığını gösteriyor.
Ayrıntı: `evidence/M5/NEN-044-bundle.md`.

### Gerçek `.app` GUI doğrulaması — yapılamadı, dürüst kayıt

`NEN-101`/`NEN-102` emsaliyle bir GUI checklist planlanmıştı; computer-use
erişim isteği **kullanıcı tarafından reddedildi**. Bunun yerine — ve bunu
telafi eden — kanıt: `ContractTests.theRealAdapterPassesTheSharedContractKit`
ve `EmbeddedTextExtractionTests` zaten **gerçek libmpv + gerçek libavformat**
adapter'ını gerçek fixture'larla koşturuyor; GUI'nin kendisi yalnız bu aynı
kodu bir pencereden çağırırdı. `docs/testing-strategy.md`'nin kanıt
tablosunda bu task esasen domain/logic + security/validation'dır (UI değil);
ekran görüntüsü zorunlu değildi.

### Doğrulama

`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` **846 passed / 1 ignored**, `cargo deny check`
(`advisories ok, bans ok, licenses ok, sources ok`, yeni bağımlılık yok) —
hepsi yeşil. `bash scripts/test-macos.sh` **264 test / 33 suite**, iki ayrı
koşuda tekrarlandı, ikisi de yeşil (önceki bir tekli koşuda görülen
`PicturelessSurfaceTests` kırmızısı, dosyanın kendi doc-comment'inin
belgelediği bilinen main-actor-contention flake'i — `NEN-049`, izole koşuda
ve iki tam-suite yeniden koşuda yeşil; bu task'ın kapsamı dışı, düzeltilmedi).
`bash scripts/test.sh` **4/4**, `bash scripts/doctor.sh` (yeni `ffmpeg`
tespiti dahil) çıkış 0, `bash scripts/task-index.sh` ve `bash
scripts/check-docs.sh` yeşil.

