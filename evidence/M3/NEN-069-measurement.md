# NEN-069 — Videosuz medyada yüzeyin ölçümü

Tarih: 2026-09-06
Makine: Apple Silicon · macOS 27.0
libmpv: 2.5.0 (mpv 0.41.0_8, Homebrew)
Xcode: 26.6

Ölçüm, üründe kullanılan `MPVVideoView` + `MPVPlaybackEngine(videoView:)`
ikilisiyle yapıldı. Kareler `NEN-066`'nın offscreen framebuffer'ına, ürünün
kendi `renderFrame(intoFramebuffer:width:height:)` yolundan çizildi. Ölçüm
dosyası geçiciydi ve depoya girmedi; ürettiği sayılar burada.

## Soru 1 — Sebep hangisi?

Task dosyası tek bir sebep yazıyordu: *"mpv videosuz medyada yeni kare üretmez,
dolayısıyla render yolu hiç çağrılmaz ve GL yüzeyi son kareyi tutar."* Bu cümle
iki ayrı iddiayı taşıyor ve hangisinin doğru olduğu hem düzeltmeyi hem testi
değiştiriyordu:

- **A** — `renderFrame` çağrılsa bile mpv videosuz medyada framebuffer'a
  dokunmuyor. Düzeltme: context varken de `glClear`.
- **B** — mpv aslında siyaha boyuyor; eksik olan `needsDisplay`, yani
  `draw(_:)`'in hiç çağrılmaması. Düzeltme: yüzeye redraw işareti koymak.

`menu-clip.mkv` yüklenip 5 s'ye seek edildi, bir kare alındı; sonra
`audio-only-clip.mka` yüklendi ve `renderFrame` **elle** sürüldü:

```
videoFrame  nonBlack = 834538 / 921600
audio geometry = nil
needsDisplay-after-audio-load = false
audioFrame  nonBlack = 0        (ikinci yakalama da 0)
```

**Sebep B.** mpv videosuz medyada render edildiğinde yüzeyi kendisi siyaha
boyuyor — çizim eksik değil, **isteyen** eksik. `needsDisplay` audio-only medya
yüklendikten sonra `false` kalıyor, çünkü mpv'nin update callback'i yalnız
**yeni kare** ürettiğinde tetikleniyor ve videosuz medya hiç kare üretmiyor.

Bunun testler için bedeli doğrudan: `NEN-066`'nın `capture(from:)` yardımcısı
`renderFrame`'i doğrudan çağırdığı için `needsDisplay` yolunu atlıyor. Yani
DoD'un önerdiği biçimiyle yazılan piksel testi **düzeltmeden önce de yeşil**
olurdu. Nitekim öyle: negatif kontrol 1'de (aşağıda) redraw isteği kaldırıldığında
piksel testi yeşil kaldı, kırmızıya dönen yalnız tetikleyici testi oldu.

## Soru 2 — `needsDisplay` testten gözlenebilir mi?

Hayır, pencere olmadan gözlenemez. Penceresiz bir `MPVVideoView`'de:

```
window = nil
set true  -> needsDisplay reads false
set false -> needsDisplay reads false
```

AppKit çizilecek yer yokken bayrağı tutmuyor. Aynı görünüm borderless bir
`NSWindow`'un içine konduğunda:

```
window = true
set false -> false
set true  -> true
displayIfNeeded() -> false
```

Bu yüzden tetikleyici testi yüzeyi bir pencereye koyuyor — üründeki hâli de
budur.

## Soru 3 — Geometri `ready` anında okunabiliyor mu?

Tetikleyicinin `videoGeometry == nil` ile korunup korunamayacağını belirledi:

```
menu-clip.mkv        atReady = 160x90   later = 160x90
audio-only-clip.mka  atReady = nil      later = nil
```

Motor `ready` anında **cevap veriyor**. Ama kabuğun kendi `videoGeometry`
kopyası o anda ikisinde de `nil`: her yüklemeden önce temizleniyor ve yalnız
`VIDEO_RECONFIG` söylediğinde yeniden okunuyor (ADR-0038 Karar 2), o olay ise
`ready`'den sonra geliyor. Dolayısıyla `ready` anında bir `videoGeometry == nil`
guard'ı resimli medyada da resimsiz medyada da aynı cevabı verir ve hiçbir şeyi
sabitlemez. Tetikleyici bu yüzden koşulsuz: **açılan her medya bir redraw
alıyor**, çünkü resmi olmayan medya başka türlü hiç almıyor. Maliyeti yükleme
başına, mpv'nin zaten elinde tuttuğu karenin tek bir render'ı.

## Yan bulgu — temizlik verildiği framebuffer'a inmiyordu

`shutdown` yolunun testi yazılırken çıktı ve **kırmızıydı**: motor kapatıldıktan
sonra offscreen hedefte önceki medyanın 834538 pikseli duruyordu.

Sebep `renderFrame`'in context'siz dalıydı. mpv, kendisine verilen
framebuffer'ı **kendi** bind ediyor; context yokken çalışan `glClear` ise o
argümanı hiç kullanmıyor ve o an bağlı olan framebuffer'a iniyordu. Ekran
yolunda bağlı olan zaten 0 ve `fbo` da 0 olduğu için **üründe yanlış bir şey
olmuyordu**; offscreen yakalamada ise hiçbir zaman doğru olmuyor. `fbo`
bind edildi.

Kapanış yolunun geri kalanı için yeni kod yazılmadı ve gerekmedi:
`shutdown` → `videoView.detach()` zaten render context'i bırakıp
`needsDisplay = true` diyor. Kabuğun `stop()` çağırdığı bir yol yok
(`PlayerModel` içinde `session.stop()` hiç çağrılmıyor), bu yüzden "stop"
ayağının ürün karşılığı yok.

## Negatif kontroller

| Kaldırılan | Kırmızı | Hangi test |
|---|---|---|
| `videoView?.redrawWithoutNewFrame()` (`PlayerModel.apply(.ready)`) | **1** | `an opened medium is drawn once without mpv asking` |
| `glBindFramebuffer(…, GLuint(fbo))` (`MPVVideoView.renderFrame`) | **1** | `a medium with no picture, and then no medium, keep none of the last one` |

İkisi de ayrık: her düzeltme yalnız kendi testini kırıyor, diğer test yeşil
kalıyor. Piksel testinin ilk yarısı (mpv'nin siyaha boyaması) her iki kontrolde
de yeşil — bu yüzden kanıt değil, dayanak olduğu suite dokümanına yazıldı.

## Paralel paket üzerindeki etki (NEN-049)

Bu suite gerçek libmpv sürdüğü için main actor'ı paylaşan zamanlayıcı temelli
kabuk testlerine yük bindiriyor. İki ölçüm yapıldı ve ikisi de kapsamı
değiştirdi:

1. **Bekleme biçimi.** İlk hâlde bekleme `RunLoop.current.run(until:)` ile
   yapılıyordu (NEN-066'nın deseni). Paralel pakette `PlayerModelTests`'in
   üç testi **beş saniyelik** deadline'ları kaçırdı — NEN-066 için bir kez
   zaten genişletilmiş deadline'lar. Bekleme `await Task.sleep`'e çevrildi;
   suspend ettiği için o testlerin continuation'ları actor'ı geri alıyor ve
   bu kırmızı sınıfı tamamen kayboldu.
2. **Motor sayısı.** İki ayrı testin kendi engine'i varken paralel paket
   `ContractTests.successiveMediaReportTheirOwnDisplaySize`'ı **3/3 kırmızı**
   yapıyordu. İki test tek engine'e birleştirildi.

Birleştirmeden sonra kalan katkı **ölçülerek** aranadı. Makinenin durumu
oturum boyunca kaydığı için (aynı ağaç sabahleyin 2/5, öğleden sonra 1/4)
ardışık değil **dönüşümlü** koşuldu: aynı ağaçta suite dosyası sırayla var ve
yok edilerek dörder kez.

| Ağaç | Paralel tam paket |
|---|---|
| Suite dosyası **var** | **1/4 yeşil** |
| Suite dosyası **yok** | **1/4 yeşil** |
| Suite var, seri (`--no-parallel`) | **2/2 yeşil**, 157/157 |

Yani birleştirmeden sonra bu suite'in kırmızı oranına katkısı **ölçülemiyor**;
paralel paket bu makinede bu task olmadan da aynı oranda kırmızı.

Kırmızı olan test her seferinde aynı ve bu task'ın koduna erişmiyor:
`ContractTests` engine'lerini `videoView: nil` ile kuruyor, yani ne
`redrawWithoutNewFrame` ne de değişen `renderFrame` dalı o yolda çalışıyor.
Kusur `waitForGeometry`'nin **ilk non-nil** değeri kabul etmesinde: `loadfile`
sonrası `FILE_LOADED` geldiğinde `video-out-params` hâlâ giden medyayı
tarif edebiliyor ve `aLoadingMediumDoesNotExposeTheOutgoingDisplaySize` bu
davranışı zaten bilerek sabitliyor. Yük altında o pencere genişliyor; test
5 s'lik timeout'una hiç ulaşmadan ~0,3 s'de kırmızı oluyor, yani zaman aşımı
değil **bayat değer** kusuru.

Kural 5 gereği bu task'a alınmadı; gözlem `NEN-049`'a eklendi.

**Bu, DoD'un `bash scripts/test-macos.sh` yeşil maddesinin paralel — yani
script'in varsayılan — modda karşılanmadığı anlamına gelir.** Karşılanan:
seri modda 157/157, iki koşuda; ve bu suite tek başına art arda yeşil. Kapanış
bu ayrımı gizlemeden yapıldı.
