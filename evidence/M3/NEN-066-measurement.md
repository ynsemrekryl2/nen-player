# NEN-066 — ölçüm: motorun çizdiği replik nereye düşüyor

Koşum: **2026-08-29** · macOS 27.0 · Apple M5 · libmpv 2.5.0 (Homebrew,
dinamik) · `GL_VERSION = 2.1 Metal - 91.7`.

Kanıt medyası yalnız `fixtures/media/contract-clip.mkv` ve bellekte üretilen
sentetik bir WebVTT belgesidir. Aşağıda hiçbir replik metni, tam dosya yolu
veya özel medya metadata'sı yok — yalnız piksel sayıları ve satır aralıkları
(K23 #4).

## Sonuç

**A da B de yanlış çıktı.** Render yolu altyazıyı dört durumun **dördünde de**
kareye bileştiriyor — duraklatılmışken de, oynarken de, gömülü track için de,
enjekte edilen belge için de. Kusur çizimde değil: motorun çizdiği replik,
uygulamanın **kendi kromunun altına** düşüyor.

## Nasıl ölçüldü

`MPVVideoView.draw(_:)`'in gövdesi `renderFrame(intoFramebuffer:width:height:)`
olarak ayrıldı — davranış değişmeden, çünkü `draw(_:)` aynı gövdeyi `fbo: 0` ve
`convertToBacking(bounds)` ile çağırıyor. Ölçüm aynı metodu, aynı render
context'i üzerinden, bilinen boyutlu bir offscreen FBO'ya sürüyor ve
`glReadPixels` ile kareyi geri okuyor.

Altyazı pikselini video içeriğinden ayıran şey **aynı an, iki kare**: fixture
`testsrc` olduğu için hareketli, bu yüzden iki farklı an karşılaştırılmıyor —
aynı konumda altyazı açık ve kapalı çekilen iki kare karşılaştırılıyor. Fark
eden pikseller, tanım gereği, altyazının kendisi.

Bugüne kadar `MPVVideoView`'ı **hiçbir test sürmüyordu**: `makePlaybackSurface()`
üç testte çağrılıyor, ama hiçbiri render etmiyor. Kusurun bu kadar geç
görülmesinin sebebi de bu.

## Dört hücre — hepsi çiziyor

Yüzey 1280×720 piksel (640×360 pt @2x). Satırlar **alttan** sayılıyor.

| Senaryo | `sub-text` | Fark eden piksel | Bant (alttan) |
|---|---|---|---|
| gömülü track · duraklatılmış | dolu | 5 454 | 31–65 px = **15,5–32,5 pt** |
| gömülü track · oynarken | dolu | 4 770 | 30–64 px = **15,0–32,0 pt** |
| enjekte belge · duraklatılmış | dolu | 2 212 | 37–64 px = **18,5–32,0 pt** |
| enjekte belge · oynarken | dolu | 2 212 | 37–64 px = **18,5–32,0 pt** |

`NEN-027`'nin "duraklatılmışken çizilmiyor" ve "enjekte belge hiç çizilmiyor"
gözlemlerinin ikisi de bu tabloyla çürüyor.

## Kök neden: kendi kromumuz üstünü örtüyor

Transport barının gerçek yüksekliği ölçüldü — `TransportControls`
`NSHostingView`'a konup `fittingSize` okundu:

```
TRANSPORT BAR fitting height = 114.0 pt   (640 pt genişlikte de, 1280'de de)
```

`PlayerRootView.playerChrome` bu barı `.padding(.bottom, 20)` ile pencerenin
altına yaslıyor. Yani bar pencerenin altından **20–134 pt** arasını kaplıyor,
ve `GlassSurface` `.hudWindow` materyali + `%50` dolgu + `radius 27, y 20`
gölge ile bunu opak sayılacak kadar örtüyor.

Motorun altyazıyı koyduğu bant ise pencerenin altından **15–32,5 pt**.

```
pencerenin altı
0 pt ├──────────────────────────────
     │  ← altyazı bandı 15–32,5 pt
20pt ├──── transport barı başlıyor
     │
134pt┴──── transport barı bitiyor
```

**Bant tümüyle barın (ve gölgesinin) altında kalıyor.** Üstelik altyazı bandı
yüzey yüksekliğinin oranı (%4,2–%8,9) olarak ölçekleniyor, bar ise sabit
114+20 pt: bandın üst kenarı barın üstüne ancak pencere ~1 505 pt'den uzun
olursa çıkar. Gerçek hiçbir pencerede — tam ekran dahil — çıkmıyor.

## Bu, `NEN-027`'nin dört gözleminin dördünü de açıklıyor

Kabuğun kuralı: **kontroller yalnız oynarken gizlenir** (`PlayerModelTests`
→ "controls hide only while playing"), ve altyazı paneli açıkken
`setControlsPinned(_:)` gizlemeyi tümüyle askıya alıyor (NEN-062).

| Gözlem | Kromun durumu | Sonuç |
|---|---|---|
| Açılışta gömülü track, oynarken | 2,5 sn sonra **gizlendi** | ✅ görüldü |
| Menüden kullanıcı dosyası seçildi | panel açık → **pinned** | ❌ örtülü |
| Dosya seçiliyken cue'ların üzerinden oynatıldı | panel açık → **pinned** | ❌ örtülü |
| Gömülü track'e dönüldü, duraklatılmış 16 sn | duraklatılmış → **hiç gizlenmez** | ❌ örtülü |
| Pencere yeniden boyutlandırıldı | işaretçi hareketi → **görünür** | ❌ örtülü |

Tek başarılı gözlem, kromun gizlendiği tek andır. "Duraklatılmışken çizilmiyor"
sanılan şey, duraklatılmışken kromun **hiç gizlenmemesiydi**.

## Yan gözlem — `NEN-060` muhtemelen aynı kusur

`NEN-060` "seçilen gömülü track tam ekranda çizilmiyor" diye açılmıştı ve aynı
koşuda "pencere modunda bir kez çalıştı" diye kaydedilmişti. Tam ekranda da bar
aynı 114+20 pt'yi kaplıyor; işaretçi hareket ettiği sürece de görünür kalıyor.
Bu task kapandıktan sonra `NEN-060` yeniden koşulmalı — büyük olasılıkla
üretilemez hale gelir. Ölçülmeden `done` sayılmaz.

## Elenen adaylar

| Aday | Sonuç |
|---|---|
| Render yolu duraklatılmışken OSD'yi bileştirmiyor (**A**) | **elendi** — duraklatılmış iki hücre de çiziyor |
| Dışarıdan eklenen altyazı bu yolda hiç bileştirilmiyor (**B**) | **elendi** — enjekte belge iki hücrede de çiziyor |
| `mpv_render_context_update()` hiç çağrılmıyor | **gereksiz** — bileşim zaten oluyor |
| FBO boyutu / OSD ölçek uyuşmazlığı | **elendi** — bant yüzey yüksekliğiyle orantılı ve doğru yerde |
| `vo=libmpv` `mpv_initialize`'dan sonra ayarlanıyor | **değişken değil** — bileşim bu haliyle çalışıyor |
| `MPV_RENDER_PARAM_ADVANCED_CONTROL` yokluğu | **gereksiz** |

## Ölçümün kendi sınırı

Ekrandaki gerçek drawable'dan (FBO 0) piksel geri okuma bu ortamda **geçerli
bir enstrüman değil**: pencere görünürken bile geri okunan tampon tümüyle siyah
geliyor (video dahil), oysa gerçek `.app`te video görünüyor. Bu yüzden bütün
ölçüm offscreen FBO üzerinden yapıldı; ekranda gerçekten ne olduğu, kullanıcının
`.app` koşusuyla doğrulanacak.
