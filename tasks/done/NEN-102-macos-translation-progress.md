---
id: NEN-102
title: macOS translation progress and cancellation surface
milestone: M5
size: M
state: done
closed: 2026-09-11
depends_on: [NEN-101]
blocks: [NEN-104]
adr: []
---

# NEN-102 — macOS translation progress and cancellation surface

## Sonuç

Koşan bir çeviri işinin ilerlemesi ekranda görünüyor, kullanıcı onu iptal
edebiliyor ve iptal edilen iş ekranda yarım bir sonuç bırakmıyor.

## Bağlam

Şartname §10 progressive/yarım yayını **yasaklıyor** — yani ilerleme
göstergesi kullanıcıya "ne kadarı bitti"yi söyler, ama ekranda çizilen altyazıyı
blok blok değiştirmez. Bu ayrım bu task'ın en kolay kaçırılan yeri.

Geçici bildirim yüzeyi `PlaybackPresentation` üzerinden zaten var (`NEN-048`,
`NEN-050`), hata sunumu `ADR-0031` ile karara bağlı. Krom `NEN-067`'nin ultra
ince tasarımı — yeni bir kalıcı çubuk eklenmez.

## Kapsam

- İlerleme göstergesi (blok/yüzde) ve iptal eylemi
- İşin bitişinde sonucun hedef dil grubunda menüye düşmesi
- Hata durumunun `ADR-0031`'in sunum kurallarına bağlanması

## YAPILMAYACAK

- Çevrilen metnin blok blok ekrana yansıtılması — **yasak** (§10)
- Kullanıcı başka kaynak izlerken zorla AI çıktısına geçme — **yasak** (§9)
- Arka planda birden fazla iş kuyruğu — kapsam dışı
- Yeni kalıcı krom öğesi — `NEN-067`'nin tasarımı korunur

## Kanıt (DoD)

- [x] Swift testi: ilerleme olayları göstergeye sırayla yansıyor
- [x] Swift testi: iptal eylemi işi durduruyor ve gösterge kayboluyor
- [x] Negatif: iş yarıdayken ekrandaki altyazı **değişmiyor** (progressive yayın yok)
- [x] Negatif: iptal edilen iş menüye hiçbir `Ai` kaynağı eklemiyor
- [x] Gerçek `.app` checklist: ilerleme görülüyor, iptal ediliyor, ekran ve menü temiz kalıyor — `evidence/M5/NEN-102-checklist.md`

## Kanıt kaydı

**Ekranda `PlayerModel.translationProgress` (blok sırası + o bloğun kendi
`done`/`total`'ı) yeni bir `TranslationStatusPill`'e akıyor** —
`PlayerRootView`'in mevcut geçici-mesaj yuvasında, `NEN-067`'nin kalıcı
kromuna hiçbir öğe eklemeden. `translateSelectedSubtitle()` artık
`FfiTranslationEngine.start`'a gerçek bir `sink` (yeni `TranslationProgressRelay`)
veriyor ve gelen olayları bir `AsyncStream` üzerinden sıralı tüketiyor;
`İptal` düğmesi ve `Altyazı` menüsünün iş koşarkenki `AI Çevirisini İptal Et`
komutu, yeni `nonisolated PlayerModel.cancelTranslation()`'ı çağırıyor —
Rust'ın `FfiTranslationJob.cancel()`'ı, `TranslationCancellation` adlı küçük,
paylaşılan bir izleyici üzerinden.

**Yol üstünde `nen-ffi`'de gerçek, üretimde de geçerli bir kusur bulundu ve
aynı task'ta düzeltildi.** `FfiTranslationJob::cancel()` yalnız `JobState`
`Running` iken çalışan handle'a erişebiliyordu; ama `join()`'ün **ilk işi**
state'i `Taken`'a çevirmekti — `join()` çağrıldığı an (ki her çağıran bunu
`start()`'tan hemen sonra yapar, `translateSelectedSubtitle()` dahil)
`cancel()` kalıcı olarak etkisiz kalıyordu, iş dakikalarca sürse bile.
Bu, `nen-ffi/tests/translation_gate.rs`'e eklenen yeni bir regresyon testiyle
(`cancelling_after_join_has_already_been_called_still_stops_the_job`)
doğrudan ölçüldü (join() ayrı bir thread'de çağrılıp bittikten *sonra*
cancel() çağrılınca iş bitmiş oluyordu). Düzeltme: `nen-app::translation`'a
`join()`'ün tükettiği `JobState`'ten bağımsız, `TranslationJobHandle::cancel_handle()`
ile alınan ve `FfiTranslationJob`'a `state` mutex'inin **dışında** saklanan
yeni bir `TranslationCancelHandle` — ADR-0004 Karar 2'nin gate'i değişmedi,
yalnız ona her zaman erişilebilir hale geldi.

**Beş kapı elle mutasyona uğratıldı, her birinde tam olarak beklenen
test(ler) kırmızıya döndü:** işin başlamadan önce iptal kontrolü (yalnız
`cancelBeforeStartNeverStartsAJob`), `TranslationCancellation.adopt(job)`
(yalnız `cancelMidRunStopsTheJob` + `cancelledJobAddsNoAiSource`), bitişte
`translationProgress = nil` sıfırlaması (yalnız `translationProgress == nil`
iddia eden iki test) — kontrol sağır değil. Medya değişimindeki revizyon
koruması (`for await` döngüsünün içindeki) mevcut testlerin hiçbiriyle
kırmızıya döndürülemedi (tek bir olayın *üretilmiş ama henüz tüketilmemiş*
olması gereken çok dar bir yarışı koruyor); bu, olduğu gibi kaydediliyor —
sağır bir kontrol koddan çıkarılmadı çünkü üretimde gerçek bir korumaya denk
geliyor, yalnız bugünkü test takımının onu tetikleyecek hassasiyeti yok.

**Testler, `cancel()`'ın gerçekten çalışan bir işi durdurduğunu, Rust
tarafının kendi testinin (`translation_gate.rs`) aynı deseniyle ölçüyor** —
işi duraklatan gözlemci callback'i (`Rendezvous`), paylaşılan kilide gerçek
bir OS `Thread` üzerinden ulaşan bir iptalci. `Task.detached`'in kendisi bu
iş için kullanılmadı: bir Swift test rendezvous'u parklıyken kuyruklanan bir
`Task.detached` iptalcinin gerçekten *ne zaman* çalışacağının garantisi
olmadığı, ölçülerek bulundu (kuyruklanan iptalci bazen bloğu, iş zaten bitmiş
olduktan sonra elde ediyordu) — gerçek `Thread` bu garantiyi veriyor.

**Gerçek `.app` kabulü** (`evidence/M5/NEN-102-checklist.md`):
`fixtures/media/contract-clip.mkv` (gömülü Türkçe + İngilizce) ve
`fixtures/subtitles/blocks/layout-sample.srt` (95 cue, 3 blok) ile — kullanıcı
altyazısı yüklendi, seçildi, komut verildi; menüde `Türkçe ▸ AI çevirisi (tr)`
belirdi, seçili altyazı **değişmeden** kaldı (§9). Negatif: hedef dille aynı
gömülü Türkçe kaynak seçilince komut soluk görüldü. Mock provider'ın anlık
bitişi yüzünden ilerleme hapı/İptal düğmesi canlı ekran görüntüsünde
yakalanamadı — bu, `NEN-101`'in kendi kaydının öngördüğü bilinen risk;
deterministik kanıt Swift testlerinde. Yol üstünde ölçülen ayrı bir gerçek:
medyanın gömülü **İngilizce** track'i (`NEN-044`, backlog) `NoDocument` ile
kalıcı olarak reddediliyor — kusur değil, embedded metin çıkarımının henüz
uygulanmadığının canlı teyidi.

`bash scripts/test-macos.sh` **256 passed / 31 suites** (iki ayrı koşuda
tekrarlandı, ikisi de yeşil — `NEN-101` baseline 249 + bu task'ın 7 testi:
`TranslationProgressTests`). `cargo test --workspace` **832 passed / 1
ignored** (`NEN-100` baseline 831 + `cancelling_after_join_has_already_been_called_still_stops_the_job`).
fmt, clippy, `cargo deny check` (yeni dış bağımlılık yok), `bash
scripts/test.sh` **4/4** ve `bash scripts/check-docs.sh` yeşil. Değişen Rust
dosyaları: `nen-app::translation` (yeni `TranslationCancelHandle`),
`nen-ffi::translation` (`FfiTranslationJob`'un `cancel()`'ı artık ona
dayanıyor — dış imza değişmedi, binding yeniden üretimi gerekmedi ama yine de
yapıldı). Kanıt: `evidence/M5/NEN-102-checklist.md`.
