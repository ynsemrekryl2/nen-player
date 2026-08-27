# NEN-061 — çerçevesiz krom ve cam transport kabulü

Tarih: 2026-08-27

Ortam: Apple Silicon · arm64 · macOS 27.0 (26A5416b) · Xcode 26.6
(17F113) · Swift 6.3.3 · libmpv 2.5.0 · debug, ad-hoc imzalı `.app`.

Kanıt yalnız telif-temiz sentetik fixture'larla üretildi:

- parlak yüzey: `fixtures/media/menu-clip.mkv`
- koyu yüzey: `fixtures/media/glass-dark-clip.mkv`
- koyu fixture'ın deterministik reçetesi:
  `fixtures/media/glass-dark-clip.ffmpeg.txt`

Ekran görüntüleri:

- `NEN-061-bright.png`: parlak renk çubukları üzerinde medya adı, süre, ses,
  `Türkçe` CC etiketi, cam kenar ve track'ler
- `NEN-061-dark.png`: koyu gradient üzerinde aynı yüzey ve `Kapalı` CC etiketi
- `NEN-061-fullscreen.png`: tam ekranda sistem trafik lambaları olmadan
  oynatıcı kromu

Hiçbir görüntü tam yol, query, motor adı veya özel medya metadata'sı içermiyor.

## Yüzey ve zamanlama

| Deneme | Ölçülen sonuç | Sonuç |
|---|---|---|
| Pencere modu | İçerik üst kenara kadar uzandı; standart kırmızı/sarı/yeşil düğmeler, 18 pt üst şeritte yalnız `mediaName` ve 22/20 pt dış boşluklu cam bar birlikte göründü | ✅ |
| Parlak fixture | Yerel üst gradientte `menu-clip.mkv`; HUD tint üstünde süre, %95 ses ve `Türkçe` etiketi okundu | ✅ |
| Koyu fixture | `glass-dark-clip.mkv`, süre, %95 ses ve `Kapalı` etiketi koyu videodan ayrıldı; açık kenar ve gölge kaybolmadı | ✅ |
| Oynarken otomatik gizleme | Kontroller görünürken oynatıldı; 3,2 sn sonra erişilebilirlik ağacında yalnız pencere + menü çubuğu kaldı. Bar, medya adı ve üç standart düğme birlikte yoktu | ✅ |
| Geri çağırma | Video alanındaki pointer hareketinden sonra bar, medya adı ve üç standart düğme aynı görünür duruma döndü | ✅ |
| Duraklatma | Oynatma duraklatıldı, 3,2 sn beklendi; bar, medya adı ve üç standart düğme görünür kaldı | ✅ |
| Görünmez düğme hit-test'i | Gizli durumda kapatma/tam ekran/küçültme düğümleri AX ağacından da çıktı; görünür durumda yeniden etkin düğüm oldu | ✅ |
| Tam ekran | Standart yeşil düğmeyle tam ekrana girildi; sistem trafik lambaları AX ağacından çıktı, uygulama barı çalıştı. `Esc` ile pencere moduna dönünce üç düğme geri geldi | ✅ |

Animasyon süresi ürün kodunda tek sabit olarak `.easeOut(duration: 0.24)` ve
AppKit düğme animasyonunda `0.24` saniye / `easeOut` kullanıyor. Gizleme
tamamlanınca düğmeler `isEnabled = false` ve `isHidden = true` oluyor.

## Transport davranışı

| Deneme | Ölçülen sonuç | Sonuç |
|---|---|---|
| `+5 sn` | Kalan süre `−00:14` → `−00:09` | ✅ |
| `−5 sn` | Kalan süre `−00:09` → `−00:14` | ✅ |
| Süre toggle | `00:06 / 00:20` görünümü `−00:14 / 00:20` görünümüne geçti | ✅ |
| Seek track tıklama | Track merkezine tıklama 20 sn fixture'ı `−00:10 / 00:20` konumuna taşıdı | ✅ |
| Seek sürükleme | 10 sn fixture'ta thumb sondan ortaya sürüklendi; `−00:00` → `−00:05` | ✅ |
| Volume sürükleme | Thumb sürüklemesi ses değerini `%95` → `%46` yaptı | ✅ |
| CC, seçim var | `menu-clip.mkv` otomatik Türkçe seçimi `CC  Türkçe` gösterdi | ✅ |
| CC, seçim yok | Altyazısız koyu fixture `CC  Kapalı` gösterdi | ✅ |

## Klavye ve VoiceOver checklist'i

| Deneme | Ölçülen sonuç | Sonuç |
|---|---|---|
| Odak görünürlüğü | Seek kontrolü klavye odağı aldığında sistem aksanlı 2 pt odak halkası çizildi (`NEN-061-bright.png`) | ✅ |
| Seek klavyesi | Odaklı seek üzerinde `→`, değeri 5 saniye artırdı ve commit etti | ✅ |
| Seek adjustable | AX ağacı `Oynatma konumu`, değer ve `Increment` / `Decrement` eylemlerini yayımladı | ✅ |
| Volume adjustable | AX `Decrement` eylemi `%100` → `%95`; sonraki sürükleme `%46` değerini yayımladı | ✅ |
| Düğme adları | Oynat/Duraklat, 5 saniye geri/ileri, süre toggle ve CC düğmelerinin ad/help metinleri AX ağacında ayrı ayrı okundu | ✅ |

Sonuç: özel slider'lar küçük görsel track'e karşın 22 pt etkileşim alanını,
pointer tıklama/sürükleme, klavye ayarı ve VoiceOver'ın kullandığı adjustable
eylemleri koruyor.

## NEN-046 yaşam döngüsü regresyonu

1. Fixture açıkken kırmızı kapatma düğmesi çalıştı ve pencere kapandı.
2. Uygulama listesinde `player.nen.macos` aynı anda `isRunning: true` kaldı.
3. Uygulama yeniden etkinleştirildiğinde tek temiz `Nen Player` penceresi geri
   geldi; eski transport görünmedi.
4. Son medya eylemi `glass-dark-clip.mkv` fixture'ını yeniden açtı.

Sonuç: **4/4 geçti**. NEN-046'nın pencereyi kapat / süreci yaşat / temiz
pencereyi geri getir kontratında regresyon yok.

## Otomatik kanıt

- `bash scripts/build-macos-app.sh`: çıkış 0
- `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`:
  çıkış 0
- `bash scripts/test-macos.sh`: **87 test / 10 suite**, 0 failure
- `SubtitleMenuTests`: seçim yoksa `Kapalı`, seçili token varsa girdinin
  görünen etiketi, stale token varsa güvenli `Altyazı` fallback'i
