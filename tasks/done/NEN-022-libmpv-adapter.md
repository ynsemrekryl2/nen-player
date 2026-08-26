---
id: NEN-022
title: libmpv playback adapter for macOS
milestone: M3
size: L
state: done
depends_on: [NEN-021, NEN-004]
blocks: [NEN-023, NEN-024, NEN-043]
adr: [11, 12]
---

# NEN-022 — libmpv playback adapter for macOS

## Sonuç

Gerçek libmpv adapter'ı, fake adapter ile aynı contract test kitini geçer.

## Kapsam

- libmpv bağlama ve yaşam döngüsü (init/shutdown)
- load · play · pause · stop · absolute/relative seek · position · duration ·
  rate · volume
- State eşlemesi: buffering / ready / ended / failure
- Event stream

## YAPILMAYACAK

- Track enumeration/seçimi → NEN-023
- Altyazı render → NEN-027
- Dağıtım/notarization/lisans kararı → ADR-0012 (bu task'ta yalnız geliştirme
  ortamında çalışır hale getirilir; **karar ayrı**)

## Kanıt (DoD)

- [x] Aynı contract kiti gerçek libmpv ile geçiyor
- [x] Yerel klip fixture'ıyla oynat/duraklat/seek doğrulandı
- [x] Duration'ın ötesine seek → `ended` state (kenar durum)
- [x] Bozuk/eksik dosyada `failure` state, panik yok
- [x] Negatif: adapter loglarında medya yolu/URL **yok**
- [x] Ek: motor adı `nen-ffi` koduna da girmiyor (guard taraması genişletildi)

## Kanıt kaydı

**Ortam:** Apple M5 · macOS 27.0 · Xcode 26.6 · Swift 6.3.3 · libmpv 2.5.0
(Homebrew `mpv 0.41.0_8`) · ffmpeg 9.0.1 · debug build.

### DoD #1 — aynı kit, gerçek motor

```
bash scripts/test-macos.sh
✔ Test theRealAdapterPassesTheSharedContractKit() passed after 12.293 seconds.
✔ Test run with 9 tests in 2 suites passed after 12.293 seconds.
```

Kit **ikinci kez yazılmadı** (ADR-0011 Karar 4): `nen-ports`'un senaryo listesi
`nen-ffi`'ın `run_playback_contract`'ı üzerinden sürülüyor. Senaryo sayıları:
toplam **23**; tam capability'li motorda 19, taban-only motorda 18, libmpv
adapter'ında (`PlaybackRate` + `Volume`) **18**. Swift testi `applied`'ı kitin
kendi `applicableScenarioCount`'una eşitliyor — sayı elle yazılmadığı için
senaryo eklendiğinde eşik kendiliğinden sıkılaşıyor.

### DoD #2/#3/#4/#5 — gerçek klip

`fixtures/media/contract-clip.mkv` (30 s · 160x90 @5fps · 2 audio + 2 subtitle,
73 KB) ve `fixtures/media/broken-clip.mkv` (8 KB deterministik çöp). İkisinin de
üretim komutu yanlarında `.ffmpeg.txt` olarak duruyor.

| Test | DoD |
|---|---|
| `playPauseAndSeekOnTheLocalClip` | #2 — seek 12 000 ms'e 12 000 ms'de indi; duration 30 008; 2+2 track |
| `seekingPastTheDurationEndsPlaybackInsteadOfFailing` | #3 — 9 999 999 ms → `ended`, hata değil; medya yüklü kaldığı için duration hâlâ cevap veriyor |
| `aFileThatIsNotMediaFailsWithoutCrashing` · `aMissingFileFailsWithoutCrashing` | #4 |
| `shutdownIsIdempotentAndRefusesEverythingAfterwards` | port şartı |
| `aFailedLoadReportsNothingAboutWhichMedium` · `aContractReportNamesNoMediumEither` · `theCheckWouldCatchARealLeak` | #5 |

### Ölçüm: libmpv takılmıyor, takılan mpv CLI'ı

`scripts/doctor.sh` "mpv CLI'ı çalıştırmak takılabildiği için ÖNCE kütüphaneyi
arıyoruz" diyor. Bu makinede doğrulandı: `mpv --frames=1` 20 sn'de dönmedi,
**libmpv aynı dosyada 0.5 sn'de tamamladı**. Adapter'ın dayandığı yol kütüphane
olduğu için bu M3 için engel değil — ama doctor'ın yorumu artık ölçülmüş bir
gözlem.

### Ölçüm: mpv'nin gerçekte ne yaptığı

C smoke testleriyle ölçüldü (adapter yazılmadan önce, `core/spikes/` dışında —
atılabilir ölçüm kodu):

| Konu | Ölçüm | Adapter'a etkisi |
|---|---|---|
| `seek 5 absolute+exact` | `time-pos` = **5.000000** (tam) | Tolerans 100 ms tanındı ama gerek kalmadı; marj bu klip için değil, karşılaşılmamış medya için |
| Track id'leri | Tür başına 1 tabanlı — audio 1,2 **ve** sub 1,2 | Port'un id uzayı türler arasında **tek**; adapter `ff-index`'i dışarı veriyor, mpv'nin `id`'si dosyadan çıkmıyor |
| Yükleme olayları | `start-file` → `audio-reconfig`×2 → `file-loaded` | Alt-dizi eşleşmesi zorunlu: kit tam eşitlik isteseydi geçmezdi |
| Bozuk dosya | `end-file` reason=**STOP**(2), error=none | Sebep kodu tek başına ayırt etmiyor |
| Var olmayan dosya | `end-file` reason=**ERROR**(4), error="loading failed" | İkisi de `LoadFailed` |
| EOF'ta `end-file` | `keep-open=yes` ile **gelmiyor** | `Ended`, `eof-reached` gözlemiyle bildiriliyor |

### Kitin gerçekçi hâle getirilmesi — ve bunun bedeli

Kit üç yerde gerçek motoru **haksız yere** çaktırıyordu; üçü de düzeltildi:

1. **Anında durum.** `Load` → `State == Ready` bekliyordu; `loadfile` asenkron.
   → `Action::Settle { until }`.
2. **Tam eşitlikte olay dizisi.** Gerçek motor daha fazlasını raporluyor.
   → alt-dizi eşleşmesi, **artı** beklenmeyen `Failed`/`EventsLost` yasağı.
3. **Senkron seek.** `Seek` → hemen `Position` sorulunca eski pozisyon
   okunuyordu. → `Action::AwaitEvent { shape }` ve `Outcome::Events`'in
   zaman aşımına kadar **beklemesi**. Bu aslında portun kendi tasarımı:
   `SeekCompleted` tam da "seek indi mi" sorusunun cevabı.

Ayrıca kitte **fake'in biyografisi** gömülüydü: `DurationMs(Some(120_000))`,
`TrackCount(2)`, `TrackId(1)`. Bunlar `ContractInputs`'a taşındı; adım artık
sayı değil **rol** adlandırıyor (`TrackRef::Known`, `Outcome::FixtureDuration`).
Gerçek fixture'ın 120 saniye olmaya zorlanması tam olarak ADR-0011 Karar 4'ün
önlemek istediği durumdu.

**Gevşetmenin bedeli ayrıca ödendi.** `contract_kit_is_not_vacuous.rs`, doğru
bir motoru kitin artık tolere ettiği her yönde bozuyor ve altısında da kırmızı
olmasını şart koşuyor: toleransın dışına düşen seek · `Ready`'ye hiç gelmeyen
motor · düşürülen kritik olay · sırası bozulmuş olaylar · sorulmadan gelen
`Failed` · her id'yi kabul eden seçim. Yedinci test kontrolün kontrolü: doğru
motor aynı gevşek girdilerle geçiyor.

### Kitin bulduğu dört gerçek kusur

Gerçek adapter ilk koşuda 4 senaryoda düştü. Üçü yukarıdaki kit kusurlarıydı;
**dördüncüsü gerçek bir adapter hatasıydı:**

> mpv, arka arkaya verilen iki seek'i **tek** `playback-restart`'a birleştiriyor.
> Sayaç her restart'ta bir azaldığı için ikinci seek'in cevabı hiç gelmiyordu —
> o cevabı bekleyen bir kabuk sonsuza kadar beklerdi. Artık bekleyen her seek
> cevaplanıyor; hepsi aynı pozisyonu bildiriyor, ki portun tanımı da bu:
> `SeekCompleted` "ulaşılan pozisyon"dur.

### Köprüde bulunan ve düzeltilen kusur (negatif kontrolle kanıtlı)

`ShellEngineBridge` her çağrıyı olduğu gibi iletiyordu, yani ADR-0011 Karar 2'nin
reentrancy yasağı yalnız **fake kendi guard'ını taşıdığı için** tutuyordu. Swift
adapter'ında böyle bir guard olamaz: işareti taşıyan thread-local Rust'ın.
Karar 2 yasağı açıkça **porta** verdiği için ("her adapter aynı yasağı miras
alır") guard köprüye kondu.

Kanıt boşta dönmüyor: `the_bridge_refuses_a_reentrant_call_before_reaching_the_engine`
her metodu **çağrılınca panikleyen** bir motorla deniyor — guard iletmeden önce
durmazsa test panikle düşer. 15 guard mekanik olarak silinip koşuldu → test
kırmızı; geri alındı → yeşil.

### Köprü Swift'ten önce Rust'ta yargılandı

`playback_bridge.rs` (9 test): fake, Swift'in giyeceği şekli giyip kitten
köprü üzerinden geçiyor. İki negatif kontrol: hiçbir olayı iletmeyen motor ve
medyası hakkında yalan söyleyen fixture — ikisi de kırmızı. Böylece Swift
koşusunun yeşili Swift hakkında bir şey söylüyor.

### K23

`FfiPlaybackError`'ın **hiçbir alanı** serbest metin değil; hepsi sınırlı enum
veya sayı. Sebebi NEN-010'un ölçtüğü şey: UniFFI `errorDescription`'ı
`String(reflecting: self)` olarak üretiyor, yani Rust'ın `Debug`'ı devrede
değil. Swift testleri en kötü durumu (`String(reflecting:)`) 7 yasak parça için
tarıyor; `theCheckWouldCatchARealLeak` kasıtlı sızdıran bir ikizle taramanın
yedisini de yakaladığını gösteriyor. Hata tipinden `operation` alanı bilinçle
çıkarıldı — çekirdek hangi metodu çağırdığını zaten biliyor.

### Sayılar

- Rust testleri **390 → 406**
- Swift testleri: **9** (`platforms/macos`, yeni)
- Yeni dış Rust bağımlılığı **yok**; `core/deny.toml` değişmedi
  (`cargo deny check` → advisories/bans/licenses/sources ok)
- `cargo fmt --check` temiz · `cargo clippy --workspace --all-targets -D warnings`
  temiz

### Kapsam dışı bırakılanlar

- Görüntü yüzeyi yok (`vo=null`/`ao=null`) — pencere NEN-024'ün işi, DoD'un
  hiçbir maddesi istemiyor
- `EmbeddedTextExtraction` ve `ExternalSubtitleInjection` capability olarak
  **bildirilmiyor**; kit ikisi için typed reddi doğruluyor → NEN-023, NEN-027
- Track enumeration port seviyesinde yapıldı (ADR-0011 Karar 3 zorunlu tabana
  koyduğu için kit onsuz geçmiyor); katalog semantiği NEN-023'te
- `.app` içine gömme / notarization → **NEN-043** (ADR-0012 Karar 3)
