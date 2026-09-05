# Durum

> **Bu dosya yalnız doğrulanmış bugünü anlatır.** Plan `roadmap.md`'de, kararlar
> `DECISIONS.md`'de, task ayrıntısı `tasks/INDEX.md`'de. Burada tekrar edilmez.
>
> Son güncelleme: **2026-09-05** (`NEN-068` done — medya boyutunda açılan,
> orana kilitli pencere; tam ekran ve ardışık medya kusurları kapatıldı.
> Önceki: `NEN-067` done — ultra-ince, zemine-bitişik oynatıcı kromu)

## Nerede duruyoruz

| | |
|---|---|
| **Mevcut milestone** | **M3 — macOS Vertical Slice** (M2 kapandı; toolchain kapısı **açık**) |
| **Aktif task** | — |
| **Son tamamlanan** | `NEN-068` — media-sized, aspect-locked player window |
| **Sıradaki READY** | `NEN-027`, `NEN-033`, `NEN-034`, `NEN-035`, `NEN-036`, `NEN-040`, `NEN-041`, `NEN-042`, `NEN-043`, `NEN-044`, `NEN-047`, `NEN-049`, `NEN-050`, `NEN-052`, `NEN-057`, `NEN-069` |
| **Task sayısı** | 69 · done 47 · active 0 · blocked 0 · canceled 2 · backlog 20 |

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

2026-09-05'te NEN-068 kapanışı için bu makinede Rust workspace **560/560**
(1 ignored benchmark), Swift paketi **150/150** geçti. `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo deny check`,
shell testleri, `.app` build'i, strict codesign, task index ve doküman kapıları
da yeşil. Ayrıntılı ürün kabulü `evidence/M3/NEN-068-checklist.md` ve
`evidence/M3/NEN-068-fullscreen.md` içinde.

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
