---
id: NEN-049
title: Serialize real libmpv platform tests
milestone: M3
size: S
state: done
closed: 2026-09-06
depends_on: [NEN-022, NEN-045, NEN-051]
blocks: []
adr: [12, 33]
---

# NEN-049 — Serialize real libmpv platform tests

## Sonuç

`scripts/test-macos.sh` içindeki gerçek libmpv testleri birbirinin zamanlamasını
bozmadan deterministik geçer; ürünün seek kontratı gevşetilmez.

## Kapsam

- Swift Testing'in tam paket koşusunda gerçek libmpv suite'lerini aynı anda
  çalıştırmaması için en dar test-runner veya suite izolasyonu
- `aSeekIsAnsweredThroughTheSession` testinin `SeekCompleted` içindeki gerçek
  konumu sınamaya devam etmesi; toleransı genişletmeme ve `0 ms`'yi kabul etmeme
- Seri/paralel farkının tekrarlı koşuyla kanıtlanması

## YAPILMAYACAK

- Playback adapter davranışını yalnız testi yeşile çevirmek için değiştirmek
- Seek UI uzlaştırması; transport sıçraması ayrı ürün işi
- Platform testlerini atlamak veya contract kitini daraltmak

## Neden ayrı task

NEN-046 kapanışında tam paralel macOS paketi iki ardışık koşuda aynı şekilde
kırmızı oldu: `aSeekIsAnsweredThroughTheSession`, beklenen `12_000 ms` yerine
`SeekCompleted(0 ms)` gördü. Test tek başına **5/5**, tüm paket açıkça
`--no-parallel` ile **31/31** geçti. NEN-046 yalnız pencere yaşam döngüsüdür;
gerçek motor testlerinin runner izolasyonu bu kapsama karıştırılmaz.

## Güncelleme (2026-08-26) — teşhis değişti

`NEN-051` açılırken kök sebep ölçüldü ve bu task'ın çıkış noktası olan
"paralel koşuda kırmızı" gözlemi **test izolasyonu sorunu değil**: mpv'nin
yüklemeden sonra yayınladığı `playback-restart`, `FILE_LOADED`'dan sonra
geldiği için `.ready` görülür görülmez verilen bir seek'i `0 ms` ile
cevaplıyordu. Paralellik yalnız o pencereyi genişletiyordu.

`NEN-051` kapandıktan sonra ölçüm yapıldı ve **ayırt edici çıkmadı**:
`bash scripts/test-macos.sh` düzeltmeden **önce de sonra da** art arda 5/5
yeşil. Yani bu makinede oran karşılaştırması bu task'ın gerekçesini ne
doğruluyor ne çürütüyor; DoD #1 (art arda ≥ 3 çıkış 0) hiçbir şey
değiştirilmeden zaten sağlanıyor ve DoD #3'ün istediği "değişiklik öncesi
paralel kırmızı" kaydı **üretilemedi**.

Bildirilen semptomun mekanizması `NEN-051`'de kaldırıldı ve deterministik
negatif kontrolle kanıtlandı. Bu task'ın kalan gerekçesi — gerçek libmpv
suite'lerinin runner izolasyonu — bağımsız bir kırmızı gözlemi olmadan
doğrulanamaz. **Karar kullanıcıya ait:** yeni bir kırmızı gözlenene kadar
beklemek ya da task'ı kapatmak. O gözlem gelmeden implementasyona
başlanmamalıdır.

### Gözlem — 2026-08-27, `NEN-025` sırasında

Aranan "bağımsız kırmızı" **bir kez gözlendi**, fakat beklenen testte değil.
`NEN-025`'in ilk paket koşusunda paralel modda `transientMessageExpires`
kırmızı geldi: 20 ms'lik geçici bildirim 300 ms'lik beklemeden sonra hâlâ
temizlenmemişti. O koşuda derleme de aynı anda sürüyordu.

Aynı ağaçta hemen ardından:

| Mod | Sonuç |
|---|---|
| yalnız o test | 1/1 yeşil |
| seri, tam paket | 3/3 yeşil (56 test) |
| paralel, tam paket (derleme sıcakken) | 4/4 yeşil |

Yani kırmızı **yükle ilişkili**, testle değil: ana aktörü meşgul eden bir
koşuda `Task.sleep`'e dayanan bir bekleyiş aç kalıyor.
`aSeekIsAnsweredThroughTheSession` değil, ama **sınıf aynı** — bu task'ın
gerekçesindeki "paralellik sebep değil, dar bir pencereyi genişleten koşul"
ifadesiyle uyumlu. Tek gözlem; oran ölçümü yapılmadı.

## İkinci gözlem (2026-08-27, NEN-058 sırasında)

Aynı sınıf ikinci kez görüldü. `NEN-058`'in kapanış koşusunda paralel tam paket
**1 kırmızı** verdi: `window resume after shutdown restarts event polling`
(`PlayerModelTests`) — beklenen `.ready` yerine `.playing`. Yine `FakeSession`
tabanlı, gerçek libmpv adapter'ıyla ilgisi olmayan bir shell testi ve yine
`Task.sleep`'e dayanan bir bekleyiş.

Aynı ağaçta **seri** paket art arda **2/2** yeşil (62 test). Oran ölçümü yine
yapılmadı; bu satır yalnız gözlemi kaydediyor, bu task'a iş eklemiyor (Kural 5).

## Üçüncü gözlem (2026-09-05, NEN-028 sırasında)

Sınıf üçüncü kez, ilk kez de **gerçek libmpv** testinde görüldü.
`NEN-028`'in ilk kapanış koşusunda paralel tam paket **1 kırmızı** verdi:
`ContractTests.successiveMediaReportTheirOwnDisplaySize` — döngüdeki
medyalardan biri için `waitForGeometry` beklenen boyutu zamanında görmedi.

Aynı ağaçta, hiçbir şey değiştirilmeden:

| Mod | Sonuç |
|---|---|
| yalnız o test, seri | 3/3 yeşil |
| paralel, tam paket | 1/1 yeşil (151 test) |
| seri, tam paket | 2/2 yeşil (151 test) |

Bu, bu task'ın özgün gerekçesine en yakın gözlem: kırmızı gelen test gerçek
motoru sürüyor ve beklemesi bir olayın zamanında gelmesine dayanıyor. Oran
ölçümü yine yapılmadı; bu satır yalnız gözlemi kaydediyor, task'a iş eklemiyor
(Kural 5).

## Dördüncü gözlem (2026-09-06, NEN-069 sırasında) — ilk oran ölçümü

Aynı test (`ContractTests.successiveMediaReportTheirOwnDisplaySize`) dördüncü
kez kırmızı geldi, ve bu kez **oran ölçüldü** — DoD #3'ün bugüne kadar
üretilemeyen kaydı budur. Hiçbir şey değiştirilmemiş `HEAD` ağacında, art arda
beş paralel tam paket koşusu:

| Ağaç | Paralel tam paket |
|---|---|
| `HEAD` (NEN-069 öncesi) | **2/5 yeşil** |
| `NEN-069` çalışması, aynı saatte | **2/5 yeşil** |
| `NEN-069`, seri (`--no-parallel`) | **2/2 yeşil** (157/157) |

Makinenin durumu oturum boyunca kaydı (aynı ağaç birkaç saat sonra 1/4), bu
yüzden ölçüm ardışık değil **dönüşümlü** tekrarlandı — aynı ağaçta NEN-069'un
suite dosyası sırayla var ve yok edilerek dörder koşu:

| | Paralel tam paket |
|---|---|
| suite dosyası **var** | **1/4 yeşil** |
| suite dosyası **yok** | **1/4 yeşil** |

Yani paralel paket bu makinede **değişiklik olmadan da** koşuların çoğunda
kırmızı, ve oran NEN-069 ile ölçülebilir biçimde değişmiyor.

**Mekanizma bu kez okundu.** `waitForGeometry` **ilk non-nil** değeri kabul
ediyor. `loadfile` sonrası `MPV_EVENT_FILE_LOADED` geldiğinde adapter'ın
`phase`'i `.loaded` oluyor ama `video-out-params` hâlâ **giden** medyayı tarif
edebiliyor — `aLoadingMediumDoesNotExposeTheOutgoingDisplaySize` bu davranışı
zaten bilerek sabitliyor ("the old VO size must really exist"). Yük altında o
pencere genişliyor ve `waitForGeometry` giden medyanın boyutunu döndürüyor;
test 5 s'lik timeout'una hiç ulaşmadan, ~0,3 s'de kırmızı oluyor. Yani bu bir
zaman aşımı değil, **bayat değer** kusuru.

**NEN-069 sırasında iki şey ölçüldü ve ikisi de o task'ın kapsamını
değiştirdi** — bu task'a iş eklemeden, ama burada kayda değer:

1. `RunLoop.current.run(until:)` ile bekleyen bir libmpv suite'i, main actor'ı
   paylaşan `PlayerModelTests`'in üç testini **beş saniyelik** deadline'larını
   kaçıracak kadar aç bırakıyor. `await Task.sleep`'e çevrilince bu kırmızı
   sınıfı tamamen kayboluyor. NEN-066'nın `settle()` deseni bu yüzden yeni
   suite'lere kopyalanmamalı.
2. Gerçek libmpv engine **sayısı** doğrudan etkili: iki ayrı engine kuran iki
   test, `successiveMediaReportTheirOwnDisplaySize`'ı 2/5'ten **3/3 kırmızıya**
   çeviriyordu. Tek engine'e birleştirilince oran taban değere döndü.

Ölçümün tamamı: `evidence/M3/NEN-069-measurement.md` → "Paralel paket
üzerindeki etki".

## Kanıt (DoD)

- [x] `bash scripts/test-macos.sh` art arda en az 3 kez çıkış 0 — **9 ardışık
      yeşil** (5/5 ve 4/4, iki turda)
- [x] `aSeekIsAnsweredThroughTheSession` hâlâ `12_000 ± 100 ms` bekliyor —
      `SessionTests.swift:75`, dosyaya dokunulmadı
- [x] Değişiklik öncesi paralel kırmızı ve sonrası tekrarlı yeşil sayıları
      bağlamıyla kaydedilmiş — dönüşümlü ölçüm, öncesi **3/9**, sonrası **9/9**
- [x] Ürün kaynak kodunda değişiklik yok — tek dosya
      `platforms/macos/Tests/NenPlaybackMPVTests/ContractTests.swift`

## Kanıt kaydı

**Kapanış tarihi:** 2026-09-06 · Tam ölçüm:
`evidence/M3/NEN-049-measurement.md`

**Kırmızı üretilebildi ve ilk kez sebebi doğrudan okundu.** Dokunulmamış
`HEAD` üzerinde paralel tam paket **2/5 yeşil**; kırmızı olan her koşuda
`ContractTests.successiveMediaReportTheirOwnDisplaySize` ve her koşuda
`~0,2 s`'de düşüyor — helper'ın 5 s'lik timeout'una hiç ulaşmadan. Geçici bir
tanı satırı (commit edilmedi) beklemenin **ne gördüğünü** yazdırdı:

```
DIAG loaded aspect-4x3-clip.mkv, saw FfiVideoGeometry(width: 160, height: 90)
DIAG loaded anamorphic-clip.mkv, saw FfiVideoGeometry(width: 160, height: 120)
```

İkisinde de görülen değer **giden medyanın** boyutu. Yani bekleme aç kalmıyor,
yanlış cevabı zamanında alıyor: `loadfile` sonrası `MPV_EVENT_FILE_LOADED`
geldiğinde `video-out-params` hâlâ giden medyayı tarif edebiliyor —
`aLoadingMediumDoesNotExposeTheOutgoingDisplaySize` bunu zaten bilerek
sabitliyor — ve `waitForGeometry` **ilk non-nil** değeri kabul ediyordu.

**Ölçüm çareyi değiştirdi: izolasyon uygulanmadı.** Bu task "gerçek libmpv
suite'lerini birbirine karşı seri kılmak" diye açılmıştı ve başlığı hâlâ öyle
diyor. Seri kılmak kırmızıyı **gizlerdi** — kusur paralellikte değil, testin
kendi beklemesindeydi. `waitForGeometry` artık **giden** medyanın boyutunu
dışlıyor (`waitForGeometry(_:after:)`), **beklenen** boyutu değil: beklenen
boyutu vermek iddiayı kendi öncülüne sordurmak olurdu. Kapsamın üç
basamağından (suite trait'i · ortak seri ebeveyn · script'te `--no-parallel`)
**hiçbiri gerekmedi**; paket paralel kalıyor.

**Oran bu kez ayırt edici çıktı** — `NEN-051`'de çıkmamıştı. Makine durumu
oturum içinde kaydığı için ölçüm dönüşümlü tekrarlandı:

| Ağaç | Paralel tam paket |
|---|---|
| Düzeltme yok (A turu) | **2/5 yeşil** |
| Düzeltme var (A turu) | **5/5 yeşil** |
| Düzeltme geri alındı (B turu) | **1/4 yeşil** |
| Düzeltme geri kondu (B turu) | **4/4 yeşil** |
| **Toplam** | öncesi **3/9** · sonrası **9/9** |

Seri mod, düzeltmeli ağaçta 2/2 yeşil (159/159).

**Negatif kontrol üç yönde.** (1) Düzeltme geri alınınca semptom geri geliyor —
B turunun ilk yarısı tam olarak bu; kusur yüke bağlı olduğu için tek koşu
kanıt sayılmadı, oranla raporlandı. (2) Beklenen boyutlardan biri bilinçli
yanlış yazılınca (`160×120` → `161×121`) test kırmızı ve gerçek değeri
söylüyor — yani bekleme çağıranın sorusunu cevaplamıyor. (3) Düzeltme "daha
uzun bekleyerek" geçmiyor: dokuz yeşil koşuda testin süresi **0,117–0,186 s**,
5 s'lik timeout'a yaklaşan koşu yok.

**Kapılar.** macOS paketi 159/159 (paralel 9 koşu, seri 2 koşu), Rust workspace
**568 passed / 1 ignored**, `cargo fmt --check`, `cargo clippy --workspace
--all-targets -D warnings`, `bash scripts/test.sh` ve `bash
scripts/check-docs.sh` yeşil.

**Kalan sınır, açıkça.** Dışlama bir **değer** karşılaştırmasıdır: art arda
gelen iki medya gerçekten aynı display boyutunu paylaşırsa bekleme timeout'u
harcayıp doğru cevabı en sonda verir. Bugünkü fixture'ların üçü de farklı
boyutta; böyle bir çift eklenirse ayırıcı değer değil, yüklemenin kendi
reconfiguration'ı olmalı. Helper'ın doküman yorumuna yazıldı.
