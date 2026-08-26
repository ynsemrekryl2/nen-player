# NEN-046 — macOS pencere yaşam döngüsü checklist

Tarih: 2026-08-26

Ortam: Apple M5 · arm64 · macOS 27.0 (26A5416b) · Xcode 26.6
(17F113) · Swift 6.3.3 · libmpv 2.5.0 · debug, ad-hoc imzalı `.app`.

Kanıt medyası yalnız telif-temiz sentetik fixture'dır:
`fixtures/media/contract-clip.mkv`.

## Manuel acceptance

1. `bash scripts/build-macos-app.sh` çıkış 0 verdi ve
   `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`
   doğrulamadan geçti.
2. Uygulama açıldığında tek `Nen Player` penceresi boş durumu gösterdi.
3. `contract-clip.mkv` dosya seçiciden açıldı. Pencere başlığı yalnız basename
   oldu; transport `Duraklat`, `Oynatılıyor` ve `00:00 / 00:30` gösterdi.
4. Oynatma sürerken pencerenin kırmızı kapatma düğmesine basıldı. Uygulama
   listesindeki Nen Player girdisi tamamen kayboldu; penceresiz, menüsü yaşayan
   süreç kalmadı.
5. `.app` yeniden açıldığında temiz boş durum geldi. Eski süre ve transport
   görünmedi; yalnız `contract-clip.mkv` son medya eylemi vardı.
6. Son medya eylemine basıldığında fixture yeniden yüklendi; transport tekrar
   `Duraklat` ve `Oynatılıyor` gösterdi. Video yüzeyi çalışan yeni oturuma
   bağlandı.

Sonuç: **6/6 geçti**. Uygulamanın seçtiği yaşam döngüsü "son pencere kapanınca
sonlan"dır; bu nedenle kullanıcı penceresiz ve komutları etkisiz bir süreçte
kalamaz.

## Otomatik kanıt

`bash scripts/test-macos.sh` çıkış 0:

- **31 test / 6 suite**, 0 failure
- `shutdown clears stale playback state and a later attach opens pending media`
- `reattaching after shutdown restarts event polling`

İlk test kapanışın eski medya, state, pozisyon ve duration'ı temizlediğini ve
oturum yokken seçilen dosyanın yeni yüzey bağlandığında yutulmadan yüklendiğini
kanıtlar. İkinci test yeni bağlanmanın polling'i yeniden başlattığını ve
`Ready` olayının otomatik oynatmaya ulaştığını kanıtlar.
