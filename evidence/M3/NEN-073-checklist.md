# NEN-073 — Player kromu sınır kabulü

Tarih: **2026-09-06** · Apple Silicon · macOS 27.0 · Xcode 26.6 · Swift 6.3.3

## Görsel kanıt

Gerçek ad-hoc imzalı `.app`, depodaki telif-temiz `contract-clip.mkv` fixture'ı
ile 693×390 pt içerik alanında çalıştırıldı. Ekran görüntüleri yalnız player
penceresini içerir; özel dosya yolu veya medya URL'si kaydedilmedi.

| Dosya | Senaryo | Sonuç |
|---|---|---|
| `NEN-073-gradient.png` | Oynatma sırasında krom görünür | Gradient trafik ışıklarının arkasındaki pencere üst kenarına ulaşıyor; medya başlığı safe area içinde; alt transport tek satırda ve iki kenar pencere içinde |
| `NEN-073-mouse-exit.png` | Oynarken pointer player dışına çıktı | Üst ve alt krom mevcut fade ile hemen gizlendi; video yüzeyi kaldı |
| `NEN-073-paused.png` | Oynatma duraklatıldı | Play düğmesi ve tüm transport kontrolleri görünür kaldı |
| `NEN-073-panel.png` | Altyazı paneli açık | Panel ve transport birlikte görünür; panel pin'i dış pointer hareketinde korunuyor |

## Manuel davranış checklist'i

| Senaryo | Gözlem | Sonuç |
|---|---|---|
| Oynarken içeride pointer hareketi | Krom görünür, 2,5 s hareketsizlikte mevcut 0,24 s fade ile gizleniyor | ✅ |
| Oynarken player dışına çıkış | Bekleyen gizleme beklenmeden krom gizleniyor; `NEN-073-mouse-exit.png` | ✅ |
| Player dışındaki imleç | Sistem imleci dış yüzeyde görünür kalıyor; shell `NSCursor.hide()` çağırmıyor | ✅ |
| Duraklatma | 3 s bekleme sonrasında transport ve üst başlık görünür; `NEN-073-paused.png` | ✅ |
| Açık altyazı paneli | Panel açıkken pointer çıkışı kromu/paneli gizlemiyor; `NEN-073-panel.png` | ✅ |

## Otomatik kanıt

- `PlayerModelTests`: oynama/duraklatma/pin görünürlüğü ve pointer çıkışında
  bekleyen timer iptali yeşil.
- `TransportControlsLayoutTests`: 693 pt kısa süre, iki saatlik süre,
  toplam/kalan modları ve uzun altyazı etiketi matrisinde play/full-screen
  sınırları içeride, seek ≥ 76 pt.
- `WindowGeometryTests` ve `WindowGeometryWriterTests`: 16:9 `693×390`, 4:3
  `693×520`, 2.39:1 `932×390`, 9:16 `693×1232` ve videosuz `693×390`; tüm
  minimumlar kendi oranını koruyor.

## Kapılar

- `bash scripts/test-macos.sh`: **178 test / 21 suite / 0 failure**.
- `swift test --package-path platforms/macos --filter
  'PlayerModelTests|TransportControlsLayoutTests|WindowGeometryTests|WindowGeometryWriterTests'`:
  **82 test / 4 suite / 0 failure**.
- `bash scripts/build-macos-app.sh`: exit 0; debug ad-hoc imzalı `.app` üretildi.
- `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`:
  exit 0.
- `bash scripts/test.sh`: shell, doctor ve doküman kapıları yeşil.
- `bash scripts/check-docs.sh`: exit 0.
