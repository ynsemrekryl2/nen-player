# NEN-067 — Manuel kabul ve ölçüm kaydı

Tarih: 2026-08-30  
Makine: Apple Silicon · macOS 27.0 (26A5421a)  
Toolchain: Xcode 26.6 · Swift 6.3.3 · Rust 1.98.0 · libmpv 2.5.0  
Build: debug, ad-hoc imzalı `NenPlayer.app`

Kanıt medyası yalnız depodaki telif-temiz sentetik fixture'lardır:

- parlak ve çok altyazılı yüzey: `fixtures/media/menu-clip.mkv`
- koyu ve altyazısız yüzey: `fixtures/media/glass-dark-clip.mkv`

## Görsel fixture'lar

| Dosya | Sahne | Sonuç |
|---|---|---|
| `NEN-067-bright-1280x720.png` | 1280×720, parlak | Tek satır; bar alt ve yan kenarlara sıfır boşlukla oturuyor; seek genişliyor; süre, ses, CC, hız ve tam ekran okunuyor |
| `NEN-067-dark-1280x720.png` | 1280×720, koyu | %10 tint ve üst saç çizgisi koyu videodan ayrılıyor; çevresel kart çerçevesi/gölgesi yok |
| `NEN-067-bright-720x450.png` | 720×450 içerik alanı, parlak | Bütün kontroller tek satır; seek sıkışıyor; ses slider'ı korunuyor; `Türkçe` güvenli biçimde kısalıyor; taşma yok |
| `NEN-067-dark-720x450.png` | 720×450 içerik alanı, koyu | Aynı sıkışma davranışı ve okunabilirlik koyu fixture'da korunuyor |

Minimum sahnenin ekran görüntüsü macOS'un 32 pt trafik-lambası şeridini de
içerdiği için dosya 720×482 pikseldir; SwiftUI içerik alanı 720×450 pt'dir.
Geniş sahne dosyaları doğrudan 1280×720 pikseldir.

## Panel, pin ve yaşam döngüsü

| Senaryo | Gözlem | Sonuç |
|---|---|---|
| Hız paneli | Barın üstüne dikey boşluksuz, sağdan 26 pt içeride 334×94 pt açıldı; alt köşeler kare ve alt çizgi yok | ✅ |
| Altyazı paneli | Hız paneli açıkken CC'ye basınca hız paneli kaybolup yalnız 440×326 pt altyazı paneli kaldı | ✅ |
| Hız çağrısı | Panelden `1.5×` seçildi; bar etiketi aynı turda `1.5×` oldu | ✅ |
| Pin/auto-hide | Panel açıkken krom görünür kaldı; panel seçimi ve dış tıkta pin bırakıldı; modelin medya/fatal/pasif/shutdown temizliği testleri yeşil | ✅ |
| Buffering | Ortalanmış `Video yükleniyor` göstergesi ve hafif karartma göründü; aynı anda hız düğmesi ve panel seçenekleri tıklanabildi | ✅ |

## Girdi ve erişilebilirlik

| Senaryo | Gözlem | Sonuç |
|---|---|---|
| Ses pointer | Slider ortasına tıklama değeri `%100` → `%50` taşıdı | ✅ |
| Ses klavye/VoiceOver | `Ses düzeyi` adı, `%50` değeri ve `Increment`/`Decrement` eylemleri erişilebilirlik ağacındaydı; `Decrement` `%45` üretti | ✅ |
| Tam ekran düğmesi | Düğmeye basınca trafik lambaları kayboldu ve pencere tam ekrana geçti | ✅ |
| `F` / `Esc` | `F` aynı geçişi yaptı; `Esc` pencere moduna ve trafik lambalarına döndü | ✅ |

## Otomatik kanıt

- `swift test --package-path platforms/macos --no-parallel`: **107 test / 12
  suite, 0 failure, 36.136 s**. Buna beş hız seçeneği, başarılı/reddedilmiş hız
  çağrısı, Türkçe hata, shutdown sıfırlaması, süre biçimleri ve panel dışlanması
  dahildir.
- ADR-0037 `Subtitle safe area` suite'i: **5/5**; gerçek `TransportControls`
  fitting yüksekliği **57.0 pt** ve libmpv altyazı piksel bandı barın üstünde.
- `bash scripts/build-macos-app.sh`: exit 0.
- `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`:
  exit 0.

## Son kapı — kapandı (2026-08-31)

Kapı, engelleyen `NEN-065` kapandıktan sonra aynı ağaçta yeniden koşuldu:

| Kapı | Sonuç |
|---|---|
| `bash scripts/test-macos.sh` (paralel) | **107/107 yeşil**, 16,5 s |
| `bash scripts/build-macos-app.sh` | exit 0 |
| `codesign --verify --deep --strict .build/NenPlayer.app` | exit 0 |
| `bash scripts/check-docs.sh` | exit 0 |
| `bash scripts/test.sh` | exit 0 |

Önceki durum tarihsel kayıt olarak: paralel koşum `pinned controls stay visible
and unpin restores the hide timer` testinde deterministik kırmızıydı ve aynı
107 test `--no-parallel` ile yeşildi. Kusur NEN-067'nin kodunda değil, o testin
gerçek-saate bağlı ölçümündeydi; `NEN-065` ile kapandı ve ürün kodu değişmedi.
