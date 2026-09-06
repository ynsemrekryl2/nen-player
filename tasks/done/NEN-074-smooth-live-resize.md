---
id: NEN-074
title: Smooth live video resize
milestone: M3
size: M
state: done
depends_on: [NEN-073]
blocks: []
adr: [38]
---

# NEN-074 — Akıcı canlı video resize

## Sonuç

Oynayan video penceresi sürükleyerek yeniden boyutlandırılırken ana thread hedef kare zamanını beklemez, pencere akıcı biçimde parmağı/fareyi izler ve resize bitince normal A/V zamanlamasına döner.

## Kapsam

- `MPVVideoView.draw(_:)` yolunda pencerenin `inLiveResize` durumunu okuyup
  libmpv render çağrısına `MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME` geçirmek.
- Normal oynatmada hedef kare zamanını beklemeyi açık tutmak; yalnız canlı
  resize sırasında kapatmak.
- Resize bittiğinde varsayılan zamanlamaya otomatik dönmek; framebuffer boyutu,
  HiDPI dönüşümü, update callback ve aspect lock akışını korumak.
- Aynı sentetik video, aynı pencere aralığı ve aynı makinede önce/sonra canlı
  resize ölçümü kaydetmek: ana-thread render süreleri, resize süresi, build
  tipi, cihaz, macOS ve libmpv sürümü.

## YAPILMAYACAK

- `video-timing-offset` değerini bütün oynatma için sıfırlamak veya normal
  oynatmanın A/V zamanlamasını gevşetmek.
- OpenGL/libmpv render mimarisini değiştirmek, yeni render thread'i kurmak veya
  başka bir playback engine eklemek.
- Aspect lock, minimum pencere boyutu ya da player kromu yerleşimini değiştirmek.
- Ölçüm yapılmadan sabit FPS veya milisaniye pass/fail eşiği koymak.

## Kanıt (DoD)

- [x] Birim test, normal çizimde hedef zamanı bekleme politikasının açık;
      `inLiveResize` çiziminde kapalı olduğunu doğruluyor.
- [x] Bağlamlı baseline raporu aynı fixture ve resize hareketinde önce/sonra
      ana-thread render sürelerini ve gözlenen takılmayı karşılaştırıyor.
- [x] Gerçek `.app` manuel checklist'i oynayan videoda sürekli köşe
      sürüklemesinin pencereyi takılmadan takip ettiğini ve videonun görünür
      kaldığını gösteriyor.
- [x] Resize bırakıldığında oynatma ilerliyor, normal render zamanlaması geri
      geliyor ve gözlenebilir A/V bozulması oluşmuyor.
- [x] Resize boyunca aspect ratio korunuyor; tam ekran giriş/çıkış ve
      videosuz yüzey regresyona uğramıyor.
- [x] Tam macOS test paketi, `.app` build'i, strict codesign ve depo doküman
      kapıları yeşil.

## Kanıt kaydı

`MPVVideoView` normal çizimde libmpv'nin hedef kare beklemesini açık
tutuyor; yalnız AppKit `inLiveResize` bildirirken
`MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME=0` geçiyor. `RenderTimingPolicyTests`
iki dalı da sabitliyor (**2/2**).

Aynı Debug `.app`, `contract-clip.mkv` ve altı eş sürükleme koşusundaki
baseline'da render sayısı `33–41`, en kötü ana-thread çağrısı **175,899
ms** idi. Düzeltmeden sonra sayı `37–51`, altı koşunun en kötüsü **17,991
ms** oldu; resize süresi iki tarafta da yaklaşık 2,4 saniye kaldı. Bunlar
eşik değil, bağlamlı baseline'dır.

Prob içermeyen son `.app`te oynayan video köşe sürüklemesini takip etti,
video görünür kaldı ve bırakınca oynatma devam etti. 16:9 oran kilidi,
`F` ile tam ekran giriş/çıkış ve video ardından audio-only siyah yüzey
doğrulandı. Tam kayıt: `evidence/M3/NEN-074-measurement.md`.

`bash scripts/test-macos.sh`: **181 test / 22 suite / 0 failure**.
`bash scripts/build-macos-app.sh`, strict codesign, `bash scripts/test.sh`,
`bash scripts/check-docs.sh` ve `bash scripts/task-index.sh --check` exit 0.
