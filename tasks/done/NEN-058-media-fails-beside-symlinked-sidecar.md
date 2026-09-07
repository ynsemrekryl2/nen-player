---
id: NEN-058
title: A medium fails to load when a symlinked sidecar sits beside it
milestone: M3
size: S
state: done
closed: 2026-08-27
depends_on: [NEN-022, NEN-025]
blocks: []
adr: [12]
---

# NEN-058 — A medium fails to load when a symlinked sidecar sits beside it

## Sonuç

Yanındaki altyazı dosyası ne olursa olsun medya açılır; teşhis edilir ve
kanıtla kapanır.

## Bağlam

`NEN-025`'in kapanış acceptance'ında **gözlemlendi, teşhis edilmedi.**
`fixtures/media/contract-clip.mkv`'nin iki kopyası aynı dizine kondu:

| Dosya | Yanındaki `.srt` | Sonuç |
|---|---|---|
| `Probe.mkv` | geçerli, düz dosya | oynadı |
| `Linked.mkv` | **symlink** (geçerli bir `.srt`'ye) | `Dosya okunamadı.` — art arda 2 kez |

`cmp` iki `.mkv`'nin **byte-eş** olduğunu doğruladı, yani fark medyada değil.
Hata `FfiLoadFailure::unreadable`, yani mpv `loadfile`'ı reddediyor.

**En olası hipotez — sınanmadı:** adapter mpv'nin `sub-auto` ayarını
kapatmıyor, mpv medyanın yanındaki `.srt`'yi kendi başına yüklemeye çalışıyor
ve symlink'te takılıyor. Doğruysa asıl bulgu daha geniş: **altyazıyı hangi
katmanın yükleyeceği** ADR-0026 ve `NEN-027` ile çekiliyor, mpv'nin kendi
kendine sidecar yüklemesi o sınırı deliyor ve kataloğu atlayan görünmez bir
kaynak yaratıyor.

Komut satırından `mpv` ile izole etme denendi ve **başarısız oldu** — kullanılan
çağrı üç dosyada da (kontrol dahil) asılı kaldı, yani ölçüm aracı çalışmadı,
dosyalar hakkında bir şey söylemedi. Teşhis bu task'ın işi.

Kural 5 gereği `NEN-025`'e eklenmedi.

## Kapsam

- Semptomu tekrar üret ve **ölç** — `sub-auto` hipotezini doğrula veya çürüt
- Doğruysa: mpv'nin kendi sidecar yüklemesi kapatılır; altyazı yükleme tek
  yerden, katalogdan geçer
- Yanlışsa: gerçek kök neden bulunur ve kaydedilir

## YAPILMAYACAK

- Hipotezi ölçmeden uygulamak
- Altyazı render'ı → `NEN-027`

## Kanıt (DoD)

- [x] Semptomun kök nedeni **ölçümle** gösterildi (kod okuması yeterli değil)
- [x] Yanında symlink `.srt` olan medya açılıyor
- [x] Negatif: düzeltme geri alındığında semptom aynen geri geliyor
- [x] `sub-auto` hipotezi doğruysa: mpv'nin katalog dışı bir altyazı
      yüklemediğini gösteren test

## Kanıt kaydı

Tam ölçüm: [`evidence/M3/NEN-058-measurement.md`](../../evidence/M3/NEN-058-measurement.md)

### Hipotez ikiye ayrıldı: yarısı doğru, yarısı yanlış

Task `sub-auto`'yu tek hipotez olarak açmıştı — "mpv sidecar'ı kendi yüklüyor
ve **symlink'te takılıyor**". Ölçüm bunu ikiye böldü.

**Yanlış olan yarısı: symlink değişken değildi.** Aynı medyanın yanına dokuz
farklı komşu kondu — düz `.srt`, kardeşe mutlak/göreli symlink, kırık symlink,
dizin dışına symlink, dizine symlink, kendine dönen symlink, boş `.srt` — ve
**hiçbiri** medyayı düşürmedi; dokuzunda da `state=ready`.

**Değişken sıraydı.** NEN-025'in elle koşusu `Probe.mkv`'yi önce, `Linked.mkv`'yi
sonra açmıştı. Tek motorda aynı düzen kurulunca semptom birebir üredi: **ikinci**
medya, hangisi olursa olsun, `LoadFailed(unreadable)` üretiyor. Ters sırada
`Linked.mkv` ilk açıldığında hiçbir hata yok. Symlink ile sıra o koşuda
birbirine karışmıştı.

**Kök neden.** `loadfile` açık bir medyanın üstüne geldiğinde mpv **giden**
entry'yi bitiriyor: `reason=STOP, error=0` — container olmayan bir dosyanın
verdiğiyle **aynı** şekil, ki kodun kendi yorumu bunu zaten kaydetmişti.
Adapter yalnız `stopRequested` ile susuyordu, dolayısıyla bu sonu yeni dosyanın
hatası sanıyordu. Motor ardından `ready` oluyor, ama kabuk yoldan geçen fatal
olayı görür görmez `Dosya okunamadı.` basıyor.

**Ayırt eden işaret mpv'nin kendisinde.** `playlist_entry_id`: giden entry `1`,
yeni entry `2`. `loadfile` yeni id'yi `mpv_command_ret` ile **senkron**
döndürüyor ve bu, giden dosyanın `end_file`'ından ~560 µs **önce** oluyor —
ölçüldü. Kör yutma kullanılmadı; NEN-051'de reddedilen yaklaşımın aynısı olurdu.
Yarış ayrıca yapısal olarak kapatıldı: `load()` komuttan **önce** id'yi
`noEntry`'ye çekiyor.

### Doğru çıkan yarısı: mpv kataloğu atlayan bir kaynak açıyordu

Fixture'ın iki gömülü subtitle track'i var (ff-index 3, 4). Yanında `.srt`
varken adapter **üçüncü** bir track raporluyordu: `[3, 4, 0]` — external
track'in ff-index'i yok. `sid=no` yalnız **gösterimi** kapatıyor; mpv'nin
default `sub-auto=exact`'i dosyayı **açmaya** devam ediyor.

Bu, kataloğun hiç görmediği, menünün hiç listelemediği (ADR-0031 Karar 4/5) ve
NEN-025'in dört kapısının hiç incelemediği bir kaynak demektir — **symlink
kapısı dahil**: symlink'li komşu da `[3, 4, 0]` veriyordu. `sub-auto=no` eklendi.

### Testler ve negatif kontrol

`SidecarLoadingTests.swift` — 6 test. Durum tek başına kanıt değil: düzeltmeden
**önce de** motor sonunda `ready` oluyordu, kanıt olan şey yoldan geçen `failed`
olayı.

| Kaldırılan | Kırmızı | Hangi testler |
|---|---|---|
| `playlist_entry_id` guard'ı | **2** | `asecondMediumOpens...` · `theOrderOfTheTwoMedia...` |
| `("sub-auto", "no")` | **2** | `aSidecarBeside...` · `aSymlinkedSidecar...` |

Birincisi semptomu birebir geri getiriyor. İkisi ayrık; her düzeltmenin kendi
testleri var. Ayrıca bir test guard'ın sağır kalarak sessizlik satın almadığını
ölçüyor: açık bir medyadan sonra **gerçek** bir yükleme hatası hâlâ raporlanıyor.

**Ölçümün kendisi bir kez düzeltildi.** İlk turda negatif kontroller
`git checkout <path>` ile geri alınıyordu; dosyalar commit edilmediği için bu iki
düzeltmeyi birden HEAD'e döndürdü ve B kontrolü 4 kırmızı gösterdi. Sayılar
izole ölçümle yeniden alındı.

### Koşular

- macOS paketi **seri**: `62 test / 8 suite`, art arda **2/2** yeşil (56 → 62).
- Rust workspace: **489 passed / 0 failed** — bu iş Rust'a dokunmadı.
- macOS paketi **paralel**: 1 kırmızı, `PlayerModelTests` içindeki
  `FakeSession` tabanlı bir shell testi. `NEN-049`'un aradığı sınıfın ikinci
  görülüşü; oraya not düşüldü, iş eklenmedi (Kural 5).

### Kapsam dışında bırakılanlar

Yeni bir ADR yazılmadı. `sub-auto=no`, `sid=no`'nun yorumunun zaten yazdığı
niyetin eksik kalmış yarısı; ADR-0011 Karar 3 ile ADR-0031 Karar 4/5 sınırı
çoktan çiziyor.
