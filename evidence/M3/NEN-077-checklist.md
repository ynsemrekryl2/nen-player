# NEN-077 kabul checklist'i

Tarih: 2026-09-07
Ortam: Apple Silicon (`arm64`), macOS 27.0 (26A5425), Xcode 26.6,
Swift 6.3.3, debug/ad-hoc imzalı `NenPlayer.app`.

## Gerçek uygulama

Her fixture uygulamanın `Dosya > Aç…` panelinden açıldı. Pencereye Accessibility
üzerinden `300×200 pt` boyutu verilerek AppKit minimumuna zorlandı. Görsel
denetimde player'ın eklediği siyah bant yoktu; video yüzeyi pencerenin dört
kenarını doldurdu. Transport bar tam yükseklikteydi ve play/pause, iki seek,
zaman, ses, altyazı, hız ve tam ekran kontrollerinin tamamı sınırlar içindeydi.

| Display oranı | Minimum pencere | Sonuç | Kanıt |
|---|---:|---|---|
| 16:9 | `750–751×422 pt` | geçti | [NEN-077-16x9.png](NEN-077-16x9.png) |
| 4:3 | `693×520 pt` | geçti | [NEN-077-4x3.png](NEN-077-4x3.png) |
| 2.39:1 | `1008×422 pt` | geçti | [NEN-077-cinema.png](NEN-077-cinema.png) |

16:9 fixture son imzalı build'de tam ekrana alınıp geri çıkarıldı. Accessibility
ölçümü `750×422 → 1470×923 → 751×422 pt` oldu; çıkıştan sonra oran kilidi,
video dolumu ve tam transport geri geldi. Fixture görüntüsündeki siyah bölgeler
videonun test desenine gömülüdür; yüzey çevresinde player kaynaklı bant yoktur.

## Otomatik regresyon ve negatif kontrol

- Saf geometri matrisi videosuz, 16:9, 4:3, 2.39:1 ve 9:16 durumlarını hem
  `0 pt` hem `32 pt` titlebar payıyla doğruladı. Tam içerik minimumu krom
  gereksinimini karşılarken oranı korudu; SwiftUI minimumu payı yalnız bir kez
  çıkardı.
- Gizli başlıklı gerçek `NSWindow` matrisi dört video oranında pencere,
  `NSHostingView` ve `MPVVideoView` sınırlarını eşledi. Aynı test transport
  yüksekliğini `57 pt`, kenar kontrollerini sınırlar içinde ve seek'i en az
  `76 pt` ölçtü.
- Tam ekran sırasında son pencere safe-area ölçümünün korunması ayrıca
  regresyon testiyle sabitlendi.
- Negatif kontrolde `minimumLayoutSize` safe-area telafisini bilerek yok saydı.
  Gerçek pencere testi 10 issue ile kırıldı: 16:9 `694×422`, 4:3 `693×552`,
  sinema `933×422` pencereleri display oranını kaybetti; 9:16 yüzeyi
  `693×769` iken pencere `693×1232` kaldı. Doğru uygulama hemen geri getirildi
  ve aynı matris geçti. İlk kök-neden probu da `693×390` pencereye karşı
  `693×422` hosting root taşmasını yeniden üretti.

## Kapılar

- `bash scripts/test-macos.sh`: 202 test / 22 suite, 0 issue, exit 0.
- `bash scripts/build-macos-app.sh`: exit 0; ad-hoc imzalı `.app` üretildi.
- `codesign --verify --deep --strict --verbose=2`: `valid on disk` ve
  `satisfies its Designated Requirement`.
- `bash scripts/test.sh`: 2 shell test dosyası, tamamı geçti, exit 0.
- `git diff --check` ve kapanış `bash scripts/check-docs.sh`: exit 0.
