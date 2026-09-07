---
id: NEN-047
title: Scope playback shortcuts to the player surface and echo them on screen
milestone: M3
size: S
state: done
closed: 2026-09-06
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

- [x] Ayarlar penceresi öndeyken `Space`, `←`, `→`, `↑`, `↓` oynatmayı
      etkilemiyor: konum ve ses değişmiyor (checklist)
- [x] Oynatma penceresi öndeyken aynı yedi kısayol çalışmaya devam ediyor
      (checklist)
- [x] Kontroller gizliyken `→` basıldığında transport geri geliyor ve yeni
      konumu gösteriyor (checklist)
- [x] `seekRelative` ve `adjustVolume` sonrası `controlsVisible == true`
      olduğunu gösteren model testi

## Kanıt kaydı

- **Uygulanan yol:** `@FocusedValue` — `PlayerFocusedValues.swift` yeni
  dosya, `PlayerRootView` `.focusedSceneValue(\.playerModel, model)`
  yayınlıyor, `PlayerCommands`'teki yedi maddenin tamamı
  `.disabled(focusedModel == nil)`. Menü maddeleri görünür kalıyor, tuşun
  kime ait olduğu değişti. `seekRelative`/`adjustVolume` artık
  `pointerMoved()` çağırıyor.
- Manuel acceptance: Apple M-serisi · arm64 · macOS 27.0 (26A5421a) ·
  Xcode 26.6 · Swift 6.3.3 · libmpv 2.5.0 · debug, ad-hoc imzalı `.app` ·
  yalnız sentetik fixture (`contract-clip.mkv`). Ayarlar öndeyken beş
  kısayolun etkisiz kalması, oynatma penceresinde çalışmaya devamı,
  kontrollerin klavye seek'inden sonra geri gelip yeni konumu göstermesi
  ve `⌘O`/kapalı pencere regresyonu (`NEN-046` #7) **4/4 geçti**. Ayrıntı:
  `evidence/M3/NEN-047-checklist.md`.
- `bash scripts/test-macos.sh` çıkış 0: **163 test / 18 suite**, 0 failure.
  Yeni testler: `a keyboard seek brings hidden controls back`,
  `a keyboard volume nudge brings hidden controls back`,
  `controls a keyboard seek re-shows still hide again while playing`,
  `full-screen state starts false and follows setFullScreen`.
- `bash scripts/build-macos-app.sh` ve
  `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`
  çıkış 0.
- **`Esc`/tam ekran kısmi kanıt:** kod `PlayerModel.isFullScreen` +
  `PlayerRootView`'da pencere ve tam-ekrana taranmış bir
  `NSEvent` local monitor ile uygulandı (`NSMenu`'nun düz `Esc` key
  equivalent'ının hiç tetiklenmediği bu incelemede ölçüldü — task'tan
  önce de var olan, bağımsız bir AppKit davranışı). Canlı `Esc` tuşu bu
  oturumda **hiçbir uygulama için** (bağımsız bir `NSOpenPanel`'in kendi
  Vazgeç-on-Esc'i dahil) teslim edilemediğinden gerçek tam-ekrandan-çıkış
  klavye ile doğrulanamadı — ortamın tuş iletim kısıtı, kod bu noktaya hiç
  ulaşmıyor. Fare ile giriş/çıkış ve `setFullScreen → isFullScreen` model
  testi doğrulandı. Ayrıntı: `evidence/M3/NEN-047-checklist.md`.
- ADR-0031 `accepted`; yeni mimari karar veya dış bağımlılık yok.
