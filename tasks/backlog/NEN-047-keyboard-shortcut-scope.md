---
id: NEN-047
title: Scope playback shortcuts to the player surface and echo them on screen
milestone: M3
size: S
state: backlog
depends_on: [NEN-024]
blocks: [NEN-037]
adr: [31]
---

# NEN-047 — Scope playback shortcuts to the player surface and echo them on screen

## Sonuç

Yedi oynatma kısayolu yalnız oynatma penceresi öndeyken çalışır ve klavyeyle
yapılan her transport eylemi kontrolleri ekrana geri getirir.

## Kapsam

- Kısayolların **menü key equivalent'ı** olmaktan çıkarılıp oynatma yüzeyine
  bağlanması (`@FocusedValue` ile pencereye bağlama ya da `PlayerRootView`
  üzerinde view-scoped tuş işleme). Menü maddeleri görünür kalır; değişen,
  tuşun **kime** ait olduğu
- `seekRelative` ve `adjustVolume`'un kontrolleri görünür kılması —
  `pointerMoved()` ile aynı davranış: göster ve gizleme sayacını yeniden başlat
- `Esc`'in yalnız tam ekranda anlam taşıması; başka bağlamda menüden
  yakalanmaması

## YAPILMAYACAK

- Kısayol kümesini değiştirmek — yedi kısayol `NEN-024` kapsamında sabitlendi
- Kullanıcıya kısayol tanımlatmak
- Ayrı bir OSD katmanı tasarlamak — mevcut transport yüzeyi yeterli
- Pencere yaşam döngüsü → `NEN-046`

## Neden ayrı task

`NEN-024` incelemesinde çalışan `.app` üzerinde iki ölçüm yapıldı:

1. **Ayarlar penceresi öndeyken** `↓` beş kez basıldığında ses seviyesi düştü
   ve `←` konumu `00:30`'dan `00:25`'e aldı. `PlayerCommands` yedi kısayolu
   modifier'sız menü key equivalent'ı olarak kaydediyor; AppKit bunları key
   window'un responder zincirinden önce dağıtıyor. `NEN-037` aynı sahneye iki
   dil seçici koyacak — `↑`/`↓` seçicide gezinemeyecek, `Space` popup'ı
   açamayacak. Bu yüzden `NEN-037` bu task'a bağlandı.
2. Oynatma sırasında kontroller gizlendikten sonra `→` basıldı: konum gerçekten
   ilerledi (fixture sayacı 14 → 21) ama transport geri gelmedi — kullanıcı
   seek'in olup olmadığını ekrandan **göremiyor**.

## Kanıt (DoD)

- [ ] Ayarlar penceresi öndeyken `Space`, `←`, `→`, `↑`, `↓` oynatmayı
      etkilemiyor: konum ve ses değişmiyor (checklist)
- [ ] Oynatma penceresi öndeyken aynı yedi kısayol çalışmaya devam ediyor
      (checklist)
- [ ] Kontroller gizliyken `→` basıldığında transport geri geliyor ve yeni
      konumu gösteriyor (checklist)
- [ ] `seekRelative` ve `adjustVolume` sonrası `controlsVisible == true`
      olduğunu gösteren model testi

## Kanıt kaydı

<!-- done olurken doldurulacak -->
