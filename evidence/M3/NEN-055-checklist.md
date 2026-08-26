# NEN-055 — Komut ekranda bir sonraki tik'i beklemez

Tarih: 2026-08-27

Ortam: Apple arm64 · macOS 27.0 (26A5416b) · Xcode 26.6 (17F113) ·
Swift 6.3.3 · libmpv 2.5.0.

Kanıt medyası yalnız telif-temiz sentetik fixture'dır:
`fixtures/media/contract-clip.mkv`.

## 1. Ölçüm — olay komut dönerken gerçekten hazır mı

Task'ın dayandığı iddia şuydu: `MPVPlaybackEngine.pause()` durum olayını
dönmeden önce kuyruğa koyuyor ve `ShellEngineBridge` her komuttan sonra
`pull()` çağırıyor, dolayısıyla olay komut dönerken çekirdeğin kuyruğunda
hazır. `NEN-053`'te teşhisin ölçümle değiştiği görüldüğü için bu da
uygulanmadan **önce** ölçüldü.

Geçici ölçüm testi (commit edilmedi) gerçek libmpv oturumunda komuttan hemen
sonra drain etti:

```
MEASURE play  round 1: drained 51 us later: state(playing)
MEASURE pause round 1: drained 97 us later: state(paused)
MEASURE play  round 2: drained 118 us later: state(playing)
MEASURE pause round 2: drained 160 us later: state(paused)
MEASURE play  round 3: drained 181 us later: state(playing)
MEASURE pause round 3: drained 149 us later: state(paused)
```

**İddia doğrulandı.** Olay komut döndükten 50–180 **mikrosaniye** sonra
görülebiliyor. Yani ikonun geç dönmesinde çekirdeğin veya motorun payı yok;
gecikme tamamen kabuğun elindeki cevaba bir sonraki poll tik'ine (0–50 ms)
kadar bakmamasıydı.

Poll aralığı bu yüzden **değiştirilmedi**: kazanılacak gecikme aralığın kendisi
değil, bakma anıydı.

## 2. Otomatik testler

`bash scripts/test-macos.sh` çıkış 0: **48 test / 6 suite**, 0 failure.

Yeni test: `the icon turns on the click, not on the next poll tick`. Test
poll'u kapalı bir modelde çalışır ve hiçbir olay elle verilmez — `togglePlayback`
çağrısından sonra durumun dönmüş olması, yalnız komutun kendi ürettiği olayın
aynı turda okunmasıyla mümkündür.

`FakeSession` bu task'ta gerçek adapter'a yaklaştırıldı: `play`/`pause` artık
durum olayını kuyruğa koyuyor (§1'de ölçülen davranış). Kalan 47 test bu
değişiklikle de yeşil.

## 3. Negatif kontrol

`togglePlayback` içindeki `drainSessionEvents()` kaldırıldığında yeni test
**iki beklentiyle kırmızı**:

```
✘ "the icon turns on the click, not on the next poll tick"
  Expectation failed: (model.playbackState → .playing) == .paused
  Expectation failed: (fixture.playCount → 1) == 2
```

İkinci satır bildirilen semptomun ikinci yüzüdür: durum geç döndüğü için
ikinci tıklama `togglePlayback`'i **yanlış dalda** buluyor ve duraklatılmış bir
medyayı oynatmak yerine yeniden duraklatmaya çalışıyor. Yani gecikme yalnız
görsel değildi, tuşun kendisini de yanlış yönlendiriyordu.

Drain geri konduğunda 48/48 yeşil.

## 3b. Bir kırmızı koşu — atlanmadı, kaydedildi

Drain geri konduktan sonraki ilk tam koşu **tek bir issue** ile kırmızı oldu;
hemen ardından art arda **4 tam koşu 4/4 yeşil**. Kırmızı koşunun hangi testi
olduğu **yakalanamadı** (çıktı issue satırı için grep'lenmemişti), dolayısıyla
aşağıdaki atıf kanıt değil çıkarımdır: `scripts/test-macos.sh` `swift test`i
**paralel** çalıştırıyor ve gerçek-libmpv testlerinin paralel koşuda kırmızıya
dönmesi `NEN-049`'un açık konusudur. Bu task'ın değiştirdiği kod yalnız kabuk
tarafındadır ve kabuk testleri (`--filter NenPlayerShellTests`, 25 test)
her koşuda yeşildi.

## 4. Elle acceptance

Ekran kontrolü bu oturumda reddedilmişti, elle geçişi kullanıcı kendisi yaptı.
Uygulama `bash scripts/build-macos-app.sh` ile derlendi (çıkış 0) ve
`.build/NenPlayer.app` olarak çalıştırıldı.

- [x] Kullanıcı düzeltilmiş yapıda boşluk tuşu ve düğmeyle oynat/duraklat
      denedi ve **"güzel duruyor, herhangi bir sorun göremedim"** diye
      bildirdi.

Sınırı açıkça yazılıyor: bu tek bir kullanıcı onayıdır, ölçülmüş bir gecikme
sayısı değildir. Sayı §1'dedir; §3'teki negatif kontrol de bu task'ın asıl
kanıtıdır.
