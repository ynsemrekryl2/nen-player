# NEN-053 — Bayat konum olayı inmiş bir seek'i ezmez

Tarih: 2026-08-26

Ortam: Apple arm64 · macOS 27.0 (26A5416b) · Xcode 26.6 (17F113) ·
Swift 6.3.3 · libmpv 2.5.0.

Kanıt medyası yalnız telif-temiz sentetik fixture'dır:
`fixtures/media/contract-clip.mkv`.

## 1. Ölçüm — semptom gerçek libmpv ile üretildi

İlk teşhis "mpv seek'i servis edene kadar bayat `time-pos` yayınlar, kabuk
onları uygular" idi. **Ölçüm bunu yalanladı:** geçici bir ölçüm testi
(commit edilmedi) 5 ms çözünürlükte drain ederek gösterdi ki mpv, seek komutunu
alır almaz `time-pos`'u hedefe taşıyor — seek'ten 5 ms sonra gelen ilk rapor
zaten `20000`. Kuyruk da yardım ediyor: `PositionChanged` coalescing olduğu
için eski değer yenisi geldiğinde kuyruktan siliniyor.

Kusur bunun yerine **bakma anındaydı.** Kaydırıcı tutulurken AppKit iç içe bir
izleme döngüsü çalıştırır ve kabuğun 50 ms'lik poll'u aç kalır; bırakma anında
poll aynı runloop turunda uyanır, yani seek komutundan **mikrosaniyeler**
sonra. O anda kuyruktaki en yeni konum hâlâ sürükleme öncesinin oynatma
başıdır — mpv'nin yeni `time-pos`'u henüz olay iş parçacığından çıkmamıştır.

Ölçüm bu sırayı birebir kurdu (1 sn boyunca drain edilmedi, sonra seek, sonra
aynı turda drain):

```
MEASURE from 2000 ms
MEASURE holding the knob for 1000 ms without draining
MEASURE +0 ms  positionChanged(positionMs: 2800)
MEASURE +0 ms  position(2800)
MEASURE +8 ms  seekCompleted(20000), position(20000)
MEASURE +163 ms  position(20200)
MEASURE +370 ms  position(20400)
MEASURE +560 ms  position(20599)
MEASURE +766 ms  position(20800)
MEASURE +0 ms  shown: old=2800 new=20000
MEASURE +8 ms  shown: old=20000 new=20000
MEASURE +163 ms  shown: old=20200 new=20200
MEASURE +370 ms  shown: old=20400 new=20400
MEASURE +560 ms  shown: old=20599 new=20599
MEASURE +766 ms  shown: old=20800 new=20800
```

`old=` satırları kabuğun eski kuralını, `new=` satırları guard'ı aynı kayıt
üzerinde oynatır. Eski kural bırakma anında **2800 ms** gösteriyor — bildirilen
"top eski yerine ışınlanıyor" — ve 8 ms sonra `20000`'e sıçrıyor. Guard hiç
`20000`'den ayrılmıyor.

Yani gecikme mpv'nin seek'i yavaş servis etmesi değil; kuyruğun elinde seek'ten
önceki konumun durması ve kabuğun tam o anda bakmasıdır.

## 2. Otomatik testler

`bash scripts/test-macos.sh` çıkış 0: **47 test / 6 suite**, 0 failure.
Yeni testler (`PlayerModelTests`):

- `a position report from before a seek does not move the knob back`
- `the seek's own answer is what lets position reports through again`
- `two seeks in a row stay guarded until both are answered`
- `an answer that never arrives does not freeze the position forever`
- `a refused seek leaves no guard behind`
- `EventsLost drops the guard along with the events`

## 3. Negatif kontrol

`consume`'daki tek satırlık guard (`guard !seekIsInFlight else { break }`)
kaldırıldığında **4 test kırmızı**, ve ilki bildirilen semptomun birebir
kendisi:

```
✘ "a position report from before a seek does not move the knob back"
  Expectation failed: (model.positionMilliseconds → 4080) == (20_000 → 20000)
✘ "the seek's own answer is what lets position reports through again"
  Expectation failed: (model.positionMilliseconds → 4040) == (14_000 → 14000)
✘ "two seeks in a row stay guarded until both are answered"
  Expectation failed: (model.positionMilliseconds → 9040) == (14_000 → 14000)
✘ "an answer that never arrives does not freeze the position forever"
  Expectation failed: (model.positionMilliseconds → 4040) == (14_000 → 14000)
```

Guard geri konduğunda 47/47 yeşil.

## 4. Otomatik testin kapsamadığı

`commitSeek()` içindeki sıra değişikliği (önce seek, sonra önizlemenin
silinmesi) **testle ayırt edilemiyor**: iki sıra da aynı turda tamamlandığı için
SwiftUI ikisinde de tek render yapıyor ve gözlemlenebilir bir fark kalmıyor.
Değişiklik yine de yapıldı — reddedilen bir seek'te gösterilenin doğru
kalmasını sıraya bağlı olmaktan çıkarıyor — ama kanıtı **yok**, tahmin
edilmemesi için burada açıkça yazılıyor.

## 5. Elle acceptance

Ekran kontrolü istendi ve kullanıcı **reddetti**, bu yüzden elle geçişi
kullanıcı kendisi yaptı. Uygulama `bash scripts/build-macos-app.sh` ile
derlendi (çıkış 0) ve `.build/NenPlayer.app` olarak çalıştırıldı.

- [x] Kullanıcı düzeltilmiş yapıda kaydırıcıyı sürükleyip bıraktı ve sonucu
      **"düzgün görünüyor"** diye bildirdi — geri sıçrama gözlenmedi.

Kapsamın sınırı açıkça yazılıyor: bu tek bir kullanıcı onayıdır. İleri/geri
sürükleme ve oynarken/duraklatılmışken ayrımlarının ayrı ayrı denendiği
**bildirilmedi**; §1'deki ölçüm ile §3'teki negatif kontrol bu task'ın asıl
kanıtıdır.
