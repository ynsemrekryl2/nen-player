---
id: NEN-052
title: Decide and pin what a seek during loading does
milestone: M3
size: S
state: done
closed: 2026-09-07
depends_on: [NEN-051]
blocks: []
adr: [11, 42]
---

# NEN-052 — Decide and pin what a seek during loading does

## Sonuç

Medya yüklenirken verilen bir seek'in ne yaptığı **kararlaştırılmış** ve
contract kitinde yazılıdır; fake ile gerçek adapter aynı cevabı verir.

## Kapsam

- Portun `.buffering` durumundaki seek için ne vaat ettiğinin netleştirilmesi:
  tipli ret mi, yoksa yükleme bitince uygulanan bir istek mi
- Kararın contract kitine bir senaryo olarak yazılması
- Fake ve libmpv adapter'ının aynı cevabı vermesi

## YAPILMAYACAK

- `MPV_EVENT_SEEK` kapısını gevşetmek → `NEN-051`'in kararı
- Yükleme sırasında seek'i kuyruklayıp otomatik uygulamak — karar verilmeden
  davranış eklenmez

## Neden ayrı task

`NEN-051` sırasında ölçüldü: `MPVPlaybackEngine`'in `requireMedia`'sı
`.loading` fazını **kabul ediyor**, yani port seviyesinde yüklenirken seek
etmek meşru görünüyor. Gerçek mpv ise reddediyor —
`EngineFailure(code: -12)` (`MPV_ERROR_COMMAND`). `EngineFailure` motorun
kendi sayısı demektir; bu, kabuğa "beklenmeyen motor hatası" olarak görünür,
oysa durum tamamen normal.

Contract kitinin hiçbir senaryosu yüklenirken seek etmiyor, dolayısıyla fake
ile gerçek adapter'ın bu noktada anlaşıp anlaşmadığı **bilinmiyor**. Kural 5
gereği `NEN-051`'e eklenmedi.

## Kanıt (DoD)

- [x] Kararın hangi cevabı şart koştuğu yazılı
- [x] Contract kitinde yüklenirken seek senaryosu var
- [x] Fake ve libmpv adapter'ı aynı cevabı veriyor
- [x] Negatif: adapter'ın yanlış cevap vermesi kiti kırmızıya döndürüyor

## Kanıt kaydı

**Önce ölçüldü, sonra karar yazıldı.** Gerçek libmpv adapter'ı beş ayrı
`load()` çağrısıyla ölçüldü (`evidence/M3/NEN-052-measurement.md`): `load()`
döndükten hemen sonra `state()` beşinin beşinde de `.buffering`, pencere
**2.5–12 ms** sürüyor — bir insanın tuşa basıp motoru yakalaması için
fazlasıyla dar, yani nadiren değil **her zaman** rastlanabilecek bir yarış. Bu
pencerede `seek()` `EngineFailure(code: -12)` (mpv'nin `MPV_ERROR_COMMAND`'ı)
fırlatıyordu; kabuk bunu "beklenmeyen motor hatası" diye gösteriyordu.

**`ADR-0042` `accepted`** (kullanıcı onayı): yüklenirken verilen seek
reddedilmez, tutulur ve `FILE_LOADED` anında uygulanır; ard arda gelen
ertelemelerde yalnız sonuncusu uygulanır ama çağıranın hak ettiği
`SeekCompleted` sayısı korunur; yükleme başarısız olursa, `stop`/`shutdown`
gelirse veya yeni bir `load` başlarsa ertelenmiş seek düşürülür.

**Contract kitine tek senaryo eklendi**
(`core/crates/nen-ports/src/playback/contract.rs`): "a seek issued the
instant loading starts is answered, not refused" — `Settle` YOK önce, kasıtlı
olarak: motor ister hâlâ açılıyor olsun ister zaten hazır olsun, gözlenen
cevap aynı (kabul + hedefte iniş) olmalı, senaryo hangi dala düştüğünü
bilmiyor.

**Fake ve gerçek adapter aynı cevabı veriyor, ikisi de ölçüldü.**
`nen-ports`'ta iki referans motoru eklendi (var olan `BrokenEngine` deseninde,
porta test kapısı eklemeden): `contract_fake.rs`'te `LoadingWindowEngine` —
gerçek adapter'ın ölçülen penceresini birkaç `state()` sorgusu olarak taklit
edip yüklenirken gelen seek'i tutan ve pencere kapanınca uygulayan bir
motor — **kitin tamamını** geçiyor (yalnız yeni senaryoyu değil), bu kararın
deterministik referansı. macOS tarafında `MPVPlaybackEngine`'e `deferredSeekMs`
alanı ve `FILE_LOADED` anında uygulanan erteleme eklendi
(`MPVPlaybackEngine.swift`, `MPVPlaybackEngine+Internals.swift`). Gerçek
libmpv adapter'ı, `ContractTests.theRealAdapterPassesTheSharedContractKit()`
üzerinden aynı kiti üç ardışık tam paket koşusunda (197/197) geçti; ek olarak
pencereyi doğrudan ölçen `aSeekIssuedTheInstantLoadingStartsIsAppliedNotRefused`
testi, pencereyi gerçekten yakaladığını **kendi kendine doğruluyor**
(`#expect(engine.state() == .buffering, ...)` — yarışın sessiz yeşili yasak,
NEN-049/065'in dersi).

**Negatif kontrol iki ayrı seviyede, ikisi de ayrı ayrı geri alınarak
ölçüldü.** (1) Rust: `contract_kit_is_not_vacuous.rs`'e
`Defect::RefusesASeekWhileLoading` eklendi — gerçek adapter'ın düzeltmeden
önceki davranışını (`EngineFailure(code: -12)`) birebir üreten bir ikiz —
`a_seek_refused_while_loading_is_caught` testi bunun kiti kırmızıya
döndürdüğünü **deterministik olarak** kanıtlıyor (settle beklemeden, ilk adımda
düşüyor). (2) Swift: erteleme mantığı `seek(toMs:)`'ten geçici olarak
kaldırılıp hem `theRealAdapterPassesTheSharedContractKit()` hem
`aSeekIssuedTheInstantLoadingStartsIsAppliedNotRefused()` beklenen mesajla
(`EngineFailure { code: -12 }`) kırmızıya döndüğü ölçüldü, sonra geri kondu.
Üçüncü bir negatif kontrol Karar 4'ü kapsıyor: `dropDeferredSeekUnlocked()`'ın
`stop()`'tan çağrısı kaldırılınca yeni `aDeferredSeekDoesNotSurviveTheLoadItTargeted`
testi (gerçek zamana yarışmadan, `mutate(requireLoaded: false)` ile motorun
kendi fazı zorlanarak) tam beklenen iki iddiayla (`deferredSeekMs`,
`pendingSeeks`) kırmızıya döndü, sonra geri kondu.

**Yol üstünde bulunan ve düzeltilen bir test kusuru:** ilk yazılan mekanizma
testi `settle(until: .ready)`'den hemen sonra `positionMs()`'i **tek seferde**
okuyordu — `Ready`'nin göründüğü an ile ertelenmiş `seek` komutunun event-loop
thread'inde gerçekten mpv'ye gönderildiği an aynı değil, ve tam paket
koşusunun ağır CPU yükü altında bu iki an ayrıştı (`landed = 0 ms`). Bu,
`NEN-051`'in kontrat seviyesinde öğrettiği dersin ta kendisi — "hazır" bir
işlemin tamamlandığı anlamına gelmez — testin kendi iddiasında tekrarlanmış
hâliydi. Düzeltme testin kendisiydi: `settle(engine) { positionMs() yakın mı }`
ile pozisyon **beklenerek** okunuyor artık (`playPauseAndSeekOnTheLocalClip`'in
zaten kullandığı desen). Ürün kodu bu bulgudan etkilenmedi — kontrat kitinin
kendi `AwaitSeekLanding` adımı zaten olayı bekliyordu ve üç koşuda da yeşildi.

Rust workspace **605 → 607** (+2: `LoadingWindowEngine`'in kendi testi ve
`RefusesASeekWhileLoading`'in kendi testi — yeni senaryonun kendisi veri
olduğu için var olan taramaların (`the_run_is_not_vacuous` vb.) test sayısını
değiştirmedi, kapsamlarını genişletti). fmt, clippy,
cargo-deny (yeni dış bağımlılık yok) yeşil, `cargo test --workspace` sıfır
kırmızı. macOS Swift paketi **197/197**, üç ardışık temiz paralel koşu; `.app`
build'i ve strict codesign yeşil. Değişiklik Rust çekirdeği
(`nen-ports`: `contract.rs`, `contract_fake.rs`, `contract_kit_is_not_vacuous.rs`)
ve macOS `NenPlaybackMPV`'de; FFI yüzeyi, `nen-ffi` ve kabuk (`PlayerModel`)
**dokunulmadı** — yeni hata varyantı yok, yeni FFI çağrısı yok.

**Kapsam dışı bırakılan, ölçülüp belgelenen bir sınır:** `position()`'ın
`.loading` fazında `EngineFailure(code: -10)` fırlatması (bu task'tan önce de
var olan davranış) düzeltilmedi — ADR-0042 Karar 5'te açıkça kaydedildi.
Gerçek kullanıcı yüzeyinde göreli seek (`⌘←`/`⌘→`) zaten kendi tuttuğu
pozisyondan mutlak `seek(to:)`'a gidiyor (`PlayerModel.seekRelative`), yani bu
kararın kapattığı yoldan geçiyor; Rust'ın port-seviyesi varsayılan
`seek_relative`'i (`position()` + `seek()`) hiçbir ürün yüzeyinden
tetiklenmiyor.

**Gerçek `.app` üzerinde elle klavye testi yapılmadı, gerekçesi kaydedilir:**
ölçülen pencere (2.5–12 ms) bir insanın tepki süresinden (~100–300 ms) çok
daha kısa, dolayısıyla klavyeyle "dosya açılırken hemen → tuşuna basmak"
güvenilir biçimde tekrarlanamaz — otomatik test bunun **tek** güvenilir kanıt
yoludur ve üç ardışık tam paket koşusunda deterministik olarak sağlandı.

Kanıt: `evidence/M3/NEN-052-measurement.md`, `docs/adr/0042-seek-during-loading.md`.
