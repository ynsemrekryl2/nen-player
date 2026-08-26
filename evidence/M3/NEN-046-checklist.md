# NEN-046 — macOS pencere yaşam döngüsü checklist

Tarih: 2026-08-26

Ortam: Apple M5 · arm64 · macOS 27.0 (26A5416b) · Xcode 26.6
(17F113) · Swift 6.3.3 · libmpv 2.5.0 · debug, ad-hoc imzalı `.app`.

Kanıt medyası yalnız telif-temiz sentetik fixture'lardır:
`fixtures/media/contract-clip.mkv` ve `fixtures/media/bitmap-subs-clip.mkv`.

## Manuel acceptance

1. `bash scripts/build-macos-app.sh` çıkış 0 verdi ve
   `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`
   doğrulamadan geçti.
2. Uygulama açıldığında tek `Nen Player` penceresi boş durumu gösterdi;
   `bitmap-subs-clip.mkv` son medya eyleminden açılıp oynadı.
3. Oynatma sürerken kırmızı kapatma düğmesine basıldı. Pencere kapandı; aynı
   Nen Player PID'i yaşamaya devam etti.
4. Dock'tan uygulama etkinleştirildi. Tek oynatıcı penceresi temiz boş durumla
   geri geldi; eski süre ve transport kalmadı.
5. Son medya eylemine basıldığında fixture aynı süreçte yeniden yüklendi;
   transport `Duraklat`, `Oynatılıyor` ve ilerleyen süreyi gösterdi.
6. Oynatma sürerken `⌘W` basıldı. Pencere kapandı, aynı PID yaşamaya devam
   etti; Dock penceresini tekrar boş durumda geri getirdi.
7. Pencere kapalıyken `⌘O` basıldı. Oynatıcı penceresi geri geldi ve dosya
   seçici açıldı; `contract-clip.mkv` seçilince `Duraklat`, `Oynatılıyor` ve
   `00:01 / 00:30` gösterdi.
8. `⌘Q` basıldı. Nen Player PID'i süreç listesinden kayboldu.

Sonuç: **8/8 geçti**. Kırmızı düğme ve `⌘W` yalnız oynatıcı penceresini
kapatır; Dock ve `⌘O` aynı uygulama sürecinde çalışan pencereyi geri getirir;
yalnız `⌘Q` uygulamadan çıkar.

## Otomatik kanıt

`swift test --package-path platforms/macos --no-parallel` çıkış 0:

- **31 test / 6 suite**, 0 failure
- `shutdown clears stale state and window resume opens pending media`
- `window resume after shutdown restarts event polling`

İlk test kapanışın eski medya, state, pozisyon ve duration'ı temizlediğini ve
oturum yokken seçilen dosyanın saklanan video yüzeyi resume edildiğinde
yutulmadan yüklendiğini kanıtlar. İkinci test resume'un polling'i yeniden
başlattığını ve `Ready` olayının otomatik oynatmaya ulaştığını kanıtlar.

Tam paralel platform koşusunda NEN-046'dan bağımsız gerçek-libmpv seek testi
iki kez `0 ms` ile kırmızı oldu; aynı test izole **5/5**, seri tam paket
**31/31** geçti. Runner izolasyonu ürün task'ına karıştırılmadı ve `NEN-049`
olarak backlog'a kaydedildi.
