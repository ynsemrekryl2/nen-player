---
id: NEN-008
title: Spike - large cue list across FFI
milestone: M1
size: M
state: done
closed: 2026-08-24
depends_on: [NEN-007]
blocks: [NEN-011]
adr: [28]
---

# NEN-008 — Spike: large cue list across FFI

## Sonuç

50.000 cue'luk bir dokümanın UI'a hangi modelle (tam liste vs. pencereli erişim)
sunulacağı ölçümle bilinir.

## Kapsam

- Sahte 50k cue üretimi
- İki yaklaşımın karşılaştırması:
  1. Tam listeyi FFI'dan geçirme
  2. Pencere/handle: `cues(range:)`, `activeCue(at:)`
**Baseline olarak raporlanacaklar** (pass/fail eşiği **değil** — bütçe ADR-0027
ile sonradan kabul edilir):

| Ölçüm | Not |
|---|---|
| Toplam geçiş süresi | her iki yaklaşım için |
| p50 / p95 erişim süresi | pencereli erişimde |
| Peak memory / RSS | her iki yaklaşım için |
| UI thread bloklanma süresi | gözlemlenen en uzun blok |

Her ölçümle birlikte **bağlam** kaydedilir: fixture cue sayısı ve veri boyutu ·
cihaz · OS/toolchain sürümü · debug mi release mi build. Bağlamsız sayı kanıt
sayılmaz.

## YAPILMAYACAK

- Gerçek SRT parse → M2
- Renderer entegrasyonu → NEN-027
- Spike kodunun ürüne terfisi — `core/spikes/` altında kalır

## Kanıt (DoD)

- [x] Baseline tablosu: her iki yaklaşım için süre, p50/p95, peak RSS
- [x] Ölçüm bağlamı (fixture boyutu, cihaz, OS/toolchain, build tipi) kayıtlı
- [x] Pencereli erişimin UI thread'i ne kadar blokladığı ölçüldü
- [x] Hangi yaklaşımın önerildiği ve **neden** yazıldı (sayılarla)

## Kanıt kaydı

> **Aşağıdaki sayılar baseline'dır, eşik değildir** (M1 kuralı). Performans
> bütçesi ADR-0027 ile, gerçek kullanım görüldükten sonra kabul edilir.

### Bağlam

```
fixture : 50 000 cue · 3.1 MiB (Rust tarafı payload: UTF-8 metin + cue başına 20 B sabit alan)
cihaz   : Apple M5 · arm64
OS      : macOS 27.0 (26A5416b)
rustc   : 1.98.0 (88d9e12ae 2026-08-18)      swift : Apple Swift 6.4 (swiftlang-6.4.0.30.4)
uniffi  : 0.32.0                             tarih : 2026-08-24
komut   : bash scripts/spike-cues.sh [--debug]
```

Fixture üretilmiştir, telif-temizdir, **deterministiktir**: `all_cues` iki
koşuda birebir aynı byte'ları veriyor (`generated_cues_are_deterministic`), ve
debug ile release koşularının Swift tarafındaki checksum'ları birbirinin aynı —
A için `75001045577800`, B için `34337591381145`. Yani debug/release farkı
**yalnız hızda**, veride değil.

İki yaklaşım **ayrı process'lerde** koşuyor; tek process'te peak RSS ölçümleri
birbirini kirletirdi. Çağrılar `main.swift`'in üst seviyesinde, yani **ana
thread'de** — "UI thread ne kadar bloklandı" sorusu buna dayanıyor.

### Baseline — release (karar bu satırlara göre verilir)

| Ölçüm | A — tam liste | B — pencere/handle |
|---|---|---|
| ilk çağrı | 48.379 ms | 2.669 ms *(handle kurulumu)* |
| p50 | 37.708 ms | **44.33 µs** *(40 cue'luk pencere)* |
| p95 | 38.657 ms | **59.71 µs** |
| min / max | 37.208 / 48.379 ms | — / 80.25 µs |
| **en uzun tek main-thread bloğu** | **48.379 ms** | **0.080 ms** |
| `activeCue(atMs:)` p50 / p95 / max | — | 1.50 / 1.58 / 19.00 µs |
| peak RSS | 19.5 MiB | 11.7 MiB |
| tüm dokümanı görmenin maliyeti | 37.7 ms *(tek çağrı)* | 50.7 ms *(1250 pencere çağrısı)* |

Örneklem: A için 20 tekrar; B için 10 000 rastgele pencere + 10 000 rastgele
`activeCue` (8313'ünde cue vardı, kalanı cue'lar arası 500 ms boşluğa denk
geldi — yani `None` dalı da ölçüldü). Konumlar sabit tohumlu bir LCG'den
geliyor: rastgele ama tekrarlanabilir, ve **sıralı değil** — sıralı erişim
ölçseydik yalnız cache davranışını ölçmüş olurduk.

### Baseline — debug (karşılaştırma için)

| Ölçüm | A — tam liste | B — pencere/handle |
|---|---|---|
| p50 | 65.841 ms | 54.96 µs |
| p95 | 69.507 ms | 58.79 µs |
| en uzun main-thread bloğu | 85.127 ms | 0.155 ms |
| peak RSS | 21.0 MiB | 12.4 MiB |

Debug, A'yı **1.75×**, B'nin pencere erişimini **1.24×** yavaşlatıyor. NEN-007'nin
kanıtı debug'dı; bu tablo o sayılarla bu sayıların neden karşılaştırılamayacağını
kayda geçiriyor.

### Öneri: **B — pencere/handle**, iki kayıtla

**1. Neden B?** A'nın tek çağrısı ana thread'i **48 ms** blokluyor. 60 fps'te bir
kare 16.7 ms; yani doküman açılışında yaklaşık **3 kare** düşüyor ve bu kullanıcının
göreceği bir takılma. B'nin en uzun tek bloğu **0.080 ms** — aynı bütçenin
**%0.5**'i. Fark 600×. Peak RSS de B'de **11.7 MiB**'e karşı A'da 19.5 MiB: tam
liste Swift tarafında ikinci bir kopya olarak yaşıyor.

`activeCue` **1.50 µs**. Saniyede 60 position güncellemesi bile toplam ~90 µs/s
eder, yani bir karenin binde beşi. Playback sırasında cue aramanın FFI maliyeti
pratikte **yok**; bu sayı M7'nin position çözünürlüğü tartışmasına ve NEN-029'a
girdi.

**2. Kayıt: pencere, cue başına maliyeti düşürmüyor.** Cue başına marshalling
A'da 0.754 µs, B'de 1.108 µs (44.33 µs / 40 cue) — pencere **cue başına %47 daha
pahalı**, çünkü çağrı başına sabit maliyet yalnız 40 öğeye bölünüyor. B'nin
kazancı hızdan değil, **ödemediği cue'lardan** geliyor: UI zaten aynı anda 50 000
cue göstermiyor.

**3. Kayıt: gerçekten tamamı gerekiyorsa A daha iyi.** Dokümanın tamamını pencere
pencere gezmek **50.7 ms**, tek çağrıda almak **37.7 ms** — pencereli tarama
**%35 daha pahalı**. Yani "her şey pencereli olsun" kuralı yanlış olur; tüm
dokümanı işleyen bir yol (export, yazma, toplu doğrulama) tek geçişi tercih
etmeli. Bu ayrım ADR-0003'ün (büyük veri geçiş modeli) kararına girdi.

**4. Ne öğrenmedik.** 3.1 MiB'lık payload 37.7 ms'de geçiyor → efektif ~83 MiB/s.
Bu bir bellek kopyalama hızı değil; maliyet **cue başına** struct/string
lift'lemede. Dolayısıyla A'nın maliyeti cue **sayısıyla** ölçekleniyor, cue
metninin uzunluğuyla değil — bu spike bunu ayrıştırmadı, gerekirse ayrı ölçüm
konusu.

### Doğrulama çıktıları

```
$ cargo test --manifest-path core/Cargo.toml --workspace
spike_cue_transfer : 8 passed, 0 failed
nen-app / nen-ffi  : 1 + 1 passed, 0 failed
toplam 10 passed, 0 failed (26 target)                        → exit 0
```

Spike'ın 8 testi ölçümün **doğru şeyi** ölçtüğünü kanıtlıyor — özellikle
`window_matches_the_same_slice_of_the_full_list`: B'nin döndürdüğü pencere,
A'nın tam listesinin aynı diliminin birebir aynısı. Olmasaydı iki sütun
karşılaştırılamazdı. Ayrıca `active_cue_is_none_in_the_gaps_and_outside_the_document`
ile boşluk/sınır davranışı, `no_two_cues_share_their_text` ile de fixture'ın
gizli bir dedup'tan faydalanmadığı sabitleniyor.

Ürün tarafı bozulmadı:

```
$ bash scripts/test-apple.sh
✔ Test run with 2 tests in 1 suite passed                     → exit 0
$ bash scripts/test.sh
  check-docs.test.sh ✓ · doctor.test.sh ✓                     → exit 0
$ bash scripts/check-docs.sh
  8/8 denetim geçti                                           → exit 0
```

### ADR-0028 sınırları — mekanik doğrulama

```
$ grep -rn --include='*.rs' -E "uniffi::|extern \"C\"" core/crates | grep -v crates/nen-ffi/
  (çıktı yok)                        → ürün kodunda nen-ffi dışında FFI yüzeyi yok

$ cargo metadata --manifest-path core/Cargo.toml --format-version 1 --no-deps
  spike paketleri        : ['spike-cue-transfer']
  crates/* → spikes/* kenarı : (yok)  → ADR-0006 kural 3 korunuyor

$ grep -rn "spike" platforms/apple-shared/Package.swift
  (çıktı yok)                        → ürün Swift paketi spike binding'ini tanımıyor

$ git check-ignore -v core/spikes/spike-cue-transfer/apple-harness/generated
  .gitignore:24:/core/spikes/**/generated/   → üretilen binding commit edilmiyor
```

### Not — sonraki task'ları ilgilendiren

Swift harness'ı ölçtüğü Rust crate'inin **içinde** duruyor
(`core/spikes/spike-cue-transfer/apple-harness/`), `core/spikes/` altında
kardeşi olarak değil: `core/Cargo.toml` `members = [..., "spikes/*"]` diyor,
yani `spikes/` altındaki her dizin Cargo paketi sanılıyor ve Swift paketi orada
workspace'i kırıyor. NEN-009/010/029 kendi harness'larını kurarken aynı düzeni
izlemeli. Yan fayda: spike'ın tamamı tek dizin — ADR-0028'in "geri dönüş: dizini
sil" maliyeti aynen geçerli.
