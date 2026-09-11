# Durum

> **Bu dosya yalnız doğrulanmış bugünü anlatır.** Plan `roadmap.md`'de, kararlar
> `DECISIONS.md`'de, task ayrıntısı `tasks/INDEX.md`'de. Burada tekrar edilmez.
>
> Son güncelleme: **2026-09-11** (**`NEN-110` kapandı** — ADR-0020 `accepted`,
> secure credential storage haritası karara bağlandı; M6 kırılımı aynı gün
> üretilmişti. Sıradaki adım `NEN-111` — port + in-memory fake.)

## Nerede duruyoruz

| | |
|---|---|
| **Mevcut milestone** | **M6 — Real Providers** (M5 2026-09-11'de kapandı; kırılım aynı gün üretildi — `docs/milestones/M6-real-providers.md`) |
| **Aktif task** | — |
| **Son tamamlanan** | **`NEN-110`** — ADR-0020 secure credential storage haritası kabul edildi. Ondan önce: `NEN-104` |
| **Sıradaki READY** | `NEN-035`, `NEN-105`, `NEN-107`, `NEN-109`, `NEN-111`, `NEN-114`, `NEN-119`, `NEN-124` |
| **Task sayısı** | 126 · done 101 · active 0 · blocked 0 · canceled 2 · backlog 23 |

**`NEN-110` kapandı — ADR-0020 `accepted`: kullanıcının üç API anahtarının
(OpenSubtitles · OpenAI · OpenRouter) hangi port üzerinden, hangi platform
deposunda ve hangi yaşam döngüsüyle saklanacağı karara bağlandı.** Sekiz
kararlı madde: `nen-ports::credentials::SecureCredentialStore` portu (senkron,
object-safe, kapalı `CredentialKind` enum'u) · tek `ApiKey` newtype'ı
(kind'dan bağımsız, `Debug`/`Display` → `<redacted>`, `Serialize` yok) ·
tipli payload'suz hata (`Unavailable`/`Denied`/`Corrupt`) · adapter Swift'te,
core'a reverse-FFI ile (`ForeignSecureCredentialStore`, `ForeignHttpClient`/
ADR-0039 emsali) · macOS deposu login keychain + generic password
(`kSecAttrSynchronizable` yok) · UI yalnız "kayıtlı" göstergesi verir · test
kuralı in-memory fake + contract kiti + K23 guard, gerçek Keychain yalnız
`NEN-112`'nin kendi testinde · diğer platformlar yalnız harita (M9–M11).

**Üç kullanıcı kararı plan onayı sırasında netleşti.** Anahtarın yazma/okuma
yolu her zaman Rust portundan geçer — Swift Keychain adapter'ının tek çağıranı
Rust'tır, ayarlar UI'ı dahil; kayıtlı anahtar UI'da hiçbir zaman okunmaz,
yalnız "kayıtlı" göstergesi vardır; macOS deposu login keychain + generic
password seçildi (data-protection keychain değil — ad-hoc imzalı geliştirme
build'inde application-identifier entitlement'ı olmadan `SecItemAdd`
`-34018` ile düşerdi).

Kod değişmedi (karar/doküman task'ı, Kural 1 gereği implementasyon önce ADR
ister). `docs/adr/README.md`, `docs/DECISIONS.md`, `docs/architecture.md`
tutarlı güncellendi (`NEN-097`'nin ölçtüğü ADR-0017 kusuru tekrarlanmadı);
ADR-0031'e Notlar girdisi eklendi (ayar yüzeyinin üçüncü genişlemesi, gövde
değişmedi). `bash scripts/check-docs.sh` çıkış 0. Kanıt:
`tasks/done/NEN-110-*.md`.

**M6 kırılımı üretildi (2026-09-11, `/plan-milestone M6`, kullanıcı
onayıyla).** 17 yeni backlog task'ı (`NEN-110`…`NEN-126`), üç kol: **kimlik
bilgisi** (110 ADR-0020 → 111 port → 112 Keychain → 113 ayarlar),
**çeviri sağlayıcıları** (114 ADR-0019 → 115 HTTP POST → 116 OpenAI → 117
OpenRouter → 118 gerçek ortam) ve **OpenSubtitles** (119 ADR-0021 → 120
kimlik uygulamada → 121 aday katalog → 122 güvenli indirme → 123 macOS
menü); iki ön koşul ADR task'ı (124 ADR-0046 → `NEN-034`, 125 ADR-0047 →
`NEN-038`) ve kabul (`NEN-126`). Kırılımı şekillendiren iki ölçülmüş gerçek:
`NEN-033`'ün kimlik sorgusu Rust'ta bitmiş ama **uygulamaya bağlı değil**
(`nen-ffi`'de çağıran, macOS'ta anahtar yok — `NEN-064` bu yüzden artık
`NEN-120`'yi bekliyor), ve `HttpClient` portunda **POST/gövde yok**
(`NEN-115`). Kullanıcı kararları: S3 kalite çıtası ADR-0019'da ölçümle ·
kabulde kullanıcının kendi anahtarıyla **tek** gerçek koşu (testler yine
fixture, Kural 8) · `NEN-034`/`035`/`038`/`109` M6'da kalıyor. Kod
değişmedi; `bash scripts/task-index.sh` ve `bash scripts/check-docs.sh`
çıkış 0. Önerilen ilk task: **`NEN-110`** (ADR-0020) — üç kol da credential
kapısına bağlanıyor.

**`NEN-104` kapandı — M5'in altı çıkış kriteri gerçek `.app` üzerinde, hem
sidecar hem gömülü track kaynağıyla, mock provider ile (Kural 8) uçtan uca
kanıtlandı; M5 kapandı.** `contract-clip.mkv`'nin sidecar olarak elle
yüklenen `layout-sample.srt`'si (95 cue) ve gömülü İngilizce track'i (3
cue, `NEN-044`'ün demux yolu üzerinden) ayrı ayrı Türkçe'ye çevrildi; her
ikisinin çıktısı ilgili golden fixture'la cue ID/zaman/sıra ve metin
düzeyinde **birebir** eşleşti. Hedef dil Deutsch'a değiştirilince ADR-0018'in
cache identity'si yeni bir artifact üretti (`artifacts/` 2→3); gömülü Türkçe
kaynak zaten hedef dildeyken komut `app_menu`'nün kendi "disabled" reddiyle
basılamadı.

**Restart sonrası cache reuse gerçek `.app` üzerinde ölçüldü — dosya yeniden
yazılmadı.** Uygulama kapatılıp yeniden açıldıktan, aynı komut tekrarlandıktan
sonra ilgili artifact dosyasının inode'u ve mtime'ı **birebir aynı** kaldı;
`nen-app::translation::run_job`'ın cache isabetinin provider'a hiç
gitmeden döndüğü canlı doğrulandı. İptal negatifı mock provider'ın anlık
bitişi yüzünden GUI'de zamanlanamadı (`NEN-101`/`NEN-102`'nin öngördüğü
bilinen risk) — deterministik testler (`NEN-102` progress/cancel suite'i,
`translation_gate.rs`) telafi etti.

**Bu ortamda `computer-use`'un ekran görüntüsü kaydı depoya committ
edilebilecek bir dosya yoluna erişilemedi (tooling sınırı) — kanıt metin
checklist + disk ölçümleriyle (dosya sayısı, inode, mtime) tutuldu** (Kural
3: UI kanıtı screenshot **veya** checklist). Kod değişmedi: `cargo test
--workspace` **846 passed / 1 ignored** ve `bash scripts/test-macos.sh`
**264 passed / 33 suites** (iki koşuda, ikisi de yeşil — aradaki bilinen
`PicturelessSurfaceTests` flake'i, `NEN-049`, dokunulmadı), her ikisi de
`NEN-044` baseline'la birebir aynı; fmt, clippy, `cargo deny check`, `bash
scripts/test.sh` **4/4** yeşil. Kanıt: `evidence/M5/NEN-104-checklist.md`.

**M5 retro'su `docs/milestones/M5-translation-core.md`'ye yazıldı: 2026-09-08
→ 2026-09-11, 20 task, beş ADR (0015/0016/0017/0018/0045) `accepted`, dört
yanlış çıkan varsayım (`NEN-101` store kökü, `NEN-102` cancel/join state,
`NEN-106` blok-`total`, `NEN-108` mutex fairness) — hepsi aynı task'ta
ölçülüp düzeltildi.** Üç takip task'ı M6'ya ayrıldı: `NEN-105` (kalıcı
checkpoint), `NEN-107` (belge-geneli ilerleme), `NEN-109` (uzak gömülü metin
çıkarımı). M6'nın kendi task kırılımı henüz üretilmedi — sıradaki adım ayrı
bir `/plan-milestone` onayı.

**`NEN-044` kapandı — gömülü bir metin altyazı track'inin tam metni artık
seçimde değil, yalnız açık çeviri komutunda, `libavformat`/`libavcodec`
üzerinden (ADR-0045) çıkarılabiliyor.** Yeni `EmbeddedTextExtractor.swift`
(macOS adapter) seçili medyanın container'ını mpv'nin kendi handle'ından
tamamen bağımsız, **ikinci kez** açıp ilgili subtitle stream'ini demux/decode
ediyor ve kanonik SRT olarak yeni `nen-app::session::PlaybackSession::
prepare_embedded_document`'a veriyor — bu, `nen_subtitle::srt::parse`
(NEN-013) ile parse edilip `SubtitleLibrary::attach_embedded_document`
(idempotent) ile satıra takılan **tek yeni giriş noktası**. `NEN-102`'nin
kapanışında canlı ölçülüp kaydedilen kusur — gömülü İngilizce track'in
çeviri komutunda kalıcı `NoDocument` reddi, çünkü hiçbir şey motoru
metin için hiç sormuyordu — bu yolla kapandı.

**Üç uygulama noktası kullanıcı kararıyla kapatıldı (plan onayı sırasında,
ADR'nin kendisi bunları kapsam dışı bırakmıştı).** Taşıma biçimi kaynak
codec ne olursa olsun (subrip · ass · mov_text · ...) **kanonik SRT** —
core'un zaten tek parser'ı var, ikincisini açmaya gerek yok. Çıkarım
`ShellEngineBridge`'in kilidi **dışında** çalışıyor: gerçek bir demux
saniyeler sürebilir, kilit altında `position_ms`/pump'ı o süre boyunca
dondururdu. Ve **yalnız yerel dosya** — uzak (M4 Stremio HTTP akışı)
locator `Unsupported` ile döner; sınırsız/iptal edilemez bir indirmenin
riskini taşımak yerine yeni `tasks/backlog/NEN-109-remote-embedded-text-
extraction.md` (M6) ayrıldı (Kural 5).

**Bitmap reddi için yeni, payload'suz bir port varyantı gerekti:
`PlaybackError::TrackCarriesNoText`.** Mevcut `UnknownTrack` "bu id yok"
demek, "bu id'de metin yok" demek değil — ADR-0045 Karar 3'ün ayırdığı iki
anlam. Varyant `nen-ports` → `nen-ffi` (`FfiPlaybackError`, ve ayrı bir
`FfiEmbeddedDocumentError` — `FfiTranslationStartError`'ın **düz, payload'suz**
kuralına uydurulmuş sekiz varyant) → Swift'e (kendi Türkçe mesajıyla)
uçtan uca taşındı.

**Golden ve zorunlu negatif, gerçek libmpv + gerçek libavformat'ta ölçüldü.**
`contract-clip.mkv`'nin gömülü İngilizce ve Türkçe track'lerinden çıkarılan
metin, yeni `fixtures/media/contract-clip.sub-{eng,tur}.golden` ile **birebir**
eşleşti — zamanlamalar fixture'ın kendi üretim recipe'sindeki kaynak SRT'yle
bit bit aynı, ilk denemede. `bitmap-subs-clip.mkv`'nin `hdmv_pgs_subtitle`
track'i `TrackCarriesNoText` ile reddedildi; **aynı dosyanın** metin track'i
sorunsuz çıkarıldı — red bitmap'e özel, container/codec-lookup'ta genel bir
kusur değil. İki mutasyon elle uygulanıp geri alındı (bitmap rect'lerin
metne eklenmesi; süre hesabının sabit `1ms`'e sabitlenmesi), ikisinde de
tam olarak beklenen test(ler) kırmızıya döndü.

**`nen-app`/`nen-ffi` uçtan uca kanıt, ayrıca `PlayerModel.
translateSelectedSubtitle()`'ın kendi çağrısı.** `nen-app::embedded_
extraction.rs` (7 test) `prepare_embedded_document` sonrası
`translation::prepare`'ın **artık `NoDocument` vermediğini** doğrudan
ölçüyor; ikinci çağrıda motor tekrar sorulmuyor (idempotent); kullanıcı
dosyası motora hiç gitmiyor; seçim (`show_source`) çıkarım tetiklemiyor.
Swift tarafında `translateSelectedSubtitle()` artık `FfiTranslationEngine`
kurulmadan önce `session.prepareEmbeddedDocument`'ı çağırıyor
(`PlaybackSessionClient` bu yüzden `Sendable` oldu); bir mutasyon (çağrının
kaldırılması) yalnız bu iki yeni Swift testini kırmızıya çevirdi,
`NEN-101`'in testlerinin hiçbiri etkilenmedi.

**Bundle kapanışı yeniden ölçüldü (ADR-0045 Karar 5): gömülü dylib sayısı
48 — `NEN-043`'ün orijinal kapanışıyla birebir aynı.** `otool -L` ana
ikilinin `libavformat`/`libavcodec`/`libavutil`'i artık doğrudan (yalnız
`libmpv` üzerinden transitif değil) bağladığını gösterdi; ayrıntı
`evidence/M5/NEN-044-bundle.md`. Gerçek `.app` üzerinde bir GUI checklist
planlanmıştı (`NEN-101`/`NEN-102` emsali); computer-use erişim isteği
kullanıcı tarafından reddedildi. Bunun yerine — dürüst kayıt — telafi eden
kanıt zaten koşmuş durumdaydı: `ContractTests.
theRealAdapterPassesTheSharedContractKit` ve `EmbeddedTextExtractionTests`
aynı gerçek libmpv + gerçek libavformat adapter'ını gerçek fixture'larla
koşturuyor; GUI yalnız aynı kodu bir pencereden çağırırdı.

`cargo test --workspace` **846 passed / 1 ignored** (`NEN-108` baseline 832
+ bu task'ın 14 testi). `bash scripts/test-macos.sh` **264 test / 33 suite**
(baseline 256/31 + bu task'ın 8 testi), iki ayrı koşuda tekrarlandı, ikisi de
yeşil (tek bir ara koşuda görülen `PicturelessSurfaceTests` kırmızısı,
dosyanın kendi doc-comment'inin belgelediği bilinen main-actor-contention
flake'i — `NEN-049`, izole koşuda ve iki tam-suite koşuda yeşil; bu task'ın
kapsamı dışı, dokunulmadı). `cargo fmt --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo deny check` (yeni bağımlılık yok),
`bash scripts/test.sh` **4/4**, `bash scripts/doctor.sh` (yeni `ffmpeg`
tespiti dahil) ve `bash scripts/check-docs.sh` hepsi yeşil. Kanıt:
`tasks/done/NEN-044-*.md`.

**`NEN-108` kapandı — `core/crates/nen-ffi/tests/translation_gate.rs`'teki iki
çeviri iptal testi artık CI'da tutarlı geçiyor.** `NEN-102`'nin commit'i
(`afbcb1e`) GitHub Actions'ta bir kez kırmızı verip aynı commit'te
`gh run rerun` ile yeşile dönmüştü — ölçülen kök neden, worker'ı
`TranslationCall::progress`'in tuttuğu delivery-gate kilidinin **içinde**
durduran test senkronizasyonuydu: `release()` sonrası kilidi kimin alacağı
(worker'ın kendi sonraki `checkpoint()`'i mi, bloke `cancel()` mi) std
`Mutex`'in adil olmayan (non-FIFO) davranışına kalıyordu, `thread::sleep(50ms)`
olasılığı düşürüyor ama sıfırlamıyordu.

**Düzeltme sleep büyütmek değil, duraklama noktasını değiştirmek oldu.**
`nen-app`'in kendi eşdeğer testi zaten kilidin tamamen dışında duraklıyordu
(`nen_providers::translation_mock::MockCallGate`, provider'ı bir progress
teslim edildikten **sonra** durduruyor); `nen-ffi`'nin testi bu mekanizmaya
iki dar test-seam ile ulaştı — `TranslationEnvironment::with_provider`
(`nen-app`, `new`'in sabit mock'unu parametreye çevirir) ve
`FfiTranslationEngine::with_environment` (`nen-ffi`, `#[doc(hidden)]` ve
`#[uniffi::export]` dışında — üretilen Swift binding'de yok, gerçek bir
çağıranın ulaşamayacağı tek yol). `nen-ffi/Cargo.toml`'a yalnız
`[dev-dependencies]` altında `nen-providers` eklendi; ADR-0006 kural 2/3'ün
üretim `[dependencies]` grafiği değişmedi.

**İki test artık `thread::sleep` veya ikinci bir thread içermiyor.**
`cancel()` her iki testte de inline çağrılıyor — worker kilidin dışında
duraklarken `cancel()` rakipsiz çalışıyor. İkinci testin sıralaması
(`join()` önce uçar, `cancel()` sonra gelir) yeni bir `wait_until_finished()`
yardımcı fonksiyonuyla `job.is_finished()`'ın `Taken` durumunu — `join()`'ün
**ilk** işi — gözlemleyerek sabitlendi, sabit bir gecikme yerine.

**Sağırlık kontrolü: iki ayrı mutasyon (her testten `cancel()` çağrısı
kaldırıldı), tam olarak beklenen tek testi kırdı.** Yerelde 30/30 ardışık
koşu yeşil (varsayılan yükte), ayrıca 20/20 ardışık koşu 4 paralel CPU
yükleyici altında yeşil. `cargo test --workspace` **832 passed / 1 ignored**
(`NEN-102` baseline'la birebir aynı). fmt, clippy, `cargo deny check`
(yalnız workspace-içi bir dev-dependency kenarı — yeni dış paket yok),
`bash scripts/build-apple.sh` (seam binding'de yok), `bash scripts/test.sh`
**4/4** ve `bash scripts/check-docs.sh` yeşil. Kanıt: `tasks/done/NEN-108-*.md`.

**`NEN-102` kapandı — koşan bir çeviri işinin ilerlemesi ekranda görünüyor,
kullanıcı onu iptal edebiliyor ve iptal edilen iş ekranda yarım bir sonuç
bırakmıyor.** Yeni `TranslationProgressRelay` (`ForeignTranslationProgressSink`
implementasyonu) `translateSelectedSubtitle()`'ın artık `sink: nil` değil
gerçek bir sink verdiği `FfiTranslationEngine.start`'tan gelen olayları bir
`AsyncStream`'e taşıyor; `PlayerModel.translationProgress` (blok sırası + o
bloğun kendi `done`/`total`'ı — belge-geneli yüzde değil, bkz. aşağıda) yeni
`TranslationStatusPill`'e akıyor, `PlayerRootView`'in mevcut geçici-mesaj
yuvasında — `NEN-067`'nin kalıcı kromuna hiçbir öğe eklenmedi. Yeni
`nonisolated PlayerModel.cancelTranslation()` hem hapın `İptal` düğmesinden
hem `Altyazı` menüsünün iş koşarkenki `AI Çevirisini İptal Et` komutundan
çağrılıyor; medya değişimi de (`openMedia`) çalışan işi aynı yoldan iptal
ediyor (kullanıcı kararı).

**Yol üstünde `nen-ffi`'de gerçek, üretimde de geçerli bir kusur bulundu ve
aynı task'ta düzeltildi.** `FfiTranslationJob::cancel()` yalnız `JobState`
`Running` iken çalışan handle'a erişebiliyordu; ama `join()`'ün **ilk işi**
state'i `Taken`'a çevirmekti — `join()` çağrıldığı an (ki her çağıran, bu
task'ın kendi kodu dahil, `start()`'tan hemen sonra yapar) `cancel()` kalıcı
olarak etkisiz kalıyordu, iş dakikalarca sürse bile. Yeni bir Rust regresyon
testiyle (`nen-ffi/tests/translation_gate.rs` →
`cancelling_after_join_has_already_been_called_still_stops_the_job`) doğrudan
ölçüldü. Düzeltme: `nen-app::translation`'a `join()`'ün tükettiği
`JobState`'ten bağımsız, `TranslationJobHandle::cancel_handle()` ile alınan
ve `FfiTranslationJob`'a `state` mutex'inin **dışında** saklanan yeni bir
`TranslationCancelHandle` — ADR-0004 Karar 2'nin gate'i değişmedi, yalnız
ona her zaman erişilebilir hale geldi.

**Beş kapı elle mutasyona uğratıldı, her birinde tam olarak beklenen
test(ler) kırmızıya döndü** (kontrol sağır değil); medya değişimindeki
revizyon koruması mevcut testlerin hiçbiriyle kırmızıya döndürülemedi (çok
dar bir yarışı koruyor) — bu olduğu gibi kaydedildi, sağır bir kontrol
koddan çıkarılmadı çünkü üretimde gerçek bir korumaya denk geliyor.
Cancel'ın gerçekten çalışan bir işi durdurduğunu ölçen testler, Rust'ın
kendi testinin (`translation_gate.rs`) aynı desenini kullanıyor — işi
duraklatan bir gözlemci callback'i, paylaşılan kilide gerçek bir OS
`Thread` üzerinden ulaşan bir iptalci. `Task.detached` bu iş için
**kullanılmadı**: kuyruklanan bir `Task.detached` iptalcinin gerçekten ne
zaman çalışacağının garantisi olmadığı ölçülerek bulundu (bazen iş, iptalci
bloğu elde etmeden önce zaten bitiyordu) — gerçek `Thread` bu garantiyi
veriyor.

**Belge-geneli yüzde bu task'ın kapsamı dışında bırakıldı (kullanıcı
kararı).** `FfiTranslationProgress` blok-yereldir (`checkpoint.rs` her blok
sınırında `TranslationCall::fork` kullanıyor, `done` sıfıra dönüyor) ve FFI
belgenin toplam cue sayısını hiç geçirmiyor — kabuk bugün dürüst bir payda
hesaplayamaz. Yeni `tasks/backlog/NEN-107-document-wide-translation-progress.md`
(M6, gerçek sağlayıcılardan önce) bunu Rust tarafında çözecek; oraya
`NEN-106`'nın emsali olan retry-monotonluk tuzağı not düşüldü.

**Gerçek `.app` kabulü** (`evidence/M5/NEN-102-checklist.md`):
`fixtures/media/contract-clip.mkv` (gömülü Türkçe + İngilizce) ve
`fixtures/subtitles/blocks/layout-sample.srt` (95 cue, 3 blok — `NEN-089`'un
golden fixture'ı) ile — kullanıcı altyazısı yüklenip seçildi, komut verildi;
menüde `Türkçe ▸ AI çevirisi (tr)` belirdi, seçili altyazı **değişmeden**
kaldı (§9). Negatif: hedef dille aynı gömülü Türkçe kaynak seçilince komut
soluk görüldü. Mock provider'ın anlık bitişi yüzünden ilerleme hapı/İptal
canlı ekran görüntüsünde yakalanamadı (`NEN-101`'in kendi kaydının öngördüğü
bilinen risk); deterministik kanıt Swift testlerinde. Yol üstünde ölçülen
ayrı bir gerçek: medyanın gömülü **İngilizce** track'i (`NEN-044`, backlog)
`NoDocument` ile kalıcı olarak reddediliyor — kusur değil, embedded metin
çıkarımının henüz uygulanmadığının canlı teyidi.

`bash scripts/test-macos.sh` **256 passed / 31 suites** (iki ayrı koşuda
tekrarlandı, ikisi de yeşil — `NEN-101` baseline 249 + bu task'ın 7 testi).
`cargo test --workspace` **832 passed / 1 ignored** (`NEN-100` baseline 831 +
bu task'ın regresyon testi). fmt, clippy, `cargo deny check` (yeni dış
bağımlılık yok), `bash scripts/test.sh` **4/4** ve `bash scripts/check-docs.sh`
yeşil. Kanıt: `tasks/done/NEN-102-*.md`.

**`NEN-101` kapandı — kullanıcı macOS'ta hedef çeviri dilini bir ayardan
seçebiliyor ve seçili altyazı kaynağı için menü çubuğundaki yeni `Altyazı ▸
AI ile <hedef dile> Çevir` komutuyla açıkça çeviri başlatabiliyor.** Rust
çekirdeğine hiç dokunulmadı — `NEN-100`'ün ürettiği `FfiTranslationEngine` /
`FfiTranslationJob` yüzeyi olduğu gibi çağrıldı. Yeni
`TranslationPreferenceStore.swift`, `SubtitlePreferenceStore.swift`'in
birebir deseni: hedef dil sistem dilinden **bir kez** tohumlanıyor, ikinci
bir dil tablosu açılmadı — seçici `LanguageCatalog.allCodes` +
`SubtitleMenuPresentation.endonym(for:)`'u kullanıyor.

**Komutun tek kapısı `PlayerModel.canTranslateSelectedSubtitle`** — kaynak
seçili değilse, zaten hedef dildeyse (birincil subtag karşılaştırması,
ADR-0030 granülerliği) ya da iş zaten koşuyorsa devre dışı. `translate
SelectedSubtitle()` işi ana actor dışında başlatıp `join()` eder, yalnız
medya o sırada değişmediyse (`mediaPresentationRevision` guard'ı) sonucu
kataloglar — **`selectSubtitle` hiçbir zaman çağrılmıyor**, §9'un "zorla AI
çıktısına geçilmez" kuralı. Kaynak seçmek tek başına hiçbir işi tetiklemiyor.

**Her iki kapı elle mutasyona uğratıldı, her birinde tam olarak beklenen
test(ler) kırmızıya döndü.** Dil-eşitliği koşulu kaldırılınca yalnız iki
negatif test; `translateSelectedSubtitle()`'ın kendi guard'ı kaldırılınca
yalnız "hiçbir `artifacts/` dizini oluşmadı" iddiası kırmızı oldu — Swift'in
kendi kapısının Rust'un `AlreadyTargetLanguage` reddine güvenmediğini
kanıtlıyor.

**Gerçek `.app` koşusunda yol üstünde gerçek bir kusur bulundu ve aynı
task'ta düzeltildi.** `nen-persist::FilesystemArtifactStore::new` kökü
`fs::canonicalize` ile açıyor — kökün **zaten var olmasını** şart koşuyor.
`~/Library/Application Support/NenPlayer/` hiçbir yerde yaratılmıyordu;
temiz bir kurulumda ilk komut her zaman `StoreUnavailable` ile düşerdi.
Swift testleri bunu yakalayamadı çünkü test fixture'ı kendi kökünü zaten
yaratıyor. Düzeltme: `translateSelectedSubtitle()` motoru kurmadan önce
dizini yaratıyor.

**Gerçek `.app` kabulü** (`evidence/M5/NEN-101-checklist.md`):
`fixtures/media/contract-clip.mkv` (gömülü Türkçe altyazı) ve
`fixtures/subtitles/languages/english.srt` ile — hedef dil Almanca'ya
değiştirildi, İngilizce kaynak seçildi, komut verildi; menüde `Deutsch ▸ AI
çevirisi (de)` belirdi, seçili altyazı **değişmeden** kaldı. Negatif: hedef
dil Türkçe'ye çevrilip gömülü Türkçe altyazı seçilince komut soluk/devre dışı
görüldü. Diskte tek bir içerik-adresli artifact dosyası oluştu (ADR-0017
Karar 2); kanıt toplandıktan sonra geliştirme dizini temizlendi.

**ADR-0031'e ayrı, önceden push edilen bir Notlar girdisi eklendi
(`8d0fc77`).** Karar 6'nın "M3'te tek ayar yüzeyi" kapsamının M5'te üçüncü
bir satırla genişlediği kayıt altına alındı; yasaklanan genel ayarlar ekranı
değil, gövde değişmedi (ADR-0043/`NEN-081` emsali).

`bash scripts/test-macos.sh` **249 passed / 30 suites** (bu task'ın 10
testi dahil — `TranslationPreferenceStoreTests` 4, `TranslationCommandTests`
6); `cargo test --workspace` **831 passed / 1 ignored** (`NEN-100`
baseline'la aynı — Rust'a dokunulmadı); fmt, clippy, `bash scripts/test.sh`
**4/4** yeşil. `nen-persist`, `nen-translate`, `nen-app`, `nen-ffi`
dokunulmadı — yalnız macOS kabuğu. Kanıt: `tasks/done/NEN-101-*.md`.

**`NEN-100` kapandı — `nen-ffi::translation` çeviri işini açan tek dış kapı
oldu.** Bir iş artık FFI sınırından başlatılabiliyor, ilerlemesi izlenebiliyor,
iptal edilebiliyor ve sonucu kataloğa eklenebiliyor; `nen_app::translation`'ın
(`NEN-099`) hiçbir iç tipi bu sınırı geçmiyor. `nen-app`'e yeni bir
kompozisyon kökü — `TranslationEnvironment` — eklendi: `nen-ffi` hâlâ yalnız
`nen-app`'e bağımlı (ADR-0006 kural 3, `Cargo.toml` değişmedi), ama
`nen-persist::FilesystemArtifactStore`'u M5'in mock provider'ıyla burada
eşliyor. Gate yalnız ilkel değer alıp veriyor — depo kök yolu, token, hedef
dil dizgesi — hiçbir `nen-translate` tipi FFI'ya hiç adlandırılmadı.

**İlerleme push ile taşınıyor; bu ADR-0033'ün pull yönünün istisnası değil,
kapsamı dışı bir karar.** ADR-0033'e bunu kaydeden bir Notlar girdisi ayrı
commit'te eklendi: Karar 3'ün pull tercihi playback'in ~30 Hz'lik sürekli
akışına özgü, bir çeviri işinin ilerlemesi ise blok başına birkaç kesikli
olay ve zaten `TranslationCall`'ın (ADR-0004 Karar 2/5) tek delivery gate'i
bir push'un ihtiyaç duyduğu şey. Yeni `ForeignTranslationProgressSink`
(`#[uniffi::export(with_foreign)]`) bunu taşıyor, ikinci bir kuyruk açılmadı.

**İptal, gerçek bir çok-thread'li testle ölçüldü.** Sink kendi callback'i
içinde — delivery gate'in kilidini tutarken — duraklatılıyor, başka bir
thread `cancel()` çağırıyor, callback serbest bırakılıyor. Ölçülen değişmez:
`cancel()` döndüğü an kaydedilen olay sayısı, işçi thread'i tamamen bitene
kadar **artmıyor** — ADR-0004 Karar 2'nin kendisinden doğan bir garanti. İlk
yazımda bu test 31 koşudan birinde yanlış geçti (aynı işçi thread'i kilidi
bırakır bırakmaz "barge" edip kalan işi bitirebiliyordu); düzeltme
`nen-ports`'un kendi eşdeğer testinin uyguladığı aynı çare
(`thread::sleep(50ms)` ile canceller'a kilide ulaşma payı) — sonrasında
100/100 koşu yeşil.

**Beş kapı elle mutasyona uğratıldı, her birinde tam olarak beklenen
test(ler) kırmızıya döndü:** `catalog_into`'nun `Done`-only şartı,
`join()`'ün ikinci çağrıda `AlreadyJoined` döndürmesi, `StartRefusal →
FfiTranslationStartError` eşlemesindeki bir varyant, `FfiTranslationSummary`'ye
eklenen bir diyalog-sızdıran alan (yalnız K23 guard testi kırmızı oldu, diğer
üç guard testi yeşil kaldı — sağır değil), ve `TranslationProgress`'in
`done`/`total` alan eşlemesi. K23 guard'ı (`guard_ffi_translation_debug.rs`)
gerçek bir koşuyla ölçüyor — bu sınırdaki hiçbir tipin elle yazılmış `Debug`'ı
yok, hepsi kapalı enum/sayı/dil etiketinden ibaret; sentinel diyalog + depo
kök yolu ile koşturulan gerçek işin topladığı her olay ve nihai özet taranıyor.

**`bash scripts/build-apple.sh` binding üretimini kırmadan tamamladı** ve
üretilen `nen_ffi.swift`, `FfiTranslationEngine`, `FfiTranslationJob`,
`ForeignTranslationProgressSink` ve her iki hata tipini gerçek Swift tipleri
olarak içeriyor — `NEN-101` bugünden çağrılabilir bir yüzey buluyor.

`cargo test --workspace` **831 passed / 1 ignored** (`NEN-099` baseline 818 +
bu task'ın 13 testi); fmt, clippy, `cargo deny check` (yeni dış bağımlılık
yok — `Cargo.lock` diff'i boş), `bash scripts/test.sh` **4/4** ve `bash
scripts/check-docs.sh` yeşil. `nen-persist`, `nen-translate`, `nen-providers`,
macOS kabuğu dokunulmadı. Kanıt: `tasks/done/NEN-100-*.md`.

**`NEN-099` kapandı — kaynak seçmek çeviri başlatmıyor; açık bir komut bir işi
başlatıyor ve iş kullanıcının araya giren seçimleriyle retarget edilmiyor.**
Yeni `nen-app::translation` modülü `SubtitleLibrary`'yi `nen-translate`'in
pipeline'ıyla ilk kez birleştiriyor — bugüne kadar ikisini aynı anda gören tek
yer `artifact_store_roundtrip.rs`/`artifact_index_lookup.rs` test dosyalarıydı,
ürün kodunda "kullanıcı açık komut verdi" diyen bir use-case yoktu.
`prepare(...)` kaynağı okuyup değişmez bir `TranslationJob` snapshot'ı kurar
(kaynak belgenin klonu, plan, blok düzeni, artifact metadata, önceden
hesaplanmış cache anahtarı); `start(...)` işi kendi worker thread'inde
başlatıp hemen bir `TranslationJobHandle` döner (ADR-0004 Karar 1, yeni bir
runtime veya iptal mekanizması açılmadan).

**Retarget bir çalışma zamanı reddi değil, var olmayan bir API.**
`TranslationJob` kurulduktan sonra katalogla hiçbir bağı kalmıyor; worker
yalnız kendi snapshot'ını sürüyor. Gerçek `PlaybackSession` + provider'ı havada
tutan bir gate ile ölçülen testte iş sürerken kullanıcı ikinci bir kaynağa
geçiyor — iş yine de başlangıç kaynağının fingerprint'inde tamamlanıyor ve
sonucu kataloğa ekledikten sonra bile ekranda zorla görünmüyor, çünkü
kataloglama (`SubtitleLibrary::add_translation`) ayrı ve açık bir adım.

Cache isabeti worker'ın ilk adımı: isabet provider'a hiç gitmeden dönüyor,
kaçırma normal pipeline'a düşüyor. İptal `NEN-093`'ün zaten kanıtlanmış
`TranslationCall` gate'inin üstünde duruyor — ikinci bir mekanizma açılmadı.
Dört kapı elle mutasyona uğratıldı (cache lookup atlatma, dil-eşitliği kapısı,
katalog yazımı, cache identity'nin blok düzenini görmezden gelmesi), her
birinde tam olarak beklenen test(ler) kırmızıya döndü. Kanıt:
`tasks/done/NEN-099-*.md`.

`cargo test --workspace` **818 passed / 1 ignored** (`NEN-106` baseline 809 +
bu task'ın 9 testi); fmt, clippy, `cargo deny check` (yeni dış bağımlılık
yok), `bash scripts/test.sh` **4/4** ve `bash scripts/check-docs.sh` yeşil.
`nen-ffi`, macOS kabuğu, `nen-persist`/`nen-translate`/`nen-ports` dokunulmadı.

**`NEN-106` kapandı — `NEN-096` yazılırken ölçülmüş ve Kural 5 ile ayrılmış
bağımsız kusur giderildi: çok bloklu bir çeviri işi artık kendi
sağlayıcısının bildirdiği ilerleme yüzünden düşmüyor, `NEN-099`'u bloke eden
tek engel kalktı.** `translate_checkpointed`
(`core/crates/nen-translate/src/checkpoint.rs`) bütün bloklar için tek bir
`TranslationCall` kullanıyordu; bir sağlayıcı yalnız kendi bloğunu görüp
`total` olarak blok cue sayısını bildirdiğinde, `TranslationCall::progress`'in
ADR-0004 Karar 4'ün doğru koruduğu monoton-`total` kuralı ikinci bloğu
`Permanent` ile düşürüyordu — 50 cue'luk bir belge (`37`+`13`) düşerken aynı
kod 30 cue'da (tek blok) geçiyordu. Kusur sağlayıcıda değil orkestrasyondaydı:
`NEN-092`'nin retry için zaten kullandığı `TranslationCall::fork` deseni —
bağımsız ilerleme dizisi, paylaşılan iptal geçidi — artık blok sınırında da
uygulanıyor; blok sınırı `checkpoint()` ve `commit()` dışarıdaki paylaşılan
`call`'da kalıyor, yalnız sağlayıcı işi `&call.fork()` üzerinden sürülüyor.
`nen-ports`'un monotonluk kuralı gevşetilmedi, yeni tip veya ADR açılmadı.

`nen-app`'te bu kusur yüzünden konmuş iki geçici fixture kalktı: iterleme
bildirmeyen `SilentEchoProvider` (`artifact_store_roundtrip.rs`) silindi ve
çok bloklu test gerçek `MockTranslationProvider`'a geçirildi; `layout.blocks()
== 1` iddiası (`artifact_index_lookup.rs`) kaldırıldı. Kırmızı-önce
doğrulaması: düzeltmeden önce üç yeni test `Block(Provider(Permanent))` ile
düştü, düzeltmeden sonra `cargo test --workspace` 809 passed / 1 ignored;
fmt, clippy, `cargo deny check`, `bash scripts/test.sh` 4/4 yeşil. Kanıt:
`tasks/done/NEN-106-*.md`.

**`NEN-098` kapandı — bir artifact artık kendi ADR-0018 cache identity'sini
taşıyor, ve `nen-persist`'in dizin taramasından türeyen bir index bu kimliğe
göre arama yapabiliyor; şartname §11'in "restart sonrası reuse" cümlesi ve S4
(ağsız okuma) kararı ölçülür hale geldi.** Kimlik artifact'in kendi bilgisi
oldu — `ValidatedSubtitleArtifact` `assemble`'ın kendisine verdiği
`BlockLayout` config'ini saklıyor ve yeni `cache_identity()` metodu
`CacheIdentity::of`'u yalnız kendi alanlarından çağırıyor; yanlış kimlikli bir
artifact üretmek yapısal olarak mümkün değil. `nen-ports::persistence`'e
`ContentAddress`'in birebir deseniyle yeni `CacheKey` newtype'ı,
`ArtifactIndexEntry` ve ayrı bir `ArtifactIndex` trait'i (`entries` · `find` ·
`latest_for_target`) eklendi — sorgu yüzeyi de port'ta durduğu için ADR-0017'nin
"SQLite'a geçiş port sınırında soğurulur" iddiası bugün de geçerli.

**Index, ayrı bir dosya değil — ADR-0017 Karar 1'in dediği gibi dizinden
türüyor.** `FilesystemArtifactStore` `artifacts/`'i tarayıp her `<64hex>.json`
adayını **mevcut `get()` yolundan** (hash doğrulaması, boyut sınırı, sembolik
bağ reddi dahil) geçiriyor; kullanıcı kararı gereği okunamayan/bozuk bir dosya
taramayı durdurmuyor, sessizce atlanıyor. S9'un "hedef dil başına en yeni"
projeksiyonu kullanıcı kararıyla `source fingerprint + hedef dil` ile
gruplanıyor; her iki artifact de diskte kalıyor, yalnız hangisinin gösterileceği
seçiliyor.

**Dört kapı elle mutasyona uğratıldı, her birinde tam olarak beklenen test(ler)
kırmızıya döndü.** Taramanın hex/uzantı filtresi kaldırılınca 7 test (her
index-bağımlı senaryo) kırmızı oldu; `get()` hatasının atlanması yerine
yayılması yalnız corrupt-skip testini; `latest_for_target`'ın en yeni yerine en
eskiyi seçmesi yalnız projeksiyon + kontrat testini; `find()`'ın istenen
anahtarı yok sayması nen-persist'te iki testi **ve** nen-app'te gerçek pipeline
üzerinden çalışan değişen-bileşen testini kırdı — kontrol sağır değil.

**Ağsız okuma iddiaya değil ölçüme dayanıyor.** Yeni
`nen-persist/tests/offline_lookup.rs` sayaçlı bir `HttpClient` tutarken tam bir
yaz/tara/bul/oku döngüsü çalıştırıp sayacın `0` kaldığını, ardından doğrudan bir
`send()` çağrısıyla sayacın gerçekten `1`'e çıktığını (sağırlık kontrolü)
gösteriyor. Yeni `nen-app/tests/artifact_index_lookup.rs`, "cache identity
bileşeni değişince eski artifact bulunmuyor" iddiasını `nen-translate`'in saf
`HashMap` testinin (`NEN-097`) ötesinde gerçek `MockTranslationProvider` ve
gerçek dosya sistemi deposu üzerinden kanıtlıyor.

K23 guard'ları (`nen-ports/tests/guard_persistence_index_debug.rs`, 3 test)
`CacheKey`/`ArtifactIndexEntry`'nin `Debug`/`Display`'inde hex/hash sızmadığını
ve kasıtlı `#[derive(Debug)]` ikizinin sızdırdığını gösteriyor;
`guard_persist_debug.rs` `ArtifactRecord.cache_identity`'i de kapsayacak
şekilde genişletildi. Dosya formatı 1→2'ye bindi (`cache_identity` alanı); eski
format sessizce yanlış okunmak yerine `Corrupt` ile reddediliyor.

`cargo test --workspace` **806 passed / 1 ignored** (`NEN-097` baseline 793 +
bu task'ın 13 testi); fmt, clippy, `cargo deny check` (yeni dış bağımlılık
yok — `Cargo.lock` diff'i boş), `bash scripts/test.sh` **4/4** ve `bash
scripts/check-docs.sh` yeşil. `nen-ffi`, macOS kabuğu dokunulmadı. Kanıt:
`tasks/done/NEN-098-*.md`.

**`NEN-097` kapandı — cache identity artık şartname §11'in saydığı her
bileşenden mekanik olarak türüyor, "prompt/schema/pipeline semantiği
değişince uyumsuz cache kullanılmamalıdır" cümlesi kod düzeyinde zorlanıyor.**
Yeni `nen-translate::identity` modülü, `nen_subtitle::fingerprint`'in
(ADR-0007) desenini birebir izleyen açık, sabit sıralı byte kodlaması +
BLAKE3 ile bir `CacheIdentity` üretiyor — `serde_json`/`JSON.stringify` yok,
çünkü alan sırası derive detayıdır, platformlar arası garanti değil. Yeni
`nen-translate::versions` beş versiyon sabitini (`pipeline` · `prompt` ·
`schema` · `block-layout` · `translation-session`) tek yerde topluyor;
`artifact::PIPELINE_VERSION` ve `blocks::BLOCK_LAYOUT_VERSION` eski
yollarında `pub use` re-export olarak kaldı, mevcut çağrı yerleri değişmedi.

**ADR-0018 önce kabul edildi (2026-09-09), ayrı commit'te.** Beş kararın
tamamı taslakta olduğu gibi kaldı; kullanıcı iki noktayı netleştirdi:
`media context` yalnız `MediaHash` (başlık/yıl/dosya yolu kimliğe girmiyor —
`SourceFingerprint` zaten tam diyalog+zaman eşleşmesi istiyor), ve hesaplanan
kimlik bu task'ta `ArtifactRecord`'a **yazılmıyor** — nereye yazılacağı
`NEN-098`'in kapsamı, `NEN-096`'nın yeni kapanmış port yüzeyi dokunulmadı.
**Yol üstünde ADR-0017'nin kendi kusuru bulundu:** `NEN-095` (`8b145ef`)
ADR'yi kabul etmişti ama `docs/adr/README.md`'nin durum sütununu ve
`docs/DECISIONS.md`'nin ADR sayacını hiç güncellememişti; Kural 5 gereği ayrı
bir doküman-düzeltme commit'i aldı (`3dcfed2`'nin aynı deseni). M5'in beş
ADR'sinin tamamı artık `accepted`.

**Negatif test, ADR-0018 Karar 2'nin 13 adlı bileşeninden 14 mutasyona
ayrıldı** (`provider/model` iki ayrı kodlanmış alan olduğu için iki ayrı
test): 11'i `tests/cache_identity_negative.rs`'te (source fingerprint ·
source/target language · provider · model · media hash ×2 · glossary ×2 ·
block size · overlap, her biri hem `assert_ne!` hem eski kimlikle kurulmuş
bir `HashMap` lookup'ının yenisiyle **miss** verdiğini iddia ediyor), 5'i
version bump'ları için `src/identity.rs`'in kendi `#[cfg(test)]` modülünde
(crate-private `of_with_versions` gerektirdiğinden dışarıdan görülemiyor).
**14 mutasyonun hepsi elle ölçüldü** — `encode`'daki ilgili satır/blok
kaldırılınca tam olarak beklenen test(ler) kırmızıya döndü, başkası
etkilenmedi; kontrol sağır değil (NEN-094'ün yedi kapısı, NEN-096'nın on
mutasyonu emsali).

K23 guard'ları (`tests/guard_cache_identity_debug.rs`, 4 test) glossary adının
ve provider/model kimliğinin `CacheIdentityInput::Debug`'a sızmadığını,
`CacheIdentity`'nin kendi hex'inin kendi `Debug`/`Display`'inde geçmediğini
(`ContentAddress` emsali) ve kasıtlı `#[derive(Debug)]` ikizinin aynı
sentinel'ı sızdırdığını (guard sağır değil) gösteriyor.

`cargo test -p nen-translate` **89 passed**; workspace **793 passed / 1
ignored** (NEN-096 baseline 768 + bu task'ın 25 testi); fmt, clippy, `cargo
deny check` (yeni paket yok — `Cargo.lock` diff'i tek satır), `bash
scripts/test.sh` **4/4** ve `bash scripts/check-docs.sh` yeşil. `nen-ffi`,
macOS kabuğu, `nen-persist` dokunulmadı. Kanıt: `tasks/done/NEN-097-*.md`.

**`NEN-096` kapandı — bir çeviri artık uygulamadan çıkınca kaybolmuyor.**
`core/crates/nen-persist/` üç satırlık iskelet olmaktan çıktı: yeni
`nen-ports::persistence` portu (`ContentAddress` · `ArtifactRecord` ·
`ArtifactStore` · payload'sız `ArtifactStoreError` · contract kiti) ve onu
implemente eden `nen-persist::FilesystemArtifactStore`. Bir artifact tek bir
kanonik JSON dosyası (`<root>/artifacts/<64hex>.json`), adresi kendi tam
içeriğinin `blake3` hash'i — ADR-0017 Karar 2'nin tarifi birebir.

**İzdüşüm tek yönlü, ve bu kasıtlı.** `ValidatedSubtitleArtifact::to_record()`
eklendi; tersi yok. Okuma bir `ArtifactRecord` üretir, hiçbir zaman bir
`ValidatedSubtitleArtifact` — `NEN-094`'ün *"yalnız `assemble` doğrulanmış
artifact üretir"* değişmezi tip düzeyinde duruyor, diskten gelen baytlar
doğrulanmış bir çeviri gibi davranamıyor. `ArtifactId` de kayda girmiyor:
depodaki kimlik içerik adresidir. Kayıt yalnız `NEN-094`'ün zaten açtığı
getter'ların izdüşümü — ADR-0018 kabul edilmediği için hiçbir cache identity
semantiği kararlaştırılmadı (Kural 4).

**Atomiklik ve kök kapısı, on ayrı mutasyonla ölçüldü — her birinde tam olarak
bir test kırmızıya döndü.** Yazım ADR-0017 Karar 3'ün tam dizisi (geçici ad →
`fsync` → `rename` → dizin `fsync`); kesilen bir commit adreste hiçbir şey
bırakmıyor, ve **kalıcı** bir karşı-kontrol (`a_plain_write_leaves_a_readable_
partial_artifact`) aynı kesintiyi naif biçimde yaparak yasaklanan durumun
gerçekten oluştuğunu her koşuda gösteriyor. Kök dışına çıkma iki katmanda
kapalı: `ContentAddress::from_hex` yalnız tam 64 küçük-harf hex kabul ediyor
(`..` bir adres olamıyor, bir yola hiç ulaşamıyor), `resolve()` ise sözdizimsel
tek-bileşen şartından sonra hedefin dizinini `canonicalize` edip kökün altında
kaldığını doğruluyor — sembolik bağla kaçırılmış bir dizini yalnız bu ikinci
katman yakalıyor.

**Dedup testi ölçüm sonucu güçlendirildi.** İlk hâli yalnız dosya sayıyordu ve
dedup erken dönüşü kaldırıldığında yeşil kalıyordu — çünkü atomik `rename`
yeniden yazımı zaten idempotent yapıyor. Test artık dosyanın **aynı dosya**
kaldığını (inode değişmemiş) iddia ediyor; kapı gerçekten bağlayıcı.

**`serde` workspace dep'i oldu ama yeni dış bağımlılık gelmedi.** `Cargo.lock`
diff'i tek bir yeni `[[package]]` göstermiyor; `serde` ve `serde_derive`
grafikte `serde_json` ve `uniffi` üzerinden zaten duruyordu. `cargo deny`
yüzeyi ve `NEN-043`'ün bundle kapanışı büyümedi — ADR-0017 Karar 2'nin şartı
sağlandı.

**Yol üstünde bağımsız bir kusur ölçüldü ve `NEN-106` olarak ayrıldı (Kural 5).**
`translate_checkpointed` bütün bloklar için tek bir `TranslationCall`
kullanıyor; `TranslationCall::progress` ise aynı çağrı içinde `total`'ın
değişmesini haklı olarak `Permanent` ile reddediyor. Ama bir sağlayıcı yalnız
kendi bloğunu görür ve `total` olarak blok cue sayısını bildirir — 50 cue'luk
bir belge (`37` + `13`) ikinci blokta düşüyor, **aynı kod** 30 cue'da (tek blok)
geçiyor. `nen-translate`'in kendi testleri bunu görmemişti çünkü fixture
sağlayıcısı hiç `progress` çağırmıyor. `NEN-099`'u blokluyor.

`cargo test -p nen-persist` **21 passed**; workspace **768 passed / 1 ignored**;
fmt, clippy, `cargo deny check`, `bash scripts/test.sh` **4/4** ve
`bash scripts/check-docs.sh` yeşil. `nen-ffi` ve macOS kabuğu dokunulmadı.
Kanıt: `tasks/done/NEN-096-*.md`.

**`NEN-095` kapandı — doğrulanmış artifact'lerin nerede ve nasıl saklanacağı
`accepted` bir ADR ile sabitlendi.** `docs/architecture.md` → Portlar
tablosunun `Persistence` satırı ve `docs/DECISIONS.md` → "Ertelenmiş
kararlar" dünden beri "aday: SQLite + CAS" diyordu; bugün ADR-0017 üç
kullanıcı kararıyla kapandı: **(1)** index teknolojisi yalnız dosya sistemi
— SQLite/redb reddedildi, yeni dış bağımlılık yok (bir kullanıcının diskinde
ömür boyu birkaç yüz artifact birikir; dizin taraması milisaniyeler sürer,
SQLite'ın sorgu gücü bu ölçekte kullanılmaz ama `cargo deny` yüzeyi ve
`NEN-043`'ün bundle kapanışı her zaman büyür); **(2)** artifact tek kanonik
dosya — metadata + normalize cue'lar + WebVTT birlikte, adres dosyanın
tamamının `blake3` hash'i, ayrı bir `.vtt` yazılmaz; **(3)** blok
checkpoint'i (`NEN-093`) kalıcı olacak ama implementasyonu **M6**'ya
bırakıldı — mock provider'la çalışan M5'te baştan başlamanın maliyeti bugün
ölçülemeyecek kadar düşük, gerçek maliyet (kredi/kota kaybı) M6'da doğuyor.

**"Aday" ifadeleri her geçtiği yerde karara çevrildi**: `docs/architecture.md`
(giriş kutusu, ASCII diyagram, `Persistence`/`nen-persist` satırları,
adaptör-seçim paragrafı), `docs/DECISIONS.md` ("Ertelenmiş kararlar"dan
çıkarıldı, "Kararlı" tablosuna taşındı), `docs/testing-strategy.md`
(`Persistence` kiti paragrafı), `docs/milestones/M5-translation-core.md`
(bu vesileyle 0015/0016/0045'in de zaten `accepted` olduğu kapanış paragrafı
da düzeltildi). `docs/product-spec.md` §11'e dokunulmadı — orijinal şartname
metni, ADR-0016 kapanışında §10'un da değişmediği emsalle aynı.

**Karardan doğan iki takip kaydı ayrı tutuldu (Kural 5).** Kalıcı checkpoint
implementasyonu için yeni `tasks/backlog/NEN-105-resumable-checkpoint-store.md`
(M6, `docs/milestones/M6-real-providers.md`'e eklendi); `NEN-098`'in dosya adı
artık reddedilmiş bir teknolojiyi andığı için `NEN-098-sqlite-artifact-index.md`
→ `NEN-098-artifact-metadata-index.md` yeniden adlandırıldı (içerik
değişmedi).

Kod değişmedi (karar/doküman task'ı; Kural 1 gereği implementasyon önce ADR
ister). `bash scripts/task-index.sh` ve `bash scripts/check-docs.sh` çıkış 0.
Kanıt: `tasks/done/NEN-095-*.md`.

**`NEN-094` kapandı — çeviri sonucu artık yalnız belgenin tamamı doğrulandıktan
sonra var olan bir `ValidatedSubtitleArtifact` ve onun UTF-8 WebVTT çıktısı.**
Yeni `nen-translate::artifact` modülü şartname §11'in saydığı alanların
tamamını birleştiriyor — kendi fingerprint'i veya WebVTT yazıcısı açmadan,
`nen_subtitle::fingerprint`'i ve NEN-014'ün WebVTT yazıcısını yeniden
kullanarak. `assemble()` yalnız `NEN-093`'ün ürettiği `CompletedBlocks`'u
kabul ediyor (yarım bir çalışmadan bu tip zaten üretilemiyor), ama girdiye
körü körüne güvenmiyor: her cue'nun ID'si ve zamanlaması kaynak belgeyle
birebir karşılaştırılıyor, sahte/yanlış eşleşen bir sonuç sessizce
birleştirilmek yerine reddediliyor. Artifact ID dışarıdan verilir (kullanıcı
kararı — `NEN-096`/`NEN-097` henüz `accepted` değil, Kural 4), pipeline
versiyonu bugün var olan iki alanla sınırlı tutuldu.

**Yedi kapının her biri elle mutasyon testinden geçti** (kaldırılınca ilgili
negatif test kırmızıya döndü); bir sekizinci planlanan kapı (`BlockOrder`)
`BlockLayout` ve `BlockCheckpoints`'in kendi değişmezleri gereği hiçbir
girdiyle tetiklenemeyeceği ölçülünce koddan tamamen çıkarıldı — sağır bir
kontrol tutulmadı. Kanıt: `tasks/done/NEN-094-*.md`.

**`NEN-093` kapandı — yalnız tamamen doğrulanmış bir blok checkpoint'leniyor,
iptal edilen bir çeviri işi hiçbir koşulda sonradan bir şey commit etmiyor.**
`nen-ports::translation::TranslationCall`'a checkpoint yazımını progress/
result teslimatıyla aynı kilit sınırında tutan bir `commit<T>` primitifi
eklendi (ADR-0004 Karar 2/5, kullanıcı onaylı): gate kapalıysa kapanan closure
hiç çalışmıyor, `cancel()` süren bir commit'in bitmesini bekliyor, döndükten
sonra hiçbir commit çalışamıyor. Yeni `nen-translate::checkpoint` modülü
blokları sırayla sürüyor — zaten checkpoint'li blok provider'a hiç gitmeden
atlanıyor, blok sınırında iptal kontrol ediliyor, yalnız `NEN-092`'nin
onarım bütçesinden geçen blok commit ediliyor. `BlockCheckpoints::into_completed`
bütün bloklar checkpoint'lenmeden `CompletedBlocks` üretmiyor — yarım yayın
yasağı tip düzeyinde.

Kanıt: gerçek çok-thread'li iptal testi (bir blok cevabı barrier'la havada
tutulurken başka thread `cancel()` çağırıyor, cevap `finish()` içinde
reddediliyor) ve blok sınırı/gate mutasyon kontrolleri — kaldırılan her iki
kontrol de ayrı, tam sayıda testi kırmızıya döndürdü (kontrol sağır değil).
`cargo test -p nen-translate` **36 passed** (+ 4 negatif dosyada); workspace
**724 passed / 1 ignored**; fmt, clippy, cargo-deny, `bash scripts/test.sh`
**4/4** ve `bash scripts/check-docs.sh` yeşil. Kanıt: `tasks/done/NEN-093-*.md`.

**`NEN-091` kapandı — provider'ın typed blok cevabı artık yerel ve
authoritative validation'dan geçmeden teslim edilmiyor.** ADR-0016 kabul edildi;
`nen-translate::validation::validate_block`, exact cue count, izinli/tekil ID,
boş olmayan metin ve kaynak sırasına normalizasyonu uyguluyor. Başarılı çıktı
`SubtitleDocument`'in özgün `TimeSpan`'lerini koruyor; provider zaman
taşıyamıyor. Eksik, tekrarlı veya boş metinli beklenen ID'ler deterministik
repair kümesine giriyor; yalnız fazladan bilinmeyen ID full-block retry'a
bırakılıyor. Hata ve başarı tiplerinin `Debug`/`Display` yüzeyleri cue metni
taşımıyor.

Kanıt: `cargo test -p nen-translate` **32 passed**; workspace **702 passed / 1
ignored**; fmt, clippy, cargo-deny, `bash scripts/test.sh` **4/4** ve
`bash scripts/check-docs.sh` yeşil. ADR commit'i `94c7e10`, CI run'ı
`34265789758` yeşil. Ham structured-output parse hataları M6 provider adapter
kararına bırakıldı.

**`NEN-092` kapandı — blok çevirisi artık validation bütçesiyle sonlanıyor.**
İlk cevap geçersizse yalnız `repair_cue_ids` için en fazla iki targeted repair,
ardından en fazla bir full-block retry deneniyor; sürekli bozuk cevapta toplam
çağrı üst sınırı 4 ve kısmi `ValidatedBlock` teslimi yok. Repair istekleri
özgün bağlamı koruyor, provider hataları validation retry'ından ayrılıyor.
`TranslationCall::fork` retry progress toplamlarını bağımsız başlatırken
cancellation geçidini paylaşıyor. Kanıt: `nen-translate` **38 passed**;
workspace **712 passed / 1 ignored**; fmt, clippy, cargo-deny ve shell/doc
kapıları yeşil.

**`NEN-090` kapandı — provider sınırı runtime'sız ve caller-owned kaldı.**
`nen-ports::translation::TranslationProvider`, yalnız `CueId + text`
taşıyan provider-neutral istek/cevap tiplerini, provider/model kimliğini ve
payload'sız hata sınıflarını açıyor. Yeni `TranslationCall`, cancellation,
monoton sayısal progress ve sonuç teslimini aynı kilit altında tutuyor;
`cancel()` döndükten sonra geç callback veya sonuç teslim edilemiyor. Bu sözleşme
önce ADR-0004 olarak `accepted` oldu; ayrı `31619b8` commit'inin CI'ı yeşil.

**`nen-providers::translation_mock::MockTranslationProvider` tamamen ağsız ve
deterministik.** Aynı istek aynı provider/model kimliği, cevap ve progress
dizisini üretiyor. Ortak contract kiti geçiyor; eksik cue döndüren kasıtlı
adapter kiti kırmızıya döndürüyor. Bariyer kontrollü negatif test, havadaki
çağrı iptal edildikten sonra geç sonuç/progress sayısının sıfır olduğunu
kanıtlıyor; panikleyen callback kilidi zehirlese bile gate fail-closed.

K23 guard'ları istek, cevap, context/cue metni ve provider kimliğinin
`Debug`/`Display` yüzeyine çıkmadığını; kasıtlı `derive(Debug)` ikizinin ise
sentinel'ı sızdırdığını gösterdi. `cargo test -p nen-ports`: **106 passed**;
`cargo test -p nen-providers`: **18 passed**; workspace: **695 passed / 1
ignored**. fmt, clippy, deny ve shell/doc kapıları yeşil. Kanıt:
`tasks/done/NEN-090-*.md`.

**`NEN-103` kapandı — ADR-0045, gömülü metin çıkarımı için macOS playback
adapter'ında `libavformat`/`libavcodec` yolunu kabul etti.** `EmbeddedTrackExtractor`
portu ve `nen-identity`'nin I/O'suz sınırı korunuyor; bitmap track'ler merkezi
`TrackDescriptor.is_text` sınıflandırmasıyla çeviriye kapalı kalıyor. Mevcut
libmpv dylib kapanışının libav kütüphanelerini zaten içerdiği ölçüldü; gerçek
bundle doğrulaması implementasyon task'ı `NEN-044`e bırakıldı.

**`NEN-089` kapandı — `SubtitleDocument` artık her zaman aynı, yeniden
üretilebilir overlapping bloklara ayrılıyor ve belgenin tamamından çıkarılan
bağlam her bloğa aynı biçimde giriyor.** `core/crates/nen-translate/`
üç satırlık iskeletten çıktı: `blocks.rs` (`BlockLayoutConfig`,
`BlockLayoutError`, `TranslationBlock`, `BlockLayout`) ve `context.rs`
(`ContextTerm`, `DocumentContext`). Ön koşulu olan **ADR-0015 `accepted`
oldu** (aynı gün) — taslağın iki eksik maddesi (bağlam çıkarım kuralının
somut tanımı, overlap'in blok boyutu gibi doğrulanan bir parametre olması)
kullanıcı kararıyla tamamlandı.

**Blok sınırı sabit cue sayısıyla belirleniyor, son pencere
`cueCount − blockSize`'a sabitleniyor, overlap penceresinin sözü çağrıdan
önce floor-orta noktasında bölünüyor** (ADR-0015 Karar 1/2) — bir cue hiçbir
zaman iki blokta birden çevrilmiyor. **Overlap artık blok boyutu gibi
doğrulanan bir parametre:** `1 ≤ overlap < blockSize / 2`; bu üst sınır,
sabitlenmiş son pencerede bile bir bloğun çıktı kümesinin asla boş
kalamayacağını garanti ediyor (task'ın kanıt kaydındaki matematik). **Bağlam
analizi**, belge genelinde tekrar eden, büyük harfle başlayan ve en az bir
kez cümle başı dışında görünen terimleri (üst sınır 32) çıkaran, tamamen
yerel ve deterministik bir kural — provider'a hiç gitmiyor.

**Kanıt golden + unit + iki zorunlu negatif katmanında, hepsi elle
doğrulandı.** Golden fixture (`fixtures/subtitles/blocks/layout-sample.srt`,
95 cue) için üretilen pencereler (`0..40`, `34..74`, `55..95`) ve çıktı
sınırları (`37`, `64`) elle hesaplanıp ADR-0015'in kurallarıyla birebir
eşleştiği doğrulandı; bağlam terimleri (`Killua 48 · Gon 29 · Kurapika 19`)
elle sayıldı ve `Leorio`'nun — her zaman cümle başında geçtiği için — kural
tarafından bilerek elendiği gösterildi. Negatif (zorunlu): aralık dışı blok
boyutu (29, 61) ve aralık dışı overlap (0, `blockSize/2`) tipli hatayla
reddediliyor; boş belge ayrı bir hata, bir bloktan küçük belge boş blok
üretmeden tek blok oluyor.

**K23 guard'ı (`docs/security-policy.md` §1) bağlam terimlerini kapsıyor** —
çıkarılan terim altyazı diyaloğunun bir parçası. `ContextTerm`/
`DocumentContext` `nen_domain::subtitle::Cue` emsaliyle elle `Debug` yazıyor,
yalnız sayı basıyor. Üç negatif kontrol ayrık ölçüldü: overlap sınır kuralı
sabit sınıra çekilince yalnız golden test kırmızı oldu (unit'in kendi aralığı
gevşek olduğu için bunu yakalamadı — plandan farklı ama gerçek ölçülen
sonuç), overlap doğrulama kapısı kaldırılınca 21 testten yalnız 1'i, K23
guard'ı `#[derive(Debug)]`'a çevrilince yalnız o test kırmızıya döndü ve
gerçek terim metnini bastığını gösterdi.

`cargo test -p nen-translate`: **25 passed** (21 unit + 1 golden + 3 guard).
`cargo test --workspace`: **688 passed / 1 ignored**. `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`
(yeni dış bağımlılık yok), `bash scripts/test.sh`, `bash
scripts/check-docs.sh` hepsi yeşil. Değişiklik yalnız `nen-translate` ve
doküman tarafı; `nen-ffi`, macOS kabuğu, `nen-persist` dokunulmadı. Kanıt:
`tasks/done/NEN-089-*.md`.

**M5 — Translation Core kırılıma bağlandı; sıradaki iş `NEN-089`.** M4'ün
kapanış ritüelinin son maddesi (`docs/roadmap.md` → "Milestone kapanış ritüeli":
*"sonraki milestone'un task kırılımı"*) yapılmamıştı —
`docs/milestones/M5-translation-core.md` hâlâ *"(henüz kırılmadı)"* diyordu ve
M5'in gövdesi (blok pipeline, strict validation, checkpoint/cancel,
artifact/cache, persistence, mock provider) için **sıfır task** vardı.
`nen-translate` ve `nen-persist` bugün hâlâ üçer satırlık boş iskelet.

**16 yeni task açıldı (`NEN-089`…`NEN-104`), `NEN-044` grafiğe bağlandı.**
Zincir: blok düzeni (`089`) → provider portu + deterministic mock (`090`) →
strict doğrulama (`091`) → repair bütçesi (`092`) → checkpoint/iptal (`093`) →
artifact + WebVTT (`094`) → persistence kararı (`095`) → içerik adresli depo
(`096`) → cache identity (`097`) → index + offline okuma (`098`) →
orkestrasyon (`099`) → FFI (`100`) → macOS komut/ayar (`101`) → ilerleme/iptal
(`102`) → acceptance (`104`). Ayrı kol: demux kararı (`103`) → gömülü metin
çıkarımı (`044`) → `104`. Hiçbir task `L` değil; macOS yüzeyi ve persistence
bilerek ikiye bölündü (`tasks/README.md` → bölme testi). Mevcut hiçbir ID
yeniden numaralandırılmadı.

**M5'in altı çıkış kriterinin her birinin kanıtlayan task'ı var** — eşleme
`docs/milestones/M5-translation-core.md` → "Çıkış kriteri → kanıt eşlemesi".
Altı task (`091` · `092` · `093` · `096` · `097` · `098`) `tasks/README.md`'nin
security/validation satırına giriyor, yani **negatif test zorunlu**.

**Beş ADR `proposed` olarak açıldı:** 0015 (blok stratejisi) · 0016 (doğrulama
ve onarım) · 0017 (persistence adapter) · 0018 (cache identity) · 0045 (gömülü
metin demux yolu). Numaralar kullanıcı kararıyla planlanan tablodan alındı
(0015–0018), çünkü `docs/DECISIONS.md` ve `docs/architecture.md` persistence
kararını zaten adıyla **ADR-0017** diye anıyordu. Kural 4 gereği hiçbiri
`accepted` değil; ilgili ADR kabul edilmeden o task'ın implementasyonuna
geçilmez ve SQLite/CAS gibi adlar **aday** kalır.

**Üç açık soru kullanıcı kararıyla kapandı (2026-09-08).** **S3:** M5'in ölçütü
dilsel değil **yapısal doğruluk** — M5 yalnız mock provider ile bittiği için
dilsel çıta M6'ya bırakıldı. **S4:** kaydedilmiş artifact ağ olmadan açılır,
ayrı bir offline modu anahtarı yok (`NEN-098`'in DoD'unda negatif test).
**S9:** farklı provider/model/glossary ile üretilen artifact'ler diskte yan
yana durur, M5'te menüde hedef dil başına yalnız en yeni görünür.

**Yol üstünde bağımsız bir doküman kusuru bulundu ve ayrı commit'e ayrıldı
(Kural 5).** `docs/adr/README.md` kendi "Yazılmış" tablosunda ADR-0034, ADR-0035
ve ADR-0042'yi hiç listelemiyordu; `docs/DECISIONS.md`'nin sayımı da 22 accepted
diyordu (gerçek: 28). Kırılıma katılmadı, kendi commit'ini aldı (`3dcfed2`).

Kod değişmedi (planlama task'ı; Kural 1 gereği kırılım kodun ön koşulu).
`bash scripts/task-index.sh` ve `bash scripts/check-docs.sh` çıkış 0.

**`NEN-072` kapandı — M5'in ilk task'ı, uzak medyanın ilk 64 KiB byte
penceresinden Matroska/MP4 container title/year'ı artık `MediaEvidence`'a
taşınıyor.** `MediaEvidence.container` alanı `NEN-036`'dan beri vardı ama hiçbir
üretim yolu doldurmuyordu; `nen-identity/src/container.rs`'e yeni bağımlılık
eklenmeden (kapsam zaten tam demux değil, ADR-0009 Karar 6'nın işaret ettiği
dar bir okuma) elle yazılmış, sınırlı bir EBML/ISOBMFF walker eklendi ve
`nen-app::remote_evidence::collect_with_policy`'nin zaten çektiği
`head_window.body`'ye bağlandı.

**Yol üstünde gerçek bir panik bulundu ve aynı task içinde düzeltildi:**
EBML'in 8 baytlık size VINT kodlaması (`contract-clip.mkv`-tarzı her Matroska
dosyasında sıradan) marker-mask hesaplamasında `u8` üzerinde 8-bit sağa
kaydırma deniyordu — bu, adversarial girdi değil kendi golden fixture'ının
kendisiyle ilk testte tetiklendi. Düzeltme `checked_shr`; crate'in zaten
`#![deny(clippy::indexing_slicing)]` kuralı, elle indekslemenin tamamının
`.get()` tabanlı, panik yapamayan erişime çevrilmesini de zorladı.

**Kanıt hem format hem security kesişiminde.** Golden: gerçek ffmpeg-üretilmiş
`fixtures/media/container/valid-title.{mkv,mp4}` title/year'ı uçtan uca
doğru veriyor. Negatif (zorunlu), üç ayrı senaryo: oversized declared size
(buffer'a clamp, panik yok), 2000 seviye iç içe geçmiş element (walker şema-
sabit tek seviyeye iniyor, girdinin iddia ettiği derinliğe göre değil — panik
yok, hızlı), NEN-022'nin `broken-clip.mkv`'si + elle kesilmiş header'lar
(boş sonuç). `nen-app` tarafında tanınmayan pencerenin `MediaEvidence`'a hiç
container eklemediği ayrıca test edildi.

`cargo test --workspace`, `cargo fmt --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo deny check` (yeni bağımlılık yok),
`bash scripts/test.sh`, `bash scripts/check-docs.sh` hepsi yeşil. Değişiklik
yalnız `nen-identity`/`nen-app`; FFI ve macOS kabuğu dokunulmadı. Kanıt:
`tasks/done/NEN-072-*.md`.

**`NEN-088` M4'ü kapattı.** Gerçek Stremio 5.1.26 “MPV içinde oynat” eylemi,
kurulu ve bütünlüğü doğrulanmış geri alınabilir köprü üzerinden taze Nen Player
`.app`'ini hem soğuk hem sıcak durumda açtı. Stremio'nun ölçülen `0` başlangıç
değeri baştan oynatma olarak gözlendi; metadata yokluğu akışı bozmadı. Geçici
log taramalarındaki URL, query, özel yol ve handoff argv sayaçları sıfırdı.

**`NEN-084` kapandı — Nen Player'ın MPV argv sözleşmesi kanıtlandı.** Gerçek
Stremio 5.1.26'da harici oynatıcı yüzeyi yeniden görüldü; ADR-0043 sınırı gereği
doğrudan Nen Player hedefi iddia edilmedi. Aynı ölçülmüş CLI sözleşmesi gerçek
`.app` üzerinde `contract-clip.mkv` ile 00:12'de oynadı; metadata'sız opak
loopback fixture 00:09'da oynadı. Unified log ve stdout/stderr negatif
taramasında locator, URL/query, özel yol ve argv parçaları için eşleşme 0.
Ekran ve adım kaydı: `evidence/M4/NEN-084-checklist.md`.

**`NEN-083` kapandı — handoff girdisi hiçbir log yüzeyine ulaşamıyor.** Core ve
FFI guard’ları argv’den çıkan request’i, locator’ları ve hata `Display`’ini
yasak parçalarla tarıyor; `HandoffMetadata` doğrudan shape-only debug ile
korunuyor. macOS’ta `HandoffOutcome.rejected` payload’sız hale getirildi ve
handoff sonucu ile üretilen FFI locator/request için `String`/Debug/reflection
gösterimleri güvenli türevlerle sınırlandı. 239 serial Swift testi ve workspace
gate’leri yeşil. Kanıt: `evidence/M4/NEN-083-checklist.md`.

**`NEN-082` kapandı — locator kanıtı opsiyonel kaldı.** `PlayerModel` uzak
`http`/`https` handoff'u hemen yüklerken mevcut `collectRemoteEvidence`
adaptörünü detached task'ta çağırıyor. URL path'i, son yönlendirme URL'si ve
`Content-Disposition` adı mevcut `MediaEvidence`/ADR-0009 sırasına bırakıldı;
metadata yokluğu veya typed HTTP hatası fatal/transient playback yüzeyine
taşınmıyor. Yerel handoff, `⌘O` ve recent media akışları evidence çağrısı
yapmıyor. Kanıt: `evidence/M4/NEN-082-checklist.md`.

**`NEN-081` kapandı — handoff'un taşıdığı başlangıç pozisyonu artık
uygulanıyor.** `NEN-080` `start_position_ms`'i zaten ayrıştırıp taşıyordu,
`PlayerModel.handleHandoff` onu bilerek kullanmıyordu; bu task son adımı attı.
Kapsam yalnız macOS kabuğu (`PlayerModel.swift`) — Rust çekirdeğinde kod
değişmedi.

**Planlama sırasında ADR-0043 Karar 2'de bir tutarsızlık bulundu ve kullanıcı
kararıyla netleşti (ADR'nin Notlar'ına eklendi, supersede edilmedi).** Karar
hem pozisyonun ADR-0042'nin erteleme mekanizmasından (`load` ile birlikte,
`FILE_LOADED` anında) geçmesini hem de süreyi aşan bir değerin sessizce
düşüp medyanın **baştan** açılmasını istiyordu — ama `load` anında süre
bilinmiyor (mpv onu ancak `FILE_LOADED`'da biliyor), yani ertelenmiş, süreyi
aşan bir seek medyayı sonda açardı, baştan değil. Kullanıcı kararı: pozisyon
kabukta (`pendingHandoffStartPositionMs`) tutulur, `apply(_:)`'ın `.ready`
dalında — `play()`'den **önce** — uygulanır; süre okunamayan medyada (canlı
yayın) pozisyon yine uygulanır, kapı yalnız "süre biliniyor ve pozisyon ≥
süre" olduğunda kapanır.

**Kanıt hem sahte hem gerçek motorla.** `FakeSession` üzerinden 10 test
(seek `play`'den önce — çağrı sırası kaydedilerek doğrulandı —, süreyi
aşan/aşmayan/okunamayan pozisyon, başarısız yükleme ve ikinci handoffun
öncekini düşürmesi, soğuk açılış kuyruğu). DoD'un kendi maddesi gerçek
libmpv de istiyordu: yeni `HandoffStartPositionRealEngineTests`,
`PlayerModel`'in gerçek `sessionFactory`'siyle (`PicturelessSurfaceTests`'in
zaten kurduğu pencereli-yüzey deseni, ilk kez `PlayerModel` üzerinden
sürüldü) `contract-clip.mkv`'yi 12.000 ms'de açıp inen pozisyonu bekleyerek
okuyor, sonra aynı medyayı 999.000 ms'le (30.008 ms'lik süreyi kat kat aşan)
yeniden açıp düştüğünü doğruluyor. Üç ayrı negatif kontrol ayrık ölçüldü.

**Paralel test koşusunda, bu task'tan bağımsız bir contention bulgusu
izole edildi.** Bu oturumun kendi masaüstü yükü altında `bash
scripts/test-macos.sh`'ın varsayılan paralel koşusu ara sıra, önceden
belgelenmiş main-actor zamanlama testlerinde (`NEN-049`/`NEN-066` sınıfı)
kırmızı çıktı; `swift test --skip HandoffStartPositionRealEngineTests` ile
yeni suite tamamen çıkarıldığında **aynı** kırmızı **aynen** kaldı — sebep bu
task değil. Kesin kanıt `swift test --no-parallel`: iki ardışık koşu,
232/232, 0 kırmızı.

Rust workspace **646 passed / 1 ignored** (kod değişmedi, yalnız iki yorum
güncellendi), fmt/clippy/`cargo deny check` temiz. macOS Swift paketi
**221 → 232**. Gerçek `.app` kabulü: doğrudan argv exec, `contract-clip.mkv`
üzerinde `--start=5` (transport bar `00:05`, zaten oynuyor) ve `--start=999`
(medya baştan açıldı, `00:02`, hata yok) — ekran görüntüsüyle kanıtlı. Kanıt:
`evidence/M4/NEN-081-checklist.md`.

**Push sırasında bağımsız bir CI kusuru görüldü, ayrı task'a yazıldı.** Yalnız
`docs/adr/0043-*.md` değiştiren commit (`244baee`) GitHub Actions'ta `cargo
test` adımında `spike-async-cancel::tests::uncancelled_job_completes_and_commits_exactly_once`
ile kırmızı döndü (`join()`'ün döndüğü an `live_jobs()` henüz sıfır değildi);
**aynı** commit `gh run rerun` ile yeniden koşulduğunda tamamen yeşil geldi —
bir yarış, bu task'ın içeriğiyle ilgisiz. `NEN-085` olarak dosyalandı (M1,
spike koduna ait); geçerli olan işi bekletmedi.

**`NEN-080` kapandı — başka bir uygulamanın Nen Player'a verdiği medya
`⌘O` ile açılmış gibi aynı yoldan oynuyor.** ADR-0043'ün üç yüzeyi
(`argv` + open-with + `nenplayer://` scheme) core'da tek bir ayrıştırıcıya
(`nen_app::handoff::parse_argv`/`parse_url`) indirgendi — bayrak adları
(`--start`/`--start-time`/`--start-position`, iki yazım), tanınmayan
bayrağın sessizce atlanması (NEN-078 Bulgu 4'ün `--no-terminal`'i), `#t=`
fragment'i, `file://`/`http`/`https` çözümü (uzak şema kapısı
`remote_evidence::validate_url`'in **aynısı**, ikinci kopya açılmadı) ve
pozisyonun saniye→ms dönüşümü. FFI gate aynı ayrımı gate'e taşıdı; locator
bu kez gate'ten **çıktığı** için (subtitle path'lerinin tersine) `Debug` elle
yazıldı. macOS'ta yeni `HandoffIntake`/`HandoffCoordinator`,
`PlayerModel.openMedia(at:)`'in `http`/`https`'i de kabul etmesi ve
`Info.plist`'e `CFBundleDocumentTypes`/`CFBundleURLTypes` eklenmesiyle
tamamlandı.

**Yol üstünde, handoff'tan bağımsız bir pencere yaşam döngüsü kusuru
bulundu ve düzeltildi.** Gerçek `.app` kabulünün ilk denemesinde medya hiç
açılmadı: `Window(id:)` + `.windowResizability` sahnesi her başlatmada,
pencere görünür olmadan **önce**, kök view'ı bir kez tam söküp
(`onDisappear` → `model.shutdown()`) ~100 ms içinde yeniden kuruyordu; video
yüzeyi bu sarsıntıyı sağ çıkıyordu ama ikinci kuruluşta bir daha hiç
`attach` edilmiyordu, yani oturum kalıcı `nil` kalıyordu. `Info.plist`'in
yeni girdileri kaldırılınca da aynen gözlendi — sebep onlar değildi; `⌘O`
bugüne kadar bu ~100 ms'lik pencereden çok sonra tetiklendiği için kusur
hiç görünmemişti. Düzeltme tek satır: `PlayerRootView`'in `.onAppear`'ına
`model.resume()` — `NEN-046`'nın Dock'tan yeniden açma için zaten kullandığı
aynı kendini-onarma çağrısı.

Rust workspace **646 passed / 1 ignored** (bu task'ın 39 yeni testi), macOS
Swift paketi **221/221** (19 yeni test, biri gerçek `.app`'te bulunan sıralama
kusurunun regresyon testi), `cargo fmt`/`clippy`/`cargo deny check` ve
`bash scripts/test-macos.sh` iki ardışık koşuda yeşil. Gerçek `.app` kabulü
dört senaryoda: uygulama kapalıyken `open -a` (open-with), uygulama açıkken
ikinci `open -a`, gerçek Stremio mekanizmasıyla doğrudan argv exec
(`--start=5 --no-terminal`), ve negatif (`ftp://` panik yaratmadan reddedildi).
`nenplayer://` scheme'i ADR-0043 Bulgu 9 gereği bu makinede uçtan uca
doğrulanamıyor (Developer ID yok), roadmap **S11**'e bağlı — kod yolu birim
testleriyle kanıtlı. Kanıt: `evidence/M4/NEN-080-checklist.md`.

**`NEN-079` kapandı — macOS handoff alıcı yüzeyi `accepted` bir ADR ile
sabitlendi.** `NEN-078`'in ölçümü kesin bir öneri üretmemişti (adayları
karşılaştırıyordu); bu oturumda kullanıcıyla dört karar noktası netleştirildi
ve `docs/adr/0043-macos-handoff-surface.md` beş numaralı Karar olarak yazıldı:
(1) alıcı yüzey **üçü birden** — `CFBundleDocumentTypes` (open-with),
`CFBundleURLTypes` (`nenplayer` scheme'i) ve argv; (2) başlangıç pozisyonu
argv'de üç eşdeğer bayrak (`--start=`/`--start-time=`/`--start-position=`,
`NEN-078` Bulgu 6'nın gösterdiği "sabitlenen CLI sözleşmesi, uygulama kimliği
değil" bulgusuna dayanarak — taklit değil) veya scheme'de `#t=` fragment'i ile
gelir, ADR-0042'nin `deferredSeekMs` kontratından geçer; (3) handoff'un kendi
portu **yok** — inbound bir OS olayı sayılır, core çağırmaz OS kabuğa iter;
(4) ayrıştırma core'da (`nen-app`), kabuk yalnız ham olayı taşır; (5) handoff
metadata'sının ayrı bir alanı yok, locator'ın kendisi ADR-0009'un kanıt
katmanına girer.

**`docs/architecture.md`'nin kendi içindeki tutarsızlığı kapandı.** Sınır
kuralı "Stremio handoff"u `port + platform adapter` sınıfına koyuyordu ama
port tablosunda böyle bir port yoktu. "Stremio handoff" ve "lifecycle" sınır
kuralı listesinden çıkarıldı; yerine "Inbound OS olayları port değildir" alt
bölümü eklendi (ADR-0043 Karar 3'e referansla).

**Dürüstçe kaydedilen iki sınır.** Stremio'nun kendi ayar listesi
(`Etkisizleştirildi · MPV · IINA · Infuse · M3U Playlist`) ve oynatıcı
ekranının `...` menüsü (sabit VLC/MPV) kapalı küme — Nen Player bunlara
giremiyor, çünkü ADR bayrak-taklidini kabul edip bundle-kimliği taklidini
reddediyor; `NEN-084`'ün kabul koşusu bu yüzden Stremio'nun bu iki yüzeyinden
değil, kabul edilen sözleşmeyi konuşan bir gönderici üzerinden yapılacak.
`nenplayer://` scheme'i bu makinede Developer ID eksikliği yüzünden
(`NEN-078` Bulgu 9) kanıtlanamıyor, roadmap **S11**'e bağlı.

Kod değişmedi (karar/doküman task'ı). `bash scripts/check-docs.sh` çıkış 0.
Kanıt: `tasks/done/NEN-079-*.md`.

**`NEN-078` kapandı — Stremio'nun macOS'ta harici oynatıcıyı hangi mekanizmayla
çağırdığı gerçek Stremio 5.1.26 ile ölçüldü.** `NEN-025`/ADR-0034 emsali izlendi:
depoya girmeyen, geçici bir prob (`HandoffProbe.app`) `mpv:`/`iina:`/`vlc:`
scheme'lerini claim etti ama Stremio'nun iki tetiklemesinde de **hiç**
çağrılmadı — Stremio custom URL scheme kullanmıyor. Bunun yerine hedef
oynatıcıyı **doğrudan, kendi CLI biçimiyle** başlatıyor: `ps aux`'un yalnız
bayrak adları okundu (`--start=`/`--no-terminal` mpv için, `--start-time=`/
`--no-video-title-show` VLC için — URL hiçbir dosyaya yazılmadı, K23).

**Ayar var ama kapalı bir liste** (Ayarlar → Oynatıcı → Gelişmiş → "Harici
oynatıcıda oynat": `Etkisizleştirildi · MPV · IINA · Infuse · M3U Playlist`,
serbest metin yok) ve bu ayarı değiştirmek oynatmayı etkilemiyor — harici
oynatıcı yalnız oynatıcı ekranının `...` menüsünden elle tetikleniyor, o menü
de ayardan bağımsız hep aynı iki sabit seçeneği (VLC/MPV) gösteriyor.

**Pozisyon taşınmıyor.** Aynı medya iki farklı gerçek oynatma konumunda
(~48 sn ve ~15 dk 29 sn) harici oynatıcıya gönderildi; başlangıç bayrağı her
ikisinde de **0** geldi. Metadata için ayrı bir alan yok, yalnız URL'nin
dosya adı segmenti zımni ipucu taşıyor. Bu iki bulgu M4'ün "doğru pozisyondan
oynuyor" kriterini ve `NEN-081`/`NEN-082`'nin kapsamını doğrudan etkiliyor;
kesin karar `NEN-079`'a (ADR) bırakıldı — ölçüm kesin bir yüzey önerisi
üretmiyor, iki adayı (custom scheme vs. isim bazlı CLI taklidi) ölçülmüş
verilerle karşılaştırıyor.

**Sağırlık kontrolünün kendisi ayrı bir bulgu üretti:** bu makinede gerçek
Developer ID imzası yok (`security find-identity` → 0, `NEN-043`'ün de
kaydettiği durum) ve ad-hoc imzalı prob `spctl -a -vv` altında `rejected` —
böyle bir uygulama hiçbir göndericiye scheme handler olarak görünmüyor
(kontrol grubu: düzgün imzalı `claude://`/`bitwarden://` normal çözülüyor).
Stremio zaten scheme kullanmadığı için bu ölçümü engellemedi, ama gelecekte
bir scheme yüzeyi seçilirse bu kısıt notarization boşluğuyla birleşecek.

Kod değişmedi (ölçüm task'ı). Prob temizliği doğrulandı: `lsregister -u`
sonrası `lsregister -dump`'ta prob referansı **0**; Stremio'nun kendi ayarı ve
seek edilen konum ölçüm sonunda özgün haline geri getirildi. `bash
scripts/check-docs.sh` çıkış 0. Kanıt: `evidence/M4/NEN-078-measurement.md`.

**M3 kapandı — 2026-08-25'te başladı, 2026-09-07'de bitti, 46 task
(44 done / 2 canceled), 14 gün.** 2026-09-06 kullanıcı kararı gereği beş çıkış
kriteri `NEN-028`'de kanıtlandıktan sonra da beklendi ve `milestone: M3`
etiketli **her** task bitirildi; son iş kalemi `NEN-043` idi. Kapanış ritüeli
(`docs/roadmap.md` → "Milestone kapanış ritüeli") uygulandı: retro yazıldı
(süre · yanlış çıkan varsayımlar · ADR kararları · sonraki milestone), M3
doküman'ının 10 task sayan bayat listesi 46'nın tamamına genişletildi, roadmap'in
`🔵 sıradaki — toolchain blocker` satırı ve *"Sırada `NEN-023`/`NEN-024`"*
diyen giriş paragrafı düzeltildi.

**Retro'nun taşıdığı asıl bulgu:** M3'ün task sayısı 10'dan 46'ya çıktı ve
büyümenin sebebi kapsam kayması değil, **teşhis**ti — ölçüm dört kez
(`NEN-066`, `NEN-049`, `NEN-069`, `NEN-076`) task dosyasının kendi yazdığı
teşhisi çürüttü. 15 ADR yazıldı (14 accepted, ADR-0036 rejected); hiçbiri
`superseded` olmadı, ama iki kez bu ihtimal ölçülüp reddedildi ve "karar
değişti" ile "karar geçersiz" ayrımı ADR-0035/ADR-0041 precedent'iyle
sabitlendi.

**Sıradaki milestone M4 — Stremio Handoff (macOS)** (kullanıcı kararı,
2026-09-07); kırılımı kapanışın ikinci yarısı olarak üretildi:
`NEN-078` ölçüm → `NEN-079` ADR → `NEN-080` alıcı yüzey →
{`NEN-081` başlangıç pozisyonu · `NEN-082` metadata → kanıt ·
`NEN-083` log denetimi} → `NEN-084` CLI sözleşmesi kanıtı → `NEN-086` launcher
ölçüm düzeltmesi → `NEN-087` geri alınabilir bridge kurulumu. M4'ün gerçek
Stremio akışını doğrulayacak `NEN-088` backlog'dadır.

**Kırılım ölçümle başlıyor, çünkü alıcı yüzey ölçülmeden seçilemez.**
`platforms/macos/Resources/Info.plist` bugün ne `CFBundleDocumentTypes` ne
`CFBundleURLTypes` taşıyor ve `NenPlayerApp.swift`'in `AppDelegate`'inde
`application(_:open:)` yok — yani yüzey sıfırdan kurulacak, ve hangisinin
kurulacağını yalnız gönderen taraf söyler. Stremio 5.1.26 bu makinede kurulu,
dolayısıyla `NEN-078` tahmin değil ölçüm yapabilir (`NEN-025`/ADR-0034
emsali: ölçüm kod yazılmadan önce yapılır ve planlanan çözümü elemeye
yetkilidir).

**Kırılım sırasında bulunan mimari tutarsızlık `NEN-079`'a yazıldı:**
`docs/architecture.md`'nin sınır kuralı "Stremio handoff"u `port + platform
adapter` sınıfında sayıyor, ama aynı dosyanın port tablosunda böyle bir port
yok. ADR ya tabloya satır ekleyecek ya sınır kuralını düzeltecek.

**`NEN-043` kapandı — Nen Player'ın `.app`'i artık Homebrew kurulu olmayan
bir Mac'te açılıyor.** `scripts/bundle-macos.sh` (yeni) ve onun graf işini
üstlenen `scripts/lib/rewrite_macho_deps.py` (yeni — bash 3.2 ilişkisel dizi
desteklemediği için) libmpv'nin **48 dylib'lik** geçişli Homebrew kapanışını
hesaplayıp `Contents/Frameworks/`'e kopyalıyor, `install_name_tool` ile tüm
yolları `@rpath`/`@loader_path`'e çeviriyor, `Contents/Resources/licenses/`
altına `LICENSE` + mpv'nin kendi lisans metinlerini + üretilen bir
`THIRD-PARTY.md`'yi (formül + sürüm + SPDX lisansı, her satır) koyuyor, ad-hoc
imzalıyor ve bundle'ı bağımsızca tarayıp Homebrew referansı kalmadığını
doğruluyor.

**Kapsam kullanıcı kararıyla ikiye bölündü.** DoD'un iki maddesi
(`spctl -a -vv`, notarization ticket) bu makinede kanıtlanamıyordu:
`security find-identity -v -p codesigning` → 0 kimlik, `xcrun notarytool
history` → kimlik bilgisi yok. Developer ID imzası, hardened runtime,
notarization ve `spctl`, `NEN-043`'ün YAPILMAYACAK'ına ve roadmap **S11**'e
yazıldı; Apple Developer Program üyeliği alındığında numaralandırılmış bir
task açılacak. Bundle'a giren lisans metinleri de kullanıcı kararıyla
daraltıldı: 48 formülün tam metni değil, mpv'nin kendisi + üretilen bir
bildirim tablosu.

**Negatif kontrol gerçek makinede, gerçek `.app`'te koşuldu.**
`/opt/homebrew/Cellar` geçici olarak yeniden adlandırıldığında (aynı gizli
pencerede, tek shell çağrısında, `trap` ile garantili geri yükleme):
gömme-öncesi (Homebrew'a dinamik bağlı) bir snapshot `dyld: Library not
loaded` ile çöktü — kontrol sağır değil; gömülü `.app` **aynı pencerede**
açıldı, uygulamanın kendi son-açılanlar listesinden gerçek bir dosyayı
(`GTAVI_An_Extended_Look.mp4`) oynattı (ekran görüntüsü kanıtlı, playhead
ilerliyor); `vmmap` yüklü image listesinde tek bir `/opt/homebrew` girdisi
kalmadığını gösterdi. Aynı mekanizma `scripts/tests/bundle-macos.test.sh` ile
deterministik hale getirildi: sahte bir Homebrew düzeni üzerinde gerçek bir
`mainbin → liba → libb` zinciri derlenip yeniden yazılıyor ve vendor prefix'i
diskten kaldırıldıktan **sonra çalıştırılarak** sınanıyor; ayrı bir negatif
kontrol, yeniden yazma atlanmış ham kopyanın aynı koşulda çöktüğünü ve
tarama deseninin bunu yakaladığını doğruluyor.

**Yol üstünde bulunan araç kusuru, kendi kendine düzeltildi:**
`scripts/task-index.sh`'ı `bash scripts/task-index.sh > tasks/INDEX.md` ile
çağırmak dosyayı bozuyor — script `tasks/INDEX.md`'yi zaten **kendi içinde**
yazıyor, dıştaki yönlendirme onun stdout'undaki yalnızca `"Üretildi:
tasks/INDEX.md"` satırını dosyanın başına, script'in içerideki tam yazımından
**sonra** açtığı ayrı bir dosya tanıtıcısıyla çakışarak yazıyor ve `# Task
Index` başlığını kısmen eziyor. Düzeltme kod değişikliği değil, çağrı biçimi:
script argümansız çalıştırılır, yönlendirilmez. `bash scripts/task-index.sh
--check` bunu şimdi doğruluyor.

Rust workspace ve macOS Swift paketi **dokunulmadı** (fmt/clippy/`cargo test
--workspace`/`cargo deny check` **202/202 Swift**, sırasıyla regresyon
kontrolü olarak koşuldu, hepsi yeşil); değişiklik yalnız `scripts/` altında.
`bash scripts/test.sh` yeni `bundle-macos.test.sh` dahil **3/3**, `bash
scripts/check-docs.sh` çıkış 0. Kanıt: `evidence/M3/NEN-043-checklist.md`
(iki gerçek ekran görüntüsü dahil).

**`NEN-077` kapandı — minimum pencerede video oranı ile player kromu artık
aynı koordinatta.** `693×390 pt` krom tabanı safe-area içi kullanılabilir alan
olarak ayrıldı; gerçek pencere minimumu ölçülen titlebar payıyla video oranında
türetiliyor ve SwiftUI root bu payı minimumundan yalnız bir kez çıkarıyor.
Gizli başlıklı gerçek `NSWindow` matrisi window/root/`MPVVideoView` eşitliğini,
`57 pt` transport'u ve kenar kontrollerini 16:9, 4:3, 2.39:1 ve 9:16 için
sabitledi. Gerçek `.app` kabulünde minimumlar sırasıyla `750–751×422`,
`693×520` ve `1008×422 pt`; tam ekran çıkışı yeniden `751×422 pt` ölçüldü.
Safe-area telafisinin kaldırıldığı negatif kontrol 10 issue üretti. Tam macOS
paketi 202 test / 22 suite ile, `.app` build'i, strict codesign ve depo kapıları
yeşil. Kanıt: `evidence/M3/NEN-077-checklist.md`.

**`NEN-052` kapandı — yüklenirken verilen seek artık kaybolmuyor.** Fake motor
`.buffering` durumunda verilen bir seek'i her zaman kabul ediyordu, gerçek
libmpv adapter'ı ise reddediyordu (`EngineFailure(code: -12)`) — kabuk bunu
"beklenmeyen motor hatası" diye gösteriyordu, oysa durum normaldi.
`evidence/M3/NEN-052-measurement.md` gerçek adapter'da beş ölçümle doğruladı:
yükleme penceresi **2.5–12 ms** — bir insanın tuşa basıp yakalaması için
fazlasıyla dar, yani nadiren değil her zaman rastlanabilecek bir yarış.

**`ADR-0042` accepted: seek reddedilmez, tutulur, `FILE_LOADED` anında
uygulanır.** Contract kitine tek senaryo eklendi ("a seek issued the instant
loading starts is answered, not refused") — kasıtlı olarak `Settle` öncesinde
hiç yok, motor ister hâlâ açılıyor ister zaten hazır olsun aynı cevabı
(kabul + hedefte iniş) vermeli. Rust'ta `LoadingWindowEngine` gerçek adapter'ın
ölçülen penceresini birkaç `state()` sorgusu olarak taklit edip **kitin
tamamını** geçiyor; macOS'ta `MPVPlaybackEngine`'e `deferredSeekMs` eklendi ve
gerçek libmpv adapter'ı aynı kiti üç ardışık tam paket koşusunda (197/197)
geçti.

**Negatif kontrol iki seviyede.** Rust'ta `Defect::RefusesASeekWhileLoading`
düzeltmeden önceki davranışı üretip kiti deterministik olarak kırmızıya
döndürüyor. Swift'te erteleme geçici olarak kaldırılıp hem paylaşılan kit hem
pencereyi kendi kendine doğrulayan yeni mekanizma testi beklenen
`EngineFailure { code: -12 }` mesajıyla kırmızıya döndü; üçüncü kontrol
Karar 4'ü (yükleme başarısız/`stop` olunca ertelenen seek'in düşmesi) ayrıca
kanıtladı.

**Yol üstünde bulunan bulgu, ürün koduna değil teste aitti:** ilk mekanizma
testi `Ready`'yi görür görmez `positionMs()`'i tek seferde okuyordu — tam
paket koşusunun yükü altında bu, ertelenmiş `seek` komutunun event-loop
thread'ine gerçekten ulaşmasından önceye denk geldi (`landed = 0 ms`).
`NEN-051`'in kontrat seviyesinde öğrettiği ders ("hazır" tamamlandı anlamına
gelmez) testin kendi iddiasında tekrarlanmıştı; düzeltme pozisyonu bekleyerek
okumaktı (`playPauseAndSeekOnTheLocalClip`'in zaten kullandığı desen). Kontrat
kitinin kendi `AwaitSeekLanding` adımı zaten olayı bekliyordu, üç koşuda da
yeşildi.

Rust workspace **605 → 607**, macOS Swift paketi **197/197**; fmt, clippy,
cargo-deny, `.app` build'i, strict codesign, shell ve doküman kapıları yeşil.
FFI yüzeyi, `nen-ffi` ve kabuk (`PlayerModel`) dokunulmadı. Gerçek `.app`'te
elle klavye testi yapılmadı — ölçülen pencere insan tepki süresinden çok daha
kısa, otomatik test tek güvenilir kanıt yolu. Kanıt: `tasks/done/NEN-052-*.md`.

**(2026-09-07 itibarıyla tamamlandı.)** O tarihte geçerli olan karar şuydu:
*"M3 kapanmıyor: `milestone: M3` etiketli task'ların hepsi bitecek"*
(kullanıcı kararı, 2026-09-06) — beş çıkış kriteri `NEN-028`'de kanıtlanmıştı
ve kanonik task listesi (`NEN-021`…`028`, `061`, `062`) tamamlanmıştı, ama
kriterlerle erken kapanış yapılmadı. Son kalan backlog `043` de kapanınca M3
2026-09-07'de kapatıldı; roadmap'in M3 satırı ve milestone dokümanının bayat
task listesi o kapanışta düzeltildi (bkz. yukarıdaki kapanış kaydı). `NEN-073`, kullanıcının gerçek `.app`te minimum pencere
sınırının uygulanmadığını göstermesiyle yeniden açıldı ve düzeltici kapanışta
gerçek köşe sürüklemesiyle kanıtlandı; canlı resize regresyonu `NEN-074`
ile kapandı.

**`NEN-076` kapandı — `check-docs.sh`'ın STATUS güncellik denetimi CI'ın
shallow clone'unda artık her done task'ı bugün sanmıyor.** Kusur `NEN-075`
çalışılırken ölçüldü: yalnız ADR dosyaları içeren `1ad194d` (ADR-0041 kabulü)
GitHub Actions run **34086963805**'i `failure` ile bitirdi, aynı ağaç yerelde
`ok` veriyordu. Sebep `.github/workflows/ci.yml`'in varsayılan `fetch-depth: 1`
kullanması — tek commit'lik bir geçmişte o commit **bütün** dosyaları eklemiş
görünür, dolayısıyla adım 9'un eski `git log -1 --format=%cs -- "$f"` sorgusu
her done task için **HEAD'in** tarihini bildiriyordu. `NEN-075`'in kapanışı
STATUS'u ileri taşıdığı için CI o commit'te yeşile döndü (run 34088324092)
ama mekanizma durmuyordu: STATUS'a dokunmayan, yeni bir günde atılan **her**
commit — her ADR-only ve her tooling commit'i — aynı yanlış hatayı verecekti.

**Kullanıcı kararı: kapanış tarihi git geçmişinden değil, done task'ın kendi
`closed` frontmatter alanından okunur.** İki alternatif — `fetch-depth: 0` ve
shallow'da denetimi atlamak — reddedildi: ilki denetimin kendisini shallow'da
yanlış bırakıyordu (yalnız CI'ın bugünkü derinliği yeterli olduğu için
görünmezdi), ikincisi denetimi tam koşması gereken yerde (CI) hiç
koşturmuyordu. `closed` alanı denetimi geçmişin derinliğinden **tümüyle**
bağımsız kılıyor — adım 9 artık git'e hiç dokunmuyor, `fm_get "$f" closed`
okuyor. Yeni adım 3c, `state: done` her task'ın `closed`'ının `YYYY-MM-DD`
biçiminde dolu olduğunu zorluyor.

**64 done task'a `closed:` backfill edildi, elle değil ölçülerek.** Her dosya
için iki bağımsız git sorgusu (`git log -1` ve `git log --reverse | head -1`)
**64'ünde de birebir aynı tarihi** verdi — göç yeni bir doğru/yanlış
üretmedi, yalnız kaynağı değiştirdi. `fetch-depth: 1` kalıyor;
`.github/workflows/ci.yml`'e dokunulmadı, çünkü kararın anlamı tam olarak
denetimin artık geçmiş derinliğini önemsememesi.

Yeni test paketi `scripts/tests/check-docs.test.sh` T14–T17 (toplam 20
doğrulama). T14/T15 `1ad194d`'nin şeklini üreten iki commit'lik bir fixture'ı
`git clone --depth 1` ile klonlayıp — CI'ın kendisiyle aynı derinlikte —
denetimin doğru cevabı verdiğini ve bayat bir STATUS'u yine yakaladığını
gösteriyor; T16/T17 `closed` alanının eksik/bozuk biçimini yakalıyor. Üç ayrık
negatif kontrol izole ölçüldü: adım 9 eski git-tabanlı hâline dönünce yalnız
T14/T15 kırmızı, biçim kapısı kaldırılınca yalnız T16/T17 kırmızı, adım 9'un
karşılaştırması sabitlenince T13 ve T15 kırmızı. `bash scripts/test.sh` ve
`bash scripts/check-docs.sh` yeşil. Rust ve Swift'e dokunulmadı. Kanıt:
`tasks/done/NEN-076-*.md`.

**`NEN-075` kapandı — `Film.mkv`'nin yanındaki `Film.tr.srt` artık elle
seçilmeden bulunuyor.** `NEN-057`'nin ölçtüğü kapsam sınırı kapatıldı:
sidecar taraması bugüne kadar yalnız tam basename eşleşmesine
(`Film.mkv` → `Film.srt`) bakıyordu, dil alt-uzantılı adlar taramadan hiç
geçmiyordu. `subtitle_files::sidecars_of` artık medyanın kendi dizinini
**bir kez, özyinelemesiz** listeliyor ve `<basename>.` önekli, `.srt` ile
biten her girdiyi aday sayıyor; `admit()`'in symlink, traversal,
regular-file ve boyut kapıları her aday için **teker teker** aynen
çalışıyor — büyüyen tek şey aday kümesi. `NEN-057`'nin dil ipucu
(`from_file_name`) zaten ortak yükleme yolundaydı, dolayısıyla dil hattına
dokunulmadı.

**Önce ADR-0041 yazıldı, kullanıcı onayladı.** ADR-0034 Karar 3'ün "dizin
listelemez" cümlesi bu kararla değişti; ADR-0034 supersede edilmedi, gövdesi
duruyor ve Notlar'a işaret eklendi (ADR-0035'in kurduğu precedent). Üç
kullanıcı kararı alındı: dizin listelemesi (sabit dil-kodu tablosu değil —
ikinci bir ISO tablosu ve kombinatoryal patlama anlamına gelirdi), her
alt-uzantı aday (dili çözülemeyen `Film.backup.srt` da kabul edilir, dili
içerikten gelir), ve **16** adaylık belgeli bir üst sınır — tam basename
eşleşmesi sıralamada her zaman ilk gelip sınırdan her zaman kurtulacak
şekilde.

**Dört negatif kontrol ayrık ölçüldü:** dizin listelemesi geri alınınca
**8** kırmızı, sınır sıralaması geri alınınca **1** kırmızı, `admit()`'in
symlink kapısı kaldırılınca **2** kırmızı, harf-duyarsız dedup geri alınınca
**1** kırmızı. Yol üstünde ölçülen bir kenar durum (`Film.SRT`'nin
büyük/küçük harf farkıyla ikinci bir aday sayılmaması) ayrı testle kapatıldı.

**Gerçek `.app` kabulü** `Film.mkv`'nin yanına üç sidecar (`Film.srt`,
`Film.en.srt`, `Film.tr.srt`), bir symlink (`Film.fr.srt`) ve başka bir
"medyaya" ait bir dosya (`Baska.tr.srt`) konarak koşuldu: `⇧⌘O`'ya hiç
dokunulmadan üçü de menüde (`Kullanıcı Altyazıları 3`), otomatik seçim
Türkçe sidecar'ı açtı ve repliği ekranda çizdi; symlink sessizce yok,
`Baska.tr.srt` hiç aday olmadı. Kanıt: `evidence/M3/NEN-075-checklist.md`.

Rust workspace **594 → 605** (+11, dokuzu NEN-075'in kendi testleri, biri
kenar durum), üç ardışık temiz koşuda 0 kırmızı; fmt, clippy, cargo-deny
(yeni dış bağımlılık yok) yeşil. macOS Swift paketi **195/195**, `.app`
build'i ve strict codesign yeşil. Değişiklik yalnız Rust çekirdeği ve tek
satırlık bir Swift çağrı yeri güncellemesi; FFI yüzeyi `Vec` dönecek şekilde
genişledi.

**Yol üstünde bulunan, kapsam dışına ayrılan kusur:** `check-docs.sh` adım
9'un STATUS güncellik denetimi CI'ın shallow clone'unda (`fetch-depth: 1`)
her done task'ı HEAD'in tarihiyle okuyor — yalnız doküman içeren, yeni bir
günde atılan bir commit (`1ad194d`, ADR-0041 kabulü) bunu ilk kez ortaya
çıkardı. `NEN-076` olarak dosyalandı.

**`NEN-057` kapandı — `Film.tr.srt` gibi bir dosya adı artık dilini
söylüyor.** Bugüne kadar kullanıcı altyazısının dili yalnız içerikten tespit
ediliyordu (`resolve_language` her zaman `metadata: None` ile çağrılıyordu),
oysa imza metadata'yı zaten bekliyordu (ADR-0029 Karar 5). Yeni
`nen_subtitle::language::from_file_name` dosya adının son alt-uzantısını okur
ve `LanguageTag::parse`'a verir; ikinci bir ISO tablosu açılmadı, ADR-0032'nin
kanonikleştirmesi (`Film.eng.srt` → `en`) ve ADR-0030'un primary-subtag
gruplaması (`Film.pt-BR.srt` → grup `pt`, region korunur) bedava geldi.

**İki kullanıcı kararı alındı.** Birincisi: küçük, belgeli bir işaretçi kümesi
(`sdh`, `cc`, `forced`) dil sayılmıyor — `LanguageTag::parse` bunları da
geçerli iki/üç harfli kod olarak kabul ederdi ve `Film.sdh.srt` sahte bir
`SDH` menü grubu açardı; bu bir regresyon olurdu, çünkü o dosyanın dili bugün
içerikten doğru bulunuyor. Sondaki işaretçi atlanıp bir önceki alt-uzantıya
bakılıyor (`Film.en.sdh.srt` → `en`). İkincisi: `hi` işaretçi kümesine
**girmedi** — gerçek bir ISO 639-1 kodu (Hintçe) ve iki harfli bir kodu
tümüyle erişilemez kılmak tutarsız olurdu.

**Aday, önünde gerçek bir segment olmadan kabul edilmiyor.** `tr.srt` ve
`.tr.srt` ipucu üretmiyor — orada "dil" adın kendisi, bir başlığın niteleyicisi
değil. Bu üç mekanizma (metadata besleme, işaretçi atlama, gerçek-önek
koşulu) ayrı ayrı geri alınıp yalnız kendi testinin kırmızıya döndüğü
ölçüldü: sırasıyla 1/25, 2/11, 1/11.

**Bir kapsam sınırı ölçüldü ve ayrı task'a dosyalandı.** Sidecar taraması
yalnız medyanın tam basename'ini arıyor (`Film.mkv` → `Film.srt`), yani
`Film.tr.srt` bugün sidecar olarak hiç bulunmuyor — bu task'ın etkisi yalnız
`⇧⌘O` ile elle seçilen dosyalar. Genişletmek dizin listelemesi gerektirir ve
kendi kararını istiyor (`NEN-075`).

Rust workspace **586 → 594** (bu task ile +8; `568` STATUS'ta `NEN-042`
girişinin tarihsel sayısıydı, temiz `HEAD` zaten 586 veriyordu — `NEN-033`'ün
OpenSubtitles testleri aradaki fark). fmt, clippy, cargo-deny (yeni dış
bağımlılık yok), shell ve doküman kapıları yeşil. Değişiklik yalnız Rust
çekirdeğinde; FFI ve macOS kabuğu dokunulmadı. Kanıt:
`tasks/done/NEN-057-*.md`.

**`NEN-042` kapandı — boş durum artık tek "son açılan" satırı yerine 5
kayıtlık, sıralı bir liste gösteriyor.** `RecentMediaStore` yeniden yazıldı:
`RecentMediaEntry` (yalnız `id` + `displayName`, yol taşımıyor — ADR-0031
Karar 2), `UserDefaultsRecentMediaStore` dedup'lı ve kapasiteye kırpılan bir
liste, uzak (http/https) URL'leri sessizce reddediyor. `NEN-050`'de bilinçli
olarak ertelenen kusur bu geçişte kapandı: bayat bookmark artık security
scope **açıkken** tazeleniyor, tazeleme başarısız olsa bile zaten çözülmüş
URL kayıptan düşmüyor. `PlayerModel.openRecentMedia(_:)` çözülemeyen kaydı
artık yalnız **kendisi** olarak düşürüyor — önceki tek-slot tasarımın
`clear()`'ı tüm depoyu siliyordu. `PlayerCommands`'e `Son Açılanları Temizle`
maddesi eklendi (liste boşken devre dışı). Kullanıcı kararları: N=5, temizleme
komutu Dosya menüsünde, bugünkü tek kayıt bir kez göç ediyor.

Beş ayrı negatif kontrol (tazeleme-hatası yutma, dedup, kapasite kırpması,
`isFileURL` kapısı, `PlayerModel`'de per-entry `remove`) her biri kendi
başına geri alınıp yalnız kendi testini kırmızıya çevirdi. Swift paketi
paralel ve seri **195/195** (önceki 184), Rust workspace **568 passed / 1
ignored** (bu task Rust'a dokunmadı), fmt/clippy, `.app` build, strict
codesign, shell ve doküman kapıları yeşil. Gerçek `.app`te ad-hoc imzalı
build, yalnız `fixtures/media/*.mkv` ile: altı fixture açılıp tam kapatılıp
yeniden açıldığında 5 satır doğru sırada; gerçekten silinen bir dosyanın
satırı tek başına düştü, kalan dört satır ve uygulama sağlam kaldı; temizleme
komutu listeyi anında ve kalıcı olarak boşalttı. Yol üstünde bir OS gözlemi:
aynı birim içinde yeniden adlandırılan dosya security-scoped bookmark
tarafından hâlâ çözülüyor (Apple'ın kasıtlı dayanıklılığı) — "taşınan" DoD
maddesi bu yüzden dosyanın gerçekten kaldırılmasıyla sınandı. Kanıt:
`evidence/M3/NEN-042-checklist.md`.

**`NEN-050` kapandı — son açılan medya deposundan türeyen üç geçici bildirim
yolunun artık testi var.** `NEN-048` motor hatasından türeyen sınıfı
kapatmıştı ama aynı `presentTransient` yüzeyini kullanan, kaynağı
`recentStore` olan üç yol testsiz kalmıştı (`evidence/M3/NEN-048-checklist.md`
bunu kaydediyordu). `MemoryRecentStore` `FakeSession`'ın deseninde hata
enjeksiyonu ve `clearCount` kazandı — sayaç olmadan "depo temizlendi" ile
"zaten boştu" ayrışmazdı. Üç metin `PlayerModel`'den `PlaybackPresentation`'a
taşındı (metin değişmedi) ve kapalı küme testi bu ikisini de artık kapsıyor.

Negatif kontrol dört yönde ve ayrık: bildirim çağrısı kaldırılınca yalnız o
yolun testi kırmızı, `clear()` kaldırılınca yalnız throw yolunun testi
kırmızı (sayaç gerçekten ayırt edici), save hatası yükü de bloklayınca yalnız
"medya yine de yükleniyor" iddiası kırmızı, ve metne rakam eklenince yalnız
kapalı küme testi kırmızı (kontrol sağır değil). macOS Swift paketi
**182 → 186**, Rust workspace **568 passed / 1 ignored** (bu task Rust'a
dokunmadı), fmt/clippy, `scripts/test.sh` ve doküman kapıları yeşil.
`UserDefaultsRecentMediaStore.resolve()`'daki bilinen store kusuruna
dokunulmadı — DoD'u `NEN-042`'de. Kanıt: `tasks/done/NEN-050-*.md`.

**`NEN-041` kapandı — doctor PATH'te bulunan ama çalıştırılamayan aracı artık
hazır saymıyor.** `swift --version` sıfırdan farklı döndüğünde veya sürüm satırı
okunamadığında M1/M3 kapıları kapanıyor; aynı boru hattı kusuru cargo, rustc,
cargo-deny, JDK, Gradle, Xcode ve libmpv'nin pkg-config kolunda da giderildi.
Shadow-PATH paketi **49 doğrulama**, tüm shell paketi **2/2** yeşil. Gerçek
makinede M3 kapısı Swift 6.3.3 ve Xcode 26.6 ile çıkış 0 verdi.

**`NEN-074` kapandı — canlı resize artık hedef kare zamanını ana thread'de
beklemiyor.** `MPVVideoView`, normal çizimde libmpv'nin A/V zamanlamasını açık
tutuyor; yalnız `inLiveResize` boyunca
`MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME=0` geçiyor. Aynı fixture ve altı eş
sürüklemede önceki en kötü **175,899 ms** ana-thread çağrısı, düzeltmeden
sonra en kötü **17,991 ms** oldu; sabit performans eşiği konmadı. Gerçek
`.app` kabulü aspect lock, tam ekran dönüşü ve videosuz yüzeyi de korudu.
macOS paketi **181/181**, app build, strict codesign, shell ve doküman kapıları
yeşil. Kanıt: `evidence/M3/NEN-074-measurement.md`.

**`NEN-073` düzeltici kapanışla tamamlandı — önceki minimum sınırı kanıtı
geçersizdi.** `WindowGeometryWriter` AppKit `contentMinSize` değerini yazarken
`PlayerRootView` SwiftUI'a `0×0` minimum bildiriyor, normal `Window` sahnesi de
bu içerik minimumunu uyguluyordu. Minimum artık aspect-correct değerle root
view'dan yayımlanıyor, sahne açıkça `.windowResizability(.contentMinSize)`
kullanıyor ve AppKit writer yalnız aspect lock/açılış boyutunu yönetiyor.
Negatif kontrolde eski `0×0` davranışı yeni testte **10 expectation failure**;
düzeltmeyle **1/1 yeşil**. Gerçek `.app` köşe sürüklemesinde 16:9 pencere
**693×390**, 4:3 pencere **693×520** frame'de durdu. Tam macOS paketi
**179/179**, app build, strict codesign, shell ve doküman kapıları yeşil.
Kanıt: `evidence/M3/NEN-073-checklist.md`.

**`NEN-037` kapandı — Settings'in tek sahnesi artık birinci ve ikinci tercih
edilen altyazı dilini taşıyor, ADR-0010 Karar 4/10'un menü sırası ilk kez
üründe kanıtlandı.** Çekirdek iki tercihi baştan beri destekliyordu
(`SubtitlePreferences::new`, FFI `menu`/`auto_selection`); eksik olan macOS
kabuğuydu — `PlayerModel` birinciyi sistem dilinden sessizce okuyup ikinciyi
hep `nil` geçiyordu, Settings sahnesi de "Henüz ayarlanabilir bir seçenek
yok." diyordu. Yeni `SubtitlePreferenceStore` (`RecentMediaStore`'un
deseninde) ve `LanguageCatalog` (var olan `endonym(for:)`'u tekrar kullanan,
yeni bir dil tablosu açmayan liste) bu boşluğu kapattı.

**İki kullanıcı kararı alındı.** Birincisi: uygulama ilk açıldığında sistem
dili birinci tercih olarak **görünür ve değiştirilebilir** şekilde depoya
yazılır (`subtitlePreferencesSeeded` bayrağıyla tam bir kez) — bugünkü
"Türkçe makinede Türkçe track otomatik açılır" davranışı böylece korunuyor,
ama artık kullanıcının bildiği ve boşaltabildiği bir tercih olarak. İkincisi:
dil listesi ikinci bir tabloyla curate edilmedi, Foundation'ın adlandırabildiği
tüm diller listelendi.

**Tercih değişikliği ekrandaki altyazıyı değiştirmiyor.** Otomatik seçim
medya başına bir kez çalışıyor (ADR-0031 Karar 4.3); `updateSubtitlePreferences`
menüyü yeniden sıralıyor ama `hasAutoSelected`'a dokunmuyor — otomatik test
(`changingPreferenceReordersWithoutMovingSelection`) ve gerçek `.app`'te bunu
ayrı ayrı kanıtladı.

**Dört DoD maddesi de gerçek `.app` üzerinde tek tek koşuldu**, sentetik
`menu-clip.mkv` fixture'ıyla (İngilizce/Fransızca/Türkçe gömülü track +
dilsiz bir track — §8'in kanonik kümesi): iki tercih ayarlanıp `⌘Q` ile tam
kapatılıp yeniden açıldığında `Français`/`English` aynen geri geldi (diskteki
`.plist` doğrudan da okunup doğrulandı); birinci tercih değişince CC menüsü
`Kapalı, Türkçe, English, Français, Dil Belirsiz` → `Kapalı, Français,
English, Türkçe, Dil Belirsiz` sırasına döndü; ikinci tercih birinciyle aynı
seçilince seçici anında `Yok`'a döndü; iki tercih de boşken menü sırası
ADR-0010 Karar 10'un "tercih ayarlanmamış" örneğiyle (`en < fr < tr`) birebir
aynı çıktı. Swift paketi **177/177** (7 yeni + 4 yeni + 3 eklenen test),
negatif kontrol iki yönde ve ayrık. Rust workspace **568 passed / 1 ignored**
(bu task Rust'a dokunmadı), fmt, clippy, cargo-deny, shell ve doküman
kapıları yeşil. Kanıt: `evidence/M3/NEN-037-checklist.md`.

**`NEN-047` kapandı — yedi kısayol artık yalnız oynatma penceresine ait,
klavye eylemi ekranda görünür oluyor.** `NEN-024` incelemesinde ölçülen iki
kusur giderildi: yedi kısayol `PlayerCommands`'te menü key equivalent'ı
olmaktan çıkıp `@FocusedValue` ile oynatma sahnesine bağlandı — Ayarlar
penceresi öndeyken artık ne oynatmayı ne sesi etkiliyor; `seekRelative` ve
`adjustVolume` artık `pointerMoved()` çağırıp gizli kontrolleri geri
getiriyor. `NEN-037`'yi bekleten bağımlılık kalktı, artık READY.

**Yol boyunca bir AppKit sınırı ölçüldü:** `Esc`'in tam ekrandan çıkışı
`NSMenu`'nun düz (modifikatörsüz) key equivalent'ı üzerinden hiç
tetiklenmiyor — task'tan önce de var olan, bağımsız bir davranış. Gerçek
çıkış artık `PlayerRootView`'daki pencereye ve `isFullScreen`'e taranmış bir
`NSEvent` local monitor'e taşındı; menü maddesi keşif ve fare tıklaması için
kaldı. Canlı `Esc` tuşu bu oturumun uzak masaüstü ortamında hiçbir uygulama
için (bağımsız bir `NSOpenPanel`'in kendi Vazgeç-on-Esc'i dahil) teslim
edilemediğinden gerçek tam-ekrandan-çıkış klavye ile doğrulanamadı; kod
doğru AppKit birincili ile yazıldı, `isFullScreen` durumu model testiyle
kanıtlı. Kanıt: `evidence/M3/NEN-047-checklist.md`.

**`NEN-049` kapandı — paralel macOS paketi artık deterministik ve kusur
gizlenmeden kaldırıldı.** Dört kapanışta gözlenen kırmızı
(`ContractTests.successiveMediaReportTheirOwnDisplaySize`) dokunulmamış `HEAD`
üzerinde yeniden üretildi — **2/5 yeşil** — ve ilk kez sebebi doğrudan okundu.
Test her koşuda `~0,2 s`'de düşüyordu, yani 5 s'lik timeout'a hiç ulaşmıyordu.
Geçici bir tanı satırı beklemenin **ne gördüğünü** yazdırdı: 4:3 klip
yüklenirken `160×90`, anamorphic klip yüklenirken `160×120` — ikisi de **giden
medyanın** boyutu. Bekleme aç kalmıyor, yanlış cevabı zamanında alıyordu:
`loadfile` sonrası `FILE_LOADED` geldiğinde `video-out-params` hâlâ giden
medyayı tarif edebiliyor ve `waitForGeometry` **ilk non-nil** değeri kabul
ediyordu.

**Ölçüm çareyi değiştirdi — izolasyon uygulanmadı.** Task "gerçek libmpv
suite'lerini birbirine karşı seri kılmak" diye açılmıştı; seri kılmak kırmızıyı
**gizlerdi**, çünkü kusur paralellikte değil testin kendi beklemesindeydi.
Bekleme artık **giden** medyanın boyutunu dışlıyor, **beklenen** boyutu değil —
beklenen boyutu vermek iddiayı kendi öncülüne sordurmak olurdu. Kapsamın üç
basamağından hiçbiri gerekmedi; paket paralel kalıyor ve ürün kaynak kodu
değişmedi (DoD #4): tek dosya `ContractTests.swift`.

**Oran bu kez ayırt edici çıktı** — `NEN-051`'de çıkmamıştı. Makine durumu
oturum içinde kaydığı için dönüşümlü ölçüldü: düzeltmesiz **2/5** ve **1/4**,
düzeltmeli **5/5** ve **4/4** — öncesi **3/9**, sonrası **9/9**. Seri mod
düzeltmeli ağaçta 2/2 (159/159). Negatif kontrol üç yönde: düzeltme geri
alınınca semptom oranla geri geliyor; beklenen boyut bilinçli yanlış yazılınca
test kırmızı ve gerçek değeri söylüyor (yani bekleme çağıranın sorusunu
cevaplamıyor); dokuz yeşil koşuda testin süresi **0,117–0,186 s**, yani
düzeltme "daha uzun bekleyerek" geçmiyor. Rust workspace **568 passed / 1
ignored**, fmt/clippy, shell ve doküman kapıları yeşil. Kanıt:
`evidence/M3/NEN-049-measurement.md`.

**Kalan sınır açıkça kaydedildi:** dışlama bir **değer** karşılaştırmasıdır.
Art arda gelen iki medya gerçekten aynı display boyutunu paylaşırsa bekleme
timeout'u harcayıp doğru cevabı en sonda verir. Bugünkü üç fixture'ın boyutları
farklı; böyle bir çift eklenirse ayırıcı değer değil, yüklemenin kendi
reconfiguration'ı olmalı.

**`NEN-036` kapandı — uzak medya evidence portu ve macOS URLSession adapter'ı
kanıtlandı.** Provider API'leri HTTPS + approved-host olarak kaldı; kullanıcı ve
handoff medya URL'leri keyfi http/https hostları için core policy üzerinden
doğrulanıyor. HEAD, final redirect basename'i, Content-Disposition, bounded
Range pencereleri ve OpenSubtitles hash'i deterministic fake ve internetsiz
URLProtocol testleriyle sınandı. Yeni container parser kapsamı `NEN-072`'ye
ayrıldı. Rust workspace, fmt/clippy/cargo-deny, macOS Swift paketi **159/159**,
`.app` build/codesign, shell ve doküman kapıları yeşil.

**`NEN-069` kapandı — videosuz medya açıldığında ekranda önceki videonun son
karesi kalmıyor.** Ölçüm task dosyasının yazdığı teşhisi **ikiye ayırdı ve
yarısını çürüttü:** ürünün kendi render yolu elle sürüldüğünde videosuz medyada
yakalanan kare 0/921600 aydınlık piksel veriyor — mpv çizimi zaten yapıyor.
Eksik olan **isteyen**di: update callback yalnız yeni kare üretildiğinde
tetikleniyor, videosuz medya hiç kare üretmiyor, dolayısıyla `draw(_:)` hiç
çağrılmıyor. Açılan her medya artık bir redraw alıyor.

Bunun bedeli test tasarımına indi: DoD'un önerdiği biçimiyle yazılan piksel
testi `renderFrame`'i doğrudan çağırdığı için `needsDisplay` yolunu atlar ve
**düzeltmeden önce de yeşil** olurdu — nitekim negatif kontrolde yeşil kaldı.
Kusuru ölçen test bu yüzden tetikleyiciye bakıyor. `videoGeometry == nil`
guard'ı da elendi: motor `ready` anında cevap veriyor ama kabuğun kopyası o an
resimli medyada da `nil`, yani guard hiçbir şeyi sabitlemezdi.

**Yan bulgu: temizlik verildiği framebuffer'a inmiyordu.** `renderFrame`'in
context'siz dalı `fbo` argümanını kullanmıyor, o an bağlı olana `glClear`
ediyordu. Ekran yolunda ikisi de 0 olduğu için üründe yanlış bir şey olmuyordu;
offscreen yakalamada hiç doğru olmuyor ve shutdown testi bu yüzden 834538
pikselle kırmızı geldi. Kapanış yolunun geri kalanı için kod gerekmedi —
`shutdown` zaten context'i bırakıyor ve kabuğun `stop()` çağırdığı bir yol yok.

Negatif kontrol iki yönde ve ayrık: her düzeltme yalnız kendi testini kırıyor
(1 + 1 kırmızı). Swift paketi **seri 157/157** (2/2 koşu), Rust workspace
**560 passed**, fmt/clippy, `scripts/test.sh`, `.app` build'i ve strict
codesign yeşil.

**Karşılanmayan tek DoD maddesi açıkça kaydedildi:** `scripts/test-macos.sh`'in
varsayılan **paralel** modu yeşil değil. Kırmızı olan test her koşuda aynı
(`ContractTests.successiveMediaReportTheirOwnDisplaySize`) ve bu task'ın koduna
erişmiyor. Katkı dönüşümlü ölçüldü — aynı ağaçta suite dosyası sırayla var ve
yok edilerek dörder koşu: **1/4 ve 1/4 yeşil**. Yani paralel paket bu makinede
bu task olmadan da aynı oranda kırmızı; mekanizma `waitForGeometry`'nin ilk
non-nil değeri kabul edip giden medyanın boyutunu okuması. Gözlem `NEN-049`'a
**dördüncü** kez ve ilk kez oran ölçümüyle yazıldı — o task'ın DoD #3'ünün
bugüne kadar üretilemeyen kaydı budur. Yol üstünde iki ölçüm bu task'ın kendi
kapsamını değiştirdi: `RunLoop.run(until:)` ile bekleyen bir libmpv suite'i
`PlayerModelTests`'in üç testini beş saniyelik deadline'larını kaçıracak kadar
aç bırakıyor (`await Task.sleep`'e çevrildi), ve gerçek engine sayısı doğrudan
etkili (iki test tek engine'e birleştirildi). Kanıt:
`evidence/M3/NEN-069-measurement.md`.

**`NEN-028` kapandı — M3'ün beş çıkış kriteri üründe kanıtlandı.** Kabul
senaryosu `docs/milestones/M3-macos-slice.md`'ye yazıldı ve gerçek `.app`te
baştan sona koşuldu: medya `Aç`'tan 0,7 s sonra oynuyor ve menü taramayı
beklemiyor; oynatma sürerken yüklenen bozuk `.srt` oynatmayı kesmiyor, satırı
menüde `biçim hatalı` diye işaretleniyor; menüde `Kapalı` her zaman var,
gruplar tekrarsız ve dilsiz gömülü track `Dil Belirsiz` içinde; geçerli bir
SRT'ye işaret eden **kısayol sidecar sessizce reddediliyor**; seçim anında ve
her seek sonrası doğru replik anında ekranda, cue'suz anda ekran boş. Rust
workspace **560 passed / 1 ignored** (72 hedef), macOS paketi **154/154**,
`scripts/test.sh`, `.app` build'i ve strict codesign yeşil. Kanıt:
`evidence/M3/NEN-028-checklist.md`, sekiz kare ve yerel ekran kaydı.

**Koşu bir kusur buldu ve düzeltildi:** `PlayerModel.loadSubtitleFile(at:)`
menüyü yenilemediği için `⇧⌘O` ile yüklenen altyazı dosyası menüye hiç
girmiyordu — sidecar yolu taramanın yenilemesi sayesinde çalıştığından kusur
bugüne kadar görünmemişti. Üç test önce kırmızı görüldü, düzeltme tek satır.

**M3, `NEN-028` ile biçimsel olarak kapanmadı.** Çıkış kriterleri işaretlendi
ama retro ve roadmap'in milestone durumu bilinçli olarak `NEN-028` kapsamı
dışında bırakıldı (kullanıcı kararı, 2026-09-05). O bekleyen adım 2026-09-07'de
atıldı — bkz. yukarıdaki M3 kapanış kaydı.

Koşudan çıkan `NEN-071` işi kapandı: `⇧⌘O` panelinin kısayolu kendisi çözmesi,
davranış kararı ve ürün yüzeyinde testi. `NEN-049`'a üçüncü gözlem: paralel
tam paket bu kez **gerçek libmpv** testinde bir kez kırmızı verdi
(`successiveMediaReportTheirOwnDisplaySize`); aynı ağaçta test tek başına 3/3,
paralel paket 1/1, seri paket 2/2 yeşil.

**`NEN-027` kapandı — seçilen altyazı ekranda ve seek sonrası doğru replik
anında görünüyor.** Kod 2026-08-29'da bitmişti; karşılanamayan tek DoD maddesi
"seçim anında altyazı görünüyor" idi ve o kusur bu task'ın enjeksiyon yolunda
değil, cam transportun altyazı bandını örtmesindeydi — `NEN-066` ADR-0037'nin
güvenli alanıyla kapattı. Görsel kabul 2026-09-05'te gerçek `.app`te koşuldu:
gömülü track'in cue'suz anında (00:05) ekran boşken kullanıcı dosyası seçildi
ve replik **oynatma gerekmeden** çizildi; +5 sn ile 00:10'da 2. replik anında
geldi; 00:15'te ekran boşaldı; iki kaynağın da cue'su olan 00:11'de gömülü
track'e geçildiğinde **tek** replik kaldı; `Kapalı` aynı anda ekranı boşalttı.
Rust workspace **72 hedef / 560 passed**, macOS paketi temiz koşuda
**151/151**, `.app` build'i, strict codesign, shell ve doküman kapıları yeşil.
Kanıt: `evidence/M3/NEN-027-checklist.md` ve dört kare.

Yol üstünde bir gözlem kaydedildi ve kapsama alınmadı: ilk tam Swift koşusu
`ContractTests` içinde bir kez kırmızı verdi, temiz ikinci koşu yeşil. Aynı
kararsızlık `NEN-070` koşusunda da görülmüştü; `NEN-049` tam olarak bu gözlemi
bekleyen açık task.

**`NEN-070` kapandı — ekran genişliğine açılan videoda play/pause ve tam ekran
düğmeleri artık kırpılmıyor.** Seek slider'ın yüksek layout priority'si 1470 pt
pencerede satırı iki kenardan taşırıyordu. Önce play `−6…28`, tam ekran
`1470…1500` ölçüldü; düzeltmeden sonra sırasıyla `22…56` ve `1418…1448`, seek
880 pt. 693/1470 pt, kısa/95 dakika/bir saat sonrası ve geniş altyazı etiketi
matrisi gerçek SwiftUI frame testiyle korunuyor. Tam macOS paketi **151/151**,
uygulama build'i, strict codesign, shell ve doküman kapıları yeşil. Kanıt:
`evidence/M3/NEN-070-checklist.md`.

**`NEN-068` kapandı — pencere medyanın gerçek display boyutunda açılıyor ve
canlı resize boyunca oranı koruyor.**
ADR-0038'in taşıdığı tek sayı porttan, FFI'den ve kabuktan geçip
`WindowGeometry`'ye ulaşıyor: pencere medyanın oranına kilitleniyor, minimum
boyut krom tabanından (600×390 pt) o orana göre türetiliyor ve video yokken
kilit kurulmuyor. Ölçüm `evidence/M3/NEN-068-measurement.md`'de: libmpv
headless de `MPV_EVENT_VIDEO_RECONFIG` üretiyor ve anamorphic klipte
720×576 yerine **1024×576** veriyor.

Elle kabul üç kusuru kapattı: video yüzeyi safe area içinde kalıp siyah çerçeve
çiziyordu; sıfır oran ataması macOS 27'de native tam ekran çıkışını yarıda
bırakıyordu; yükleme sırasında eski VO boyutu yeni medyanın ilk pencere
boyutlandırmasını tüketebiliyordu. Gerçek uygulamada 16:9, 4:3, 2.39:1,
anamorphic ve audio-only koşuları; tam ekran `F`/düğme/`Esc`; play/pause, seek
ve canlı resize ölçüldü. Swift paketi **150/150**, Rust workspace **560/560**
(1 ignored benchmark); fmt, clippy, cargo-deny, shell testleri, `.app` build'i,
strict codesign ve doküman kapıları yeşil. Tam kayıt:
`tasks/done/NEN-068-*.md` · `evidence/M3/NEN-068-checklist.md` ·
`evidence/M3/NEN-068-fullscreen.md`.

**`NEN-067` kapandı — oynatıcı kromu tek satırlık, 57 pt'lik, pencerenin alt
ve yan kenarlarına sıfır boşlukla oturan cam bir transport.** Tek sırada
oynat/duraklat, ±5 sn, geçen süre, esnek seek, kalan/toplam, ses, CC, hız ve
tam ekran var; birbirini dışlayan altyazı ve hız panelleri barın üstüne
bitişik açılıyor ve panel açıkken krom pinleniyor. ADR-0037'nin güvenli alanı
artık barın gerçek SwiftUI yüksekliğinden ölçülüyor. Paralel Swift paketi
**107/107**, uygulama build'i, strict codesign ve doküman kapıları yeşil;
manuel kabul `evidence/M3/NEN-067-checklist.md`'de.

Bu kapanış `NEN-068`'i READY yaptı — medya boyutunda açılan, orana kilitli
pencere; mimari kararı ADR-0038 ile alınmış durumda.

**`NEN-065` kapandı — kontrol gizleme testi artık makine yüküne bakmıyor.**
Kusur davranışta değil ölçümdeydi: model gizleme zamanlayıcısını gerçek saatte
kuruyor, test 2 ms'lik gecikmeye karşı gerçek saatte 10 ms bekliyordu ve paralel
koşumdaki gerçek libmpv testleri o 5×'lik payı yiyordu. Testin iki iddiası
olumsuz olduğu için — "pin'liyken gizlenmiyor", "duraklıyken gizlenmiyor" —
uykuyla zaten kanıtlanamıyordu; ikisi `hideControlsNow()` doğrudan çağrılarak
yapısal hale getirildi, kalan olumlu iddia koşula beklemeye çevrildi. Ürün
kodu değişmedi. Paralel paket 10 ardışık koşumda yeşil, biri altı meşgul-döngü
süreci altında; `NEN-067`'nin engeli kalktı.

**`NEN-066` kapandı — motorun çizdiği altyazı gerçek video yüzeyinde ve
transport katmanının üstünde görünür.** A/B ölçümü render bileştirme
hipotezlerini eledi: libmpv gömülü ve kullanıcı track'lerini oynarken ve
duraklatılmışken çiziyordu; 134 pt'lik cam transport bu piksel bandını
örtüyordu. ADR-0037 ile kabuk görünür kromun gerçek alt inset oranını playback
session'a taşıyor, adapter bunu `sub-pos`'a mapliyor ve krom gizlenince
sıfırlıyor.

Gerçek libmpv offscreen piksel testi dahil Swift paketi **102/102**, Rust
workspace ve renderer kontratları yeşil. Ad-hoc imzalı `.app`te gömülü/kullanıcı
altyazısı, oynatma/duraklatma, görünür/gizli krom, `F`/`Esc` tam ekran ve
`Kapalı` koşuları geçti; iki ekran kanıtı kaydedildi. `NEN-060` aynı kök nedene
katlanıp canceled oldu, `NEN-027` artık READY. Tam kayıt:
`tasks/done/NEN-066-*.md` · `evidence/M3/NEN-066-checklist.md`.

**`NEN-062` kapandı — altyazı menüsü artık popover değil, transportun sağ
kenarına hizalı pencere içi cam panel.** Panel ve bar aynı `GlassSurface`'i
kullanıyor; aralarında 12 pt var, panel 440×260 pt ve kolonlar toplam genişliği
büyütmeyen 190/250 pt sınırında. Grup vurgusu bakılan kolonu, 7 pt aktif nokta
gerçek kaynağı göstermeye devam ediyor; NEN-026'nın sıra, endonim, dedup,
kusurlu satır ve boş durum semantiği değişmedi.

Panel açıkken `setControlsPinned(_:)` otomatik gizlemeyi askıya alıyor; gerçek
`.app`te 3,2 sn sonra panel, transport ve chrome görünür kaldı. CC, video alanı
ve transport tıklamaları paneli kapatıyor, transport eylemi kaybolmuyor; medya,
fatal, pasifleşme ve shutdown sınırları stale panel/pin bırakmıyor. Aynı
basename taşıyan farklı dosyalar monoton `mediaPresentationRevision` ile
ayrılıyor. Parlak fixture ve kusurlu sidecar kanıtları kaydedildi; geçici
sidecar kaldırıldı. Swift paketi **91/91** yeşil, `.app` build ve strict
codesign doğrulaması geçti. Tam kayıt:
`tasks/done/NEN-062-*.md` · `evidence/M3/NEN-062-checklist.md`.

**`NEN-061` kapandı — oynatıcı yüzeyi artık videoya kadar uzanan macOS
kromu ve tek cam transport katmanı.** Medya adı, trafik lambaları ve bar aynı
0,24 sn ease-out görünürlük döngüsünde; düğmeler gizlenince hit-test'ten de
çıkıyor, tam ekranda sistem yönetimine bırakılıyor. HUD materyali üstündeki
özel seek/volume kontrolleri pointer sürükleme ve tıklamayı, klavye odağını ve
VoiceOver adjustable eylemlerini koruyor. Süre sesin yanında ve tıklanabilir;
CC etiketi `Kapalı` veya seçili kaynağın görünen adını kullanıyor. Parlak/koyu
sentetik fixture, otomatik gizleme, duraklatma ve NEN-046 yaşam döngüsü gerçek
`.app` üzerinde geçti; Swift paketi **87/87** yeşil.

**Teknik video kalitesi rozeti iptal edildi.** `NEN-063` uygulanmadan
`canceled` oldu; ADR-0036 `rejected`. Playback portu, FFI ve libmpv yüzeyine
çözünürlük, codec veya SDR/HDR metadata hattı eklenmedi. M3 boyunca üst
şerit mevcut basename'i göstermeye devam eder. Resmi API'deki kesin hash
eşleşmesine dayanan zengin medya kimliği ayrı `NEN-064` task'ıyla M6'ya
ertelendi; bu task `NEN-033` bitmeden READY değildir.

**`NEN-026` kapandı — altyazı menüsü ekranda ve §8'in görünen metni ilk kez
var.** Menü çekirdeğin projeksiyonunu çiziyor: gruplama, sıra, dedup ve "boş
grup görünmez" kuralı `nen-catalog`'da kaldı ve NEN-019'un dört golden'ı onu
tutuyor; Swift yalnız **metni** üretiyor — endonim başlıklar (`English` ·
`Français` · `Türkçe`) ve Türkçe chrome (`Kapalı` · `Kullanıcı Altyazıları` ·
`Dil Belirsiz`). ADR-0010 Karar 7'nin "dilin adı kendi dilindedir" kuralı
Foundation'ın kendi tablosundan geliyor, ikinci bir kopya shipping edilmedi.

**Yüzey Claude Design mockup'ından alındı, semantiği §8'de bırakıldı.** Sabit
yükseklikli iki kolonlu panel: kolon 1 projeksiyonun bölümleri, kolon 2 seçili
bölümün girdileri. Yükseklik sabit çünkü kaynak geldikçe büyüyen bir panel
işaretçinin altındaki satırı kaydırırdı (ADR-0031 Karar 4.2). Mockup'ın üçüncü
kolonu — `AI İLE ÇEVİR`, gecikme, manuel ve otomatik senkronizasyon — M5 · M7 ·
M8 olduğu için **yazılmadı**; grid ileride açılacak biçimde kuruldu.

**`Kapalı` bir eylem, gezinme hedefi değil.** Kullanıcı kararı: tıklanınca
altyazı kapanıyor **ve** kolon 2'yi devralıp `Altyazılar kapalı.` diyor. Kural
tek: kolon 1'deki vurgu her zaman kolon 2'nin gösterdiğidir. Bunun karşılığı
`Bu dil için altyazı yok.` satırının hiç yazılmaması — kolon 1 yalnız dolu
grupları listelediği için o durum üretilemez (ADR-0035 Karar 3'ün mantığı).

**Token kimlik değil, ve sebebi ölçülmüş.** `SubtitleSourceId`'nin anahtarı bir
yol digest'i (K23 #8) ve Swift'in üretilmiş struct'ı her alanını
`String(reflecting:)` ile basar — NEN-023'te sınırı çizilen şey. Menü satırı
bunun yerine ömür boyu sabit, opak bir sayaç taşıyor: basacak bir şeyi yok ve
liste büyürken değişmiyor.

**Elle koşu üç kusur buldu, üçü de düzeltildi.** (1) Otomatik seçim Türkçe'yi
açtığında menü `Kapalı`'ya bakıyor kalıyordu ve kolon 2 **ekranda altyazı
varken** `Altyazılar kapalı.` yazıyordu; seçim artık bakılan grubu da taşıyor.
(2) Panel video üzerinde okunmuyordu — popover'ın kendi materyali parlak bir
karede yetmiyordu; opak zemin kondu. (3) Sebep etiketi çift kararma yüzünden
(satır %40 opaklık × ikincil renk) panelin en soluk yazısıydı — oysa satırın
var olma sebebi tam olarak o bilgi.

**Negatif kontrol on yönde, ve kontrollerin kendisi iki kez düzeltildi.**
`Kapalı` testi ilk turda **0 kırmızı** verdi: test, `Kapalı`'ya basmadan önce
zaten `Kapalı`'ya bakıyordu. Geç-sidecar testinin sidecar'ı İngilizce metindi
ve Türkçe tercihle hiç eşleşmiyordu — yani testi geçiren şey kuralın kendisi
değil, eşleşmenin yokluğuydu. Üçüncüsü kaydedildi: **tek-atış guard'ını tek
başına kaldırmak hiçbir testi kırmıyor**, çünkü özelliği bugün çağrı yerinin
kendisi tutuyor; guard ancak ikinci bir çağrı yeri eklendiğinde taşıyıcı
oluyor, ve o senaryoda 2 test kırmızıya dönüyor.

**Tam ekran gözlemi `NEN-066` içinde kapandı; `NEN-060` canceled.** Seçilen
track'in görünmemesi tam ekrana özgü değildi: aynı transport örtüşmesi pencere
modunda da ölçüldü. ADR-0037 güvenli alanı ve gerçek `.app` koşusu hem pencere
hem tam ekranı kapsıyor.

**Yol üstünde bir test iskeleti kusuru bulundu.** `TempFixture`'ın dizin adı
yalnız etiket + pid'di; swift-testing paralel koştuğu için aynı etiketi kullanan
iki test aynı dizini paylaşıyor ve `init`'teki `removeItem` kardeşinin
fixture'larını yarı yolda siliyordu. Sayaç eklendi. Ayrıca `PlayerModel` tercihi
`Locale.preferredLanguages`'tan statik okuyordu, yani otomatik seçim testleri
makinenin sistem diline bağlıydı; enjekte edildi.

Rust **491 → 513**, Swift **62 → 86**. Yeni dış bağımlılık yok. Tam kanıt:
`tasks/done/NEN-026-*.md` · `evidence/M3/NEN-026-checklist.md`.

**`NEN-056` kapandı ve `ADR-0035` accepted oldu — menünün sebep etiketi kümesi
artık üretilebilen durumlarla birebir.** ADR-0031 Karar 5 iki madde taşıyordu ve
ikisi aynı anda doğru olamazdı: birincisi kataloğa giren bozuk kaynağın
taşıyacağı etiketleri `okunamadı` · `biçim hatalı` · **`çok büyük`** diye
sayıyor, ikincisi boyut sınırını güvenlik kapısına koyup kapıdan dönen dosyayı
kataloğa **hiç** sokmuyordu. Boyut kapıda eleniyorsa katalogdaki hiçbir kaynak
`çok büyük` taşıyamaz — yani `NEN-026` **üretilemeyecek bir durum için UI
yazmak** üzereydi.

**Tutarsız olan kod değil, dokümandı.** `admit()` sınırı, dosya açılmadan önce
elde olan `symlink_metadata`'ya soruyor; `MAX_SUBTITLE_BYTES` ayrıca
`encoding::MAX_INPUT_BYTES` ile aynı sayı (NEN-015) ve ayrışmalarını bir test
engelliyor. Kullanıcı kararıyla bu yarı korundu: **boyut bir güvenlik
kapısıdır**, `çok büyük` kümeden düştü. Diğer yarı ölçülen bir maliyet yüzünden
elendi — kataloğa sokmak `NEN-025`'in kapılarını değiştirir ve kullanıcının
kendi seçtiği büyük dosyanın geçici bildirimini götürürdü (Karar 1'e göre
kaynak-düzeyi hata yalnız menüde görünür), yani `.app` üzerinde kanıtlanmış bir
davranış geri alınırdı. Kabul edilen maliyet açık: 10 MiB üstü bir **sidecar**
sessizce görünmez kalıyor — ölçülen en büyük gerçek altyazı 3.1 MiB, sınır onun
3 katından fazlası.

**ADR-0031 düzenlenmedi, tümüyle de supersede edilmedi.** Gövde olduğu gibi
duruyor, Notlar'a ADR-0035'e işaret eden bir madde eklendi (ADR-0001'in açık
istisnası). Tümüyle supersede etmek `adr:` alanıyla ADR-0031'e referans veren
**7 done task**'ı `check-docs.sh` adım 6'da kırmızıya döndürürdü.

**Asıl bulgu kümenin diğer yarısındaydı: `okunamadı`'yı üreten hiçbir test
yoktu.** `biçim hatalı`'nın üreticisi vardı, ötekinin yoktu — menünün çizeceği
etiket, hiçbir testin yürümediği bir kod yoluna dayanıyordu. Yeni test gerçek
bir dosyaya UTF-16LE BOM + tek başına yüksek surrogate yazıyor: §4'ün kapılarını
geçiyor, sonra ADR-0008 gereği mojibake yerine reddediliyor, yani sonuç
`Rejected` değil `Defective(Unreadable)` — kaynak katalogda kalıyor.
İkinci test kümeyi kapatıyor.

**Negatif kontrol iki yönde.** Beklenti `Malformed`'a çevrilince **1** kırmızı
(`left: unreadable / right: malformed` — test tam olarak bu kusuru ölçüyor);
`SourceDefect`'e üçüncü varyant eklenince guard **derlenmiyor**
(`E0004: non-exhaustive patterns`), yani ADR-0035 Karar 3 mekanik olarak
zorlanıyor. İkisi de geri alındı.

Rust testleri **489 → 491**, Swift değişmedi (bu iş Swift'e dokunmadı). Ürün
kodu değişmedi. Tam kanıt: `tasks/done/NEN-056-*.md`.

**`NEN-059` kapandı — push'ta görülen CI kırmızısı yerelde bütünüyle
giderildi.** Rust 1.98.0 `rustfmt`, NEN-025/NEN-051'den kalan beş dosyada satır
düzeni farkı buldu; `cargo fmt` yalnız bu beş dosyayı değiştirdi. Whitespace
dışı içerik dört dosyada birebir aynı, beşincideki tek token farkı Rust
1.98'in `clippy::assertions_on_constants` lint'inin istediği
`const { assert!(...) }` wrapper'ı. Tam yerel workflow — fmt, clippy, workspace
testleri, cargo-deny, shell testleri, task-index ve docs kapıları — çıkış 0.
Task commit'i `0e434d6` için GitHub Actions `CI` run **33042520202**, tüm
adımları **1 dk 57 sn** içinde geçerek `success` tamamlandı.

**`NEN-058` kapandı — bildirilen hipotez ölçümle ikiye ayrıldı.** Task
"yanında symlink `.srt` olan medya açılmıyor" diye açılmıştı ve tek hipotezi
mpv'nin sidecar'ı kendi yükleyip symlink'te takılmasıydı. Ölçüm hipotezin
**yarısını çürüttü, yarısını doğruladı** — ve iki ayrı kusur çıkardı.

**Symlink değişken değildi.** Aynı medyanın yanına dokuz farklı komşu kondu —
düz `.srt`, kardeşe mutlak/göreli symlink, kırık symlink, dizin dışına symlink,
dizine symlink, kendine dönen symlink, boş `.srt` — ve dokuzunda da medya
açıldı. **Değişken sıraydı:** NEN-025'in elle koşusu `Probe.mkv`'yi önce,
`Linked.mkv`'yi sonra açmıştı ve semptomu üreten şey ikinci olmaktı. Ters
sırada aynı `Linked.mkv` hiçbir hata üretmiyor. Symlink ile sıra o koşuda
birbirine karışmıştı; sidecar seyirciydi.

**Kök neden: `loadfile` giden dosyayı bitiriyor ve bu son, gerçek bir hatadan
sebep koduyla ayırt edilemiyor.** İkisi de `reason=STOP, error=0` — kodun kendi
yorumu bunu zaten kaydetmişti. Adapter yalnız `stopRequested` ile susuyordu,
dolayısıyla giden dosyanın sonunu yeni dosyanın hatası sanıyor, kabuk da yoldan
geçen fatal olayı görüp `Dosya okunamadı.` basıyordu — motor bir an sonra
`ready` olsa bile. **Ayırt eden işaret mpv'nin kendisindeydi:**
`playlist_entry_id`. `loadfile` yeni id'yi `mpv_command_ret` ile senkron
döndürüyor ve bu, giden dosyanın sonundan ~560 µs **önce** oluyor (ölçüldü).
Kör yutma kullanılmadı — NEN-051'de reddedilen yaklaşımın aynısı olurdu.

**Hipotezin doğru çıkan yarısı ikinci bir kusurdu ve mimari sınırı deliyordu.**
`sid=no` yalnız **gösterimi** kapatıyor; mpv'nin default `sub-auto=exact`'i
yanındaki `.srt`'yi **açmaya** devam ediyordu. İki gömülü subtitle track'i olan
fixture `[3, 4, 0]` raporluyordu — üçüncüsü external. Yani kataloğun hiç
görmediği, menünün hiç listelemediği (ADR-0031 Karar 4/5) ve NEN-025'in dört
kapısının hiç incelemediği bir kaynak vardı; **symlink kapısı dahil**, çünkü
symlink'li komşu da açılıyordu. `sub-auto=no` eklendi. Bu, `NEN-026`'nın menü
iddiasının zeminini düzeltiyor: menü "kaynakların tamamı burada" diyecekse
motorun arkadan kaynak açmaması gerekiyordu.

**Negatif kontrol iki yönde ve ayrık:** entry-id guard'ı kaldırılınca **2**
kırmızı (semptomu birebir üretiyor), `sub-auto=no` kaldırılınca **2** kırmızı
(track listesi). Her düzeltmenin kendi testleri var. Ayrıca guard'ın sağır
kalmadığı ölçülüyor: açık bir medyadan sonra gerçek bir yükleme hatası hâlâ
raporlanıyor. **Ölçümün kendisi bir kez düzeltildi** — ilk turda negatif
kontroller `git checkout` ile geri alınıyordu ve dosyalar commit edilmediği için
bu iki düzeltmeyi birden siliyordu.

Swift **56 → 62** (seri 2/2), Rust **489** (değişmedi — bu iş Rust'a
dokunmadı). Kanıt: `evidence/M3/NEN-058-measurement.md`.

**`NEN-025` kapandı — altyazı dosyası yükleniyor, sidecar bulunuyor, kapılar
gerçek dosya sistemiyle sınanıyor.** Rust **489** test (`NEN-051`'de 453'tü),
Swift **56** (seri 2/2, paralel 4/4). Kapılar `nen-app`'te düz `std::fs` ile:
fake bir port "bu symlink" demekten ibaret olurdu, bu yüzden testler gerçek
symlink, gerçek FIFO, gerçek dizin-symlink'i ve seyrek 10 MiB dosya kuruyor.

**Sidecar keşfi kod yazılmadan önce ölçüldü ve planlanan çözüm elendi.**
Sandbox altında, medyanın security scope'u **açıkken**: kardeş `.srt` `EPERM`,
dizin listeleme `EPERM`, ve Apple'ın bu iş için gösterdiği related-item
koordinasyonu (`NSIsRelatedItemType` + `NSFileCoordinator`) da üç ayrı
Info.plist kurulumunda `EPERM`. Yani karar "sandbox mı, kolaylık mı" değildi.
Kullanıcı kararıyla **ADR-0034** kabul edildi: App Sandbox kaldırıldı, Mac App
Store non-goal listesine eklendi, notarization (`NEN-043`) etkilenmedi.
Karşılığı açıkça yazıldı — `security-policy.md` §4 kapıları artık **tek**
savunma hattı. Ölçüm: `evidence/M3/NEN-025-sandbox-measurement.md`.

**Boyut sınırı icat edilmedi; var olanı paylaşıldı.** Plan 16 MiB'lık yeni bir
sabit öngörüyordu, oysa `nen_subtitle::encoding::MAX_INPUT_BYTES` (10 MiB,
NEN-015) zaten vardı. İkinci ve daha büyük bir sınır, aradaki bandın önce
tamamen okunup sonra reddedilmesi demek olurdu — §4 #4'ün yasakladığı şey. Bir
test ikisinin ayrışmasını engelliyor.

**Bir DoD maddesi beklenenden farklı yoldan karşılandı.** `NSOpenPanel`
symlink'i **çözüyor** (ölçüldü), yani kullanıcı panel üzerinden symlink teslim
edemiyor. Symlink kapısı tarama yolunda çalışıyor ve gerçek `.app`'te
gösterildi: symlink sidecar sessizce elendi, geçerli sidecar bulundu, 11 MiB'lık
dosya elle yüklendiğinde "Bu altyazı dosyası çok büyük." bildirimi çıktı ve
oynatma sürdü. Kanıt: `evidence/M3/NEN-025-checklist.md`.

Üç yan bulgu ayrıldı: **`NEN-056`** (ADR-0031 Karar 5'in `çok büyük` etiketi
üretilemez durumda — ADR kendi içinde çelişiyor), **`NEN-057`** (dosya adından
dil ipucu), **`NEN-058`** (yanında symlink `.srt` olan medya açılmıyor —
gözlendi, teşhis **edilmedi**). Ayrıca `NEN-049`'a yük altında gözlenen bir
paralel kırmızı işlendi: aranan bağımsız kırmızı sınıfı bir kez görüldü, ama
`aSeekIsAnsweredThroughTheSession`'da değil.

**Kullanıcı üç transport gecikmesi bildirdi; üçü de dosyalandı.** Semptomlar:
ses düzeyi değişimi geç duyuluyor, play/pause bazen geç dönüyor, kaydırıcı
bırakıldığında top önce eski konuma ışınlanıp sonra bırakılan yere geliyor.
Kod okundu, üç kök neden **farklı** çıktı ve üçü de kabukta:

- `NEN-053` — **kapandı.** `consume` bayat `positionChanged`'ı inmiş bir
  seek'in üstüne yazıyordu; artık uçuştaki seek'i tutan bir sayaç var.
- `NEN-054` — **kapandı.** Ölçüm üç adaydan ikisini eledi (bir sürüklemenin
  tam yükü 1.1 ms); kalan ses hattıydı. Ses düzeyi artık cihazda ayarlanıyor
  (`ao-volume`), mpv'nin 200 ms'lik ses tamponunun ötesinde.
- `NEN-055` — **kapandı.** Durum olayının komut dönerken hazır beklediği
  ölçüldü (50–180 us); kabuk artık komuttan sonra aynı turda bakıyor. Negatif
  kontrol semptomun ikinci yüzünü gösterdi: durum geç döndüğü için ikinci
  tıklama `togglePlayback`'i yanlış dalda buluyordu.

`NEN-054` ile `NEN-055`'in teşhisleri **kod okumasıdır**, henüz ölçülmedi.
`NEN-053`'ün teşhisi ölçüldü ve **değişti**: sanılan neden mpv'nin seek'i
servis edene kadar bayat `time-pos` yayınlamasıydı; ölçüm mpv'nin `time-pos`'u
komutu alır almaz hedefe taşıdığını gösterdi. Asıl neden bakma anıydı —
kaydırıcı tutulurken AppKit iç içe izleme döngüsü çalıştırdığı için poll aç
kalıyor, bırakma anında seek komutundan mikrosaniyeler sonra uyanıyor ve
kuyrukta hâlâ sürükleme öncesinin konumunu buluyor. Kayıt:
`evidence/M3/NEN-053-checklist.md`.

`NEN-055` ile `NEN-054`'ün teşhisleri bu yüzden uygulanmadan önce ölçüldü:
birincisi doğrulandı, ikincisinde üç adaydan ikisi elendi. Üçünün de elle
geçişini kullanıcı yaptı — ekran kontrolü reddedilmişti.

Üçünde de ortak sınır: gecikmelerin **sayısal öncesi/sonrası** ölçümü yok,
kullanıcı onayı var. Kanıtlar `evidence/M3/NEN-053|054|055-checklist.md`.

**`NEN-051` kapandı — bir seek artık yalnız kendi cevabını alıyor.**
`NEN-049` "paralel koşuda `aSeekIsAnsweredThroughTheSession` kırmızı" diye
açılmıştı; bağlam okunurken teşhis **yanlış çıktı**. Kusur test izolasyonunda
değil, adapter'daydı: `MPV_EVENT_PLAYBACK_RESTART` bekleyen her seek'i
cevaplıyordu ve tek koruması `pendingSeeks > 0`'dı. Oysa mpv `loadfile` için de
bir restart yayınlıyor ve bu, `state()`'i `Ready` yapan `file-loaded`'dan
**sonra** geliyor. `Ready`'yi görür görmez seek eden bir kabuk bu yüzden
yüklemenin restart'ıyla yarışıyor: kazanan o olursa seek, çekirdek onu daha
servis etmeden `time-pos` ile — yani `0 ms` ile — cevaplanıyor, seek'in kendi
restart'ı ise sayacı boş bulup hiçbir şey söylemiyor.

**Ölçüm düzeltmeyi de seçti.** Geçici bir olay logu (commit edilmedi) gerçek
sırayı gösterdi ve üçüncü bir şeyi ortaya çıkardı: mpv bir seek başladığında
`MPV_EVENT_SEEK` yayınlıyor, o seek'in restart'ından önce ve aynı kuyrukta.
Planlanan iki aday (async komut cevabı · yüklemenin restart'ını körü körüne
yutmak) bu yüzden kullanılmadı — aranan işaret zaten oradaydı. `seekInFlight`
bayrağı bunu tutuyor; bir restart artık `pendingSeeks > 0` **ve** başlamış bir
seek istiyor. Tolerans genişletilmedi, `0 ms` kabul edilmiyor.

**Asıl bulgu tek platformda kalmadı.** Kitin bunu neden yakalamadığı ayrıca
ölçüldü: `EventShape` payload taşımıyor, yani kit `SeekCompleted`'ın **hangi**
konumu taşıdığını hiç sormuyordu; ardından gelen `Position` adımı motora
doğrudan sorduğu ve o ana kadar seek indiği için senaryo yeşil kalıyordu. Olay
yanlıştı, kimse bakmıyordu. `Action::AwaitSeekLanding` eklendi ve üç senaryonun
dört seek adımı buna çevrildi; yargı `seek_tolerance_ms` ile, yani
`Outcome::PositionNear`'ın marjıyla. `EventShape` değiştirilmedi. Android aynı
hatayı artık yazamaz.

**Negatif kontrol iki yönde ve ikincisi kalıcı.** `seekInFlight` guard'ı
kaldırılınca yeni Swift testi bildirilen semptomun birebir kendisini üretiyor:
`(answered → [0]).isEmpty → false`. Fake `Duration::ZERO` raporlamaya
zorlanınca `nen-ports` ve `nen-ffi` köprüsü boyunca **8** test kırmızı.
İkincisi `contract_kit_is_not_vacuous.rs`'e yedinci defect olarak yerleşti —
doğru konuma giden, cevabı doğru sırada veren, yalnız olayın taşıdığı konumu
yanlış söyleyen ikiz. Kontrolün kontrolü de yapıldı: yeni karşılaştırma devre
dışı bırakılınca bu ikiz **tek başına** kırmızı oluyor, dosyadaki diğer yedi
test yeşil kalıyor.

**Oran ölçümü ayırt edici çıkmadı ve bu açıkça kaydedildi.** `NEN-049`'un
istediği öncesi/sonrası kırmızı oranı alındı: paket düzeltmeden **önce de
sonra da** art arda 5/5 yeşil. Yani bu makinede oran, düzeltilmiş ile bozuk
kurulumu ayırt etmiyor; kanıt orana değil, deterministik negatif kontrole ve
doğrudan olay-sırası ölçümüne dayanıyor. Aynı sebeple `.app` üzerinde elle
`→`'ya basma maddesi **düşürüldü** — pencere elle vurulamayacak kadar dar
olduğu için o madde iki kurulumda da aynı sonucu verirdi. 20 tekrarlı otomatik
test bile kusur yerindeyken yeşil kaldı; o test bir oran koruması, mekanizma
kanıtı değil.

**`NEN-049` bu yüzden askıda.** DoD #1'i hiçbir şey değiştirilmeden zaten
sağlanıyor, DoD #3'ün istediği "değişiklik öncesi paralel kırmızı" kaydı ise
üretilemedi. Yeni bir kırmızı gözlenene kadar implementasyona başlanmamalı;
beklemek ya da kapatmak **kullanıcı kararı**. Ayrıca yeni bir yan bulgu
ayrıldı → **`NEN-052`**: port `.loading` sırasında seek'e izin verirken gerçek
adapter `EngineFailure(-12)` ile reddediyor ve contract kiti bu durumu hiç
kapsamıyor.

Rust testleri **452 → 453**, Swift **39 → 41**. Yeni dış bağımlılık yok,
`deny.toml` değişmedi. Tam kanıt: `tasks/done/NEN-051-*.md`.

**`NEN-048` kapandı — boş durumda sahte bildirim yok, geçici sınıf kanıtlı.**
`resynchronize()` artık hatayı kullanıcıya haber etmiyor: resync kullanıcının
eylemi değildir, reddedilmesi ADR-0031 Karar 1'in geçici sınıfına girmez.
Uygulama medya yokken her öne geldiğinde çıkan `Önce bir medya açın.` bildirimi
böylece kayboldu. Kullanıcı eyleminden türeyen üç yol (`seek`, `setVolume`,
`togglePlayback`) olduğu gibi duruyor. `FakeSession` artık çağrı bazında hata
enjekte edebiliyor — NEN-024'te bu sınıfın kanıtsız kalmasının sebebi buydu.
Sekiz yeni testle seri paket **39 test / 6 suite** (art arda 2/2); gerçek
`.app` üzerinde **4/4** manuel acceptance geçti. Testlerin boş olmadığı iki kez sınandı: düzeltme geri
alınınca sessizlik testleri tam olarak bildirilen semptomu üretti, metne
bilinçli sızıntı enjekte edilince negatif test yakaladı. Son-medya deposundan
türeyen üç geçici yol testsiz kaldı ve `NEN-050`'ye ayrıldı. Kanıt:
`evidence/M3/NEN-048-checklist.md`.

**`NEN-049` hakkındaki eski ölçüm — artık `NEN-051` ile açıklanıyor.**
Kusur "her paralel koşuda kırmızı" değil, yük altında aralıklıydı: aynı gün
paket önce paralel modda 4/4 yeşil geldi, kapanış doğrulamasında paralel 3/4 ·
seri 2/2 ölçüldü ve kırmızı olan hep `aSeekIsAnsweredThroughTheSession`'dı.
Bunun sebebi `NEN-051`'de ölçüldü ve giderildi; paralellik sebep değil, dar bir
zamanlama penceresini genişleten koşuldu.

**`NEN-046` kapandı — pencere ve uygulama kapanışı artık ayrı.** Oynatıcı
sahnesi `WindowGroup` yerine tek `Window`. Kırmızı düğme ve `⌘W` oynatmayı
temizleyip pencereyi kapatıyor fakat aynı uygulama PID'i yaşıyor; Dock ve
pencere kapalıyken `⌘O` tek pencereyi yeni oturum/polling ile geri getiriyor.
Yalnız `⌘Q` uygulamayı sonlandırıyor. Model eski medya, state, pozisyon ve
süreyi temizliyor; saklanan video yüzeyine resume ile yeniden bağlanıyor.
İki model testi, seri macOS paketinin **31 test / 6 suite** koşusu ve gerçek
`.app` üzerinde **8/8** manuel yaşam döngüsü geçti. Paralel pakette ölçülen
bağımsız libmpv test izolasyonu kusuru `NEN-049`'a ayrıldı. Kanıt:
`evidence/M3/NEN-046-checklist.md`.

**`NEN-024` kapandı — macOS artık tek pencerede gerçek video oynatıyor.**
SwiftUI kabuk; AppKit/libmpv render yüzeyi, transport, yedi kısayol grubu,
otomatik gizlenen kontroller, Settings sahnesi, kapalı-küme Türkçe hata sunumu,
tek security-scoped bookmark ve sessiz `EventsLost` resync davranışını taşıyor.
Geliştirme `.app` paketi App Sandbox + app-scoped bookmark + kullanıcı-seçimli
salt-okunur dosya yetkisiyle ad-hoc imzalandı ve `codesign --verify` geçti;
libmpv hâlâ Homebrew'dan dinamik bağlı — uygulama içine gömme/notarization
`NEN-043` kapsamında. **29 Swift testi / 6 suite** ve repo tooling'inin **2 test
dosyası** geçti. Sentetik fixture ile 10 saniyelik oynatma kaydı, transport
ekranı ve tam manuel checklist `evidence/M3/NEN-024-*` altında.

**Kabuk çalışır `.app` üzerinde incelendi; üç kusur ölçüldü, biri kapandı.** `NEN-024`'ün
kapanışı geçerli — kapsam maddelerinin karşılığı kodda var, ADR-0031'in
gizlilik kararları gerçekten uygulanmış. Ama kanıt kaydının kapsamadığı üç
davranış canlı uygulamada doğrulandı: (1) pencere kapatılınca uygulama menü
çubuğunda yaşamaya devam ediyor ve `⌘O` ile dosya seçilse bile hiçbir şey
olmuyor — geri dönüş yolu yok → **`NEN-046` ile kapandı**; (2) yedi kısayol menü key
equivalent'ı olduğu için **Ayarlar penceresi öndeyken de** çalışıyor (`↓`
sesi düşürdü, `←` konumu `00:30 → 00:25` aldı) ve kontroller gizliyken
klavyeyle yapılan seek ekranda hiçbir iz bırakmıyor → `NEN-047`; (3) medya
yokken uygulama her öne geldiğinde `Önce bir medya açın.` geçici bildirimi
çıkıyor — kullanıcı hiçbir şey yapmamışken, ADR-0031 Karar 1'in geçici sınıf
tanımına aykırı → `NEN-048`. Üçüncüsünün görülmeme sebebi de aynı yerde:
`FakeSession` hiç throw etmediği için geçici hata sınıfının **hiçbir testi
yok** ve manuel checklist'te de maddesi yok; fatal sınıf kanıtlı, geçici sınıf
kanıtsız kapanmış. Ayrıca `UserDefaultsRecentMediaStore` bayat bookmark'ı
scope açılmadan tazelemeye çalışıyor ve başarısız olursa çözülmüş kaydı
siliyor — bu `NEN-042`'nin kapsamına eklendi. `NEN-037` artık READY değil:
kısayol kapsamı düzelmeden iki dil seçicisi klavyeyle kullanılamaz.

**`NEN-045` kapandı — kabuk artık çekirdek üzerinden oynatıyor.** `NEN-024`
planlanırken çıkan boşluktu: `nen-ffi` bugüne kadar yalnız contract kitini
sürmek için açılmıştı (`version()`, `runPlaybackContract()`, motor trait'leri),
yani kabuğun `load`/`play`/`seek` diyebileceği bir nesne **hiç yazılmamıştı**.
ADR-0026 oturumu çekirdeğe koyduğu için kabuk motoru doğrudan da çağıramazdı.
`PlaybackSession` bu boşluğu kapattı: gövde `nen-app`'te, `nen-ffi` yalnız
tipleri çeviriyor. Rust testleri **438 → 452**, Swift **16 → 21**. Yeni dış
bağımlılık yok, `deny.toml` değişmedi.

**Asıl bulgu teslimat yönünü belirledi → `ADR-0033`.** Bounded kuyruk
çekirdekte, ama **adapter'ın kendi `pending` dizisi sınırsız**. Kimse
`drainEvents()` çağırmazsa — uygulama arka planda — birikme kuyruğa hiç
girmiyor, dolayısıyla `EventQueue`'nun 64'lük sınırı hiçbir şeyi sınırlamıyor ve
ADR-0011 Karar 1'in `EventsLost` garantisi kâğıt üstünde kalıyordu. Çözüm
çekirdekte sürekli çeken bir pump: birikme her zaman kuyrukta olur, taşma
`EventsLost`'a döner, öne gelen kabuk eksikliği **öğrenir** (ADR-0031 Karar 3).

**Teslimat push değil pull.** Push (foreign `EventSink`) aynı birikmeyi adapter
dizisinden MainActor dispatch kuyruğuna taşırdı — orada ne sınır ne coalescing
var — ve olay başına ~77 µs hop eklerdi. Portun `EventSink`/`deliver_all` yolu
duruyor ama FFI'da kullanılmıyor; ters çağrı olmadığı için reentrancy de bu
yolda oluşamıyor.

**Pump'ın değeri ölçülünce daraldı ve netleşti.** `drain_events()` zaten
kendisi çekiyor, yani düzenli drain eden bir kabuk pump olmadan da her olayı
görür. Pump'ın kapattığı tek senaryo **kimsenin drain etmediği** an — ve kanıt
kaydı bunu iki teste bölerek yazıyor: biri gözetimsiz motorun sınırsız
biriktirdiğini, diğeri pump'ın sormadan çektiğini ölçüyor.

**Outbound olay tipi ayrı yazılmak zorunda kaldı.** `FfiPlaybackEvent`
`EventsLost` taşımıyor ve bu bilinçli: bir adapter onu iddia edebilseydi teslim
edemediği akışı gizleyebilirdi. Ama kabuk öğrenmek zorunda. İki yön iki tip
oldu; `FfiSessionEvent` yalnız o variant'ı fazladan taşıyor.

**Negatif kontrol beş yönde.** Pump hiç spawn edilmiyor → **3** kırmızı ·
shutdown pump'ı durdurmuyor → **2** · shutdown guard'ı yok → **1** ·
`PositionChanged` kritik sınıfa alınıyor → **7** (2'si nen-app) · `EventsLost`
sınırda düşürülüyor → **1**.

**Güvenlikte yeni bir yön açıldı ve yazıldı.** Track başlığı ilk kez **geri** de
geçiyor (`session.tracks()`). Argüman round-trip olması: değer kabuğun kendi
ürettiği değer, değişmeden sahibine dönüyor. `nen-ffi`'nin modül dokümanı
"inbound only" ifadesinden bu kurala güncellendi; yasak olan **Rust'ın onu
basması** ve o kural yerinde duruyor. Oturum nesnesi locator tutmuyor ve `Debug`
türetmiyor — buna dair test **yazılmadı**, çünkü bir trait'in yokluğunu ölçen
test boşta döner.

**Kabul edilen maliyet: `nen-app` artık bir thread'e sahip.** Bugüne kadar
tamamen çağrı-güdümlüydü. Yaşam döngüsü iki testle bağlı ve ikisi de pump
durdurulmadığında kırmızı.

Tam kanıt: `tasks/done/NEN-045-*.md`.

**`NEN-023` kapandı — gömülü track'ler artık ürün tarafında.** Bugüne kadar
track listesi yalnız port seviyesinde vardı ve kataloğa giden hiçbir kod yolu
yoktu; ADR-0031 Karar 4 ise menünün açıldığı anda gömülü track'leri göstermeyi
şart koşuyor. `nen-app::embedded` bu boşluğu kapattı: subtitle track'leri
`SubtitleSourceCatalog` girdilerine dönüyor, seçilen girdi tekrar bir `TrackId`'ye
çözülüyor. Rust testleri **406 → 438**, Swift **9 → 16**. Yeni dış bağımlılık
yok, `deny.toml` değişmedi.

**Bitmap ayrımı yanlış yerdeydi, düzeltildi.** codec → metin/bitmap listesi Swift
adapter'ının içindeydi, yani Android kendi kopyasını yazmak zorunda kalacaktı.
Liste `nen-ports`'a taşındı (`subtitle_carries_text`); `FfiTrackDescriptor.is_text`
alanı **kalktı** — adapter'ın yanlış doldurabileceği bir alan değil, çünkü
gönderdiği bir alan değil. Yön muhafazakâr: **bilinmeyen codec metin sayılmıyor**,
çünkü okuyamadığımız bir biçim için metin vaat etmek, çeviri istendiği anda boş
belge üretir.

**Ölçüm plana girmemiş gerçek bir kusur buldu → `ADR-0032`.** Track listesi ilk
kez gerçek fixture'dan okununca çıktı: **Matroska ISO 639-2 yazıyor**, libmpv
`eng` · `tur` · `fre` veriyor. `LanguageTag::parse` 2–3 harfli primary'yi geçerli
saydığı için `eng` hatasız ayrışıyor ve `en`'den **farklı** bir etiket oluyor.
Sonuç: kullanıcının `Movie.en.srt`'si ile aynı filmin gömülü İngilizce track'i
menüde **iki ayrı grup**, ve tercihi `en` olan kullanıcı için hiçbir gömülü track
otomatik açılmıyor. Bu ADR-0030'un çözdüğü problemin aynısı, başka bir eksende —
orada region (`en` / `en-us`), burada kod standardı (`en` / `eng` / `fre`).

**ADR-0032 accepted: indirgeme `parse`'ın kendisinde.** Konteyner sınırında değil,
çünkü aynı üç harfli kod `.nfo`, dosya adı ve ileride OpenSubtitles kapılarından
da giriyor — kanonikleştirmeyi kapılara dağıtmak, ADR-0030'un düzelttiği "aynı
soruyu birden çok yerde sormak" hatasının birebir tekrarı olurdu. Tablo 204
satır: 184 ISO 639-1 dili ve 20 /B–/T çifti (`fre`/`fra`, `ger`/`deu`). Karşılığı
olmayan kod (`fil`, `haw`, `nds`) **olduğu gibi kalıyor** — hiçbir şey tahmin
edilmiyor, hiçbir şey kesilmiyor. Kabul edilen maliyet: `parse("eng")` artık
`"en"` döndürüyor, yani girdiyle çıktı birebir aynı değil.

**Extraction `NEN-044`'e taşındı (kullanıcı kararı).** libmpv bir subtitle
track'inin **tam** metnini veren API sunmuyor (`sub-text` yalnız o anki cue);
gerçek çıkarım konteyneri demux etmeyi gerektiriyor ve kendi ADR'sini istiyor
(libavformat mı, Rust konteyner parser'ı mı). M3'te bu metni tüketen hiçbir şey
yok — gömülü track seçilince motor kendisi çiziyor, NEN-027'nin injection'ı harici
belgeler için, çeviri M5'te — ve şartname §7 extraction'ı "yapılabilir" diyerek
zorunlu kılmıyor. **Lazy kuralının kanıtı NEN-023'te kaldı:** `extract_text`
çağrılınca panikleyen bir motor üzerinden katalog kurma yolu baştan sona koşuyor.

**Bitmap fixture'ı ffmpeg ile üretilemedi.** `Subtitle encoding currently only
possible from text to text or bitmap to bitmap` — ffmpeg metinden bitmap
rasterize etmiyor. Minimal bir HDMV PGS akışı (PCS · WDS · PDS · ODS · END +
temizleyen ikinci display set) elle üretilip `-c:s copy` ile muxlandı; üretici
deterministik ve reçetesi `fixtures/media/bitmap-subs-clip.ffmpeg.txt`'de,
çalıştığı doğrulanmış durumda.

**Negatif kontrol beş yönde, ve ölçümün kendisi iki kez düzeltildi.** Sınıflandırma
her codec'e `true` → **5** kırmızı · ADR-0032 devre dışı → **6** · FFI'da elle
`Debug` yerine `derive` → **3** · bitmap işareti taşınmıyor → **2** · ters eşleme
çözülmüyor → **1**. İlk turda `cargo test`'in **ilk kırmızı hedefte durduğu**
fark edildi (sonraki crate'ler hiç koşmuyordu, K1 yalnız 1 kırmızı görünüyordu);
`--no-fail-fast` ile gerçek sayılar alındı. İkincisi: kırmızı testleri toplayan
grep'in deseni rakam içermediği için `iso_639_2_codes_...` sayılmıyordu.

**Güvenlikte kapsanmayan taraf açıkça yazıldı.** Track başlığı FFI'yı geçen ikinci
string oldu (ilki locator). Rust tarafı korunuyor: `FfiTrackDescriptor`'ın
`derive(Debug)`'ı kaldırıldı ve kasıtlı derive'lı ikiz dört yasak parçanın hepsini
sızdırarak guard'ın boşta dönmediğini kanıtlıyor. **Swift tarafı korunamaz** —
üretilmiş düz bir struct, `String(reflecting:)` başlığı gerçekten basar. Önce
buna zayıf bir test yazıldı, sonra **kaldırıldı**: boşta dönen bir testle sınırı
örtmek, sınırı yazmaktan kötü. `RedactionTests`'in yorumu artık neyin yapısal
olarak tuttuğunu ve neyin disiplin olduğunu ayırıyor.

Tam kanıt: `tasks/done/NEN-023-*.md`.

**`NEN-022` kapandı — macOS'ta gerçek video oynuyor ve gerçek adapter
paylaşılan contract kitini geçiyor.** M3'ün "gerçek libmpv adapter'ı, fake
adapter ile **aynı** contract kitini geçmelidir" çıkış şartı karşılandı ve kit
**ikinci kez yazılmadı** (ADR-0011 Karar 4): `nen-ports`'un senaryo listesi
`nen-ffi` üzerinden sürülüyor. `platforms/macos` artık dolu (Swift + libmpv,
ADR-0012 Karar 2). Rust testleri **390 → 406**, ayrıca **9** Swift testi.
Yeni Rust bağımlılığı yok, `deny.toml` değişmedi.

**Kit gerçek motoru üç yerde haksız yere çaktırıyordu.** `Load → State == Ready`
anında bekleniyordu (`loadfile` asenkron); olay dizisi tam eşitlik istiyordu
(gerçek motor daha fazlasını raporluyor); `Seek` sonrası `Position` hemen
soruluyordu (seek asenkron). Sırasıyla `Action::Settle`, alt-dizi eşleşmesi ve
`Action::AwaitEvent` + zaman aşımına kadar bekleyen `Outcome::Events` ile
düzeltildi. Üçüncüsü aslında portun kendi tasarımıydı: `SeekCompleted` tam da
"seek indi mi" sorusunun cevabı.

**Kitte fake'in biyografisi gömülüydü.** `DurationMs(Some(120_000))`,
`TrackCount(2)`, `TrackId(1)` doğrudan senaryolardaydı — yani gerçek fixture
tam 120 saniye ve tam o id'lere sahip olmaya zorlanırdı. Bunlar
`ContractInputs`'a taşındı; adım artık sayı değil **rol** adlandırıyor
(`TrackRef::Known`, `Outcome::FixtureDuration`). Bu tam olarak ADR-0011
Karar 4'ün önlemek istediği "kit bir adapter'a göre şekillenir" durumuydu.

**Gevşetmenin bedeli ayrıca ödendi.** `contract_kit_is_not_vacuous.rs` doğru bir
motoru kitin artık tolere ettiği **her** yönde bozuyor ve altısında da kırmızı
olmasını şart koşuyor: toleransın dışına düşen seek · `Ready`'ye hiç gelmeyen
motor · düşürülen kritik olay · sırası bozulmuş olaylar · sorulmadan gelen
`Failed` · her id'yi kabul eden seçim. Yedincisi kontrolün kontrolü.

**Kit gerçek bir adapter hatası buldu:** mpv arka arkaya verilen iki seek'i tek
`playback-restart`'a birleştiriyor, dolayısıyla ikinci seek'in cevabı hiç
gelmiyordu — o cevabı bekleyen bir kabuk sonsuza kadar beklerdi. Artık bekleyen
her seek cevaplanıyor.

**Köprüde ikinci gerçek kusur bulundu ve negatif kontrolle kanıtlandı.**
`ShellEngineBridge` her çağrıyı olduğu gibi iletiyordu, yani ADR-0011 Karar 2'nin
reentrancy yasağı yalnız *fake kendi guard'ını taşıdığı için* tutuyordu. Swift
adapter'ında böyle bir guard olamaz — işareti taşıyan thread-local Rust'ın —
ve NEN-029'un ölçtüğü self-deadlock'a girilirdi. Karar 2 yasağı açıkça porta
verdiği için guard köprüye kondu; kanıt **çağrılınca panikleyen** bir motorla
alınıyor. 15 guard mekanik olarak silinince test kırmızıya döndü, geri alındı.

**Ölçüm: libmpv takılmıyor, takılan mpv CLI'ı.** `doctor.sh`'ın yorumu bu
makinede doğrulandı — `mpv --frames=1` 20 sn'de dönmedi, libmpv aynı dosyayı
0.5 sn'de bitirdi. Adapter kütüphaneye dayandığı için M3 etkilenmiyor.
Ayrıca ölçüldü: `seek absolute+exact` **tam** iniyor (tolerans 100 ms tanındı
ama gerekmedi); mpv track id'leri **tür başına** 1 tabanlı, port'un id uzayı ise
türler arasında tek — adapter bu yüzden `ff-index`'i dışarı veriyor.
Tam kanıt: `tasks/done/NEN-022-*.md`.

**`ADR-0012` accepted oldu — `NEN-022`'nin önünü açan karar.** M3'ün ikinci
kapısıydı ve son açık mimari sorusuydu; dört kararla kapandı: (1) macOS motoru
**libmpv** — aday statüsü kalktı; (2) adapter **Swift'te**, `platforms/macos/`
altında (bugün boş) — Rust-tarafı bir mpv crate'i reddedildi, çünkü ADR-0011
Karar 4'ün "kit ikinci kez yazılmasın" şartını anlamsızlaştırır ve `NEN-024`'te
video yüzeyi zaten AppKit'e ait olacak; (3) **geliştirmede dinamik link**
(`pkg-config mpv`, Homebrew dylib'i), **dağıtımda `.app` içine gömme** — gömme
bir paketleme işi olduğu ve adapter kodunu değiştirmediği için `NEN-043`'e
ayrıldı (Kural 5); (4) proje lisansı **GPL-3.0-or-later**.

**Lisans kolu böylece kapandı: depo artık lisanssız değil.** Kökte `LICENSE`
(GPL v3 tam metni, `/opt/homebrew/Cellar/gettext/1.0/COPYING`'in birebir
kopyası), `docs/licensing.md` kararı anlatan bir belgeye dönüştü, roadmap
**S12** cevaplandı, **S11** daraldı ve risk **R2** "Yüksek"ten "Orta"ya düştü.
Belirleyici ölçüm: Homebrew `mpv 0.41.0_8` lisansı `GPL-2.0-or-later AND
LGPL-2.1-or-later` — yani GPL kollu. İki yol vardı: bu ikiliyi olduğu gibi
gömüp projeyi GPL yapmak (ek iş **yok**), ya da mpv'yi `--enable-lgpl` ile
kendin derleyip kodu kapalı tutabilmek (mpv + bağımlılıklarını LGPL
konfigürasyonuyla derleyen bir build altyapısı). Kullanıcı kararı açık kaynak
olduğu için ikinci yolun tek faydası ortadan kalktı.

**Sürüm seçimi ölçümle zorunlu çıktı.** v3 tercih değil kısıt: `core/deny.toml`
allow listesinde **Apache-2.0** var (uniffi ve ağacın büyük kısmı) ve
Apache-2.0 **GPLv2 ile uyumsuz**, GPLv3 ile uyumlu — patent hükmü GPLv2'nin
kabul etmediği bir ek şart sayılıyor. mpv `GPL-2.0-**or-later**` olduğu için
v3'e yükseltilebiliyor; yani GPLv3 bu ağaçtaki tek uyumlu nokta. `deny.toml`
değişmedi (allow listesinin yedi girişinin hepsi GPL-3.0 uyumlu; MPL-2.0 kendi
§3.3'ü ile açıkça izin veriyor).

**Kabul edilen maliyetler kayıtlı:** kaynak kodu açık olacak ve bu **geri
alınamaz** (dağıtılan sürüm geri çekilemez); **App Store yolu kapandı** —
Apple'ın şartları GPL'in yasakladığı ek kısıtlar getiriyor, side-loading ve
GitHub release açık; `NEN-043` kapanana kadar kullanıcının `brew install mpv`
yapması gerekiyor. Bir de kapsam etkisi: ADR-0011 Karar 3 track enumeration/
selection'ı zorunlu tabana koyduğu için contract kiti onsuz geçmiyor —
`NEN-022` bunu **port seviyesinde** yapmak zorunda, `NEN-023` katalog
seviyesindeki semantiğe (bitmap tespiti, metin çıkarımı) daralıyor.

**M3 UI beyin fırtınası 1. turu yapıldı (kod yazılmadan).** `NEN-022`
başlamadan önce, UI'ın kodun şeklini belirleyen tarafı karara bağlandı:
[`ADR-0031`](adr/0031-macos-shell-interaction-model.md) altı kararla
açıldı ve **accepted** oldu — hata sunumunun üç sınıfı, ekranda tam yol/query yasağı ve
kanıt kaydının `fixtures/` zorunluluğu, `EventsLost` sonrası **sessiz** resync,
menünün taramayı beklememesi ve liste büyürken seçimin kaymaması, hatalı kaynak
ile güvenlikten dönen dosyanın ayrılması, M3'ün tek ayar yüzeyi. `NEN-024`,
`NEN-025`, `NEN-026`, `NEN-037` kapsamları buna göre güncellendi; boş durumun
tam "son açılanlar" listesi `NEN-042` olarak ayrıldı (kural 5). `NEN-024` ile
`NEN-037` arasındaki ayarlar-ekranı çelişkisi ADR-0031 Karar 6 ile kapandı.
`NEN-024`, `NEN-025`, `NEN-026`, `NEN-037` artık `done` yoluna açık.

**`NEN-021` kapandı ve `ADR-0011` accepted oldu — M3'ün ilk ürün kodu var.**
Boş duran `nen-ports` crate'i artık `PlaybackEngine` portunu, capability
modelini, event teslimat kurallarını, paylaşılan contract kitini ve kiti geçen
fake adapter'ı taşıyor. **Yeni dış bağımlılık yok** — `nen-ports` tek kenar
(`nen-domain`), ADR-0006 grafiği korundu. Test sayısı **337 → 390**.

**ADR-0011'in dört kararı** ADR-0026'nın bıraktığı iki ölçülmüş riski kapatıyor:
(1) event teslimatı **bounded kuyruk + sınıfa göre coalescing** — `PositionChanged`
tek slot tutuyor (mutlak değer, en yenisi doğru olan), `StateChanged` ·
`SeekCompleted` · `TracksChanged` · `EndReached` · `Failed` sıra koruyor ve
**sessizce düşmüyor**; taşma olursa kuyruk `EventsLost { dropped }` ile
kapanıyor ve tüketici resync ediyor. (2) Callback içinden senkron çağrı
**yasak** → `ReentrantCall`, deadlock değil. (3) Capability yalnız motorlar
arasında **farklılaşanı** sayıyor (dört giriş); taban zorunlu ve sorgulanmıyor,
**relative seek capability değil** — port `position` + `seek` üzerinden default
veriyor. (4) Contract senaryoları **veri**, kod değil — NEN-022 aynı listeyi
FFI'dan sürecek, ikinci kopya yazmayacak (M3 çıkış kriteri bunu şart koşuyor).

**Karar 1'in gerekçesi inceleme sırasında düzeltildi.** İlk taslak ack tabanlı
teslimatı **maliyet** gerekçesiyle reddediyordu; sayılar bu argümanı taşımıyor:
ack 60 Hz'de 4.6 ms/sn'yi ~7 ms/sn'ye çıkarırdı, yani 16.7 ms'lik karenin
%0.46'sı yerine %0.70'i — ikisi de hissedilmez, tam da ADR-0026'nın kendi A/B
karşılaştırmasında bulduğu gibi. Gerçek itiraz yapısal: **video gerçek zamanda
oynuyor, ack üreticiyi durduramaz.** Yığılma kararını yok etmiyor, üreticiye
devrediyor — sınırsız kuyruk sorunu geri geliyor ya da yine bir düşürme kuralı
gerekiyor.

**Kit'in boşta dönmediği ayrıca test ediliyor.** 23 senaryonun 19'u tam
capability'li, 18'i taban-only motora uygulanıyor; `the_run_is_not_vacuous`
sayılara alt sınır koyuyor ve her senaryonun en az bir uçta koştuğunu,
`every_capability_is_covered_in_both_directions` her capability için hem **var**
hem **yok** senaryosunun bulunduğunu şart koşuyor. 4 capability'nin **16 alt
kümesi** de ayrı ayrı koşuluyor — capability'lerin bağımsızlığı böyle
kanıtlanıyor.

**DoD #3'ün grep'i elle değil test olarak koşuyor.** `nen-ports` ve `nen-app`
altındaki tüm `.rs` dosyalarında 8 motor adı aranıyor, satır yorumları
çıkarılarak (port'un kendi dokümantasyonu bu adları serbestçe anıyor; yasak
olan adın **koda** girmesi). Üç kontrol testi taramanın kendisini doğruluyor.

**Negatif kontrol üç kusurla yapıldı:** capability kontrolünü atlamak 2 testi,
coalescing'in kritik olayları da düşürmesi **7** testi, reentrancy guard'ının
her zaman `Ok` dönmesi **4** testi kırmızıya döndürdü; üçü de geri alındı.
Güvenlik tarafında kalıcı negatif kontrol var: `#[derive(Debug)]`'lı üç ikiz
(media source, track descriptor, String taşıyan hata) aynı değerlerle gerçekten
sızdırıyor. Tam kanıt: `tasks/done/NEN-021-*.md`.

**`NEN-039` kapandı ve `ADR-0030` accepted oldu — dil gruplaması ve tercih
eşleşmesi artık primary subtag üzerinden.** `nen-domain`'e
`LanguageTag::primary()` / `primary_tag()` eklendi; `nen-catalog`'un menü
gruplaması (`BTreeMap` anahtarı) ve otomatik seçim eşleşmesi bunları kullanıyor,
`nen-subtitle`'ın aynı ayrımı elle yapan özel `primary_subtag()` yardımcısı
silindi — iki kod yolu artık aynı soruyu aynı yerden soruyor. `en` ve `en-us`
menüde **tek** grup; kaynağın tam etiketi korunuyor (ADR-0029 Karar 5 duruyor).

**İkinci kırılma yönü inceleme sırasında bulundu ve ADR'ye yazıldı:** tam
eşitlik yalnız kaynağın region'ında değil, **tercihin kendi region'ında** da
kırılıyordu. NEN-026 tercihi sistem dilinden alırsa macOS `tr-TR` verir,
katalogtaki track'ler ise `tr` etiketlidir — kullanıcı tercih ayarlar, hiçbir
grup üste çıkmaz ve hiçbir altyazı otomatik açılmazdı. İki taraf da primary'ye
indiği için bu asimetri kalktı.

**Negatif kontrol iki gerçek test kusuru buldu.** Kod geçici olarak tam-etiket
hâline döndürülünce 6 yeni testin yalnız 4'ü kırmızıya döndü: bir menü testi
`en`'i tercih ettiği için alfabetik sırayla aynı sonucu üretiyor (hoist
edilmese de geçiyor), diğeri katalogda tek kaynak olduğu için eski kodda da
mükerrer grup açamıyordu. İkisi de düzeltildi (tercih `fr-ca`, katalog `en` +
`en-us`), sonra 6/6 kırmızı — testler artık boşta dönmüyor.

**Kabul edilen maliyet kilitlendi:** tercihi `pt-br` olan kullanıcı `pt-pt`
yerine `pt-br`'yi **isteyemiyor** (grup içinde region tiebreak yok, ADR-0010
Karar 5'e dokunurdu). Bu davranış `region_is_not_a_tiebreak_inside_a_language`
testiyle sabitlendi — değişirse sessizce değil, kırmızı testle değişir.
Test sayısı **328 → 337**; yeni dış bağımlılık yok. Tam kanıt:
`tasks/done/NEN-039-*.md`.

**Eski `NEN-020` ve M2 kapanışı — subtitle dili artık offline ve güven eşikli.**
`nen-subtitle`, Whatlang 0.18 ile bütün belgeyi bir kez sınıflandırıyor;
`> 0.90` adayı kanonik iki harfli `LanguageTag` yapıyor, eşik altını
`Dil Belirsiz` bırakıyor. Metadata nihai dilde her zaman kazanıyor; güvenilir
metin farklı primary language bulursa typed conflict korunuyor, region farkı
conflict sayılmıyor. 8 doğru dil + kısa/karışık unknown golden'ı dört yazı
sistemini kapsıyor; K23 guard sonucu subtitle diyaloğu taşımıyor. Whatlang'ın
70 dil eşlemesi exhaustive. Test sayısı **317 → 328**; tam kanıt
`tasks/done/NEN-020-*.md`.

**M3 toolchain kapısı açıldı — `NEN-021` artık başlayabilir.** Bu STATUS
2026-08-25'e kadar tam Xcode ve libmpv'yi eksik sayıyordu; ölçüm ikisinin de
kurulu olduğunu gösterdi (Xcode 26.6 · libmpv 2.5.0). Eksik olan tek şey
**Xcode lisansının kabul edilmemiş olmasıydı**: `swift --version` çıkış 69 ile
`You have not agreed to the Xcode license agreements.` dönüyordu.
`sudo xcodebuild -license accept` sonrası `bash scripts/doctor.sh M3` **exit 0**
veriyor. B3 kapandı.

**Kapı açılırken `doctor.sh`'ta gerçek bir yanlış pozitif bulundu →
`NEN-041`.** Lisans kabul edilmeden önce `swift` çalışmıyordu, ama doctor onu
`✓ swift (sürüm okunamadı)` diye raporlayıp M3 kapısını **açık** gösteriyordu:
`detect_swift`, `command -v` guard'ını geçtikten sonra sürüm okunamasa da
çıkış 0 dönüyor. Bu, NEN-005'te `detect_jdk`'da bulunan kusurun aynısı ve tam
olarak NEN-032'nin önlemek istediği hata sınıfı. Kural 5 gereği bulunduğu işe
eklenmedi, kendi task'ına alındı.

**Xcode.app GUI'si macOS 27 beta'da açılmıyor — M3 için engel değil.**
`LSMinimumSystemVersion` 26.2, yani sürüm engeli değil. Repo Swift tarafını
SwiftPM ile sürüyor (`scripts/build-apple.sh`, `scripts/test-apple.sh`);
kullanılan şey `Xcode.app/Contents/Developer` ağacı (SDK'lar: macOS 26.5 ·
iOS 26.5 · tvOS 26.5, swift-testing plugin'i, Frameworks) ve o ağaç çalışıyor.
M3'ün beş çıkış kriterinin hiçbiri GUI istemiyor; GUI'nin götürdüğü şey
NEN-024'te SwiftUI Preview / Instruments / simulator olur.

**Kaydedilen sapma: Swift 6.4 → 6.3.3.** `xcode-select` artık
CommandLineTools yerine Xcode'u gösterdiğinden Xcode'un bundled toolchain'i
kullanılıyor; CLT'ninki 6.4'tü. M1 ölçümleri (NEN-008/009/010/011/029) 6.4 ile
alınmıştı — **baseline oldukları için geçersiz olmuyorlar**, ama başka bir
makineyle karşılaştırılırken bu fark bilinmeli.

**`NEN-019` kapandı — dört kaynak tek katalog ve tek menü projeksiyonunda.**
`nen-domain`'e `SubtitleSourceKind` · `SubtitleSourceId` · `SubtitleSource` ·
`LanguageTag` · `SubtitlePreferences`; `nen-catalog`'a metadata-kimlikli upsert,
dil/kullanıcı gruplama, yapısal menü projeksiyonu ve saf otomatik seçim
politikası eklendi. Otomatik seçim yalnız tercih edilen dillerde ve bugün yalnız
`embedded` → `user`; `opensubtitles` basamağı NEN-038'e kadar kapalı, `ai` hiçbir
koşulda otomatik seçilmiyor.

**ADR-0010 Karar 10'un iki menüsü byte-eşit golden.** Aynı katalog tercih yokken
`en` → `fr` → `tr`, birinci `tr` / ikinci `en` tercihinde `tr` → `en` → `fr`
grup sırasını üretiyor; kullanıcı grubu `Kapalı`'nın hemen altında, bilinmeyen
dil her zaman en sonda. `Debug` guard'ı özel tam yol, dosya adı, digest ve
private file ID'yi gizliyor; `#[derive(Debug)]`'lı bozuk ikiz dördünü de
sızdırarak negatif kontrolü kanıtlıyor. Yeni doğrudan dış bağımlılık yok,
`nen-domain` hâlâ tek düğüm. Test sayısı **284 → 317**. Tam kanıt:
`tasks/done/NEN-019-*.md`.

**`NEN-018` kapandı — `nen-identity` artık dolu.** Dokuz modül eklendi
(`os_hash` · `release_name` · `url_hints` · `dir_hints` · `declared_name` ·
`siblings` · `nfo` · `container` · `evidence`), **yeni dış bağımlılık yok** —
`nen-identity` tek kenar (`nen-domain`), `nen-domain` hâlâ tek düğüm,
`deny.toml`'a dokunulmadı. Test sayısı **121 → 284**.

**Golden'lar dört gerçek parser kusuru buldu, hiçbiri elle fark edilmemişti:**
(1) `Blade.Runner.2049.2017…` başlığı "Blade Runner", yılı **2049** sanıyordu —
arka arkaya iki yıl-benzeri token varsa artık ilki başlığın parçası; (2)
`[SubGroup] Steins Gate` başlığa fansub grubunu katıyordu; (3) `Unknown` bir
sonuç yarım başlık bırakıyordu — artık `Unknown ⇒ title = None`; (4)
`Severance (2022)/Season 02/` `movie` dönüyordu, sezon da artık dizi işareti.

**Wire biçimi kusuru elle doğrulanabilir vektörle yakalandı.** Hash ilk sürümde
`to_le_bytes()` ile saklanıyordu; OpenSubtitles `%016x` ile **big-endian**
basıyor. Tamamı sıfır olan 131 072 byte'lık dosyanın hash'inin kendi boyutu
olması gerektiği (`0000000000020000`) ilk koşuda uyuşmazlığı gösterdi — yani
gönderilecek hash baştan yanlış olacaktı. DoD #1 üç ayrı kanıt taşıyor: elle
doğrulanabilir bilinen cevap · kasıtlı olarak farklı yazılmış **bağımsız
referans implementasyonu** (açık indeks aritmetiği + elle bit kaydırma) ·
commit edilmiş golden.

**Negatif kontrol iki biçimde.** Kalıcı olanı `DerivedEvidence`: aynı değerleri
taşıyan `#[derive(Debug)]`'lı kasıtlı bozuk ikiz, 10 yasak desenin **hepsini**
sızdırdığı sürekli doğrulanıyor — sızdırmazsa test kırılır, yani "yasak desen
yok" iddiası boşta dönemez. Tek seferlik mekanik doğrulama da yapıldı:
`MediaEvidence`'ın elle yazılmış `Debug`'ına kasıtlı sızıntı sokulunca guard
2/7 testte kırmızıya döndü, dosya geri alındı.

**Üç tasarım kararı kapanışta kayda geçti.** (1) Boşluk doldurma yalnız
sayısal alanlarda — kazanan katman başlığı ve türü sahiplenir, alt katmanlar
yalnız eksik yıl/sezon/bölüm verir. (2) Bölüm **veya sezon** türü kesinleştirir;
sezonlu bir `Movie` döndürmek tutarsız olurdu. (3) **Kardeş mutabakatı
daraltıldı:** task açılışında "yanlış `SxxEyy`'leri eler" yazıyordu, bu
ADR-0009'un "düşürmez" ilkesiyle çelişiyordu — artık yalnız verdict döndürüyor
ve eksik bir dizi adını dolduruyor. Tam kanıt: `tasks/done/NEN-018-*.md`.

**Eski `NEN-018` açılışı ve `ADR-0009`.** Medya kimliği **katmanlı bir
kanıt modeliyle** çözülecek: beyan katmanları (handoff metadata · `.nfo`
sidecar · container metadata · dosya adı beyanı) tahmin katmanlarının
(üst klasör adları · kardeş dosya teyidi · URL path segmentleri) üstünde;
ilk `Unknown` olmayan katman kazanır. Gerekçe kullanıcı kararında: gömülü
altyazı yoksa altyazılar OpenSubtitles'tan gelecek ve bu ancak kimlik
çözülürse mümkün — **kullanıcıya aday listesi göstermek son çaredir**, hedef
medyaların büyük çoğunluğunda hiç sormamak.

ADR-0009 şartname §6'nın kanıt listesini **iki yönde genişletti**: (1) uzak
medyada URL'in yalnız basename'i değil **tüm path segmentleri** ipucu sayılıyor
— Stremio/debrid URL'lerinde basename çoğu zaman anlamsız (`stream.mkv`, hash
adı), anlamlı ad üst segmentte; (2) sunucunun beyan ettiği ad
(`Content-Disposition`, yönlendirme zincirinin sonu) ayrı bir kanıt katmanı
oldu. Query, fragment ve host **hiçbir koşulda** kimliğe girmiyor (§6 yasağı +
K23 #1/#2 aynen korundu). `docs/product-spec.md` §6'ya ADR'ye işaret eden bir
not düşüldü; şartname yeniden yazılmadı (ADR-0001'in izin verdiği biçim).

Kapsam `size: M` → **`L`** oldu (offline kanıt katmanları eklendi). Dört takip
task'ı açıldı: **NEN-033** (OSDb hash → IMDb ID, M6) · **NEN-034** (AI ile
release-name normalizasyonu, M6 — kendi gizlilik ADR'sini ister) ·
**NEN-035** (güven skoru ve aday sıralama, M6 — gösterim oranı ölçülebilir
kabul kriteri) · **NEN-036** (uzak kanıt portu: HEAD/`Content-Disposition`/
bounded redirect/`Range`, M3).

**Torrent/debrid non-goal olarak kaldı** — kullanıcı kararı, doküman
değişikliği yapılmadı. ADR-0009 Notlar bölümü bunun kimlik çıkarımını neden
zayıflatmadığını kaydediyor: Stremio senaryosunda URL infohash tabanlı ve opak
olsa bile kimlik handoff extras'tan ve `Range` ile okunan container başlığı +
hash'ten gelebilir.

**Eski `NEN-017` kapanışı — M2'nin cue lookup çıkış kriteri karşılandı.**
`nen-subtitle`'a `CueIndex` eklendi (`index.rs`): ödünç alınmış bir
`SubtitleDocument` üzerinde `active_cues(at)` · `active_cue(at)` ·
`cues_in(range)`. **Yeni bağımlılık yok.**

**API çakışan cue'ları düşürmüyor.** NEN-013 çakışmayı kabul ettiği için bir
`t` anında birden fazla cue aktif olabilir; birincil sorgu `active_cues`
hepsini doküman sırasında döndürüyor. Tekil `active_cue` kolaylık
sarmalayıcısı olarak kaldı ama artık *ilk* cue'yu döndürüp diğerlerini attığı
doküman yorumunda açık — NEN-027 istiflenmiş konuşmacıyı sessizce
kaybetmesin diye.

**Yapı: sıralı dizi + `max_end_prefix` artırımı** (iki `partition_point`).
Reddedilen alternatif boundary-event segment dizisiydi: sorgusu en kötü
durumda da `O(log n + k)` olurdu ama tamamı çakışan `n` cue'da `O(n²)` bellek
tüketirdi — bu crate güvenilmeyen girdi ayrıştırdığı için (10 MiB sınırı
~100k cue'ya izin veriyor) belleğin tükenmesi, tek bir sorgunun yavaşlamasından
kötü bir başarısızlık biçimi. Seçilen yapı girdi ne olursa olsun `O(n)`
bellekte. Bedeli kayıtta: aday penceresi çakışma derinliğiyle büyür; dokümanı
baştan sona kaplayan tek cue şekli parity korpusunda var, doğruluk bozulmuyor.

**Baseline** (Apple M5 · release · 3 koşunun aralığı): 50k cue'da index
lookup p50 **48–57 ns**, lineer tarama p50 **15.6–17.6 µs** → **~290–344×**.
Ölçüm `#[ignore]`'lu (`scripts/bench-cue-lookup.sh`), CI yavaşlamıyor.
**Ölçüm yönteminin kendisi bir kusur buldu:** ilk sürümde hangi tablo satırı
önce koşarsa ~2.5× yavaş çıkıyordu (sıra ters çevrilince sapma da tersine
döndü) — sebep lookup değil, taze 50k cue'luk dokümanın ilk dokunuş
page-fault'larıydı; her satır iki kez ölçülüp yalnız ikincisi raporlanarak
düzeltildi.

**Negatif kontrol:** parity testinin boşta dönmediği, `index.rs`'e iki yönde
(fazla cue döndüren / eksik cue döndüren) kasıtlı hata sokularak kanıtlandı —
ikisinde de 3/3 parity testi kırmızıya döndü, sonra dosya geri alındı.

Test sayısı 108 → **121** (+10 index unit, +3 `lookup_parity`; benchmark
`ignored`). **ADR yazılmadı** — yapı `nen-subtitle` içinde kalıyor, crate
sınırı/bağımlılık grafiği değişmiyor, product-spec §14 zaten lineer taramayı
yasaklıyor; karar task'ın kanıt kaydında. `adr:` alanı `[7]` → **`[]`**
düzeltildi (NEN-013/014/015 precedent'i). Tam kanıt:
`tasks/done/NEN-017-*.md`.

**Eski `NEN-016` kapanışı — `ADR-0007` accepted oldu.** `nen-subtitle`'a
(`nen-domain`'e değil — `docs/architecture.md`'nin crate tablosu "timeline"ı
zaten `nen-subtitle`'a veriyor ve bu, `nen-domain`'in her kapanışta
doğrulanan sıfır-bağımlılık özelliğini korur) bir `fingerprint` modülü
eklendi: `TimelineFingerprint::of` yalnız cue zamanlarını (`blake3` ile, `u32
LE` cue sayısı + sıralı `start_ms`/`end_ms`), `SourceFingerprint::of` aynısını
+ cue başına satır sayısı + uzunluk-önekli satır byte'larını hash'liyor.
`CueId` hiçbir hash'e dahil değil — sıra zaten doküman-sırası iterasyonuyla
kodlanıyor. `ADR-0007` ayrıca `subtitle.rs`'nin NEN-013'ten beri açık bıraktığı
"stable, cross-source cue identity" sorusunu kapattı: yeni bir kimlik tipi
**yok**, kaynaklar arası eşleştirme doküman-düzeyi fingerprint üzerinden
yapılacak (M5/M7). `blake3`'ün lisansı (`CC0-1.0 OR Apache-2.0`) `deny.toml`
değişikliği gerektirmedi — `Apache-2.0` kolu zaten izinli listedeydi. 8 yeni
unit test (DoD'un 4 maddesi + `CueId` etkisizliği + satır bölünmesi
ayrışması + boş doküman paniksizliği), test sayısı 100 → **108**. Tam kanıt:
`tasks/done/NEN-016-*.md`.

**Eski `NEN-015` kapanışı — encoding detection and sanitization.** `nen-subtitle`'a `srt::parse`'ın önünde çalışan bir
encoding/sanitization katmanı (`encoding::decode`) eklendi: BOM sniff (UTF-8 →
UTF-16LE → UTF-16BE), BOM yoksa önce sıkı UTF-8 denemesi, başarısız olursa
**Windows-1254**'e (tek legacy fallback) düşüş; ardından kontrol karakteri
(`\n`/`\r` hariç), bidi override ("Trojan Source" sınıfı) ve zero-width
karakter temizliği. 10 MiB boyut sınırı decode denenmeden önce uygulanıyor.
Workspace'in ilk gerçek dış bağımlılığı **`encoding_rs`** (WHATWG Encoding
Standard implementasyonu, Firefox/Servo) eklendi — lisansı
`(Apache-2.0 OR MIT) AND BSD-3-Clause` çıktığı için `core/deny.toml`'a
`BSD-3-Clause` eklendi, `cargo deny check` yeşil. Fixture korpusu
(`fixtures/subtitles/encodings/`) 8 byte-precise dosya: 7 pozitif (BOM'lu
UTF-8/UTF-16LE/UTF-16BE, BOM'suz CP1254 Türkçe metin, BOM'suz CP1252 Batı
Avrupa metni, bidi-override enjeksiyonu, zero-width enjeksiyonu) + 1 negatif
(UTF-16LE BOM + eşleşmeyen surrogate). Test sayısı 82 → **100**.

**`ADR-0008` kabul edildi** (`Karar 1`: `encoding_rs`; `Karar 2`: BOM'suz
durumda CP1252 ile otomatik ayrım **yapılmıyor** — yalnız Windows-1254,
CP1254/CP1252 ayrımı BOM'suz genel durumda çözülemez ve dil tahmini
NEN-020'nin kapsamı; `Karar 3`: bidi/zero-width karakterler escape değil
**silinir**). `docs/adr/README.md` "Yazılmış" tablosuna taşındı.

**Eski `NEN-014` kapanışı — WebVTT writer.** `nen-subtitle`'a bir WebVTT writer (`webvtt::write`)
eklendi: `SubtitleDocument` → `WEBVTT` başlığı, cue başına identifier
(`CueId`) + `HH:MM:SS.mmm --> HH:MM:SS.mmm` zaman satırı + `&`/`<`/`>` kaçışlı
metin satırları, BOM hiç yazılmıyor. Fixture korpusu 7 → **8** geçerli dosyaya
çıktı (`html-special-chars.srt`, kaçış kapsamını kanıtlamak için eklendi);
8 dosyanın hepsi için SRT → doc → WebVTT round-trip `.vtt` golden'ı commit
edildi. Test sayısı 72 → **82**. `adr:` alanı `[7]` → **`[]`** düzeltildi —
NEN-013'teki aynı gerekçe: ADR-0007'nin konusu NEN-016'nın kararı. Tam kanıt:
`tasks/done/NEN-014-*.md`.

**Eski `NEN-013` kapanışı — M2'nin ilk ürün kodu.** `nen-domain`'e subtitle
değer tipleri (`CueId` · `TimeSpan` · `Cue` · `SubtitleDocument`),
`nen-subtitle`'a strict SRT parser'ı ve **17 varyantlı** `SrtError` eklendi.
Fixture korpusu: 7 geçerli dosya + 7 `.golden` snapshot, **25 malformed**
dosya (her biri tek bozukluk), ve 17 varyantın hepsi en az bir fixture'la
kapsanıyor. Test sayısı 42 → **72**; `nen-domain` sıfır bağımlılıklı kaldı,
`nen-subtitle` yalnız `nen-domain`'e bağlı (ADR-0006 grafiği). Tam kanıt:
`tasks/done/NEN-013-*.md`.

**Üç tasarım kararı** kapanışta kayda geçti: (1) **çakışan cue'lar kabul
ediliyor** — farklı konuşmacı SRT'lerinde meşru, reddetmek NEN-025'te
kullanıcının geçerli dosyasını kırardı; sıra yine zorlanıyor
(`NonMonotonicCue`, eşit başlangıç serbest). (2) Index dizisi **tam** 1,2,3,…
olmak zorunda — atlama/tekrar/sıra bozukluğu tek varyantla ifade ediliyor.
(3) BOM **reddediliyor, atlanmıyor**; encoding NEN-015'in işi ve parser'ın
girdisi `&str`, yani kapsam sınırı tipte duruyor.

**Fuzz `cargo-fuzz` ile değil, deterministik smoke ile yapıldı** — nightly
toolchain gerektirirdi ve `doctor.sh`'a yeni bir gereksinim eklerdi. Sabit
tohumlu (`0x4E45_4E30_3133`) xorshift64* üreteci **16 281 vaka** üretiyor
(781 truncation · 10 500 mutation · 5 000 noise), 0.07 s'de koşuyor ve her
`cargo test` ile CI'da otomatik tekrarlanıyor. İddia yalnız "panik yok"
değil: `Ok` dönen her vakada doküman kendi invariant'larından geçiriliyor.

`NEN-013`'ün `adr:` alanı `[7]` → **`[]`** olarak düzeltildi — NEN-006/007/
008/009/010/011 ile aynı precedent: ADR-0007'nin konusu (cue kimliği ve
timeline fingerprint algoritması) **NEN-016**'nın kararı; NEN-013 yalnız
ADR-0006'nın zaten çizdiği crate sınırlarını dolduruyor.

**Eski yan bulgunun bugünkü durumu:** `AGENTS.md` artık izlenen giriş noktası ve
kanonik `CLAUDE.md`'ye yönlendiriyor. Yalnız `.agents/` untracked; task
commit'lerine dahil edilmiyor.

**Eski `NEN-012` kapanışı — M1 kilitlendi.** Beş spike'ın (NEN-008/009/010/011/029)
ölçümleri `ADR-0002`'de sentezlendi: **Rust shared core dili olarak kabul
edildi** (go), I1–I5 invariant'larının hepsi kanıtlı, M1'in üç no-go
koşulundan hiçbiri tetiklenmedi. Reddedilen alternatifler (Kotlin
Multiplatform · Swift core + ayrı Android · C++ core) ayrı spike edilmedi —
zaten kanıtlanmış bir adaydan geçmenin ölçülmüş gerekçesi yoktu. Aynı task
kapsamında `ADR-0027` (dört FFI performans bütçesi, ölçülen p50/p95'in
üzerine 4–11× marj) de `accepted` oldu. `docs/architecture.md`,
`docs/DECISIONS.md` ve `docs/roadmap.md` güncellendi (Rust artık "aday"
değil; M1 → kapandı, M2 → sıradaki). Tam kanıt: `tasks/done/NEN-012-*.md`.

**Yan bulgu (ayrı arka plan görevine yönlendirildi, NEN-012 kapsamı değil):**
`docs/DECISIONS.md`'nin "Ertelenmiş kararlar" tablosu `ADR-0026`'yı hâlâ
bekleyen olarak listeliyor — `NEN-029` kapanışında güncellenmemiş bir kusur.

**Eski `NEN-011` kapanışı.** NEN-008/009/010'un ölçtüğü üç Rust crate'ine
dokunulmadan, her birine Swift'teki `apple-harness/`'in eşi bir
`jvm-harness/` (Gradle wrapper tabanlı Kotlin/JVM projesi) eklendi — binding
üretimi aynı `spike-*-uniffi-bindgen` binary'lerinden, yalnız
`--language kotlin` ile (`scripts/spike-cues-jvm.sh`,
`spike-async-jvm.sh`, `spike-typed-errors-jvm.sh`).

**Ön koşul: B2 kapandı.** Bu makinede gerçek bir JDK yoktu (yalnız CLT'nin
çalışmayan `/usr/bin/java` stub'ı). `brew install --cask temurin` sudo şifresi
istediği için bu ortamda başarısız oldu; `brew install openjdk` (formula,
sudo gerektirmez) kullanıldı, `/opt/homebrew/opt/openjdk/bin` `~/.zshrc`'ye
PATH eklendi. `bash scripts/doctor.sh M1` artık JDK'yi ✓ gösteriyor. Bootstrap
için `brew install gradle` (9.7.1) geçici kuruldu — yalnız üç `jvm-harness/`'ta
bir kere `gradle wrapper` çalıştırıp kendi `gradlew`'lerini üretmek için;
bundan sonra hiçbir geliştiricinin sistem Gradle'ı kurmasına gerek yok.

**Kotlin'e özgü iki isimlendirme sapması kaynak incelemesiyle doğrulandı**
(`uniffi_bindgen` 0.32 `bindings/kotlin/gen_kotlin/mod.rs`): (1) adı "Error"
ile biten bir `uniffi::Error` tipi Kotlin'de otomatik "Exception" ile
değiştiriliyor — Rust/Swift `AppError`, Kotlin'de **`AppException`**; (2) düz
`uniffi::Enum` varyantları Kotlin'de **SCREAMING_SNAKE_CASE** (Swift'te
camelCase'ti) — Swift'in zaten bulduğu "hata PascalCase / enum camelCase"
asimetrisine üçüncü bir kural ekleniyor.

**Cue-transfer checksum'ları Swift'le birebir aynı** (`75001045577800` /
`34337591381145`, 50 000 cue, NEN-008 ile aynı fixture) — I5'in ("semantik
sonuçlar Swift ve Kotlin arasında aynı") doğrudan kanıtı. Coroutine iptali
gerçek Rust `JobHandle.cancel()`'ı tetikliyor (Rust tarafı `async fn` değil,
`JobHandleCoroutines.kt`'nin `suspendCancellableCoroutine` +
`invokeOnCancellation` sarmalayıcısı üzerinden) — I1/I2/I4 invariant'ları
Kotlin/JVM tarafında da (3/3 test) deterministik kanıtlandı. En büyük ölçüm
sapması: `checkpoint_every=1`'de dispatch-penceresi kaçağı Swift'te 0/800,
Kotlin/JVM'de 193/800 (I1'in kendisini etkilemiyor — yalnız "kullanıcı
iptale karar verdi" ile "cancel() fiilen çağrıldı" arasındaki gevşek
pencere, JVM thread scheduling + GC nedeniyle daha geniş). Typed-error eşleme
maliyeti Kotlin/JVM'de Swift'in p50'de ~5.5×, p95'te ~23×'ü — JIT ısınması +
GC, bug değil. Tüm sapmalar ve M10 etkileri task'ın kanıt kaydında "Metodolojik
sapmalar" tablosunda.

**Yan bulgu (ayrı backlog task'ına yönlendirildi, NEN-011 kapsamı değil):**
`scripts/doctor.sh`'ın `run_timeout` helper'ı `java -version`'ın (stderr'e
yazan) çıktısını kendi içindeki `2>/dev/null` ile siliyor — `doctor.sh`
raporunda "✓ JDK" satırının sürüm detayı hep boş kalıyor. Kozmetik (M1
gate'inin exit kodunu etkilemiyor); önceki hiçbir makinede gerçek bir JDK
çalışmadığı için şimdiye kadar ortaya çıkmamıştı.

`NEN-011`'in `adr:` alanı `[3]` → **`[28]`** olarak düzeltildi — NEN-006/
007/008/009/010 ile aynı gerekçe: ADR-0003 henüz `accepted` değil ve bu task
onu kararlaştırmıyor (sonuçları NEN-012 üzerinden ADR-0003'e girdi olacak);
gerçek dayanak spike-local FFI kapısını açan ADR-0028.

**Eski `NEN-005` kapanışı.** `.github/workflows/ci.yml` — macOS runner'da her
push/PR'da `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test
--workspace`, `cargo deny check` (yeni `core/deny.toml`), `scripts/test.sh`,
`task-index.sh --check`, `check-docs.sh` koşuyor; cargo registry+target
cache'leniyor, aynı branch'te eski koşu iptal ediliyor. Bu task için
`github.com/ynsemrekryl2/nen-player` adında private bir repo kuruldu (bu
repoda daha önce remote yoktu).

İlk gerçek koşu, yerelde tekrarlanamayan **3 gerçek kusur** buldu — üçü de bu
task kapsamında düzeltildi: (1) `rust-toolchain.toml`'ın 1.98.0 pin'i yalnız
cwd `core/` altındayken tetikleniyor, `--manifest-path` yetmiyor — fmt/clippy/
test adımları `working-directory: core` ile düzeltildi; (2)
`EmbarkStudios/cargo-deny-action` bir Docker container action, macOS
runner'da çalışmıyor — `taiki-e/install-action` + düz `cargo deny check`'e
geçildi; (3) **`scripts/doctor.sh`'ın `detect_jdk`'sında gerçek bir doğruluk
hatası bulundu**: diğer tüm `detect_*`'lerin aksine `command -v java`
guard'ı yoktu, `run_timeout`'un dayandığı perl `exec LIST` PATH'te java hiç
yokken sessizce exit 0 dönüyordu — yani "JDK yok" "JDK var" raporlanıyordu.
Yerel Mac'lerde bu gizli kaldı (CLT'li Mac'te JDK kurulu olmasa da
`/usr/bin/java` çalıştırılınca gerçekten hata veren bir stub'tır); GitHub'ın
macOS runner image'ında `/usr/bin/java` gerçek ve çalışan bir JDK olduğundan
kusur ilk kez CI'da ortaya çıktı. Düzeltildi; `doctor.test.sh`'ın S4 senaryosu
S7'nin (swift) kullandığı shadow-PATH tekniğiyle güncellendi.

DoD'un 4 kasıtlı ihlal kanıtı zincirleme commit'lerle toplandı (pipeline
sıralı olduğundan her biri bir öncekini düzeltip bir sonrakini bozuyor):
kasıtlı `cargo fmt` ihlali → 🔴 24s, clippy ihlali (`bool_comparison`) → 🔴
35s, bayat `INDEX.md` → 🔴 1m9s (bu koşu `scripts/test.sh`'ın CI'da fiilen
koştuğunu da doğruladı), `doctor.test.sh`'ta kasıtlı bozuk assertion → 🔴
45s, temiz duruma dönüş → 🟢. Yeşil koşu süresi: **3m58s** (soğuk cache, ilk
koşu) → **52s** (sıcak cache, sonraki koşular). Ayrıntı ve tüm run linkleri
task'ın kanıt kaydında.

Ayrıca bu task için `cargo-deny` bu makineye Homebrew ile kuruldu (0.20.2) —
toolchain tablosundaki B2 dışı "soon" eksiği kapandı.

**`NEN-029` kapandı ve `ADR-0026` accepted oldu.**
`core/spikes/spike-reverse-ffi/` — NEN-009'un delivery-gate deseni tek bir
crate içinde iki playback-ownership yönüne genelleştirildi: **A** (core-owned,
`ReverseEngine` fake motoru bir `PlaybackObserver` foreign trait'ini tekrar
tekrar çağırıyor) ve **B** (shell-owned, `ForwardSession`'a Swift kendi
`DispatchSourceTimer`'ıyla düz çağrılar yapıyor, ters çağrı yok). Release'de
60 Hz'de A'nın per-call maliyeti p50 **36.67 µs** + MainActor-hop p50
**~40 µs** (callback her zaman ana thread DIŞINDA düşüyor); B'nin maliyeti
p50 **1.33 µs**, hop hiç yok. Mutlak toplam ikisinde de küçük (A ~4.6 ms/sn,
B ~0.08 ms/sn — 1000 ms/sn'lik kare bütçesinin binde biri mertebesinde), yani
performans tek başına kararı belirlemedi. Asıl ayrım yapısaldı: bir
backgrounding proxy deneyi, A'nın Rust-taraflı üretici thread'inin Swift
tüketicisi hazır olmasa da üretmeye devam ettiğini gösterdi (Swift'in drain
kuyruğu 2 sn askıya alınırken Rust tıklamaya devam etti, sonra birikme
temizlendi) — B'de bu risk yapısal olarak yok. I1 ve I4 her iki yönde de
sağlandı; event ordering, seek/seek-complete sırası, typed error aktarımı
ikisinde de doğrulandı. Reentrancy: callback içinden `seek()` güvenli (ayrı
kilit kullanıyor), callback içinden aynı thread'den `cancel()` ise delivery
gate'in kendi kilidiyle self-deadlock — kod incelemesiyle kanıtlandı, canlı
çalıştırılmadı (gerçek bir deadlock `live_engines()` sayacını binary'nin geri
kalan testleri için kalıcı bozardı).

**`ADR-0026` A'yı önerdi, kullanıcı onayladı: merkezi Rust session (reverse
callback) kalıyor.** Gerekçe: A'nın mutlak maliyeti hiçbir makul UI bütçesini
zorlamıyor, hiçbir invariant ihlal edilmedi; merkezi session'ın asıl
gerekçesi zaten performans değil, subtitle sync/çeviri tetiklemenin tek
kaynaktan yönetilmesiydi — bu spike o gerekçeyi ölçemezdi, yalnız A'nın
uygulanabilir olduğunu doğruladı. `NEN-021`'in gerçek kontratı iki ölçülmüş
riski (backpressure/backgrounding, reentrancy disiplini) açıkça ele almak
zorunda — ADR-0026 → "Karar". `docs/architecture.md`'nin "Ownership yönü —
spike bekliyor" notu bu kararla güncellendi; `NEN-021` artık başlayabilir.

**Bulunan ve düzeltilen bir test kusuru:** ilk yazılan Rust testleri paralel
`cargo test` thread'leri arasında `LIVE_ENGINES`/`LIVE_FORWARD_SESSIONS`
global sayaçlarını paylaşıyordu. NEN-009'un aksine bu spike'ın tick'leri
gerçek wall-clock `sleep` kullanıyor (10-80 ms pencereler, NEN-009'un
mikrosaniye ölçekli busy-loop'larının aksine), bu da paralel test çakışmasını
çok daha olası kılıp 2/11 testi deterministik kırdı. Düzeltme:
`tests` modülüne bir `Mutex<()>` eklenip her test onu ilk satırda kilitledi;
üç ardışık koşuda 11/11 stabil geçti.

Kotlin ertelendi: task NEN-011'e bağımlı değil, bu makinede JDK kurulu değil
(B2). Ayrıntı, tam ölçüm tabloları (release+debug) ve ADR-0028 mekanik
denetim çıktıları task'ın kanıt kaydında.

`NEN-029`'un `adr:` alanı zaten `[26]` doğruydu — bu kez düzeltme gerekmedi.

**Eski `NEN-010` kapanışı.** `core/spikes/spike-typed-errors/` — beş varyantlı bir
`AppError` (UniFFI **rich** error, `flat_error` değil) Swift'e geçirilip
`default:` olmadan exhaustive bir `switch` ile ayrıştırıldığı gösterildi
(I3: "typed error string parse gerektirmiyor"). **Tasarım bulgusu:**
NEN-006'nın elle yazılmış `Debug` garantisi yalnız Rust `{:?}` çıktısını
korur — Swift'in kendi `String(describing:)` basımı Rust `Debug`'ını hiç
görmüyor, dolayısıyla FFI'yı geçen bir hata için koruma "inşa öncesi
sanitize et" ile sağlandı: `Parse` yalnız `nen_domain::redact::extension()`'ı
taşıyor (ham path'i asla), `Network` yalnız `redact_host()`'tan geçmiş
host'u. Bu sayede ham değer FFI teline hiç çıkmıyor; hem Rust `Debug`'ı hem
wire hem Swift'in varsayılan basımı aynı anda güvenli. Negatif kanıt
(NEN-006'daki `BadFixtureWithDerivedDebug` ile aynı teknik): sanitizing
constructor bypass edilip ham path doğrudan alana konursa, elle yazılmış
`Debug` bunu ayırt edemiyor ve gerçekten sızdırıyor — üstteki no-leak
testlerinin boşta dönmediğinin kanıtı.

**Yan bulgu:** bu uniffi sürümünde (0.32.0) `uniffi::Error` varyant adları
Swift'te PascalCase (`.Parse`), sıradan `uniffi::Enum` varyantları ise
camelCase (`.localAsr`) — iki derive makrosu arasında tutarsız isimlendirme;
M2'nin gerçek hata taksonomisi yazılırken hatırlanmalı.

**DoD #3 ("bilinmeyen varyant sessizce yutulmuyor") negatif kanıtı**
gerçek crate'e dokunmadan, tamamen ayrı bir scratch cargo+swift projesinde
aynı `uniffi::Error` mekanizmasıyla mekanik kanıtlandı: 3 varyantlı bir
enum + exhaustive switch derleniyor; switch'e dokunulmadan 4. varyant
eklenince `swift build` **"switch must be exhaustive"** ile kırılıyor.
Yani iddia çalışma zamanı davranışı değil derleme zamanı garantisi olarak
kanıtlandı — `doctor.test.sh`'ın shadow-PATH tekniğiyle aynı ruh (geçici
durum, mekanik kanıt, iz bırakmadan temizlik).

Baseline (M1-core-spike.md'nin istediği, eşik değil): 5 varyant; throw→catch→
switch eşleme maliyeti p50 **2.04 µs**, p95 **2.17 µs** (Apple M5 · macOS
27.0 · release, 10 000 tekrar).

`NEN-010`'un `adr:` alanı `[5]` → **`[28]`** olarak düzeltildi — NEN-006/
008/009'la aynı gerekçe: ADR-0005 dosyası yok, task onu kararlaştırmıyor;
gerçek dayanak spike-local FFI kapısını açan ADR-0028.

**`NEN-006` kapandı.** `nen-domain` (bağımlılıksız değer crate'i) içine bir
`redact` modülü eklendi: `Redacted<T>` (Debug/Display her koşulda
`<redacted>` basar), `extension()`, `size_class()`, `redact_host()` — hepsi
`docs/security-policy.md` §1'in "Loglanabilecekler" listesiyle sınırlı.
Guard test (`tests/guard_redaction.rs`), K23'ün 8 yasaklı deseninin
(medya URL, token query, tam özel yol, cue metni, raw provider response,
API key, OpenSubtitles private file ID, özel hash/filename) elle yazılmış
`Debug` kullanan örnek tiplerin `{:?}` çıktısında **hiç** görünmediğini
kanıtlıyor. **Negatif kanıt** (güvenlik task'ı için zorunlu):
`#[derive(Debug)]` ile yazılmış kasıtlı bozuk bir fixture aynı deseni
gerçekten sızdırıyor — yani kontrolün boşta dönmediği, gerçek bir
`derive(Debug)` hatasını yakalayacağı ayrıca kanıtlandı (NEN-032'nin
shadow-PATH kanıtıyla aynı biçim). Typed error tarafı: örnek `ErrorFixture`
enum'ının üç varyantı payload'a hiç dokunmadan, yalnız discriminant
üzerinden ayrıştırılabiliyor. `nen-domain`'in sıfır bağımlılık özelliği
korundu (`cargo tree` tek düğüm); `cargo clippy -D warnings` ve
`cargo fmt --check` (yalnız `nen-domain` kapsamında) temiz. Ayrıntı ve
tam test çıktısı task'ın kanıt kaydında.

`NEN-006`'nın `adr:` alanı `[5]` → **`[]`** olarak düzeltildi — NEN-007/008/009
ile aynı gerekçe: ADR-0005 dosyası `docs/adr/` altında henüz yok, task onu
kararlaştırmıyor, yalnız zaten kabul edilmiş `docs/security-policy.md`'yi
koda döküyor.

`NEN-032` kapandı. `doctor.sh`, `swift`i M3'ten M1'e taşıdı: NEN-007
(`test-apple.sh`) ve NEN-008 (`spike-cues.sh`) M1 içinde zaten Swift'e
bağımlıydı, ama doctor bunu M3'e kadar blocker saymıyordu — Swift'siz bir
makinede `doctor.sh M1` yanlışlıkla çıkış 0 verip hatayı ilk Swift
komutuna erteliyordu. `requirement()`'ta `swift` kendi satırına ayrıldı
(`swift:M1|swift:M3` → blocker), Xcode/libmpv'nin M3 grubu ve tam
Xcode/M1 istisnası dokunulmadan kaldı. Yeni test senaryosu (S7) "swift
yok" durumunu doğruladı — CLT kurulu bir Mac'te `/usr/bin/swift` gerçek
bir binary olduğundan, `command -v swift`'in onu bulamaması için
`/usr/bin`'in geri kalanı swift hariç bir gölge dizine bağlanıp PATH ona
yönlendirildi. `bash scripts/test.sh` ve `bash scripts/check-docs.sh`
yeşil; ayrıntı task'ın kanıt kaydında.

**`NEN-009` kapandı.** FFI sınırından geçen bir işin kooperatif iptali,
`Mutex<Option<Arc<dyn ProgressSink>>>` "delivery gate" tasarımıyla ölçüldü:
worker her checkpoint'te callback'i **kilit altında** çağırıyor, `cancel()`
aynı kilidi alıp sink'i temizliyor — böylece `cancel()` bir callback'in
ortasında dönemiyor ve döndükten sonra hiçbir checkpoint artık sink
bulamıyor. Üç invariant (I1 geç callback yok, I2 geç commit engellendi, I4
kaynak sızıntısı yok) hem release hem debug build'de Swift testleriyle
deterministik kanıtlandı — `core/spikes/spike-async-cancel/`.

Baseline (release · Apple M5 · macOS 27.0 · block_micros=20 µs sabit ·
200 koşu/satır): latency checkpoint aralığına neredeyse birebir bağlı —
checkpoint_every=1 → p50 **33 µs**, checkpoint_every=100 → p50 **2.02 ms**;
kapı maliyeti (kilit + çağrı) aralığın yanında ölçülemeyecek kadar küçük.

**Kanıt sürecinde bulunan bir kusur, sürecin kendisini de düzeltti:** ilk
yazılan Swift testi "iptal isteği" bayrağını `cancel()` çağrılmadan ÖNCE
işaretliyordu; 800 koşuluk sweep'te debug build'de 1 kaçak callback ortaya
çıktı (checkpoint_every=1'de). Bu I1'in ihlali değildi — "kullanıcı iptale
karar verdi" ile "Swift'in `cancel()`'ı fiilen çağırması" arasındaki dispatch
penceresinde meşru bir kaçaktı; testin ölçtüğü sınır I1'in gerçek sınırıyla
(cancel() **döndükten** sonra) örtüşmüyordu. Düzeltme: bayrak artık
`cancel()` döndükten SONRA işaretleniyor — kilit tasarımı gereği yapısal
olarak imkânsız bir kaçağı test ediyor, artık deterministik. Ayrıntı ve
release/debug tam tabloları task'ın kanıt kaydında.

`NEN-009`'un `adr:` alanı `[4]` → **`[28]`** olarak düzeltildi — NEN-007/008
ile aynı gerekçe: ADR-0004 (async/cancellation kontratı) ölçümler
(NEN-009+NEN-011+NEN-029) bitmeden `accepted` olamaz; task'ın gerçekte
dayandığı karar spike-local FFI kapısını açan ADR-0028.

**`NEN-008` kapandı — M1'in ilk sayıları var.** 50 000 cue'luk bir doküman
(3.1 MiB) Swift'e iki yoldan geçirilip ölçüldü (release · Apple M5 · macOS 27.0):

| | tam liste | pencere/handle |
|---|---|---|
| p50 | 37.7 ms | **44.3 µs** (40 cue'luk pencere) |
| en uzun main-thread bloğu | **48.4 ms** (~3 kare @60fps) | **0.08 ms** |
| peak RSS | 19.5 MiB | 11.7 MiB |

**Öneri: pencereli erişim** — ama iki kayıtla: (1) pencere cue **başına** %47
daha pahalı, kazancı hızdan değil ödemediği cue'lardan geliyor; (2) dokümanın
*tamamı* gerçekten gerekiyorsa tek çağrı %35 daha ucuz (37.7 ms'e karşı
50.7 ms), yani "her şey pencereli olsun" kuralı yanlış olur. `activeCue`
**1.50 µs** — playback sırasında cue aramanın FFI maliyeti pratikte yok; bu
sayı NEN-029'a ve M7'nin position çözünürlüğüne girdi. Ayrıntı, debug/release
karşılaştırması ve doğrulama çıktıları task'ın kanıt kaydında.
**Bunlar baseline'dır, eşik değil** — bütçe ADR-0027 ile kabul edilecek.

Ön koşul olarak **ADR-0028 kabul edildi**: ADR-0006 kural 2 ("`nen-ffi` tek dış
kapıdır"), kural 3 ve CLAUDE.md kural 7 birlikte M1'in FFI ölçümlerine yer
bırakmıyordu. ADR-0028 kural 2'nin kapsamını **ürün koduyla** sınırlıyor;
`core/spikes/*` altındaki bir spike crate yalnız ölçüm için kendi atılabilir
kapısını açabiliyor. Üç sınır grep + `cargo metadata` ile mekanik doğrulanıyor
ve çıktıları kanıt kaydında. ADR-0006 **düzenlenmedi** — yalnız "Notlar"ına
işaret eklendi (ADR-0001'in izin verdiği istisna).

`NEN-008`'in `adr:` alanı `[3]` → **`[28]`** olarak düzeltildi; NEN-007'de
yapılan düzeltmenin aynısı (ADR-0003 spike ölçümleri olmadan `accepted` olamaz,
üstelik dosyası da yok).

`NEN-031` kapandı: `scripts/tests/check-docs.test.sh` artık **kendi task
fixture'ını kuruyor** — canlı `tasks/` ve `INDEX.md` okunmuyor. Denetim 8'in her
iki dalı (ready listesi dolu / boş) ayrı ayrı doğrulanıyor; boş-ready dalı bugüne
kadar hiç test edilmemişti. Yan etki: koşu 16.4 s → 1.6 s (eski fixture tüm
repo'yu, `core/target` dahil 1.2 GB, tarlıyordu). Bu, **NEN-005'in (CI) ön
koşuluydu**.

Ondan önce `NEN-007` ile repository kod içermeye başlamıştı: Cargo workspace,
ADR-0006'nın tarif ettiği 11 crate ve `nen-ffi` üzerinden Swift'e geçen bir
`version()` fonksiyonu ayakta. Milestone sırası önerisi (`M1-core-spike.md`)
**NEN-009 ve NEN-010** ile devam ediyor — ikisi de aynı spike zeminini
kullanacak.

**UniFFI hâlâ aday.** NEN-007 binding teknolojisini seçmedi; ADR-0003
NEN-011/NEN-012'de karara bağlanacak. Kabul edilen mimari karar **ADR-0006** —
monorepo yapısı, crate sınırları ve `nen-ffi`'ın tek dış kapı olması.

NEN-007'nin `adr:` alanı `[3, 6]` → **`[6]`** olarak düzeltildi: ADR-0003 spike
ölçümleri olmadan `accepted` olamaz, yani task'ın kapanışını kilitliyordu.
Aynı kusur NEN-008'de bu kapanışta giderildi; geriye **`NEN-011`** kaldı.

## Toolchain

`bash scripts/doctor.sh M3` (2026-08-25, bu makine — macOS 27.0 26A5416b):

| Araç | Durum | M1'deki seviyesi |
|---|---|---|
| `cargo` / `rustc` | ✅ 1.98.0 (2026-08-18) | blocker — **karşılandı** |
| `cargo-deny` | ✅ 0.20.2 (Homebrew, NEN-005) | soon — **karşılandı** |
| JDK | ✅ OpenJDK 26.0.2.1 (Homebrew `openjdk`, NEN-011) | soon — **karşılandı** |
| `swift` | ✅ Apple Swift 6.3.3 (Xcode bundled) | M1 (blocker) — NEN-007 testi ve NEN-008 harness'ı kullanıyor |
| Tam Xcode | ✅ Xcode 26.6 (build 17F113, `/Applications/Xcode.app`) | M3 (blocker) — **karşılandı** |
| libmpv | ✅ 2.5.0 (Homebrew mpv 0.41.0_8, pkg-config) | M3 (blocker) — **karşılandı** |
| Gradle | ✅ 9.7.1 (Homebrew, yalnız wrapper bootstrap için — NEN-011) | hiçbir milestone'da blocker değil (wrapper) |
| Android SDK | ❌ eksik | M10 |

Rust `rustup` ile kuruldu (2026-08-24). Workspace `core/rust-toolchain.toml` ile
**1.98.0'a pinli** — M1 ölçümlerinin başka makinede karşılaştırılabilmesi için.

JDK, `brew install --cask temurin` sudo istediği için `brew install openjdk`
(formula) ile kuruldu; `/opt/homebrew/opt/openjdk/bin` `~/.zshrc`'ye PATH
eklendi (Homebrew'ün kendi "keg-only" uyarısının önerdiği sudo'suz yol).
Gradle yalnız üç `jvm-harness/`'ın kendi `gradlew` wrapper'ını üretmek için
geçici bootstrap amacıyla kuruldu — NEN-011 kapandıktan sonra hiçbir
geliştiricinin sistem Gradle'ı kurmasına gerek yok.

`doctor.sh M1` → **çıkış 0** · `doctor.sh` (parametresiz) →
**çıkış 0** (bilgilendirici). Kurulum komutları çıktıda; script **hiçbir şey
kurmaz** — bu, `scripts/tests/doctor.test.sh` S7 ile mekanik olarak kanıtlanıyor.

## Blocker'lar

| # | Blocker | Kimi durduruyor | Çözüm |
|---|---|---|---|
| ~~B1~~ | ~~Rust kurulu değil~~ | — | ✅ **çözüldü** 2026-08-24 — rustup, 1.98.0 |
| ~~B2~~ | ~~JDK yok~~ | — | ✅ **çözüldü** 2026-08-24 — `brew install openjdk`, NEN-011 |
| ~~B3~~ | ~~Tam Xcode + libmpv yok~~ | — | ✅ **çözüldü** 2026-08-25 — ikisi de kuruluydu; eksik olan Xcode lisans kabulüydü (`sudo xcodebuild -license accept`) |
| ~~B4~~ | ~~`check-docs.test.sh` fixture'ı canlı repo durumuna bağlı~~ | — | ✅ **çözüldü** 2026-08-24 — `NEN-031` |

**Gerçek blocker kalmadı.** M1 ve M3 kapıları açık (`doctor.sh M1` ve
`doctor.sh M3` → exit 0).

## Kullanıcı kararı bekleyenler

| # | Konu | Ne zaman gerekiyor |
|---|---|---|
| **S3** | Çeviri kalite hedefinin operasyonel ölçütü | M5 |
| **S4** | Offline/uçak modu birinci sınıf mı? S8'in "cloud sync non-goal" cevabı bunu doğrudan etkiliyor — sync yoksa offline davranış tamamen yerel cache'e bağlı | M5–M6 |
| **S6** | Local ASR modeli ve cihaz kaynak bütçesi *(privacy kısmı cevaplandı)* | M8 |
| **S7** | Android TV minimum API seviyesi ve hedef cihaz sınıfı | M10 |
| **S9** | Birden fazla AI artifact'in UI'da gösterimi | M5 |
| **S11** | İleride public dağıtım | M3 sonrası |
| **S12** | Gerçek lisans seçimi | ADR-0012 sonrası |

Hiçbiri sıradaki task'ları bloke etmiyor.

## Son doğrulama

2026-09-11'de M6 task kırılımı üretildi (doküman işi, kod değişmedi):
17 yeni task dosyası, `docs/milestones/M6-real-providers.md` kırılımı,
`NEN-034`/`NEN-038`/`NEN-064` bağımlılık düzeltmeleri, ADR README'ye 0046/0047
satırları, roadmap S3/S9 notları. `bash scripts/task-index.sh` ve `bash
scripts/check-docs.sh` çıkış 0 (döngü yok, 126 task).

2026-09-11'de `NEN-044` kapandı — gömülü bir metin altyazı track'inin tam
metni artık `libavformat`/`libavcodec` üzerinden (ADR-0045) çıkarılabiliyor;
`NEN-102`'nin kaydettiği canlı kusur (gömülü İngilizce track'in çeviri
komutunda kalıcı `NoDocument` reddi) kapandı. `cargo test --workspace`
**846 passed / 1 ignored** (`NEN-108` baseline 832 + 14). `bash
scripts/test-macos.sh` **264 test / 33 suite** (baseline 256/31 + 8), iki
ayrı koşuda tekrarlandı, ikisi de yeşil. `cargo fmt --check`, `cargo clippy
--workspace --all-targets -- -D warnings`, `cargo deny check` (yeni
bağımlılık yok), `bash scripts/test.sh` **4/4**, `bash
scripts/check-docs.sh` hepsi yeşil. Golden fixture (`contract-clip.mkv`'nin
gömülü İngilizce/Türkçe track'leri) ve zorunlu bitmap negatifi
(`bitmap-subs-clip.mkv`) gerçek libmpv + gerçek libavformat'ta ölçüldü; iki
mutasyon elle uygulanıp geri alındı, ikisinde de tam olarak beklenen
test(ler) kırmızıya döndü. Ayrıntılı kanıt kaydı: `tasks/done/NEN-044-*.md`.

2026-09-11'de `NEN-102` kapandı — koşan bir çeviri işinin ilerlemesi ekranda
görünüyor, kullanıcı `İptal` ile durdurabiliyor, iptal edilen iş ekranda ve
menüde hiçbir iz bırakmıyor. Yol üstünde `nen-ffi`'de gerçek bir kusur bulundu
ve düzeltildi: `FfiTranslationJob::cancel()` `join()` çağrıldığı an kalıcı
olarak etkisiz kalıyordu (yeni `TranslationCancelHandle`, `nen-app::translation`).
`bash scripts/test-macos.sh` **256 passed / 31 suites** (iki ayrı koşuda
tekrarlandı). `cargo test --workspace` **832 passed / 1 ignored** (`NEN-100`
baseline 831 + bu task'ın regresyon testi). `cargo fmt --check`, `cargo clippy
--workspace --all-targets -- -D warnings`, `cargo deny check` (yeni bağımlılık
yok), `bash scripts/test.sh` **4/4**, `bash scripts/check-docs.sh` hepsi
yeşil. Beş kapı elle mutasyona uğratılıp her birinde tam olarak beklenen
test(ler) kırmızıya döndüğü ölçüldü. Ayrıntılı kanıt kaydı:
`tasks/done/NEN-102-*.md`.

2026-09-09'da `NEN-094` kapandı — `ValidatedSubtitleArtifact` ve WebVTT
çıktısı `core/crates/nen-translate/src/artifact.rs`'e eklendi. `cargo test -p
nen-translate` **62 passed** (lib 40 + `artifact_golden` 1 + `artifact_negative`
7 + `block_layout_golden` 1 + `checkpoint_cancellation_negative` 4 +
`guard_artifact_debug` 2 + `guard_context_debug` 3 + `validation_negative` 4);
`cargo test --workspace` **738 passed / 1 ignored**, 0 failed. `cargo fmt
--check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny
check` (yeni bağımlılık yok), `bash scripts/test.sh`, `bash
scripts/check-docs.sh` hepsi yeşil. `assemble()`'ın yedi doğrulama kapısının
her biri elle kaldırılıp ilgili negatif testin kırmızıya döndüğü ölçüldü;
planlanan sekizinci bir kapı (`BlockOrder`) hiçbir girdiyle tetiklenemeyeceği
kanıtlanınca koddan çıkarıldı. Ayrıntılı kanıt kaydı: `tasks/done/NEN-094-*.md`.

2026-09-08'de `NEN-088` kapandı — gerçek Stremio 5.1.26 soğuk/sıcak kabulü,
transport ekran kanıtları ve negatif log taraması tamamlandı. `bash
scripts/build-macos-app.sh` çıkış 0; `bash scripts/doctor.sh M3` çıkış 0;
köprü `installed`; geçici güvenlik sayaçlarının tamamı 0. Doküman kanıtı:
`evidence/M4/NEN-088-checklist.md`.

Bu kapanışta Rust/macOS ürün kodu değişmedi; task'a özel ürün testi eklenmedi.

2026-09-07'de `NEN-071` kapanışı için gerçek `.app` fixture koşusu ve
`PlayerModelTests.subtitlePanelResolvesAliasesExplicitly` geçti; panel
`resolvesAliases = true` ile yapılandırıldı. `bash scripts/test-macos.sh`
**198 test / 22 suite**, `bash scripts/test.sh` ve `bash scripts/check-docs.sh`
yeşil; `.app` build'i de üretildi. Doğrudan symlink ve sidecar-symlink negatif
testleri yeşil kaldı. Ayrıntılı fixture adımları:
`evidence/M3/NEN-071-checklist.md`.

### Toolchain kapısı geçmiş kaydı

2026-08-25, tümü bu makinede çalıştırıldı (Apple M5 · arm64 · macOS 27.0
26A5416b · rustc/cargo 1.98.0 · **Swift 6.3.3 — Xcode 26.6 bundled** ·
uniffi 0.32.0). M3 toolchain kapısı açıldıktan sonra yeniden koşuldu;
`xcode-select` artık CommandLineTools yerine Xcode'u gösterdiği için Swift
toolchain'i 6.4'ten 6.3.3'e değişti, bu yüzden Swift'e dokunan her şey
(harness'lar ve spike'lar dahil) tekrar çalıştırıldı.

```
$ bash scripts/doctor.sh M1
SONUÇ: M1 için tüm blocker'lar hazır.            → exit 0
$ bash scripts/doctor.sh M3
  ✓ cargo 1.98.0 · ✓ rustc 1.98.0 · ✓ Xcode 26.6
  ✓ swift Apple Swift version 6.3.3 · ✓ libmpv 2.5.0
SONUÇ: M3 için tüm blocker'lar hazır.            → exit 0 (B3 kapandı)
  NOT: doctor'ın `~/.cargo/bin`'i PATH'te bulması gerekir; bulunmayan bir
  shell'de cargo/rustc yanlışlıkla eksik raporlanır (kurulum sorunu değil).

$ cargo test --manifest-path core/Cargo.toml --workspace   (NEN-021 ile 337 → 390)
nen-ports 30 (unit) + 6 (contract_fake) + 6 (event_ordering) +
          7 (guard_playback_debug) + 4 (guard_no_engine_names) = 53 ·
spike_cue_transfer 8 · spike_async_cancel 6 · spike_typed_errors 5 ·
spike_reverse_ffi 11 · nen-app 1 · nen-ffi 1 ·
nen-domain 20 (unit) + 9 (guard_redaction) = 29 ·
nen-subtitle 44 (unit) + 4 (encoding_golden) + 7 (encoding_negative) +
             4 (fuzz_smoke) + 3 (golden_valid) + 4 (guard_error_debug) +
             2 (language_detection_golden) + 4 (language_resolution) +
             3 (lookup_parity) + 3 (malformed) + 3 (webvtt_roundtrip) = 81 ·
nen-identity 125 (unit) + 7 (guard_evidence_debug) + 8 (nfo_and_container) +
             6 (os_hash_reference) + 4 (release_name_golden) +
             13 (resolution_layers) = 163 ·
nen-catalog 27 (unit) + 2 (guard_source_debug) + 3 (menu_projection_golden) = 32
390 passed, 0 failed, 1 ignored                  → exit 0 (NEN-021 ile 337 → 390)
  ignored = lookup_bench (baseline; scripts/bench-cue-lookup.sh ile koşar)

$ cargo tree -p nen-domain --edges normal
nen-domain v0.1.0                                → tek düğüm, sıfır bağımlılık
$ cargo tree -p nen-subtitle --edges normal
nen-subtitle → encoding_rs → cfg-if
nen-subtitle → blake3 → ...
nen-subtitle → whatlang → hashbrown → ...        → NEN-020 (ADR-0029) kenar
nen-subtitle → nen-domain
$ cargo tree -p nen-identity --edges normal
nen-identity → nen-domain                        → tek kenar, NEN-018 dış
                                                   bağımlılık EKLEMEDİ
$ cargo tree -p nen-ports --edges normal
nen-ports → nen-domain                           → tek kenar, NEN-021 dış
                                                   bağımlılık EKLEMEDİ
$ cargo tree -p nen-catalog --edges normal
nen-catalog → nen-domain
nen-catalog → nen-identity → nen-domain
nen-catalog → nen-subtitle → (yukarıdaki ağaç)   → NEN-019 dış bağımlılık
                                                   EKLEMEDİ, yalnız iç kenar

$ cargo clippy --workspace --all-targets --manifest-path core/Cargo.toml -- -D warnings
                                                  → exit 0, uyarı yok
$ cargo fmt --all --check --manifest-path core/Cargo.toml → exit 0
$ (cd core && cargo deny check)
  advisories ok · bans ok · licenses ok · sources ok → exit 0
  NEN-020, whatlang'ın hashbrown → foldhash kenarı için deny.toml'a Zlib
  ekledi (ADR-0029); NEN-019 deny.toml'a dokunmadı — yeni dış bağımlılık yok

$ cargo test -p nen-identity --manifest-path core/Cargo.toml
  release_name_golden  4 ✓  42 ad (8'i kasıtlı Unknown), .golden byte-eşit
  resolution_layers   13 ✓  ADR-0009 Karar 6 sırası + 17 URL + 12 yol korpusu
  os_hash_reference    6 ✓  bağımsız referans implementasyonuyla 7 boyutta
                            birebir; sınır vakaları ve tek-bit duyarlılığı
  nfo_and_container    8 ✓  4 sidecar fixture'ı; malformed olan hata değil
                            "tanınmadı" dönüyor
  guard_evidence_debug 7 ✓  10 yasak desen (K23 #1/#2/#3/#8) hiçbir çıktıda yok;
                            DerivedEvidence negatif kontrolü hepsini sızdırıyor

$ cargo test -p nen-subtitle --manifest-path core/Cargo.toml
  golden_valid   3 ✓   7 geçerli fixture, .golden snapshot'larıyla byte-eşit
  malformed      3 ✓   25 vaka, 17 varyantın hepsi kapsanıyor
  fuzz_smoke     4 ✓   16 281 vaka (781 trunc · 10 500 mut · 5 000 noise), 0.07 s
  guard_error_debug 4 ✓ hiçbir SrtError varyantı cue metni sızdırmıyor
  encoding_golden   4 ✓ 7 pozitif fixture, .decoded.golden'larıyla byte-eşit
  encoding_negative 7 ✓ boyut sınırı · undecodable · bidi/zero-width/kontrol
                        karakteri sanitization · EncodingError sızıntı yok
  lookup_parity     3 ✓ 10 000 seek + 10 000 aralık sorgusu + her cue sınırı,
                        14 dokümanlık korpusta lineer taramayla birebir

$ bash scripts/bench-cue-lookup.sh                → NEN-017 release baseline
$ bash scripts/bench-cue-lookup.sh --debug        → NEN-017 debug karşılaştırması
  50k cue: index p50 48–57 ns · lineer p50 15.6–17.6 µs → ~290–344×
  (3 koşunun aralığı; baseline'dır, eşik değil)

$ bash scripts/build-apple.sh                     → binding temiz üretildi
$ bash scripts/test-apple.sh
✔ Test run with 2 tests in 1 suite passed        → exit 0
  Tam Xcode kurulu olduğu için script ek bayrak EKLEMEDİ (düz `swift test`) —
  CommandLineTools yolu artık kullanılmıyor.

$ bash scripts/spike-cues.sh                      → NEN-008 release baseline
  checksum: 75001045577800 · 34337591381145
  → Swift 6.3.3 ile NEN-008/NEN-011'in kaydettiği değerlerle BİREBİR aynı;
    toolchain değişimi semantik sonucu değiştirmedi (I5 hâlâ geçerli)
$ bash scripts/spike-cues.sh --debug              → NEN-008 debug karşılaştırması

$ bash scripts/spike-async.sh --test-only         → NEN-009 invariant'lar (release)
$ bash scripts/spike-async.sh --debug --test-only → NEN-009 invariant'lar (debug)
✔ Test run with 3 tests in 1 suite passed        → exit 0 (ikisinde de)
$ bash scripts/spike-async.sh --measure-only      → NEN-009 release baseline
$ bash scripts/spike-async.sh --debug --measure-only → NEN-009 debug karşılaştırması

$ bash scripts/spike-typed-errors.sh
✔ Test run with 2 tests in 1 suite passed        → exit 0 (switch + redaction)
  eşleme maliyeti p50 2.04 µs · p95 2.17 µs        → NEN-010 baseline
  negatif kontrol (scratch): 4. varyant swift build'i "switch must be
  exhaustive" ile kırdı                            → DoD #3 kanıtlandı

$ bash scripts/spike-reverse-ffi.sh --test-only         → NEN-029 (release)
$ bash scripts/spike-reverse-ffi.sh --debug --test-only → NEN-029 (debug)
✔ Test run with 12 tests in 3 suites passed      → exit 0 (ikisinde de)
$ bash scripts/spike-reverse-ffi.sh                     → NEN-029 release baseline (A+B)
$ bash scripts/spike-reverse-ffi.sh --debug              → NEN-029 debug karşılaştırması
  A p50 36.67 µs + hop ~40 µs · B p50 1.33 µs (60 Hz, release)
  → NEN-029 baseline, ADR-0026'nın dayanağı

$ bash scripts/check-docs.sh
  8/8 denetim geçti                              → exit 0

$ bash scripts/test.sh
  check-docs.test.sh ✓ (11 doğrulama)
  doctor.test.sh     ✓ (24 doğrulama)            → exit 0
```

**Ölçüm build tipi artık kayıt altında.** NEN-007'nin kanıtı debug'dı; NEN-008
ikisini de koştu ve farkı ölçtü: debug, tam liste geçişini **1.75×**, pencere
erişimini **1.24×** yavaşlatıyor. İki koşunun Swift tarafındaki checksum'ları
birebir aynı — fark yalnız hızda, veride değil.

`scripts/test.sh` **`tasks/active/` dolu ve boşken ayrı ayrı** koşuldu; iki
koşunun `check-docs.test.sh` çıktısı birebir aynı (NEN-031 kanıt kaydı).

**B4 kapandı — kaydedilmiş gerekçesi de hatalıydı.** B4, `check-docs.test.sh`'ın
"bir task `active` olduğu anda" kırıldığını söylüyordu. Gerçek tetikleyici bu
değil: **ready listesinin boşalması**. NEN-031 `active/`'e alındığında listede
NEN-005/006/008/009/010 kaldı ve `test.sh` yeşil kaldı — yani "bir sonraki task
başlatıldığında yeniden kırmızıya dönecek" beklentisi de yanlıştı. NEN-007'de
kırılmasının sebebi, o an her backlog task'ının NEN-007'ye bağlı olmasıydı.
`check-docs.sh`'ın kendisi her iki durumda da doğru çalışıyordu (exit 0); kusur
yalnız fixture'daydı ve `NEN-031` ile kapandı.

## Repository'nin gerçek durumu

- **Kod var** (NEN-007 ile): `core/` altında Cargo workspace + 11 crate,
  `core/rust-toolchain.toml`, `Cargo.lock`.
- `core/crates/nen-domain/src/redact.rs` — NEN-006'nın redaction yardımcıları
  (`Redacted<T>`, `extension()`, `size_class()`, `redact_host()`); crate hâlâ
  bağımlılıksız. Guard test `core/crates/nen-domain/tests/guard_redaction.rs`
  (NEN-013 ile `Cue`/`SubtitleDocument` kapsamı eklendi).
- `core/crates/nen-domain/src/subtitle.rs` — NEN-013'ün subtitle değer tipleri:
  `CueId` · `TimeSpan` (invariant tipte: `start < end`) · `Cue` ·
  `SubtitleDocument`. `Cue` ve `SubtitleDocument` cue metni taşıdığı için
  `Debug` **elle yazılmış** (K23 #4).
- `core/crates/nen-subtitle/src/srt.rs` — NEN-013'ün strict SRT parser'ı ve 17
  varyantlı `SrtError`'ı. Crate `lib.rs`'inde panik lint kapısı
  (`unwrap_used`/`expect_used`/`panic`/`unreachable`/`indexing_slicing`,
  `cfg_attr(not(test))`). Testler: `golden_valid` · `malformed` · `fuzz_smoke`
  · `guard_error_debug`.
- `core/crates/nen-subtitle/src/webvtt.rs` — NEN-014'ün WebVTT writer'ı
  (`write(&SubtitleDocument) -> String`). `WEBVTT` başlığı, cue identifier +
  `HH:MM:SS.mmm` zaman satırı, `&`/`<`/`>` kaçışlı metin, BOM'suz. Test:
  `webvtt_roundtrip` (SRT → doc → WebVTT round-trip golden).
- `core/crates/nen-subtitle/src/encoding.rs` — NEN-015'in encoding/sanitization
  katmanı (ADR-0008), `srt::parse`'ın önünde çalışır. `decode(&[u8]) ->
  Result<String, EncodingError>`: BOM sniff (UTF-8/UTF-16LE/UTF-16BE) → sıkı
  UTF-8 denemesi → Windows-1254 fallback (tek legacy code page); `sanitize`
  kontrol karakteri/bidi override/zero-width temizliyor. `encoding_rs`'e
  bağımlı (workspace'in ilk gerçek dış bağımlılığı). Testler:
  `encoding_golden` · `encoding_negative` + 10 birim testi.
- `core/crates/nen-subtitle/src/fingerprint.rs` — NEN-016'nın timeline/source
  fingerprint'i (ADR-0007). `TimelineFingerprint::of`/`SourceFingerprint::of`
  `[u8; 32]` (`blake3`) döndürür; ikisi de `Display`/`Debug`'ı küçük harf hex
  olarak basar. `CueId` hiçbir hash'e dahil değil. `blake3`'e bağımlı
  (crate'in ikinci dış bağımlılığı, `encoding_rs`'ten sonra). 8 unit test.
- `core/crates/nen-identity/src/` — NEN-018'in dokuz modülü (ADR-0009):
  `os_hash.rs` (OSDb hash, I/O'suz `of(file_size, head, tail)`; `OsHash` `Debug`/
  `Display`'de `<redacted>`, gerçek değer yalnız `to_hex()`/`as_bytes()` ile) ·
  `release_name.rs` (`parse` → `ParsedName`, hata tipi **yok**, çözülemeyen ad
  `Unknown`) · `url_hints.rs` (path segmentleri; query/fragment/host tipe hiç
  girmez) · `dir_hints.rs` · `declared_name.rs` (RFC 6266/5987 + sanitization) ·
  `siblings.rs` (rapor eder, ezmez) · `nfo.rs` (Kodi/Plex sidecar, XML + tek
  satır URL) · `container.rs` (şekil + tag → identity; demuxer M3'te) ·
  `evidence.rs` (`MediaEvidence`, Karar 6 katman yürüyüşü, §6 aday üretimi).
  Crate'in `lib.rs`'inde `nen-subtitle`'ınkiyle aynı panik lint kapısı.
  **Yeni dış bağımlılık yok.**
- `core/crates/nen-domain/src/source.rs` — NEN-019'un kaynak değer tipleri
  (ADR-0010): dört `SubtitleSourceKind`, metadata-kimlikli `SubtitleSourceId`,
  normalize `LanguageTag`, redakte `SubtitleSource` ve iki dilli
  `SubtitlePreferences`. `nen-domain` sıfır bağımlılıklı kaldı.
- `core/crates/nen-catalog/src/` — NEN-019'un katalog, menü projeksiyonu ve
  otomatik seçim modülleri. Dedup upsert ile kimliğe göre; grup içi sıra ekleme
  sırası, dil grupları tercihlerden sonra tag sırası. Testler: 21 unit + 2
  golden + 2 negatif kontrollü `Debug` guard.
- `fixtures/catalog/` — ADR-0010 Karar 10'un tercihsiz ve `tr`/`en` tercihli
  yapısal menü golden'ları.
- `fixtures/media/` — NEN-018'in korpusu: `release-names.tsv` (42 ad, 8'i
  kasıtlı çözülemeyen) · `url-hints.tsv` (17 URL, token'lı query ve fragment
  dahil) · `dir-layouts.tsv` (12 yol) · her birinin `.golden`'ı ·
  `os-hash.golden` (7 sentetik boyut) · `nfo/` (4 sidecar, biri malformed).
  Golden'lar `UPDATE_GOLDEN=1 cargo test -p nen-identity` ile yenilenir.
- `core/crates/nen-subtitle/src/index.rs` — NEN-017'nin `CueIndex<'a>`'i:
  sıralı cue dizisi + monoton `max_end_prefix` artırımı üzerinde iki
  `partition_point`. `active_cues(at)` çakışan cue'ların **hepsini** doküman
  sırasında döndürür (yarı açık: `start_ms <= t < end_ms`); `active_cue(at)`
  ilkini döndüren kolaylık sarmalayıcısı; `cues_in(range)` pencere sorgusu.
  Yeni bağımlılık yok. Testler: 10 unit + `lookup_parity` (10 000 seek,
  lineer taramayla birebir) + `lookup_bench` (`#[ignore]`, baseline).
- `fixtures/subtitles/valid/` — 8 SRT fixture + 8 `.golden` (SRT parse)
  snapshot + 8 `.vtt` (WebVTT yazım, NEN-014) snapshot;
  `fixtures/subtitles/malformed/` — 25 fixture, her biri tek bozukluk;
  `fixtures/subtitles/encodings/` — 8 byte-precise fixture (NEN-015): 7
  pozitif + `.decoded.golden` snapshot'ları, 1 negatif
  (`undecodable-garbage.srt`). Golden biçimi (`valid/`): `cues\t<n>` başlığı +
  cue başına `id \t start_ms \t end_ms \t line_count \t escape'li metin`;
  `encodings/`'in golden'ı ise decode edilmiş tam UTF-8 metin (cue yapısı
  değil, decode doğruluğu kanıtlanıyor).
- `core/spikes/spike-cue-transfer/` — NEN-008'in ölçüm crate'i; kendi Swift
  harness'ı (`apple-harness/`) VE kendi Kotlin/JVM harness'ı (`jvm-harness/`,
  NEN-011 — Gradle wrapper tabanlı, `scripts/spike-cues-jvm.sh` üretir).
  **Ürün kodu değil, terfi etmez**; kendi FFI kapısını ADR-0028 sayesinde
  açıyor. Üretilen binding'ler commit edilmiyor.
- `core/spikes/spike-async-cancel/` — NEN-009'un ölçüm crate'i; Swift
  harness'ı hem CLI ölçüm hedefi hem swift-testing invariant test hedefi
  içeriyor (`apple-harness/Tests/`); Kotlin/JVM harness'ı (`jvm-harness/`,
  NEN-011) aynı invariant'ları `kotlin.test` ile ve bir
  `suspendCancellableCoroutine` sarmalayıcısıyla (coroutine iptali → gerçek
  `JobHandle.cancel()`) kanıtlıyor. Aynı ADR-0028 kapısı, aynı terfi yasağı.
- `core/spikes/spike-typed-errors/` — NEN-010'un ölçüm crate'i; `nen-domain`'e
  bağımlı (redaction yardımcıları), Swift harness'ı hem switch/redaction
  test hedefi (`apple-harness/Tests/`) hem baseline ölçüm hedefi
  (`apple-harness/Sources/`) içeriyor; Kotlin/JVM harness'ı (`jvm-harness/`,
  NEN-011) aynı ölçümü `AppException` (Kotlin'in "Error"→"Exception" son ek
  kuralı) ile tekrarlıyor. Aynı ADR-0028 kapısı, aynı terfi yasağı.
- `core/spikes/spike-reverse-ffi/` — NEN-029'un ölçüm crate'i; A (`ReverseEngine`,
  foreign `PlaybackObserver` trait'ini tekrar çağıran fake motor) ve B
  (`ForwardSession`, düz forward çağrılar) tek crate'te. Swift harness'ı hem
  baseline ölçüm hedefi (`apple-harness/Sources/`) hem üç ayrı swift-testing
  hedefi (`InvariantTests`/`OrderingTests`/`TypedErrorTests`) içeriyor. Aynı
  ADR-0028 kapısı, aynı terfi yasağı.
- `platforms/apple-shared/` — SwiftPM paketi (`Package.swift` + swift-testing
  test target'ı). Üretilen binding `generated/` altında ve **commit edilmiyor**.
- Diğer `platforms/*` dizinleri hâlâ boş iskelet.
- `.github/workflows/ci.yml` — NEN-005'in CI skeleton'ı; `core/deny.toml` —
  cargo-deny lisans/advisory/kaynak kapısı.
- Var olan: 6 ana doküman · 12 milestone dosyası · **10 accepted ADR**
  (0001, 0002, 0006, 0007, 0008, 0009, 0010, 0026, 0027, 0028) · 38 task ·
  14 script + 2 shell testi · `fixtures/subtitles|media|catalog/` dolu,
  `fixtures/providers/` hâlâ iskelet.
- Depo kökünde **`LICENSE` dosyası bilerek yok** — bkz. [`licensing.md`](licensing.md).
- Git: `main` branch. Remote: `github.com/ynsemrekryl2/nen-player` (private —
  NEN-005 ile kuruldu, CI'ın koşabilmesi için gerekliydi). **Bu dosya commit
  hash'i tutmaz** — commit geçmişi kanonik kayıttır ve elle tutulan hash
  satırı her kapanışta bayatlar (CLAUDE.md → "Commit politikası").

## Bu dosyayı kim günceller

Her task kapanışında (`/finish-task`) ve her milestone geçişinde.
`scripts/check-docs.sh` **denetim 8**, buradaki "Sıradaki READY" satırının
`tasks/INDEX.md` ile uyuşmasını mekanik olarak zorlar — bayatlarsa CI kırılır.
