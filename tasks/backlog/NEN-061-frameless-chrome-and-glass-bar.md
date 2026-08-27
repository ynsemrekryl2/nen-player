---
id: NEN-061
title: Frameless chrome and glass transport bar
milestone: M3
size: M
state: backlog
depends_on: [NEN-024]
blocks: [NEN-062, NEN-063]
adr: [31]
---

# NEN-061 — Çerçevesiz krom ve cam transport barı

## Sonuç

Pencere kromu, medya adı ve okunabilir cam transport barı tek bir kontrol
yüzeyi olarak birlikte görünür ve kaybolur.

## Kapsam

- `Window` sahnesi hidden-title-bar kullanır; içerik video boyunca uzanır,
  pencere başlığı ve `representedURL = nil` gizlilik kuralı korunur.
- Standart trafik lambaları tam ekran dışında `controlsVisible` ile birlikte
  0,24 saniyede görünür/kaybolur; görünmezken tıklanamaz.
- Üst şerit yalnız `mediaName` gösterir; parlak karede yerel koyu gradient ile
  okunur ve kalite rozeti için `NEN-063`'ün dolduracağı alanı bırakır.
- Alt bar mockup'ın geometri değerlerini taşır: 24 pt radius, 22 pt yatay / 20
  pt alt boşluk, 18/20/14 pt iç boşluk, koyu HUD materyali ve sistem aksanı.
- Scrubber ve volume özel ince track/büyük hit alanı ile çizilir; sürükleme,
  klavye ve VoiceOver ayarlama davranışı korunur.
- Alt satır oynat/duraklat, `-5 sn`/`+5 sn`, ses, tıklanabilir süre ve CC ikonu
  + seçili altyazı etiketidir; sürekli playback-state etiketi kaldırılır.
- `SubtitleMenuPresentation.selectionLabel(...)` görünen CC metninin tek saf
  üreticisidir.

## YAPILMAYACAK

- Altyazı panelinin popover dışına taşınması → `NEN-062`
- Teknik kalite metadata hattı ve rozeti → `NEN-063`
- Klavye kapsamı ve `Esc` davranışı → `NEN-047`
- Tam ekranda altyazı çizim kusuru → `NEN-060`
- AirPlay, PiP, settings, önceki/sonraki, seek preview, buffered progress,
  chapter bilgisi, app icon veya global aksan rengi
- Tam yol, query veya motor adının herhangi bir yüzeyde görünmesi

## Kanıt (DoD)

- [ ] `selectionLabel`: seçim yokken `Kapalı`, seçili gömülü track varken
      girdinin görünen etiketi (`SubtitleMenuTests`)
- [ ] `bash scripts/test-macos.sh` yeşil
- [ ] Parlak ve koyu fixture karelerinde süre, ses ve CC etiketi okunuyor
- [ ] Bar + medya adı + trafik lambaları oynarken birlikte kaybolup geliyor,
      duraklıyken kalıcı; tam ekranda sistem düğmeleriyle yarışılmıyor
- [ ] Özel seek/volume kontrolleri sürükleme, tıklama, klavye ve VoiceOver
      checklist'ini geçiyor
- [ ] `NEN-046` regresyonu yok: pencere kapanıyor, uygulama yaşıyor ve yeniden
      açılıyor

## Kanıt kaydı

<!-- Kapanışta gerçek test ve acceptance çıktısıyla doldurulur. -->
