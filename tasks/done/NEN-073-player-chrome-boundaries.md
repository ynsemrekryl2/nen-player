---
id: NEN-073
title: Player chrome boundary behaviour
milestone: M3
size: M
state: done
depends_on: []
blocks: []
adr: [37, 38]
---

# NEN-073 — Player kromunun pencere sınırı davranışı

## Sonuç

Üst gölgelendirme pencerenin en üstünden başlar, oynatma sırasında fare player'dan çıkınca krom hemen gizlenir ve pencere transport kontrollerini bozacak kadar küçültülemez.

## Kapsam

- Üst gradient'i trafik ışıklarının arkasındaki top safe area dahil pencerenin
  en üst kenarından başlatmak; medya başlığı ile etkileşimli kontrolleri safe
  area içinde bırakmak.
- Video oynarken fare player alanından çıktığında ve kontroller pinli değilken
  üst/alt kromu mevcut 0,24 saniyelik fade ile hemen gizlemek.
- Fare çıkışında bekleyen gizleme task'ını iptal etmek ve player dışındaki
  sistem imlecini gizlememek; duraklatılmış oynatma ile açık altyazı/hız
  panelinin görünürlük davranışını korumak.
- `WindowGeometry.chromeBase` genişliğini 693 pt yapmak ve minimumları aynı
  aspect-ratio formülüyle türetmek: 16:9 `693x390`, 4:3 `693x520`, 2.39:1
  `932x390`, 9:16 `693x1232`; video geometrisi yokken minimum `693x390`.
- Minimum genişlikte kısa/uzun süre etiketleri, seek, volume, altyazı seçimi
  ve iki kenardaki düğmelerin tek satır içinde kalmasını sağlamak.

## YAPILMAYACAK

- Transport kontrol sırasını değiştirmek, kontrol saklamak veya responsive
  alternatif bir menü eklemek.
- Seek ya da volume slider'ının mevcut minimum genişliğini küçültmek.
- Paused durumunda kontrolleri otomatik gizlemek.
- Pencerenin aspect lock politikasını veya core/FFI playback sözleşmesini
  değiştirmek.
- Canlı resize performansını düzeltmek; bu `NEN-074` kapsamıdır.

## Kanıt (DoD)

- [x] Window-backed görsel kontrol veya screenshot, gradient'in trafik
      ışıklarının arkasından pencerenin üst kenarına kadar ulaştığını ve medya
      başlığının safe area içinde kaldığını gösteriyor.
- [x] Model testleri oynarken ve pinli değilken `pointerLeft()` çağrısının
      kromu hemen gizlediğini; paused ve pinli durumların görünür kaldığını
      kanıtlıyor.
- [x] Fare player dışına çıktığında sistem imleci görünür kalıyor.
- [x] 693 pt genişlikte kısa ve iki saatlik süre, kalan/toplam modu ve uzun
      altyazı etiketi matrisinde ilk/son kontrol pencere içinde, seek genişliği
      en az 76 pt.
- [x] Geometry testleri yeni 16:9, 4:3, 2.39:1, 9:16 ve videosuz minimumlarını;
      her video minimumunun kendi aspect ratio'sunu koruduğunu doğruluyor.
- [x] Tam macOS test paketi, `.app` build'i, strict codesign ve depo doküman
      kapıları yeşil.

## Kanıt kaydı

- `evidence/M3/NEN-073-checklist.md`: gerçek `.app` üzerinde gradient, safe-area
  başlık, pointer çıkışı, paused/panel görünürlüğü ve dış imleç checklist'i;
  `NEN-073-{gradient,mouse-exit,paused,panel}.png` görsel kanıtları.
- `PlayerModelTests`: pointer çıkışında anında gizleme, paused/pinned koruması
  ve bekleyen gizleme timer'ı iptali yeşil.
- `TransportControlsLayoutTests`: 693 pt kısa/iki saatlik toplam-kalan süre ve
  uzun altyazı etiketi matrisi; kenar kontrolleri içeride, seek ≥ 76 pt.
- `WindowGeometryTests` ve `WindowGeometryWriterTests`: 16:9 `693×390`, 4:3
  `693×520`, 2.39:1 `932×390`, 9:16 `693×1232`, videosuz `693×390` yeşil.
- `bash scripts/test-macos.sh`: 178 test / 21 suite / 0 failure.
- `bash scripts/build-macos-app.sh`: exit 0; `codesign --verify --deep --strict
  platforms/macos/.build/NenPlayer.app`: exit 0.
- `bash scripts/test.sh` ve `bash scripts/check-docs.sh`: exit 0.
