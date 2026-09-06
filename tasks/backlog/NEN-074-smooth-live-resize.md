---
id: NEN-074
title: Smooth live video resize
milestone: M3
size: M
state: backlog
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

- [ ] Birim test, normal çizimde hedef zamanı bekleme politikasının açık;
      `inLiveResize` çiziminde kapalı olduğunu doğruluyor.
- [ ] Bağlamlı baseline raporu aynı fixture ve resize hareketinde önce/sonra
      ana-thread render sürelerini ve gözlenen takılmayı karşılaştırıyor.
- [ ] Gerçek `.app` manuel checklist'i oynayan videoda sürekli köşe
      sürüklemesinin pencereyi takılmadan takip ettiğini ve videonun görünür
      kaldığını gösteriyor.
- [ ] Resize bırakıldığında oynatma ilerliyor, normal render zamanlaması geri
      geliyor ve gözlenebilir A/V bozulması oluşmuyor.
- [ ] Resize boyunca aspect ratio korunuyor; tam ekran giriş/çıkış ve
      videosuz yüzey regresyona uğramıyor.
- [ ] Tam macOS test paketi, `.app` build'i, strict codesign ve depo doküman
      kapıları yeşil.

## Kanıt kaydı

<!-- done olurken gerçek test çıktısı ve baseline raporu ile doldurulacak -->
