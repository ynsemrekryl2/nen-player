---
adr: 0033
title: Çekirdek playback oturumunun FFI yüzeyi ve olay teslimat yönü
status: accepted
milestone: M3
tasks: [NEN-045, NEN-024]
date: 2026-08-26
---

# ADR-0033 — Çekirdek playback oturumunun FFI yüzeyi ve olay teslimat yönü

## Durum

`accepted`

## Bağlam

[ADR-0026](0026-playback-renderer-ownership.md) playback session'ın sahibini
**çekirdek** olarak belirledi (yön A) ve [ADR-0012](0012-macos-playback-engine.md)
Karar 2 motoru Swift'e koydu. Aradaki eklem `nen-app`'in
`ShellEngineBridge`'idir: bounded kuyruk, coalescing, `EventsLost` ve reentrancy
yasağı ([ADR-0011](0011-playback-port-contract.md) Karar 1 ve 2) orada, tek
kopya olarak uygulanıyor.

Eksik olan, bu gövdenin **FFI'dan görünmemesi**. `nen-ffi` bugün beş şey dışa
veriyor ve hepsi contract kitini sürmek için var:

| Dışa verilen | Ne için |
|---|---|
| `version()` | NEN-007 smoke |
| `ForeignPlaybackEngine` · `ForeignEngineFactory` | Kitin yargılayacağı motor |
| `runPlaybackContract()` · `applicableScenarioCount()` | Kitin kendisi |

Yani kabuğun `load` / `play` / `seek` diyebileceği bir nesne **hiç yazılmadı**.
`NEN-024` (macOS kabuk) bunsuz başlayamaz; `NEN-026` ve `NEN-027` de aynı
nesnenin üstüne binecek.

**Kararı asıl zorlayan ikinci bulgu, teslimat yönüyle ilgili.** Bounded kuyruk
çekirdekte; adapter'ın kendi biriktirdiği yer değil. macOS adapter'ında olaylar
mpv'nin kendi thread'inde `pending: [FfiPlaybackEvent]` dizisine yazılıyor ve
bu dizinin **sınırı yok** — `drainEvents()` çağrılana kadar büyüyor. Kuyruğun
`EventsLost` üretebilmesi için önce birinin adapter'dan çekmiş olması gerekiyor:

> Uygulama arka planda, kimse çekmiyor. Kuyruk boş, çünkü kuyruğa hiçbir şey
> girmedi. Sınırsız büyüyen dizi adapter'da. Öne gelindiğinde tüketici
> `EventsLost` değil, **birikmiş bin olay** görüyor.

Bu, ADR-0011'in "üretici bloke olmaz, tüketici ya eksiksiz akış görür ya da
eksik olduğunu öğrenir" garantisinin sessizce boşa düşmesidir — ve
[ADR-0031](0031-macos-shell-interaction-model.md) Karar 3'ün `NEN-024`'e verdiği
"arka plandan dönünce sessiz resync" kanıtı tam olarak bu yola bağlı.

**Karar verilmezse:** `NEN-024` yüzeyi implementasyon anında icat eder. İki
olası ad hoc çözümün ikisi de pahalı — kabuk Swift motorunu doğrudan çağırırsa
ADR-0026 geçersiz olur ve sync/çeviri mantığı platformlara dağılır; kabuk kendi
olay yolunu yazarsa Android aynı yolu ikinci kez yazar.

## Karar

Üç karar birlikte alınır.

### Karar 1 — Çekirdek oturumu FFI'da tek bir `PlaybackSession` objesidir

Kabuk motorunu **doğrudan çağırmaz**. Kabuk motoru (`ForeignPlaybackEngine`)
oturuma verir, ondan sonra yalnız oturumla konuşur. Oturum `ShellEngineBridge`'i
sarar; komutlar birebir porta gider.

Sorumluluk ayrımı ADR-0006 kural 2 ve 3'ün yazdığı gibidir:

- **Gövde `nen-app`'te.** Pump, kilit, yaşam döngüsü ve kuyruk orada; bindings
  üretmeden Rust testiyle yargılanabilir.
- **`nen-ffi` yalnız geçit.** Tip çevirisi ve `uniffi::Object` sarmalayıcısı;
  mantık taşımaz.

Oturumun yüzeyi `PlaybackEngine`'in **taban** kümesidir (ADR-0011 Karar 3):
load · play · pause · stop · seek · position · duration · state · track
enumeration/selection · olay akışı · shutdown, artı capability'ye bağlı
`set_rate` / `set_volume`. Katalog, altyazı kaynağı ve metin çıkarımı yüzeyi bu
objeye **girmez** — sırasıyla `NEN-026`, `NEN-027` ve `NEN-044`'ün işidir.

### Karar 2 — Oturum kendi pump'ına sahiptir ve adapter'dan sürekli çeker

Oturum, kabuk istese de istemese de adapter'dan düzenli aralıklarla çeker
(`drain_events` → bounded kuyruk). Aralık çekirdekte tanımlıdır, platform başına
değil.

Bu, Bağlam'daki boşluğun kapandığı yerdir: sınırsız büyüyebilen tek yer
adapter'ın kendi dizisidir, ve pump onu sürekli boşalttığı için birikme her
zaman **bounded kuyrukta** olur. Dolayısıyla taşma `EventsLost`'a dönüşür ve
tüketici eksikliği öğrenir.

### Karar 3 — Teslimat **pull**'dur: kabuk `drain_events()` çağırır

Çekirdek kabuğa olay **itmez**. Kabuk hazır olduğunda — kendi run loop'unda,
kendi cadence'ıyla — `drain_events()` çağırır ve kuyrukta ne varsa alır.

`EventSink` / `deliver_all` ([event.rs](../../core/crates/nen-ports/src/playback/event.rs))
portta kalır ama **FFI'da kullanılmaz**. Ters çağrı olmadığı için ADR-0011
Karar 2'nin yasakladığı durum bu yolda **oluşamaz**; `guard_reentrancy` yine de
her komutta çalışır, çünkü yasak porta aittir ve başka bir tüketici (ileride
`NEN-027`) sink yolunu kullanabilir.

## Gerekçe

**Karar 1.** Alternatifi kabuğun motoru doğrudan çağırmasıydı; bu ADR-0026'yı
fiilen iptal ederdi. ADR-0026'nın gerekçesi performans değildi — subtitle sync
ve çeviri tetikleme mantığının **tek, platformdan bağımsız bir kaynaktan**
yönetilmesiydi. Kabuk motoru doğrudan sürerse o mantık macOS'ta bir kere,
Android'de bir kere yazılır.

Gövdenin `nen-app`'te olması pratik bir kazanç veriyor: oturumun en kritik
davranışları (taşma, coalescing, shutdown sırası) `cargo test` ile, **hiç
binding üretmeden** ölçülebiliyor. `nen-ffi`'de olsalardı her ölçüm için
`build-apple.sh` koşmak gerekirdi.

**Karar 2.** Pump'sız bir pull tasarımı çalışıyor **görünür** ve tam olarak
yanlış anda bozulur. Sayı şu: `EventQueue::DEFAULT_CAPACITY` **64**, ve yorumu
"kısa süre geride kalan bir tüketici için (arka plandan dönen uygulama), gitmiş
olan için değil" diyor. Pump yoksa o 64'lük sınır hiçbir şeyi sınırlamaz —
sınırlanan şey henüz çekilmemiştir. Yani ADR-0011 Karar 1 kâğıt üstünde
kalırdı.

Aralık 30 Hz mertebesinde seçiliyor: `PositionChanged` coalescing sınıfında
olduğu için daha sık çekmek yalnız aynı değeri tazeler, daha seyrek çekmek ise
kritik olayların gecikmesi demektir. Maliyeti saniyede ~30 kilit alma ve bir
`drain_events` çağrısı — NEN-029'un ölçtüğü per-call 36.67 µs ile saniyede
~1.1 ms, yani 1000 ms/sn bütçenin binde biri mertebesi.

**Karar 3.** Push (foreign `EventSink`) yolunun maliyeti olay **başına** bir FFI
+ MainActor hop'u: NEN-029'un ölçtüğü 36.67 µs + ~40 µs ≈ **77 µs**. Bu tek
başına belirleyici değil (60 Hz'de bile ~4.6 ms/sn). Belirleyici olan, push'un
Bağlam'daki problemi **yerinden oynatması**: uygulama askıdayken çekirdek
üretmeye devam eder ve her olay için bir MainActor hop'u dispatch kuyruğunda
birikir. Böylece sınırsız birikme adapter dizisinden dispatch kuyruğuna taşınır
— ki orada bounded kuyruğun coalescing kuralı da işlemez.

Pull'da böyle bir şey yok: askıdaki kabuk hiçbir şey biriktirmez, öne geldiğinde
`drain_events()` çağırır ve — 64 dolduysa — `EventsLost` görür. ADR-0031
Karar 3'ün istediği davranış birebir budur.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Kabuk Swift motorunu doğrudan çağırsın, çekirdek dışarıda kalsın | ADR-0026'yı fiilen iptal eder; sync/çeviri mantığı her platformda yeniden yazılır. `ShellEngineBridge`'in uyguladığı kuyruk, coalescing ve reentrancy garantileri de her adapter'a dağılır |
| Push: foreign `EventSink` + `deliver_all` | Olay başına ~77 µs hop, ve daha önemlisi askıdaki tüketicide birikme dispatch kuyruğuna taşınır — orada ne sınır ne coalescing var. Portun sink yolu duruyor, yalnız FFI'da kullanılmıyor |
| Pump yok; kabuk hem çeker hem tüketir | Adapter'ın kendi dizisi sınırsız kalır, `EventsLost` hiç üretilmez ve ADR-0011 Karar 1 kâğıt üstünde kalır. Ayrıca her platform kendi cadence'ını yazar |
| Pump aralığını platforma bıraktık | Aynı garantinin platform başına yeniden ayarlanması demek; unutan ilk platform sessizce bayat UI üretir |
| Oturum gövdesini `nen-ffi`'ye koymak | ADR-0006 kural 2/3: `nen-ffi` yüzeydir, mantık değil. Her ölçüm için binding üretmek gerekirdi |
| Oturumu `NEN-024`'ün içinde yazmak | Kanıt tipleri karışır: oturumun kanıtı Rust testi, kabuğunki ekran kaydı ve checklist. Task iki katmana yayılır ve `M`'den çıkar |
| Yüzeye katalog/extraction'ı da koymak | `NEN-026`, `NEN-027` ve `NEN-044`'ün işi; şimdi eklemek kullanılmayan bir yüzeyi dondurmak olur |

## Sonuçlar

**Olumlu:**

- `NEN-024` başlayabilir; `NEN-026` ve `NEN-027` aynı objenin üstüne biner.
- ADR-0011 Karar 1'in garantisi ilk kez **gerçekten** yürürlüğe girer — bugün
  adapter dizisi yüzünden yürürlükte değil.
- Android aynı yüzeyi devralır; olay yolu ikinci kez yazılmaz.
- Oturumun taşma/coalescing/shutdown davranışı binding üretmeden test edilir.

**Olumsuz / kabul edilen maliyet:**

- **Çekirdek bir thread'e sahip olur.** Bugüne kadar `nen-app` tamamen
  çağrı-güdümlüydü; pump'ın yaşam döngüsü (durdur, join, sonra motoru kapat)
  artık doğru yapılması gereken bir şey.
- Oturum bir kilit taşır; yavaş bir foreign çağrı pump'ı o süre boyunca
  bekletir. Coalescing sayesinde sonucu gecikmedir, kayıp değil.
- `EventSink` yolu FFI'da kullanılmadan durur — portta ölü kod değil (`NEN-027`
  yeniden değerlendirecek), ama şu an kanıtı yalnız `nen-ports`'un kendi
  testlerinde.

**Geri dönüş maliyeti: ucuz.** Üç kararın hiçbiri port kontratına veya domain
modeline dokunmuyor. Push'a dönmek `drain_events()`'in yerine `deliver_all`
çağıran bir pump demek — oturumun içinde tek bir metot; kabuk tarafında bir
`EventSink` implementasyonu. Pump aralığı zaten tek bir sabit.

## İlgili task'lar

`NEN-045` (yüzeyin kendisi) · `NEN-024` (ilk tüketici) · ileride `NEN-026`,
`NEN-027`

## Notlar

Bu ADR yalnız **çekirdek** yüzeyini karara bağlıyor. macOS'un video yüzeyi
(mpv render API) ve `.app` paketlemesi `NEN-024`'ün kararlarıdır ve kendi
ADR'sine yazılacaktır; ADR-0013 (`SubtitleRenderer` stratejisi) ise `NEN-027`'ye
ait, ikisi de bu kararın kapsamı dışında.

**Karar 3'ün kapsamı `NEN-100`'de netleşti (2026-09-10).** Karar 3'ün pull
tercihi *playback olay akışı* içindir: ~30 Hz'lik pozisyon olayı, olay başına
ölçülen ~77 µs hop ve — daha önemlisi — askıdaki bir tüketicide birikmenin
dispatch kuyruğuna taşınması (yukarıdaki "Reddedilen alternatifler" tablosu).
Bir çeviri işinin ilerlemesi bu profile girmiyor: blok başına birkaç olay,
sürekli değil kesikli. `NEN-100` bu yüzden çeviri ilerlemesini bir foreign
`ForeignTranslationProgressSink` ile **push** olarak taşıyor — `ADR-0004`
Karar 1/2/5'in zaten tanımladığı `JobHandle` + tek delivery gate'in dış
kabuğu. Bu, `EventSink`/`deliver_all`'ın FFI'da genel olarak yeniden
açılması değil: playback oturumunun kendi olay yolu pull kalıyor, ADR-0033
supersede edilmedi. Uygulama ve kanıt: `tasks/done/NEN-100-*.md`.
