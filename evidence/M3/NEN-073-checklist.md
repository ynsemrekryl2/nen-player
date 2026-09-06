# NEN-073 — Player kromu sınır kabulü

Tarih: **2026-09-06** · Apple Silicon · macOS 27.0 · Xcode 26.6 · Swift 6.3.3

## Düzeltici yeniden doğrulama

İlk kapanışın minimum pencere kanıtı geçersizdi: `NEN-073-gradient.png`
kontrollerin 693×390'da sığdığını gösteriyor, fakat pencerenin bu değerin altına
indirilemediğini göstermiyordu. Kullanıcı güncel build'i `⌘Q` sonrasında
başlatıp pencereyi 693 pt altına sürükleyebildi; task yeniden açıldı.

Kök neden iki ayrı minimum sahibiydi. `WindowGeometryWriter` AppKit
`contentMinSize` değerini yazarken `PlayerRootView` SwiftUI'a `0×0` minimum
bildiriyordu. Normal `Window` sahnesinin içerikten türettiği minimum, AppKit
yazısını etkisiz bırakıyordu. Düzeltmede root view aspect-correct minimumu
bildiriyor, sahne açıkça `.windowResizability(.contentMinSize)` kullanıyor ve
AppKit writer yalnız aspect lock ile açılış boyutunu yönetiyor.

### Negatif kontrol

Yeni `playerRootAdvertisesTheWindowMinimum` testi düzeltmesiz `0×0` root ile
ayrı koşuldu: **1 test / 10 expectation failure**. Beş senaryonun tamamı
beklenen minimumdan küçük fitting size üretti. Aynı test düzeltme geri
konduğunda **1/1 yeşil** oldu; yani test yalnız hesaplanan sayıyı değil,
SwiftUI root'un pencereye bildirdiği gerçek minimumu ölçüyor.

### Gerçek `.app` pencere sürüklemesi

Ad-hoc imzalı uygulama tam kapatılıp yeniden build edildi. Pencere kontrollü
bir konuma alındı; CoreGraphics fare olaylarıyla sağ alt köşe 300×200 hedefinin
ötesine sürüklendi ve Accessibility üzerinden sonuç ölçüldü.

| Medya | Sürükleme öncesi frame | Sürükleme sonrası frame | Sonuç |
|---|---:|---:|---|
| 16:9 `contract-clip.mkv` | 693×422 | **693×390** | Genişlik 693 pt altında değil; oran korunuyor |
| 4:3 `aspect-4x3-clip.mkv` | 693×552 | **693×520** | Genişlik 693 pt altında değil; oran korunuyor |

Videosuz pencerede ayrıca 300×200 Accessibility resize isteği verildi; pencere
**693×422** frame'de kaldı. Cinema fixture aynı istekten sonra **931×422**
frame'de kaldı; fixture'ın gerçek display oranındaki bir puanlık fark saf
2.39:1 hesabının test edilen **932×390** sonucunu değiştirmiyor.

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

- `playerRootAdvertisesTheWindowMinimum`: videosuz, 16:9, 4:3, 2.39:1 ve 9:16
  senaryolarında SwiftUI root fitting size'ı sırasıyla `693×390`, `693×390`,
  `693×520`, `932×390`, `693×1232`.
- `PlayerModelTests`: oynama/duraklatma/pin görünürlüğü ve pointer çıkışında
  bekleyen timer iptali yeşil.
- `TransportControlsLayoutTests`: 693 pt kısa süre, iki saatlik süre,
  toplam/kalan modları ve uzun altyazı etiketi matrisinde play/full-screen
  sınırları içeride, seek ≥ 76 pt.
- `WindowGeometryTests` ve `WindowGeometryWriterTests`: 16:9 `693×390`, 4:3
  `693×520`, 2.39:1 `932×390`, 9:16 `693×1232` ve videosuz `693×390`; tüm
  minimumlar kendi oranını koruyor.

## Kapılar

- Düzeltici koşu `bash scripts/test-macos.sh`: **179 test / 21 suite / 0 failure**.
- `swift test --package-path platforms/macos --filter
  'TransportControlsLayoutTests|WindowGeometryTests|WindowGeometryWriterTests'`:
  **32 test / 3 suite / 0 failure**.
- `bash scripts/build-macos-app.sh`: exit 0; debug ad-hoc imzalı `.app` üretildi.
- `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`:
  exit 0.
- `bash scripts/test.sh`: shell, doctor ve doküman kapıları yeşil.
- `bash scripts/check-docs.sh`: exit 0.
