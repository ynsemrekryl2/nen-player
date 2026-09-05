# NEN-070 — Transport yatay sınır kabulü

Tarih: **2026-09-05** · Apple Silicon · macOS 27.0 · Swift 6.3.3

## Yeniden üretim ve sonuç

Bildirilen özel yerel medya yalnız uygulamada açıldı; yolu, dosya adı ve telifli
görüntüsü bu kayda alınmadı. Pencere ve erişilebilirlik frame'leri sayısal
olarak ölçüldü.

| Durum | Pencere | Play/pause | Tam ekran | Sonuç |
|---|---:|---:|---:|---|
| Önce | 1470×796 pt | x = −6…28 | x = 1470…1500 | İki kenarda kırpılıyor |
| Sonra | 1470×796 pt | x = 22…56 | x = 1418…1448 | İki kenarda 22 pt boşluk |

Son koşuda seçili altyazı düğmesi 109 pt ile en geniş hâlindeydi; seek alanı
880 pt kaldı. Play/pause ile tam ekran görünür ve erişilebilirlik ağacında
tıklanabilir düğmelerdi.

## Otomatik kanıt

`Transport controls layout` testi gerçek `NSHostingView` yerleşiminden play,
seek ve tam ekran frame'lerini preference ölçümüyle okuyor. Matris:

- 693 pt minimum genişlik + kısa süre,
- 1470 pt genişlik + 95 dakikalık süre,
- 1470 pt genişlik + geçen sürenin bir saati aşmış biçimi,
- üçünde de seçili ve maksimuma sıkışmış altyazı etiketi.

Her durumda ilk/son düğme 22 pt iç boşlukta, seek en az 76 pt. Negatif
kontrolde eski `.layoutPriority(1)` geri kondu: test play için **16 < 21,5**,
tam ekran için **1466 > 1448,5** ölçümleriyle dört assertion kırmızısı verdi.
Doğru kod geri konduğunda suite yeniden yeşil oldu.

## Kapılar

- `bash scripts/test-macos.sh`: **151 test / 16 suite / 0 failure**.
- İlk paralel koşuda `NEN-049` ile zaten kayıtlı gerçek-libmpv ardışık medya
  yarışı bir kez kırmızı verdi; temiz ikinci tam koşu bütünüyle yeşil.
- `bash scripts/build-macos-app.sh`: exit 0.
- `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`: exit 0.
- `bash scripts/test.sh`: iki shell test dosyası yeşil.
- `bash scripts/check-docs.sh`: exit 0.
