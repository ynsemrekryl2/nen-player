# M1 — Core Technical Spike

## Amaç

Shared core dilini **ölçüme dayanarak** kilitlemek. **Aday** Rust core'un Swift
ve Kotlin'e; async progress, cancellation, typed error, büyük cue listeleri ve
**reverse callback ile playback ownership** açısından kabul edilebilir maliyetle
bağlanıp bağlanamadığı bu milestone'da belirlenir. Çıktı kod değil **karardır**
(ADR-0002, ADR-0026).

> Bu milestone bitene kadar Rust, UniFFI ve C ABI **adaydır**. Bkz.
> `docs/architecture.md` → "Karar statüsü".

## Kapsam

- Cargo workspace + `nen-ffi` binding iskeleti (aday: UniFFI)
- Üç ölçüm spike'ı (büyük cue listesi, async+cancellation, typed error)
- Aynı üçünün Kotlin/JVM paritesi
- **Reverse-FFI ownership spike'ı** (NEN-029): core mu platform shell mi
  playback session'ın sahibi?
- Milestone-aware toolchain doctor (NEN-030)
- CI iskeleti ve log redaction yardımcıları (ikisi de crate gerektirdiği için
  M0'dan buraya taşındı)
- Spike raporu ve ADR-0002

## Kapsam dışı

- Gerçek domain modeli, SRT parse → M2
- Android cihazda çalıştırma → M10 (buradaki Kotlin testi JVM üzerinde)
- Spike kodunun ürüne terfisi — `core/spikes/` altında kalır

## Baseline ölçümleri (pass/fail **değil**)

M1'in işi eşik doğrulamak değil, **bütçeyi öğrenmek**. Ölçüm yapılmadan konulan
sayı keyfîdir; bu yüzden aşağıdakiler raporlanır, geçilmesi gereken sınır olarak
kullanılmaz. Performans bütçesi baseline ve gerçek kullanıcı deneyimi
değerlendirildikten sonra **ADR-0027** ile kabul edilir.

Her ölçümde birlikte kaydedilecek bağlam:

| Bağlam | Neden |
|---|---|
| fixture cue sayısı ve veri boyutu | Sayı, girdisi olmadan anlamsız |
| cihaz · OS · toolchain sürümü | Farklı makinede karşılaştırılabilsin |
| debug / release build | Debug build 10–100× yanıltabilir |

| Ölçüm | Raporlanacak | Task |
|---|---|---|
| 50k cue FFI erişimi | Toplam süre · p50/p95 · peak memory/RSS · tam liste vs. pencereli karşılaştırma | NEN-008 |
| Cancellation | Cancellation latency · iptal sonrası callback sayısı · release doğruluğu | NEN-009 |
| Typed error | Varyant sayısı · eşleme maliyeti | NEN-010 |
| Kotlin paritesi | Üç ölçümün JVM karşılığı ve Swift'ten sapmalar | NEN-011 |
| Reverse-FFI | A/B: thread dönüş maliyeti · event ordering · position update frekansı (iki uçta) · FFI çağrı hacmi | NEN-029 |

## Invariant'lar (pass/fail — değişmez)

Bunlar ölçüm sonucuna göre gevşetilemez; K15 ve K20'nin doğrudan karşılığıdır:

| # | Invariant | Task |
|---|---|---|
| I1 | Cancellation sonrası **late callback yok** | NEN-009, NEN-029 |
| I2 | Cancellation sonrası **late commit yok** | NEN-009 |
| I3 | Typed error **string parse gerektirmiyor** | NEN-010, NEN-011 |
| I4 | **Resource/thread sızıntısı yok** (tekrarlı iptal ve shutdown sonrası) | NEN-009, NEN-029 |
| I5 | Semantic sonuçlar **Swift ve Kotlin arasında aynı** | NEN-011 |

## No-go sinyalleri

`no-go`, "baseline beklediğimizden yavaş çıktı" demek **değildir** — bütçe
sonradan ADR-0027 ile ayarlanabilir. `no-go` şu durumlarda verilir:

- Bir **invariant** (I1–I5) sağlanamıyorsa
- Baseline, hiçbir makul tasarımla kullanılabilir bir ürüne izin vermiyorsa
  (ör. pencereli erişim de UI'ı donduruyorsa)
- Reverse-FFI'ın **her iki yönü** de (A ve B) kabul edilemez çıkıyorsa

Bu durumda ADR-0002'de alternatifler değerlendirilir: Kotlin Multiplatform core ·
Swift core + Android'de ayrı implementasyon · C++ core. `no-go` halinde M2
dışındaki tüm plan yeniden yazılır.

## Çıkış kriterleri

- [ ] Swift test target'ı core fonksiyonunu çağırıp geçiyor
- [ ] Beş baseline ölçümünün **sayıları ve bağlamı** kaydedildi
- [ ] Invariant'lar I1–I5 kanıtlandı
- [ ] Reverse-FFI A/B karşılaştırması yapıldı ve M7 position çözünürlüğüne
      etkisi raporlandı
- [ ] Redaction guard testi geçiyor ve `derive(Debug)` eklenince kırılıyor
- [ ] CI yeşil; kasıtlı ihlalde kırmızı
- [ ] ADR-0002 `accepted`, reddedilen alternatifler dolu
- [ ] ADR-0026 (playback ownership yönü) `accepted`
- [ ] `bash scripts/doctor.sh M1` çıkış kodu 0

## Task'lar

`NEN-005` · `NEN-006` · `NEN-007` · `NEN-008` · `NEN-009` · `NEN-010` ·
`NEN-011` · `NEN-012` · `NEN-029` · `NEN-030`

## Bağımlılıklar

M0. **Bu milestone bir kapıdır** — NEN-012 kapanmadan M2 dışında hiçbir
milestone başlamaz. NEN-012 artık hem NEN-011'i hem **NEN-029'u** bekler.

Sıra önerisi: **NEN-030** (toolchain gerektirmez) → NEN-007 → NEN-008/009/010 →
NEN-029 → NEN-011 → NEN-005/006 → NEN-012.

## Retro

<!-- M1 kapanışında doldurulacak -->
