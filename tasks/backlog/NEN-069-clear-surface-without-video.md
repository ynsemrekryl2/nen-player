---
id: NEN-069
title: Clear the video surface when the medium has no video
milestone: M3
size: S
state: backlog
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

- [ ] `NEN-066`'nın offscreen capture desenini kullanan bir test: videolu bir
      medyadan videosuz bir medyaya geçildiğinde yakalanan karede önceki
      medyadan kalan piksel yok.
- [ ] Negatif kontrol: temizlik kaldırıldığında testin kırmızıya döndüğü
      gösterilir — yoksa test boşuna yeşil olur.
- [ ] `bash scripts/test-macos.sh` yeşil.

## Kanıt kaydı

<!-- done olurken doldurulacak -->
