# NEN-058 — Ölçüm kaydı

> Kanıt tipi: **defect** (`docs/testing-strategy.md` → "Kanıt formatı") — kök
> neden ölçümü + kalıcı test + negatif kontrol.
>
> Ortam: macOS 27.0 · libmpv 2.5.0 · Swift 6.3.3 · 2026-08-27.
> Medya: `fixtures/media/contract-clip.mkv` kopyaları. K23: hiçbir yerde özel
> tam dosya yolu yok.

## 1. Hipotez çürütüldü — symlink değişken değildi

Task `sub-auto`'yu "en olası hipotez" olarak açmıştı: mpv yanındaki `.srt`'yi
kendi yüklemeye çalışıyor ve **symlink'te takılıyor**. Hipotezin ikinci yarısı
yanlış çıktı.

Aynı dizine `contract-clip.mkv`'nin bir kopyası kondu ve yanına sırayla dokuz
farklı komşu kuruldu. Ölçüm libmpv ile alındı (`vo=null`, `ao=null`) — mpv CLI
bu makinede dönmüyor, NEN-022 bunu zaten kaydetmişti ve NEN-058'in açılış
denemesi de aynı yerde asılmıştı.

| # | Yanındaki dosya | `state` | `tracks(.subtitle)` |
|---|---|---|---|
| A | yok | `ready` | `[3, 4]` |
| B | düz `.srt` | `ready` | `[3, 4, 0]` |
| C | symlink → kardeş `.srt` (mutlak) | `ready` | `[3, 4, 0]` |
| D | symlink → kardeş `.srt` (göreli) | `ready` | `[3, 4, 0]` |
| E | kırık symlink | `ready` | `[3, 4]` |
| F | symlink → dizin dışı `.srt` | `ready` | `[3, 4, 0]` |
| G | symlink → dizin | `ready` | `[3, 4]` |
| H | kendine dönen symlink | `ready` | `[3, 4]` |
| I | boş `.srt` | `ready` | `[3, 4]` |

**Symlink'in hiçbir şekli medyayı düşürmüyor.** Semptom bu eksende üremedi.

## 2. Semptom üredi — değişken sıraydı

NEN-025'in elle koşusu `Probe.mkv`'yi **önce**, `Linked.mkv`'yi **sonra**
açmıştı. Aynı düzen tek motorda kurulunca semptom birebir çıktı:

```
PROBE 1 Probe.mkv:        state=ready
PROBE 2 Linked.mkv:       state=ready
PROBE 2 events=[buffering, failed, failed(LoadFailed(unreadable)), ready, tracksChanged, positionChanged(0)]
PROBE 3 Linked.mkv again: state=ready   (aynı olay dizisi — "art arda 2 kez")
PROBE 4 Linked.mkv first: state=ready   (ters sıra)
PROBE 5 Probe.mkv second: state=ready
```

Motor sonunda `ready` oluyor, ama **yolda `failed` üretiyor** ve kabuk fatal
mesajı gördüğü anda basıyor — kullanıcının `Dosya okunamadı.` görmesinin sebebi
bu. PROBE 4/5 dosyaların değil **sıranın** değişken olduğunu gösteriyor: aynı
`Linked.mkv` ilk açıldığında hiçbir hata üretmiyor.

Symlink ile sıra NEN-025'in koşusunda birbirine karışmıştı; sidecar seyirciydi.

## 3. Kök neden ve ayırt edici işaret

Ham olay alanları yazdırıldı (geçici enstrüman, commit edilmedi):

```
RAW loadfile rc=0 newEntry=2   t=560699
RAW end_file  reason=2 error=0 entry=1 t=561258
RAW start_file                 entry=2 t=561344
```

- `loadfile` açık bir medyanın üstüne gelince mpv **giden** entry'yi bitiriyor:
  `reason=2` (`MPV_END_FILE_REASON_STOP`), `error=0`.
- Adapter'ın `END_FILE` dalı yalnız `stopRequested` ile susuyordu. Bu bir stop
  değil, dolayısıyla olay `loadFailure(from:)`'ın fallback koluna düşüyor ve
  `unreadable` oluyor.
- **Bu, gerçek bir yükleme hatasından sebep koduyla ayırt edilemez:** kodun
  kendi yorumunun kaydettiği gibi, container olmayan bir dosya da
  `REASON_STOP` + `error=0` veriyor.

Ayıran tek şey **hangi entry'nin** bittiği, ve mpv bunu `playlist_entry_id` ile
kendisi söylüyor: giden entry `1`, yeni entry `2`.

**Zamanlama ölçüldü:** `loadfile` yeni entry id'sini `mpv_command_ret` ile
**senkron** döndürüyor ve bu, giden dosyanın `end_file`'ından ~560 µs **önce**
oluyor. Yani id, karşılaştırmanın gerektiği anda biliniyor. Yarış ayrıca yapısal
olarak kapatıldı: `load()` komutu yollamadan önce `currentEntryId`'yi
`noEntry`'ye çekiyor, dolayısıyla id bilinmeden gelen bir son hiçbir yüklemeye
ait olamıyor.

Kör yutma **kullanılmadı** — NEN-051'de reddedilen yaklaşımın aynısı olurdu.

## 4. İkinci kusur — hipotezin doğru çıkan yarısı

Tablo 1'deki `[3, 4, 0]` sütunu: fixture'ın **iki** gömülü subtitle track'i var
(ff-index 3 ve 4, `fixtures/media/contract-clip.ffmpeg.txt`). Yanında `.srt`
varken **üçüncü** bir track görünüyor; ff-index'i `0`, yani external.

`sid=no` yalnız **gösterimi** kapatıyor; mpv'nin default `sub-auto=exact`'i
dosyayı **açmaya** devam ediyor. Bu, kataloğun hiç görmediği, menünün hiç
listelemediği (ADR-0031 Karar 4/5) ve NEN-025'in dört kapısının hiç
incelemediği bir kaynak demek — symlink kapısı dahil, ki C/D/F satırları tam
olarak bunu gösteriyor.

`("sub-auto", "no")` eklendi.

## 5. Düzeltme

| Yer | Değişiklik |
|---|---|
| `MPVPlaybackEngine.load` | `loadfile` `mpv_command_ret` ile yollanıyor, yeni entry id'si `currentEntryId`'ye yazılıyor; komuttan önce `noEntry`'ye çekiliyor |
| `MPVPlaybackEngine+Properties.loadFile` | yeni yardımcı — node map'ten `playlist_entry_id` okuyor |
| `MPVPlaybackEngine+Internals` `END_FILE` | `end.playlist_entry_id == currentEntryId` değilse olay bu yüklemeye ait değil |
| `MPVPlaybackEngine.init` | seçeneklere `("sub-auto", "no")` |

## 6. Kalıcı testler

`platforms/macos/Tests/NenPlaybackMPVTests/SidecarLoadingTests.swift` — 6 test:

| Test | Ne kanıtlıyor |
|---|---|
| `aMediumOpensBesideASymlinkedSidecar` | DoD #2 |
| `asecondMediumOpensWithoutReportingAFailure` | Semptomun kendisi: ikinci medya hiç `failed` üretmiyor |
| `theOrderOfTheTwoMediaDoesNotMatter` | Ters sıra — değişken dosya değil, konum |
| `aGenuineFailureIsStillReportedAfterAMediumIsOpen` | Guard sağır kalarak sessizlik satın almıyor |
| `aSidecarBesideTheMediumDoesNotEnterTheTrackList` | DoD #4 |
| `aSymlinkedSidecarDoesNotEnterTheTrackListEither` | mpv, NEN-025'in symlink kapısını arkadan dolanmıyor |

`asecondMediumOpens...` yalnız duruma bakmıyor: **düzeltmeden önce de motor
sonunda `ready` oluyordu.** Kanıt olan şey yoldan geçen `failed` olayı.

## 7. Negatif kontrol (DoD #3) — iki yönde, ayrı ayrı

| Kaldırılan | Kırmızı | Hangi testler |
|---|---|---|
| `playlist_entry_id` guard'ı | **2** | `asecondMediumOpens...` · `theOrderOfTheTwoMedia...` |
| `("sub-auto", "no")` | **2** | `aSidecarBeside...` · `aSymlinkedSidecar...` |

Birincisi semptomu birebir geri getiriyor:
`failures → [stateChanged(failed), failed(LoadFailed(unreadable))]`.

İkisi ayrık: her düzeltmenin kendi testleri var, hiçbiri diğerinin yerine
geçmiyor ve hiçbiri boşta dönmüyor.

**Ölçümün kendisi bir kez düzeltildi.** İlk turda negatif kontroller
`git checkout <path>` ile geri alınıyordu; dosyalar henüz commit edilmediği için
bu, **iki düzeltmeyi birden** HEAD'e geri aldı ve B kontrolü 4 kırmızı gösterdi.
Yukarıdaki sayılar dosya kopyasıyla yeniden alınmış, izole ölçümlerdir.

## 8. Koşular

- macOS paketi, **seri**: `62 test / 8 suite` — art arda **2/2** yeşil
  (NEN-025'te 56'ydı; +6).
- Rust workspace: **489 passed / 0 failed** (değişmedi — bu iş Rust'a dokunmadı).
- macOS paketi, **paralel**: 1 kırmızı —
  `window resume after shutdown restarts event polling` (`PlayerModelTests`,
  `FakeSession` tabanlı, libmpv adapter'ıyla ilgisi yok). Bu, `NEN-049`'un
  aradığı bağımsız kırmızı sınıfının ikinci görülüşü; oraya not düşüldü, iş
  eklenmedi (Kural 5).

## 9. Elle acceptance

<!-- kullanıcı koşusu -->
