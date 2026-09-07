---
id: NEN-077
title: Keep the minimum player surface aspect-correct across the titlebar safe area
milestone: M3
size: M
state: done
closed: 2026-09-07
depends_on: [NEN-068, NEN-073]
blocks: []
adr: [38]
---

# NEN-077 — Minimum player yüzeyi titlebar safe area boyunca oranını korur

## Sonuç

Tam ekran dışında minimuma küçültülen player'da pencere, SwiftUI root ve video
yüzeyi medyanın display aspect ratio'sunda kalır; uygulamanın ürettiği siyah
bant veya kırpılmış transport kontrolü oluşmaz.

## Kapsam

- `693×390 pt` krom tabanını safe-area içindeki kullanılabilir alan olarak
  yorumlamak ve gerçek pencere minimumunu ölçülen safe-area payıyla, medyanın
  oranını bozmadan türetmek.
- Pencerenin gerçek safe-area payını AppKit writer'dan SwiftUI root'a taşımak;
  root minimumu ile aspect lock'un aynı tam pencere boyutunu istemesini
  sağlamak.
- Açılış boyutu hesabını aynı safe-area-aware minimumla sınırlamak.
- Gerçek, gizli başlıklı `NSWindow` içinde window/root/video yüzeyi eşitliğini
  ve 57 pt transport yüksekliğini regresyon testiyle sabitlemek.

## YAPILMAYACAK

- Videoyu crop, stretch veya kullanıcı tarafından seçilen başka bir oranla
  göstermek; videonun kendi karelerine gömülü siyah bantları kırpmak.
- Tam ekran letterbox politikasını değiştirmek.
- Transport sırasını, görünürlüğünü veya `57 pt` yüksekliğini değiştirmek.
- Core, FFI veya playback engine sözleşmesini genişletmek.

## Kanıt (DoD)

- [x] Saf geometri testleri videosuz, 16:9, 4:3, 2.39:1 ve 9:16 oranlarında
      sıfır ve 32 pt titlebar safe area için krom tabanı ile aspect ratio'yu
      birlikte koruyor.
- [x] App-style gerçek `NSWindow` testi minimumda pencere, hosting root ve
      `MPVVideoView` boyutlarının eşit olduğunu; yüzey oranının display
      oranıyla aynı kaldığını ve root'un kırpılmadığını gösteriyor.
- [x] Minimum player testinde transport 57 pt yüksekliğinde ve ilk/son
      kontroller pencere sınırları içinde.
- [x] Safe-area telafisi kaldırılan negatif kontrolde gerçek pencere testi
      mevcut `693×390` pencere / `693×422` root uyuşmazlığını yakalıyor.
- [x] Gerçek `.app`te 16:9, 4:3 ve 2.39:1 minimum resize ile tam ekran
      giriş/çıkış checklist'i; player kaynaklı siyah bant ve kırpılmış kontrol
      yok.
- [x] `bash scripts/test-macos.sh`, `.app` build'i, strict codesign,
      `bash scripts/test.sh` ve doküman kapıları yeşil.

## Kanıt kaydı

- Safe-area-aware geometri ve gerçek pencere regresyonları
  `WindowGeometryTests`, `WindowGeometryWriterTests`,
  `VideoSurfaceFillTests` ve `TransportControlsLayoutTests` içinde geçti.
  Dört video oranında window/root/surface aynı display oranında; transport
  `57 pt`, seek en az `76 pt` ve kenar kontrolleri pencere içinde ölçüldü.
- Telafisiz negatif kontrol gerçek pencere testini 10 issue ile kırdı; 16:9,
  4:3, sinema ve 9:16 durumlarında oran/yüzey uyuşmazlığını yakaladı.
- Gerçek `.app` kabulü 16:9 `750–751×422`, 4:3 `693×520`, 2.39:1
  `1008×422 pt` minimumlarında geçti. Tam ekran giriş/çıkışı
  `750×422 → 1470×923 → 751×422 pt` ölçüldü.
- `bash scripts/test-macos.sh`: 202 test / 22 suite, 0 issue; `.app` build'i,
  strict codesign ve `bash scripts/test.sh` exit 0.
- Ayrıntılı ölçüm, checklist ve ekran görüntüleri:
  [`evidence/M3/NEN-077-checklist.md`](../../evidence/M3/NEN-077-checklist.md).
