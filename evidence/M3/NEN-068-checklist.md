# NEN-068 — Kabul ve kanıt kaydı

İlk ölçüm: 2026-08-31 · Son doğrulama: 2026-09-05
Makine: Apple Silicon · macOS 27.0
Toolchain: Swift 6.3.3 · Rust · libmpv 2.5.0
Build: debug, ad-hoc imzalı `NenPlayer.app`

Kanıt medyası yalnız depodaki telif-temiz sentetik fixture'lardır. Bu kayıt
beşini kullanıyor; son dördü bu task'ta eklendi. Üretim blokları `.ffmpeg.txt`
eşlerinde:

| Fixture | Saklanan | libmpv'nin verdiği display boyutu |
|---|---|---|
| `contract-clip.mkv` | 160×90 | 160×90 (16:9) |
| `aspect-4x3-clip.mkv` | 160×120 | 160×120 (4:3) |
| `aspect-cinema-clip.mkv` | 382×160 | 382×160 (≈2.39:1) |
| `anamorphic-clip.mkv` | 720×576 | **1024×576** (16:9) |
| `audio-only-clip.mka` | — | yok |

## Ölçüm

`evidence/M3/NEN-068-measurement.md` — üç soru, ürünün kendi mpv
seçenekleriyle kurulmuş headless bir handle üzerinde ölçüldü:

1. `MPV_EVENT_VIDEO_RECONFIG` **headless de üretiliyor** (yükleme başına iki
   kez), yani ADR-0038 Karar 2'nin adlandırdığı tetikleyici doğrudan
   kullanılabilir ve contract kiti gerçekten aynı davranışı yargılıyor.
2. Taşınan sayı **gerçekten display boyutu**: anamorphic klipte 720×576 değil
   1024×576 (ADR-0038 Karar 3).
3. Videosuz medyada iki property de okunamıyor **ve hiç reconfig üretilmiyor**
   — `None` dalı uydurma değil, motorun davranışı.

## Otomatik kanıt

### Rust — port, uygulama katmanı ve FFI

`cargo test --manifest-path core/Cargo.toml`: **tümü yeşil**,
`cargo clippy --all-targets` uyarısız.

| Kanıt | Nerede |
|---|---|
| `VideoGeometry` sıfır boyutu onarmak yerine reddediyor | `nen-ports/src/playback/geometry.rs` |
| `VideoGeometryChanged` coalescing; pozisyonla karışmıyor ve kayıp sayılmıyor | `nen-ports/src/playback/event.rs`, `tests/event_ordering.rs` |
| Kit'in geometry adımı **hem** olayı **hem** değeri zorluyor | `nen-ports/src/playback/contract.rs` |
| Kit videolu ve videosuz medyada ayrı ayrı yeşil, **ve** çapraz fixture'da kırmızı | `tests/contract_fake.rs` |
| Negatif kontrol: olayı yutan twin ve boyutu ters veren twin kırmızı oluyor | `tests/contract_kit_is_not_vacuous.rs` |
| `EventsLost` sonrası boyut yeniden okunabiliyor | `tests/event_ordering.rs` |
| K23: `VideoGeometry` sızdırmıyor **ve** sayılarını hâlâ yazdırıyor (ADR-0038 Karar 5) | `tests/guard_playback_debug.rs` |
| Bridge `video_geometry`'yi reentrant çağrıda motora ulaşmadan reddediyor | `nen-app/tests/playback_bridge.rs` |
| Session sorgusu ve shutdown sonrası `ShutDown` refüzü | `nen-app/tests/playback_session.rs` |
| Bozuk boyut FFI'de `None`'a düşüyor, sağlam boyut round-trip ediyor | `nen-ffi/src/session.rs` |

### Swift — adapter ve kabuk

`bash scripts/test-macos.sh`: **150 test / 15 suite, 0 failure**
(NEN-067 kapanışında 107'ydi).

| Suite | Kanıt |
|---|---|
| `ContractTests` | Gerçek libmpv adapter'ı paylaşılan kiti geçiyor; fixture artık 160×90 geometry beyan ediyor |
| `ContractTests` | Idle motorda `NotLoaded`; yüklemede olay **ve** 160×90 değeri |
| `ContractTests` | Anamorphic klip **1024×576** veriyor, 720×576 değil |
| `ContractTests` | Yükleme sırasında önceki VO boyutu görünmüyor; ardışık 16:9 → 4:3 → anamorphic → audio-only kendi boyutlarını veriyor |
| `ContractTests` | Audio-only klip `nil` veriyor **ve** hiç olay üretmiyor |
| `Window geometry` (13 test) | Türetilen minimumlar: 16:9 → 693×390 · 2.39:1 → 932×390 · 4:3 → 600×450 · 9:16 → 600×1067; her biri kendi oranında |
| `Window geometry` | Ekrandan büyük medya oran korunarak ölçekleniyor; krom tabanı ekran yüksekliğini yenerken pencere tepe köşesini erişilebilir tutuyor |
| `Window geometry writer` (14 test) | **Gerçek `NSWindow` üzerinde**: `contentAspectRatio` ve `contentMinSize` uygulanıyor; 4:3 medya 16:9 kilidini devralmıyor; video yokken kilit bırakılıyor |
| `Window geometry writer` | Yeni medya pencereyi boyutlandırıyor; **aynı** medyanın yeniden yapılandırılması kullanıcının yerleştirdiği pencereyi oynatmıyor |
| `Window geometry writer` | Tam ekrana girerken kilit bırakılıyor, çıkarken geri konuyor |
| `macOS player shell` (6 yeni test) | Olay okumayı tetikliyor; audio-only'de `nil`; yeni medyada, fatal'da ve shutdown'da sıfırlanıyor; `eventsLost` sonrası yeniden okunuyor |
| `Subtitle safe area` | ADR-0037 kanıtı, sabit 720×450 minimum kalktıktan sonra da yeşil |
| `Video surface fill` (2 test) | Video yüzeyi içerik görünümünü dört kenarda da tam örtüyor; ikinci test pencerenin gerçekten safe area bildirdiğini doğrulayarak birincinin boşuna yeşil olmasını engelliyor |

### Kapılar

- `bash scripts/test.sh`: geçti
- `bash scripts/build-macos-app.sh`: exit 0
- `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`: exit 0
- `bash scripts/check-docs.sh`: tüm denetimler geçti

## Elle kabul

Gerçek `NenPlayer.app` sürülerek yapıldı. Pencere boyutları gözle değil,
`CGWindowListCopyWindowInfo` ile **ölçülerek** kaydedildi.

| # | Adım | Ölçülen | Beklenen | Sonuç |
|---|---|---|---|---|
| 1 | `contract-clip.mkv` (16:9) | **693×390** | 693×390 | ✅ |
| 2 | `aspect-4x3-clip.mkv` | **600×450** | 600×450 | ✅ |
| 3 | `aspect-cinema-clip.mkv` | **931×390** | 931×390 | ✅ |
| 4 | `anamorphic-clip.mkv` | **1024×576** | 1024×576, 720×576 değil | ✅ |
| 5a | 1920×1080 klip açılış | **1470×827** (oran 1,7775) | ekrana sığdırılmış 16:9 | ✅ |
| 5b | Aynı klip, kenardan canlı sürükleme | **1470×827 → 1029×579 → 1020×574** | her adımda oran 1,777 | ✅ |
| 5c | Sürükleme boyunca siyah bar | düz sarı klipte hiçbir kenarda yok | yok | ✅ |
| 6 | Tam ekran **girişi** | pencere **1470×923**, oran **1,5926** | 16:9 olmayan bir şekil alabilmesi = kilidin bırakılması | ✅ |
| 6b | Tam ekran **çıkışı** (2026-09-05) | **1470×923 → 693×390**, sonraki resize **892×502** | kilit geri konmuş olmalı | ✅ |
| 7 | `audio-only-clip.mka` | **843×474 → 883×461 → 923×447 → 963×434** | oran kilidi yok, serbest resize | ✅ |
| 8 | Boş durum | **963×434 → 940×447 → 916×461 → 893×474** | serbest resize | ✅ |

Merkez korunması da ölçüldü: 693×390 penceresi (merkez 454,329) 600×450'ye
geçtiğinde origin tam olarak (154,104) oldu. 931×390'a geçişte hesaplanan
origin −11,5 çıktı ve ekran kenarına **0**'a kenetlendi — boyut değişmeden,
`recentredFrame`'in yaptığı iş.

### Elle kabul sırasında bulunan ve düzeltilen kusur

Kullanıcı, pencere doğru orana kilitlendiği hâlde videonun çevresinde siyah
çerçeve kaldığını gördü ve düz sarı bir test klibiyle bunu tartışmasız hâle
getirdi. Ölçüldüğünde: **solda ve sağda ~28 pt, üstte ~32 pt**, her boyutta ve
her çözünürlükte, medya açılır açılmaz.

Sebep renderer değil, kabuğun kendi yerleşimiydi: `PlayerRootView` içinde
`Color.black` pencerenin safe area'sını yok sayıyordu, `VideoSurface` ise
saymıyordu. Video yüzeyinin örtmediği şerit, arkasındaki siyahtan görünüyordu —
yani **oynatıcı letterbox'ı kendi çiziyordu**. Oran kilidi doğru çalışsa bile
ürünün vaadi bu yüzden karşılanmıyordu.

Düzeltme `VideoSurface`'e `.ignoresSafeArea()` eklemek. Krom bilinçli olarak
safe area'ya uyuyor: transport sırası ve trafik lambaları oraya aittir.

Regresyon testi `VideoSurfaceFillTests` — **gerçek bir `NSWindow` içinde**,
çünkü penceresiz bir hosting view'ın safe area'sı yoktur ve test boşuna
yeşil olurdu. İkinci test tam olarak bunu, yani pencerenin gerçekten bir safe
area bildirdiğini iddia eder. Negatif kontrol koşuldu: `.ignoresSafeArea()`
kaldırıldığında test **"a 32.0 pt black margin"** diyerek kırmızıya döner —
ekranda ölçülen sayının aynısı.

### Tam ekran — 2026-09-05 kapanış doğrulaması

Kullanıcı tam ekranda video, ses ve kontrollerin durduğunu doğruladı. Önceki
kaydın bunu yalnız otomasyon katmanına bağlaması **kanıtlanmamıştı**; gerçek
uygulamada kusur yeniden üretildi ve düzeltildi.

Kök neden, oran kilidini kaldırmak için `contentAspectRatio = .zero`
kullanılmasıydı. Oran getter'ı `(0,0)` görünse de `resizeIncrements` `(0,0)`
kalıyordu. Gerçek tam ekran çıkışı `willExitFullScreen` sonrasında takılıyor,
`didExitFullScreen` gelmiyor ve pencere 1470×923'te kalıyordu. Yalnız sıfırlama
çağrısı `resizeIncrements = NSSize(width: 1, height: 1)` ile değiştirildiğinde
çıkış tamamlandı, pencere 693×390'a döndü ve oynatma ilerledi.

Ayrıca girişten çıkışın tamamlanmasına kadar geometri yazımları erteleniyor.
Bu sırada açılan yeni medyanın son boyutu tutuluyor; çıkış tamamlandığında
uygulanıyor. Videosuz medyaya geçilmişse eski oran geri getirilmiyor.

Regresyonun negatif kontrolleri ayrık: eski sıfırlama çağrısı **4 testi / 5
assertion'ı**, geçiş korumasının kaldırılması **2 testi / 7 assertion'ı**
kırmızıya çevirdi. Doğru kod geri konunca **14/14** pencere testi geçti.

Geçici tanılama ürün kodundan kaldırıldıktan sonra gerçek ad-hoc imzalı
uygulamada `F`, tam ekran düğmesi ve `Esc`; tam ekranda oynat/duraklat ve
ileri seek; oynarken ve duraklatılmışken tam ekran geçişleri doğrulandı.
Pencere 1470×923'ten her iki çıkış yolunda 693×390'a döndü. Sonraki gerçek
kenar sürüklemesinde 693×390 → 759×427 → 825×464 → 892×502 ölçüldü; video
pencereyi kenarlara kadar doldurdu. Kullanıcının değiştirdiği 892×502 boyutu,
bir sonraki tam ekran giriş/çıkışında korundu.

Ayrıntılı yeniden üretim, A/B ölçümü ve kanıt:
[NEN-068-fullscreen.md](NEN-068-fullscreen.md).

- [16:9, tam ekran dönüşünden sonra resize](NEN-068-window-16x9.png)
- [Tam ekran](NEN-068-fullscreen.png)
- [4:3, tam ekran dönüşünden sonra resize](NEN-068-aspect-4x3-clip.png)
- [2.39:1, tam ekran dönüşünden sonra resize](NEN-068-aspect-cinema-clip.png)
- [Anamorphic 16:9, tam ekran dönüşünden sonra resize](NEN-068-anamorphic-clip.png)
