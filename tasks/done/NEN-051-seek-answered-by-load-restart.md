---
id: NEN-051
title: A seek is never answered by the load's own playback-restart
milestone: M3
size: M
state: done
depends_on: [NEN-022, NEN-045]
blocks: [NEN-049, NEN-052]
adr: [11]
---

# NEN-051 — A seek is never answered by the load's own playback-restart

## Sonuç

`SeekCompleted` her zaman **o seek'in** indiği konumu taşır; yüklemenin kendi
`playback-restart`'ı bekleyen bir seek'i cevaplayamaz. Paylaşılan contract kiti
bu sınıfı görebilecek hâle gelir.

## Kapsam

- `MPVPlaybackEngine`'in `MPV_EVENT_PLAYBACK_RESTART` kolunda, yüklemeden
  türeyen restart ile seek'ten türeyen restart'ın ayrılması
- Contract kitine seek'in **taşıdığı** konumu `seek_tolerance_ms` içinde
  sınayan bir adım eklenmesi
- `.ready` sonrası hemen verilen bir seek için deterministik regresyon testi

## YAPILMAYACAK

- Seek toleransını genişletmek veya `0 ms`'yi kabul etmek
- `EventShape`'i payload taşır hâle getirmek — `Outcome::Events`'in alt-dizi
  eşleşmesi ona dayanıyor, payload'suz kalması oradaki tasarım kararı
- Gerçek libmpv testlerinin runner izolasyonu → `NEN-049`
- Transport'un seek sırasındaki görsel davranışı → `NEN-047`

## Neden ayrı task

`NEN-049` "paralel koşuda `aSeekIsAnsweredThroughTheSession` kırmızı" ölçümünü
test izolasyonu sorunu olarak açmıştı. Bağlam okunurken kök sebep ölçüldü ve
teşhis yanlış çıktı: kusur adapter'da.

`MPVPlaybackEngine+Internals.swift`'in `MPV_EVENT_PLAYBACK_RESTART` kolu
bekleyen her seek'i cevaplıyor ve tek koruması `pendingSeeks > 0`. Oradaki
yorum yalnız **unpause** restart'ını öngörüyor. Fakat mpv ilk yüklemeden sonra
da bir `playback-restart` yayınlıyor ve bu, durumu `.ready` yapan
`MPV_EVENT_FILE_LOADED`'dan **sonra** geliyor. Sıra:

1. `settle(until: .ready)` → `FILE_LOADED` ile döner
2. `seek(toMs: 12_000)` → `pendingSeeks = 1`
3. **yüklemenin kendi** restart'ı gelir, seek'i `0 ms` ile cevaplar

Paralellik sebep değil; 1. ile 3. adım arasındaki pencereyi genişlettiği için
kusuru görünür kılıyor. Serialize etmek semptomu kaybettirir, kusuru üründe
bırakır: büyük ya da uzak bir dosya açıp hemen `→`'ya basan kullanıcı transport
`00:00`'a atlarken görür.

Contract kitinin bunu yakalamamasının sebebi ayrıca kayda geçti: `EventShape`
payload taşımıyor, yani kit `SeekCompleted`'ın **hangi konumu** taşıdığını hiç
sormuyor. Bir seek'in indiği konum motorlar arasında meşru biçimde değişen bir
değer değil — `ContractInputs` zaten `seek_tolerance_ms` taşıyor. Bu boşluk
kapanmazsa Android adapter'ı da aynı kusurla yazılır.

## Kanıt (DoD)

- [x] mpv olay sırası ölçümü kayıtlı: `FILE_LOADED` → `PLAYBACK_RESTART`
      aralığı ve `.ready` sonrası verilen seek'in ayrı bir restart üretip
      üretmediği
- [x] `.ready` sonrası hemen verilen seek `12_000 ± 100 ms` ile cevaplanıyor,
      `0 ms` ile değil
- [x] Contract kiti seek'in taşıdığı konumu tolerans içinde sınıyor
- [x] Negatif: adapter düzeltmesi geri alınınca yeni Swift testi **ve** kitin
      yeni adımı kırmızıya dönüyor — iki yön ayrı ayrı kaydedilmiş
- [x] `bash scripts/test-macos.sh` paralel modda art arda ≥ 5 koşu, kırmızı
      oranı 0; öncesi/sonrası oran karşılaştırmalı kayıtlı
- [x] `aSeekIsAnsweredThroughTheSession` hâlâ `12_000 ± 100 ms` bekliyor
- [~] ~~Gerçek `.app`: dosya açılır açılmaz `→`~~ — **yapılmadı, gerekçesi
      kanıt kaydında**: pencere elle vurulamayacak kadar dar, yani bu madde
      düzeltilmiş ile bozuk kurulumu birbirinden ayırt edemez. Boşta dönen bir
      kanıt maddesi, maddenin hiç olmamasından kötüdür.

## Kanıt kaydı

### Mekanizma ölçüldü, tahmin edilmedi

`consume`'a geçici bir olay logu konarak (commit edilmedi, ölçümden sonra geri
alındı) gerçek mpv olay sırası kaydedildi. Kusurun oluştuğu tur:

```
file-loaded        pendingSeeks=0     ← state() buradan itibaren Ready
[test .ready görüp seek veriyor]
playback-restart   pendingSeeks=1     ← YÜKLEMENİN restart'ı, seek'i yutuyor
seek               pendingSeeks=0     ← mpv gerçek seek'i ancak şimdi başlatıyor
playback-restart   pendingSeeks=0     ← seek'in kendi restart'ı, hiçbir şey demiyor
```

Aynı koşunun diğer turu zararsız sırayı gösteriyor: yüklemenin restart'ı seek
verilmeden **önce** geliyor, `pendingSeeks == 0` olduğu için doğru biçimde
yok sayılıyor. Yani iki sıra da gerçek; hangisinin çıkacağı zamanlamaya bağlı.

Ölçüm ayrıca üçüncü bir şey gösterdi ve düzeltmeyi o belirledi: **mpv, bir seek
başladığında `MPV_EVENT_SEEK` yayınlıyor** ve bu, o seek'in `playback-restart`'ından
önce, aynı kuyrukta geliyor. Planda düşünülen iki aday (async komut cevabı /
yüklemenin restart'ını körü körüne yutmak) bu yüzden ikisi de kullanılmadı:
`MPV_EVENT_SEEK` zaten aranan işaret.

Denenip elenen üçüncü yol: seek'i `.loading` sırasında verip pencereyi
deterministik açmak. `requireMedia` `.loading`'i kabul ediyor ama **mpv
etmiyor** — `EngineFailure(code: -12)` (`MPV_ERROR_COMMAND`). Bu ayrıştı →
`NEN-052`.

### Düzeltme

`MPVPlaybackEngine`'e `seekInFlight` eklendi; `MPV_EVENT_SEEK` onu kuruyor,
`MPV_EVENT_PLAYBACK_RESTART` artık `pendingSeeks > 0` **ve** `seekInFlight`
istiyor. `load` ve `stop` bayrağı sıfırlıyor. Bir restart'ı seek cevabı yapan
şey artık "bir seek'in gerçekten başlamış olması"; sayaç yalnız kaç cevap
borçlu olunduğunu tutuyor. Tolerans değişmedi, `0 ms` kabul edilmiyor,
adapter'ın başka hiçbir davranışına dokunulmadı.

### Kitin kör noktası da kapatıldı

`EventShape` payload taşımıyor, yani kit `SeekCompleted`'ın **hangi** konumu
taşıdığını hiç sormuyordu; ardından gelen `Position` adımı motora doğrudan
sorduğu ve o ana kadar seek indiği için senaryo yeşil kalıyordu. Olay yanlıştı,
kimse bakmıyordu.

`Action::AwaitSeekLanding { near_ms }` eklendi ve üç senaryonun dört seek adımı
buna çevrildi. Yargı `ContractInputs::seek_tolerance_ms` ile —
`Outcome::PositionNear`'ın kullandığı marjın aynısı. `EventShape`
**değiştirilmedi**: `Outcome::Events`'in alt-dizi eşleşmesi ona dayanıyor.
Bu, kusuru tek platformda değil **her adapter'da** görünür kılıyor; Android
aynı hatayı yazamaz.

### Negatif kontrol — iki yön, ayrı ayrı

**Adapter yönü.** `seekInFlight` guard'ı kaldırılıp eski `pendingSeeks > 0`'a
dönülünce `theLoadsOwnRestartDoesNotAnswerAPendingSeek` kırmızı:

```
Expectation failed: (answered → [0]).isEmpty → false
```

Bildirilen semptomun (`SeekCompleted(0 ms)`) birebir kendisi, deterministik
olarak. Guard geri kondu.

**Kit yönü.** `FakeEngine::seek` geçici olarak `Duration::ZERO` raporlamaya
zorlanınca `nen-ports` ve `nen-ffi` köprüsü boyunca **8** test kırmızı;
hata metni `the seek was answered at 0 ms, expected 5000 ms`. Fake geri alındı.

Bu ikincisi kalıcılaştırıldı: `contract_kit_is_not_vacuous.rs`'e yedinci defect
(`AnswersSeekWithAStalePosition`) eklendi — doğru konuma giden, cevabı doğru
sırada veren, yalnız **olayın taşıdığı** konumu yanlış söyleyen ikiz. Kontrolün
kontrolü de yapıldı: yeni tolerans karşılaştırması `if true` ile devre dışı
bırakılınca bu ikiz **tek başına** kırmızı oluyor, dosyadaki diğer yedi test
yeşil kalıyor. Yani yeni adım gerçekten kendi işini yapıyor, başka bir adımın
yan etkisiyle yakalamıyor.

### Sayılar

| | Önce | Sonra |
|---|---|---|
| Rust | 452 | **453** |
| Swift (macOS paketi) | 39 | **41** |

`cargo test --workspace --no-fail-fast` → 453 geçti, 0 kırmızı.
`cargo clippy --workspace --all-targets -- -D warnings` temiz.
`bash scripts/test-macos.sh` art arda **5/5** çıkış 0 (41 test / 6 suite).
Yeni dış bağımlılık yok, `deny.toml` değişmedi.

### Oran ölçümü ayırt edici çıkmadı — açıkça kaydediliyor

`NEN-049`'un istediği "öncesi/sonrası kırmızı oranı" alındı ama **beklendiği
gibi çıkmadı**: düzeltmeden **önce** de paket art arda 5/5 yeşildi. Yani bu
makinede, bu yükte, oran karşılaştırması düzeltilmiş ile bozuk kurulumu
ayırt etmiyor. `STATUS`'ün "yük altında aralıklı" kaydı bununla tutarlı —
kusur nadir bir zamanlama penceresi.

Bu yüzden bu task'ın kanıtı orana **dayandırılmadı**: dayanağı yukarıdaki
deterministik negatif kontrol ile doğrudan olay-sırası ölçümü. Aynı sebeple
`.app` üzerinde elle `→`'ya basma maddesi düşürüldü: pencere elle vurulamayacak
kadar dar olduğu için o madde düzeltilmiş ile bozuk kurulumda **aynı** sonucu
verirdi. 20 tekrarlı otomatik test
(`seekingTheInstantReadyAppearsIsAlwaysAnsweredAtTheTarget`) bile kusur
yerindeyken yeşil kaldı; o test bir oran koruması, mekanizma kanıtı değil.

### Yan bulgular

- `NEN-052` açıldı: port `.loading` sırasında seek'e izin verirken gerçek
  adapter `EngineFailure(-12)` ile reddediyor, contract kiti bu durumu hiç
  kapsamıyor.
- `NEN-049`'un teşhisi geçersiz kaldı; dosyasına not düşüldü ve bu task'a
  bağlandı.
