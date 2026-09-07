---
id: NEN-069
title: Clear the video surface when the medium has no video
milestone: M3
size: S
state: done
closed: 2026-09-06
depends_on: []
blocks: []
adr: [12]
---

# NEN-069 — Videosuz medyada yüzey temizlenir

## Sonuç

Videosuz bir medya açıldığında oynatıcı yüzeyi boş kalır. Önceki medyanın son
karesi ekranda durmaz.

## Kusur

`NEN-068`'in elle kabulü sırasında görüldü. `audio-only-clip.mka` açıldığında:

- başlık doğru şekilde `audio-only-clip.mka` oluyor,
- pencere doğru davranıyor — oran kilidi kurulmuyor, serbest resize çalışıyor,
- ama **ekranda bir önceki videonun son karesi duruyor**.

Sebep: mpv videosuz medyada yeni kare üretmez, dolayısıyla `MPVVideoView`'in
render yolu hiç çağrılmaz ve GL yüzeyi en son çizilen kareyi tutmaya devam
eder. Yüzeyi kimse temizlemiyor.

`MPVVideoView.renderFrame(intoFramebuffer:width:height:)` render context yokken
zaten `glClear` yapıyor; eksik olan, **context varken ama gösterilecek video
yokken** aynı şeyin yapılması.

## Kapsam

- Videosuz medyada yüzeyin temizlenmesi. Tetikleyici hazır: ADR-0038'in
  `videoGeometry` sorgusu `None` döndürüyor ve kabuk bunu zaten biliyor
  (`PlayerModel.videoGeometry`).
- Medya kapanınca (`stop`, `shutdown`) da aynı temizlik.

## YAPILMAYACAK

- Audio-only medya için görsel bir şey **çizmek** (albüm kapağı, dalga formu,
  görselleştirme). Boş yüzey boş kalır; kapsam temizlemektir.
- Pencere geometrisine dokunmak — `NEN-068` o davranışı kapattı ve videosuz
  medyada serbest resize doğrudur.

## Kanıt (DoD)

- [x] `NEN-066`'nın offscreen capture desenini kullanan bir test: videolu bir
      medyadan videosuz bir medyaya geçildiğinde yakalanan karede önceki
      medyadan kalan piksel yok.
- [x] Negatif kontrol: temizlik kaldırıldığında testin kırmızıya döndüğü
      gösterilir — yoksa test boşuna yeşil olur.
- [~] `bash scripts/test-macos.sh` yeşil — **seri modda** 157/157, 2/2 koşu.
      Script'in varsayılan paralel modu bu makinede bu task olmadan da
      kırmızı; ölçüm ve gerekçe aşağıda.

## Kanıt kaydı

Tarih: 2026-09-06 · Apple Silicon · macOS 27.0 · Xcode 26.6 · libmpv 2.5.0

### Ölçüm teşhisi değiştirdi

Task dosyasının yazdığı sebep — *"mpv videosuz medyada yeni kare üretmez,
dolayısıyla render yolu hiç çağrılmaz ve GL yüzeyi son kareyi tutar"* — iki
ayrı iddia taşıyordu ve **yalnız biri doğru çıktı**. Ürünün kendi
`renderFrame` yolu elle sürüldüğünde videosuz medyada yakalanan kare
**0/921600 aydınlık piksel** verdi: mpv çizimi zaten yapıyor. Eksik olan
**isteyen**di — `needsDisplay` audio-only yüklemesinden sonra `false` kalıyor,
çünkü mpv'nin update callback'i yalnız yeni kare üretildiğinde tetikleniyor.

Bunun bedeli doğrudan test tasarımına indi: DoD'un önerdiği biçimiyle,
`NEN-066`'nın `capture(from:)` yardımcısı `renderFrame`'i doğrudan çağırdığı
için `needsDisplay` yolunu atlıyor ve o test **düzeltmeden önce de yeşil**
olurdu. Nitekim negatif kontrol 1'de yeşil kaldı. Kusuru ölçen test bu yüzden
tetikleyiciye bakıyor, piksele değil.

`videoGeometry == nil` guard'ı da ölçümle elendi: motor `ready` anında cevap
veriyor (`menu-clip.mkv` → 160x90), ama kabuğun kopyası her yüklemeden önce
temizlenip yalnız `VIDEO_RECONFIG` ile yenilendiği için o anda **ikisinde de**
`nil`. Guard hiçbir şeyi sabitlemezdi; tetikleyici koşulsuz bırakıldı.

### Yan bulgu — temizlik verildiği framebuffer'a inmiyordu

`shutdown` yolunun testi yazılırken kırmızı geldi: motor kapatıldıktan sonra
offscreen hedefte önceki medyanın **834538** pikseli duruyordu.
`renderFrame`'in context'siz dalı `fbo` argümanını hiç kullanmıyor, o an bağlı
olan framebuffer'a `glClear` ediyordu. Ekran yolunda ikisi de 0 olduğu için
**üründe yanlış bir şey olmuyordu**; offscreen yakalamada hiçbir zaman doğru
olmuyor. `fbo` bind edildi.

Kapanış yolunun geri kalanı için kod yazılmadı ve gerekmedi: `shutdown` →
`videoView.detach()` zaten context'i bırakıp redraw istiyor, ve kabuğun
`session.stop()` çağırdığı bir yol **yok** — "stop" ayağının ürün karşılığı
bulunmuyor.

### Testler

`platforms/macos/Tests/NenPlayerShellTests/PicturelessSurfaceTests.swift`:

- `an opened medium is drawn once without mpv asking` — kusurun kendisi
- `nothing else in the stream asks the surface to draw` — tetikleyicinin her
  olayda ateşlenmediği; birincisinin yanlış sebeple geçmesini engelliyor
- `a medium with no picture, and then no medium, keep none of the last one` —
  gerçek libmpv, `NEN-066`'nın offscreen framebuffer'ı: videolu karede
  aydınlık piksel var, videosuz medyada ve shutdown sonrası **0**

Suite tek başına art arda yeşil (3 test).

### Negatif kontrol — iki yönde, ayrık

| Kaldırılan | Kırmızı | Hangi test |
|---|---|---|
| `videoView?.redrawWithoutNewFrame()` | **1** | `an opened medium is drawn once without mpv asking` |
| `glBindFramebuffer(…, GLuint(fbo))` | **1** | `a medium with no picture, and then no medium, keep none of the last one` |

Her düzeltme yalnız kendi testini kırıyor; diğeri yeşil kalıyor.

### Kapılar

- Swift macOS paketi, **seri**: **157/157**, art arda 2/2 yeşil
- Rust workspace: **560 passed**, 0 failed (bu iş Rust'a dokunmadı)
- `cargo fmt --check`, `cargo clippy -D warnings`: temiz
- `bash scripts/test.sh`: 2 test dosyasının hepsi geçti
- `.app` build'i ve `codesign --verify --strict`: geçti

### Karşılanmayan madde ve sebebi

`bash scripts/test-macos.sh`'in **varsayılan paralel** modu yeşil değil.
Kırmızı olan test her koşuda aynı ve bu task'ın koduna erişmiyor:
`ContractTests.successiveMediaReportTheirOwnDisplaySize`, engine'lerini
`videoView: nil` ile kuruyor. Katkı olup olmadığı dönüşümlü ölçüldü — aynı
ağaçta bu task'ın suite dosyası sırayla var ve yok edilerek dörder koşu:
**1/4 yeşil** ve **1/4 yeşil**. Yani oran bu task'la ölçülebilir biçimde
değişmiyor; paralel paket bu makinede değişiklik olmadan da kırmızı.

Yol üstünde iki şey ölçüldü ve **bu task'ın kendi kapsamını** değiştirdi:
`RunLoop.run(until:)` ile bekleyen bir libmpv suite'i `PlayerModelTests`'in üç
testini beş saniyelik deadline'larını kaçıracak kadar aç bırakıyor
(`await Task.sleep`'e çevrildi), ve gerçek engine **sayısı** doğrudan etkili
(iki test tek engine'e birleştirildi). İkisi de Kural 5 gereği `NEN-049`'a
dördüncü gözlem olarak yazıldı — oran ölçümü o task'ın DoD #3'ünün bugüne
kadar üretilemeyen kaydıdır.

Tam ölçüm: `evidence/M3/NEN-069-measurement.md`.
