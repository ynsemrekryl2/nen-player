# NEN-048 — Geçici hata sınıfı checklist

Tarih: 2026-08-26

Ortam: Apple arm64 · macOS 27.0 (26A5416b) · Xcode 26.6 (17F113) ·
Swift 6.3.3 · libmpv 2.5.0 · debug, ad-hoc imzalı `.app`.

## Manuel acceptance

1. `bash scripts/build-macos-app.sh` çıkış 0 verdi ve
   `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`
   doğrulamadan geçti.
2. Uygulama ilk açıldığında boş durum göründü: bırakma hedefi, `Aç...` düğmesi
   ve son medya satırı. **Hiçbir bildirim çıkmadı.**
3. Finder öne alınarak Nen Player arka plana düşürüldü.
4. Nen Player'ın başlık çubuğuna tıklanarak uygulama yeniden etkinleştirildi.
   Etkinleşmenin **hemen ardından** ve **1 sn sonra** alınan iki ekran
   görüntüsünde de boş durum bildirimsizdi — `Önce bir medya açın.` çıkmadı.

Sonuç: **4/4 geçti**. `applicationBecameActive()` → `resynchronize()` zinciri
artık kullanıcıya haber üretmiyor.

## Kusurun düzeltmeden önceki hali

Düzeltme test düzeyinde geri alındı (`resynchronize()`'ın `catch`'i yeniden
`presentTransient`'a bağlandı) ve paket koşuldu. İki test tam olarak
bildirilen semptomu üretti:

```
✘ "resynchronizing without media says nothing"
  Expectation failed: (model.transientMessage → "Önce bir medya açın.") == nil
✘ "EventsLost resynchronization stays silent when the engine refuses"
  Expectation failed: (model.transientMessage → "Önce bir medya açın.") == nil
✘ Test run with 39 tests in 6 suites failed with 2 issues
```

Düzeltme geri konduğunda aynı paket 39/39 geçti. Yeni testler boş değildir:
kusur geri geldiğinde kırmızıya dönüyorlar.

## Negatif testin boş olmadığının kanıtı

`PlaybackPresentation.errorMessage`'a bilinçli bir sızıntı enjekte edildi
(`EngineFailure` metnine motor adı + opak kod eklendi) ve paket koşuldu:

```
✘ "transient copy carries no path, engine name, or numeric code"
  Expectation failed: !(carriesDigits → <not evaluated>)
  Expectation failed: !((message → "Medya oynatılamadı (mpv 4242).").contains("mpv") → true)
✘ "fatal copy contains neither a path nor engine internals"
✘ Test run with 39 tests in 6 suites failed with 5 issues
```

Sızıntı geri alındı; `PlaybackPresentation.swift` üründe değişmedi
(`git diff` boş). Negatif test hem geçici hem fatal metni koruyor.

## Otomatik kanıt

`swift test --package-path platforms/macos --no-parallel` çıkış 0, art arda
**2/2**: **39 test / 6 suite**, 0 failure (NEN-046 kapanışındaki 31'in üzerine
8 yeni test). Bu task'ın sekiz testi her koşuda yeşildi.

| Test | Neyi kanıtlıyor |
|---|---|
| `resynchronizing without media says nothing` | Medya yokken etkinleşme bildirim üretmiyor |
| `EventsLost resynchronization stays silent when the engine refuses` | Sessiz resync davranışı korunuyor (ADR-0031 Karar 3) |
| `a refused seek is reported transiently, not fatally` | Reddedilen seek geçici bildirim veriyor, `fatalMessage` boş kalıyor |
| `a refused volume change is reported transiently, not fatally` | Reddedilen ses değişimi aynı şekilde; `volume` geri alınmıyor |
| `a refused play command is reported transiently, not fatally` | Reddedilen oynat aynı şekilde; state `failed` olmuyor |
| `transient copy carries no path, engine name, or numeric code` | 11 hata varyantının metninde `/`, rakam ve motor adı yok; metin `String(reflecting:)` ve `localizedDescription`'a düşmüyor |
| `every playback error variant has its own Turkish sentence` | 11 varyantın 11 farklı, boş olmayan Türkçe cümlesi var; bilinmeyen hata kapalı kümeye düşüyor |
| `a transient message clears itself` | Bildirim ömrü dolunca kendiliğinden siliniyor |

`FakeSession` artık çağrı bazında hata enjekte edebiliyor
(`errors[.seek] = .NotLoaded`). NEN-024'te motor hatasından türeyen dört
`presentTransient` yolunun hiçbiri testte çalışmıyordu; bunlardan
`resynchronize()` kaldırıldı, kalan üçü (`seek`, `setVolume`,
`togglePlayback`) artık doğrudan test ediliyor.

Kapsam dışı bırakılanlar: son-medya deposundan türeyen üç geçici yol
(`recentStore.save` / `resolve` hataları ve boş çözümleme) hâlâ testsiz —
bu task'ın kapsamı motor hatasından türeyen sınıftı. Ayrı bir iş olarak
`NEN-050`'ye kaydedildi.

## NEN-049 bu koşuda tekrarlandı

Kapanış doğrulaması sırasında paralel paket koşularından biri
`aSeekIsAnsweredThroughTheSession` testinde kırmızı oldu:

```
✘ SessionTests.swift:75: (landed! > 11_900 → false)
```

Ölçüm: **seri 2/2 yeşil**, **paralel 3/4 yeşil**. Kırmızı olan test
`NenPlaybackMPVTests` hedefinde gerçek libmpv ile çalışıyor; bu task'ın
değişikliği yalnız `NenPlayerShell` hedefinde (`PlayerModel.swift`, 8 satır)
ve `PlaybackPresentation.swift` üründe hiç değişmedi. İki hedef arasında
bağımlılık yok, dolayısıyla kusur bu task'tan gelmiyor — `NEN-049`'un
kaydettiği paralel izolasyon kusurunun aynısı.

Not: bu task seçilirken `NEN-049`'un öncülü doğrulanamamıştı — paket o sırada
paralel modda art arda **4/4** yeşil gelmişti. Kusur "her koşuda kırmızı"
değil, **yük altında aralıklı**. `NEN-049` bu bilgiyle yeniden ele alınmalı:
DoD'sinin istediği "değişiklik öncesi paralel kırmızı" kaydı tek koşuyla değil,
tekrarlı koşuda kırmızı **oranıyla** üretilebilir.
