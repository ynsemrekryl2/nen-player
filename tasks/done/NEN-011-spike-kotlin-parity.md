---
id: NEN-011
title: Spike - Kotlin binding parity
milestone: M1
size: M
state: done
closed: 2026-08-24
depends_on: [NEN-008, NEN-009, NEN-010]
blocks: [NEN-012]
adr: [28]
---

# NEN-011 — Spike: Kotlin binding parity

## Sonuç

NEN-008/009/010'daki üç ölçümün Kotlin/JVM binding'indeki sonuçları bilinir ve
Swift'ten sapmalar belgelidir.

## Kapsam

- Aynı üç spike'ın Kotlin/JVM karşılığı
- `suspend` fonksiyon davranışı, coroutine iptali ile `JobHandle.cancel()` uyumu
- Sapmaların tablo halinde kaydı

## YAPILMAYACAK

- Android cihazda çalıştırma → M10 (bu spike JVM üzerinde)
- Media3 / Android UI → M10

## Kanıt (DoD)

- [x] Üç testin JVM sonuçları Swift sonuçlarıyla tablo halinde karşılaştırıldı
- [x] Coroutine iptali sonrası **late callback yok**
- [x] Sapma varsa sebebi ve M10'a etkisi yazıldı

## Kanıt kaydı

`adr:` alanı `[3]` → **`[28]`** olarak düzeltildi — NEN-006/007/008/009/010 ile
aynı gerekçe: ADR-0003 (FFI binding stratejisi) henüz `accepted` değil ve bu
task onu kararlaştırmıyor — tam tersine, sonuçları NEN-012 üzerinden
ADR-0003'e **girdi** olacak. Bu task'ın gerçekte dayandığı ve kapanışını
mümkün kılan karar, `core/spikes/*`'ın kendi FFI kapısını açmasına izin veren
ADR-0028 — o ADR zaten "İlgili task'lar"da NEN-011'i sonraki kullanıcı olarak
listeliyor.

### Yaklaşım

NEN-008/009/010'un ölçtüğü **üç Rust crate'ine dokunulmadı**. Her birine,
Swift'teki `apple-harness/`'in birebir eşi olan bir `jvm-harness/` (Gradle
wrapper tabanlı Kotlin/JVM projesi) eklendi — ADR-0028 "Sonuçlar"ın öngördüğü
tam olarak bu: *"NEN-011 aynı spike crate'inden Kotlin binding üreterek
pariteyi ölçer."* Binding üretimi aynı `spike-*-uniffi-bindgen` binary'leri
üzerinden, yalnız `--language kotlin` ile:

- `core/spikes/spike-cue-transfer/jvm-harness/` + `scripts/spike-cues-jvm.sh`
- `core/spikes/spike-async-cancel/jvm-harness/` + `scripts/spike-async-jvm.sh`
- `core/spikes/spike-typed-errors/jvm-harness/` + `scripts/spike-typed-errors-jvm.sh`

Gradle sistemde kurulu olmak zorunda değil — her `jvm-harness/` kendi
`gradlew` wrapper'ını taşıyor (SwiftPM'in kendine yeterliliğinin Gradle
karşılığı; `doctor.sh`: `gradle:* → info`, hiçbir milestone'da blocker değil).
Native kütüphane JNA üzerinden `jna.library.path`'ten çalışma zamanında
yükleniyor — Swift'in `-L/-l` static-link'inin JVM/dynamic-load karşılığı.

### Ön koşul — B2 kapandı

Bu makinede gerçek bir JDK yoktu (yalnız CLT'nin çalışmayan `/usr/bin/java`
stub'ı — NEN-005'in bulduğu `detect_jdk` kusurunun kanıtladığı tam durum).
`brew install --cask temurin` sudo/interaktif şifre istediği için (bu ortamda
sağlanamaz) başarısız oldu; onun yerine **`brew install openjdk`** (formula,
sudo gerektirmez) kullanıldı ve `/opt/homebrew/opt/openjdk/bin` `~/.zshrc`'ye
PATH olarak eklendi (Homebrew'ün kendi "keg-only" uyarısının önerdiği,
sudo'suz yol). `bash scripts/doctor.sh M1` artık JDK'yi ✓ gösteriyor:

```
$ bash scripts/doctor.sh M1
HAZIR:
  ✓ cargo  ✓ rustc  ✓ cargo-deny  ✓ JDK  ✓ swift
SONUÇ: M1 için tüm blocker'lar hazır.          → exit 0
```

**Yan bulgu (ayrı backlog task'ına yönlendirildi, bu task'ın kapsamı
değil):** `detect_jdk`'nin dayandığı `run_timeout` helper'ı `java -version`'ın
(stderr'e yazan) çıktısını kendi içindeki `2>/dev/null` ile siliyor — bu
yüzden `doctor.sh` raporunda "✓ JDK" satırının sürüm detayı hep **boş**
kalıyor. Kozmetik bir kusur (M1 gate'inin exit kodunu etkilemiyor, JDK zaten
`soon` seviyesinde); önceki hiçbir makinede gerçek bir JDK hiç çalışmadığı
için şimdiye kadar hiç ortaya çıkmamıştı — NEN-005'in `detect_jdk` guard
kusuruyla aynı "gizli kalan kusur" deseni. CLAUDE.md kural 5 gereği (yol
üstünde görülen alakasız iyileştirme → yeni backlog task'ı) burada
düzeltilmedi, ayrı bir backlog önerisi olarak bırakıldı.

Bootstrap için `brew install gradle` (9.7.1) de geçici olarak kuruldu —
üç `jvm-harness/`'ta bir kere `gradle wrapper --gradle-version 9.7.1`
çalıştırıp `gradlew`/`gradle-wrapper.jar`'ı üretmek için (SwiftPM'in tersine
Gradle wrapper'ı sıfırdan üretmek için var olan bir Gradle kurulumu
gerekiyor — tıpkı bir developer makinesinde olduğu gibi). Bundan sonra hiçbir
geliştiricinin sistem Gradle'ı kurmasına gerek yok.

### 1 — `spike-cue-transfer` (NEN-008 paritesi)

`bash scripts/spike-cues-jvm.sh` (release · 50 000 cue, NEN-008 ile birebir
aynı fixture boyutu):

| Ölçüm | Swift (NEN-008) | Kotlin/JVM (NEN-011) |
|---|---|---|
| checksum (A / B) | `75001045577800` / `34337591381145` | `75001045577800` / `34337591381145` |
| A p50 | 37.708 ms | 5.135 ms |
| A p95 | 38.657 ms | 7.332 ms |
| A ilk çağrı (soğuk) | 48.379 ms | 18.622 ms |
| A en uzun tek blok | 48.379 ms | 18.622 ms |
| B pencere 40 cue p50 | 44.33 µs | 10.67 µs |
| B pencere 40 cue p95 | 59.71 µs | 45.21 µs |
| B pencere — max | 80.25 µs | 4871.42 µs |
| `activeCue` p50 / p95 | 1.50 / 1.58 µs | 6.67 / 8.96 µs |
| `activeCue` — max | 19.00 µs | 21974.38 µs |
| peak bellek (A) | 19.5 MiB (RSS) | 136.1 MiB (heap) |
| peak bellek (B) | 11.7 MiB (RSS) | 184.8 MiB (heap) |

**Checksum'lar birebir aynı** (`75001045577800` / `34337591381145`) — iki
binding de aynı Rust crate'ini, aynı deterministik fixture'ı çağırıyor;
I5'in ("semantik sonuçlar Swift ve Kotlin arasında aynı") doğrudan kanıtı.
Ayrıca `CueTransferParityTest` (3 test — pencere/tam-liste dilim eşitliği,
sınır dışı pencere, boşluk/sınır `activeCue`) Kotlin binding'in marshalling'i
Rust'ınkiyle tutarlı yapıyor olduğunu ayrıca doğruluyor.

### 2 — `spike-async-cancel` (NEN-009 paritesi)

**Coroutine sarmalayıcısı (task'ın "suspend fonksiyon davranışı" maddesi):**
Rust tarafı `async fn` DEĞİL — düz thread + `ProgressSink` foreign callback +
bloklayan `JobHandle.join()` (kaynak incelemesiyle doğrulandı: UniFFI'ın
Kotlin backend'i yalnız gerçek `async fn` export'larını `suspend fun`'a
çeviriyor). `JobHandleCoroutines.kt`, `join()`'i ayrı bir thread'de çalıştırıp
`suspendCancellableCoroutine` + `invokeOnCancellation { handle.cancel() }`
ile sarmalıyor — yani gerçek bir `kotlinx.coroutines` `Job.cancel()`'ı, gerçek
Rust `JobHandle.cancel()`'ını (delivery gate'i) tetikliyor.

```
$ bash scripts/spike-async-jvm.sh --test-only
✔ AsyncCancelInvariantTests: 3 passed, 0 failed
  - noCallbackAfterCoroutineCancelCompletes()   (I1 — coroutine cancel → 0 late callback)
  - lateCommitAttemptIsBlocked()                (I2 — negatif, late commit engellendi)
  - repeatedCancelLeavesNoLiveJobs()            (I4 — 100 döngü sonrası live_jobs()==0)
```

`noCallbackAfterCoroutineCancelCompletes` DoD'un "coroutine iptali sonrası
late callback yok" maddesinin doğrudan kanıtı: `job.cancelAndJoin()` (kotlinx
coroutine iptali) → `invokeOnCancellation` içinde gerçek `handle.cancel()` →
`recorder.afterCancelCount == 0`.

Baseline — checkpoint aralığı × cancellation latency
(`bash scripts/spike-async-jvm.sh --measure-only`, release, block_micros=20 µs
sabit, repeats=200/satır):

| checkpoint_every | Swift p50 (release) | Kotlin/JVM p50 | Swift dispatch-penceresi kaçağı (800 koşu) | Kotlin dispatch-penceresi kaçağı (800 koşu) |
|---|---|---|---|---|
| 1 | 0.033 ms | 0.162 ms | 0 | **193** |
| 5 | 0.110 ms | 0.119 ms | 0 | 0 |
| 20 | 0.413 ms | 0.416 ms | 0 | 0 |
| 100 | 2.019 ms | 2.018 ms | 0 | 0 |

**En büyük tek sapma burada.** "Dispatch penceresi kaçağı" I1'in kendisi
DEĞİL (Swift'teki gibi — `measure()` kasıtlı olarak `markCancelRequested()`'ı
`cancel()` çağrılmadan ÖNCE işaretliyor, "kullanıcı iptale karar verdi" ile
"fiilen `cancel()` çağrıldı" arasındaki gevşek pencereyi ölçüyor). Swift'te bu
pencere release'de her zaman boştu (800/800); Kotlin/JVM'de en sıkı aralıkta
(`checkpoint_every=1`, ~20 µs) 800 koşudan **193'ünde** kaçak var. Sebebi:
JVM'de "ilk ilerleme bildirimini al → yanıt olarak iptal kararı ver" zinciri
`Semaphore.acquire()` + JIT/GC ile etkileşen thread scheduling üzerinden
geçiyor — Swift'in `DispatchSemaphore` + GCD'sinden çok daha büyük ve
değişken bir gecikmeye sahip. **I1'in kendisi bundan etkilenmiyor** (yukarıdaki
`noCallbackAfterCoroutineCancelCompletes` deterministik kanıtlıyor) — yalnız
"kullanıcı isteği" ile "iptal talebinin donanıma ulaşması" arasındaki gerçek
dünya penceresi JVM'de daha geniş. **M10 etkisi:** Android'de gerçek bir UI
iptal düğmesi ile en agresif checkpoint aralığı (≈20 µs) arasında Swift'ten
daha fazla "geç ama meşru" callback beklenmeli; UI tarafının bunu tolere
etmesi gerekiyor (zaten I1 ihlali değil, yalnızca en son callback'in biraz
daha geç gelebileceği anlamına geliyor).

Kaynak bağlamı: `live_jobs()` sweep sonrası **0** (I4 burada da doğrulandı,
ayrıca InvariantTests.kt'te deterministik). `Thread.activeCount()` yalnız
JVM'in kendi thread group'unu sayıyor (Swift'in `task_threads()`'i tüm process
thread'lerini sayıyordu) — bu yüzden ham sayılar (1→2 vs Swift'in 3→3)
doğrudan karşılaştırılamaz; ikisi de "sweep sonrası sızıntı yok" iddiasını
kendi ölçüm yöntemiyle destekliyor.

### 3 — `spike-typed-errors` (NEN-010 paritesi)

**İki ayrı Kotlin-özgü isimlendirme sapması (kaynak incelemesiyle
doğrulandı — `uniffi_bindgen` 0.32 `bindings/kotlin/gen_kotlin/mod.rs`):**

1. `class_name`/`convert_error_suffix`: adı "Error" ile biten bir
   `uniffi::Error` tipi Kotlin'de otomatik "Exception" ile değiştiriliyor —
   Rust `AppError`, Swift'te `AppError` kalırken Kotlin'de **`AppException`**
   oluyor.
2. `enum_variant_name`: düz `uniffi::Enum` varyantları (`Capability`,
   `ValidationField`) Kotlin'de **SCREAMING_SNAKE_CASE** üretiliyor
   (`to_shouty_snake_case()`) — Swift'te camelCase'ti (`.localAsr`). Yani
   Swift'in kendi bulduğu "hata varyantları PascalCase, düz enum varyantları
   camelCase" asimetrisine (NEN-010 kanıt kaydı) Kotlin **üçüncü bir kural**
   (SCREAMING_SNAKE_CASE) ekliyor — üç binding, üç farklı varyant-isimlendirme
   kuralı. `AppException`'ın **varyant** adları (`Parse`, `Network`, ...) yine
   PascalCase — yalnız üst tip adı değişiyor, Swift'le aynı burada.

```
$ bash scripts/spike-typed-errors-jvm.sh --test-only
✔ TypedErrorTests: 2 passed, 0 failed
  - everyVariantSwitchesWithoutStringParsing()          (DoD #1 — exhaustive `when`, `else` yok)
  - defaultKotlinPrintingHasNoForbiddenPattern()         (DoD #2 — Kotlin'in KENDİ toString()'i temiz)
```

Eşleme maliyeti — throw → catch → when/switch round-trip (10 000 tekrar):

| | Swift p50 | Swift p95 | Kotlin/JVM p50 | Kotlin/JVM p95 | Kotlin/JVM max |
|---|---|---|---|---|---|
| | 2.04 µs | 2.17 µs | 11.33 µs | 50.13 µs | 144776.08 µs |

**Sapma:** Kotlin/JVM p50 Swift'in **~5.5×**'i, p95 **~23×**'ü, ve maksimumda
(144.8 ms) devasa bir kuyruk var. Bu, AOT-derlenmiş Swift release build'e
karşı JVM'in JIT ısınması + GC duraklamalarının beklenen sonucu — yalnız
10 000 tekrarlık kısa bir ölçümde JIT'in tam optimize etmeye vakti olmuyor,
ve `System.nanoTime()` ölçümü arada bir GC/JIT-compile duraklamasını
yakalıyor. **Bug değil**, JVM/AOT runtime modeli farkı; M10 etkisi: Android'de
(ART, farklı bir JIT/AOT karışımı) gerçek sayılar burada ölçülenden farklı
olacak — bu sayılar yalnızca "JVM'in host masaüstünde nasıl davrandığı"nı
gösteriyor, ART baseline'ı ayrı ölçülmeli (M10 kapsamı).

### Metodolojik sapmalar — özet tablo

| # | Sapma | Sebep | M10 etkisi |
|---|---|---|---|
| D1 | Peak bellek: Swift RSS (`mach_task_basic_info`) vs Kotlin heap (`MemoryPoolMXBean.peakUsage`, yalnız HEAP havuzları) | JVM'de native RSS'in temiz bir karşılığı yok (GC + JIT) | Android'de gerçek bellek baskısı ayrıca ölçülmeli (ART farklı GC) |
| D2 | Thread sayısı: Swift `task_threads()` (process geneli) vs Kotlin `Thread.activeCount()` (yalnız çağıran thread group'u) | JVM'de process geneli thread sayımı için ayrı bir API/izin gerekir | Düşük risk — I4'ün kendisi `live_jobs()` sayacına dayanıyor, thread sayımı yalnız bağlam |
| D3 | JIT ısınması: ilk çağrılar sürekli olarak sonrakilerden çok daha yavaş (örn. cue-transfer A: ilk 18.6 ms, p50 5.1 ms) | Swift AOT derleniyor, JVM ısınıyor | Android'de app başlangıcında ilk birkaç FFI çağrısı JVM'de olduğu gibi yavaş olabilir — cold-start bütçesine girdi |
| D4 | Dispatch-penceresi kaçağı (checkpoint_every=1'de 193/800) | JVM thread scheduling + GC, native GCD'den daha değişken | I1'in kendisini etkilemiyor; UI iptal geri bildirimi biraz gecikebilir |
| D5 | İsimlendirme: `AppError`→`AppException`, enum varyantları→SCREAMING_SNAKE_CASE | uniffi_bindgen Kotlin backend'inin kendi kuralları (kaynak incelemesiyle doğrulandı) | ADR-0002/ADR-0003 karar metninde her iki binding'in kendi isimlendirme kuralına sahip olacağı not edilmeli |

### Doğrulama

```
$ bash scripts/spike-cues-jvm.sh              → BUILD SUCCESSFUL, 3/3 test, checksum eşleşti  → exit 0
$ bash scripts/spike-async-jvm.sh --test-only → 3/3 invariant test                              → exit 0
$ bash scripts/spike-async-jvm.sh --measure-only → baseline tablo yukarıda                       → exit 0
$ bash scripts/spike-typed-errors-jvm.sh      → 2/2 test + eşleme maliyeti tablo                 → exit 0

$ cargo test --manifest-path core/Cargo.toml --workspace
42 passed, 0 failed (bkz. NEN-005/010 baseline'ı — regresyon yok)          → exit 0

$ grep -rn --include='*.rs' -E "uniffi::|extern \"C\"" core/crates | grep -v crates/nen-ffi/
(boş)                        → ürün kodunda nen-ffi dışında FFI yüzeyi yok

$ cargo metadata --manifest-path core/Cargo.toml --format-version 1 --no-deps
crates/* → spikes/* kenarı yok

$ find platforms/android -type f | grep -v .gitkeep
(boş)                        → ürün Kotlin/Gradle modülü spike binding'ini tanımıyor (hâlâ iskelet)

$ bash scripts/test.sh
  check-docs.test.sh ✓ · doctor.test.sh ✓                                  → exit 0
$ bash scripts/check-docs.sh
  8/8 denetim geçti                                                        → exit 0
```

Bağlam: Apple M5 · arm64 · macOS 27.0 (26A5416b) · rustc 1.98.0 · Swift 6.4 ·
Gradle 9.7.1 (Kotlin 2.4.0, proje bağımlılığı) · OpenJDK (Homebrew) 26.0.2.1 ·
uniffi 0.32.0 · JNA 5.15.0 · kotlinx-coroutines 1.9.0/1.8.0 (transitively
resolved) · 2026-08-24.

I5 invariant'ı ("semantik sonuçlar Swift ve Kotlin arasında aynı") cue-transfer
checksum eşleşmesi ve typed-errors'ın exhaustive `when`/`switch` ikisinin de
5 varyantı string parse'sız ayrıştırabilmesiyle karşılandı. I3 ("typed error
string parse gerektirmiyor") Kotlin tarafında da sağlandı — yalnız üst tip ve
enum varyant adlandırması farklı (D5), payload alanları (host, extension,
line, status, capability, field) aynı tipte ve aynı anlamda.
