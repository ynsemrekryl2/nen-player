# NEN-054 — Ses düzeyi değişimi yapıldığı anda duyulur

Tarih: 2026-08-27

Ortam: Apple arm64 · macOS 27.0 (26A5416b) · Xcode 26.6 (17F113) ·
Swift 6.3.3 · libmpv 2.5.0 · mpv 0.41.0 (ölçüm için CLI).

Kanıt medyası yalnız telif-temiz sentetik fixture'dır:
`fixtures/media/contract-clip.mkv`.

## 1. Ölçüm — üç adaydan ikisi elendi

Task ölçümle başlaması şartıyla açılmıştı. Geçici ölçüm testi (commit
edilmedi) gerçek libmpv oturumunda, pump 33 Hz'de çekerken ve medya oynarken
art arda 120 `setVolume` yazdı — yani bir sürüklemenin tam yükü:

```
MEASURE setVolume round trip us: p50=8.6 p99=25.9 max=35.3 total=1103.5
MEASURE mpv_set_property us:   p50=7.3 p99=17.2 max=17.9
MEASURE property audio-buffer = 0.200000
MEASURE property ao-volume = <unavailable>   (headless: current-ao = null)
```

- **Aday (b) — sürükleme selinin birikmesi: elendi.** Tam tur p50 **8.6 µs**;
  120 örneğin tamamı **1.1 ms** sürüyor. 120 Hz'lik bir sürükleme ana iş
  parçacığından saniyede ~1 ms alıyor. Birikecek bir şey yok.
- **Aday (a) — kaydırıcının geriden gelmesi: elendi.** Aynı sayı: çağrı aynı
  turda dönüyor, `volume` değeri gecikmeden yazılıyor. Bu yüzden kabukta
  iyimser yazma **eklenmedi** — düzeltilecek bir gecikme yok (Kural 5).
- **Aday (c) — ses hattı: kaldı.** `audio-buffer` varsayılan **0.2 s**.

mpv'nin kendi kılavuzu mekanizmayı adıyla söylüyor
(`man mpv`, `--audio-buffer=<seconds>`): tamponu büyütmek *"may make
soft-volume and other filters react slower"*, varsayılan *"0.2 (200 ms)"*.
Aynı madde şunu da diyor: *"This option should be used for testing only."*
Bu yüzden tampon **küçültülmedi**; yazılım ses düzeyini tamponun önünde
bırakmak yerine ses düzeyi tamponun **ötesine** taşındı.

## 2. Ayırt edici deney — kullanıcı

Kalan ayrım ancak duyulabilir: sürükleme seli mi, ses hattı mı. Kullanıcı
düzeltilmemiş yapıda ↑/↓ ile **tek adım** (%5, tek property yazması) denedi ve
"tek adım da belirgin geç" diye bildirdi. Tek yazmanın da geç duyulması aday
(b)'yi kullanıcı tarafından da eler ve (c)'yi bırakır.

## 3. Ölçüm — `ao-volume` gerçekten kullanılabilir mi

Headless testte `ao=null` olduğu için `ao-volume` yok. Gerçek çıkışla ne olduğu
mpv CLI'ye IPC ile sorularak ölçüldü:

```
MEASURE current-ao = avfoundation
MEASURE ao-volume = 100.0          ← var ve yazılabilir
MEASURE audio-buffer = 0.2
```

İki risk ayrıca ölçüldü:

```
MEASURE ao-volume after set = 40.0
MEASURE ao-volume after reload = 40.0     ← dosya yeniden yüklenince korunuyor
MEASURE first:  ao-volume at startup = 100.0
MEASURE second: ao-volume at startup = 100.0   ← süreç yeniden başlayınca 100
```

- Dosya değişiminde düzey **korunuyor**: ses birden %100'e fırlamıyor.
- Süreç yeniden başlayınca **hatırlanmıyor**: kabuğun UI'daki varsayılanı (%100)
  ile gerçek düzey açılışta uyuşuyor, yani "kaydırıcı %100 gösterirken ses %40"
  durumu oluşmuyor.

Ölçülmeyen: çıkışın tamamen yok olup geri geldiği durum (sesi olmayan bir
medyadan sesli bir medyaya geçiş). Bilinmiyor olarak kaydediliyor.

## 4. Yapılan

`MPVPlaybackEngine.setVolume` düzeyi **cihazda** ayarlıyor (`ao-volume`), yani
mpv'nin ses tamponunun ötesinde. Yazılım `volume` yalnız çıkış yokken (boşta,
headless) yedek. Cihaz düzeyi devraldığında filtre zinciri 100'e alınıyor —
ikisi aynı anda düzey tutarsa çarpılırlardı.

`nen-ports` port yüzeyi **değişmedi**, dolayısıyla ADR gerekmedi (Kural 4).

## 5. Otomatik testler ve sınırı

`bash scripts/test-macos.sh` çıkış 0: **49 test / 7 suite**, 0 failure.

Yeni test: `withoutAnAudioOutputTheFilterChainHoldsTheLevel` (`VolumeTests`).
Testler `ao=null` ile koştuğu için **yalnız yedek yol** otomatik kanıtlanabiliyor;
cihaz yolunun kanıtı §3'teki ölçüm ile §6'daki elle geçiştir. Bu sınır
kasıtlıdır ve burada yazılıdır.

**Negatif kontrol:** yedek kaldırılıp yalnız `ao-volume` denendiğinde test iki
beklentiyle kırmızı — çıkış yokken kullanıcının düzeyi hiçbir yerde tutulmuyor:

```
✘ Expectation failed: try engine.double("volume") == 50
✘ Expectation failed: try engine.double("volume") == 0
```

## 6. Elle acceptance

Ekran kontrolü bu oturumda reddedilmişti, elle geçişi kullanıcı kendisi yaptı.
Uygulama `bash scripts/build-macos-app.sh` ile derlendi (çıkış 0).

- [x] Kullanıcı düzeltilmiş yapıda ↑/↓ ile ses değiştirdi ve **"şu anda doğru
      çalışıyor görünüyor"** diye bildirdi.

Sınırı: bu tek bir kullanıcı onayıdır, ölçülmüş bir gecikme sayısı değildir.
Gecikmenin sayısal öncesi/sonrası ölçümü **yapılmadı** — kulakla duyulan
farkın kaynağı §1 ve §3'teki ölçümlerle belirlendi.
