---
id: NEN-029
title: Playback/renderer reverse-FFI boundary spike
milestone: M1
size: M
state: done
depends_on: [NEN-007, NEN-009]
blocks: [NEN-012, NEN-021]
adr: [26]
---

# NEN-029 — Playback/renderer reverse-FFI boundary spike

## Sonuç

Playback session'ın sahibinin **shared core mu platform shell mi** olacağı,
fake adapter'lar üzerinde ölçülmüş A/B karşılaştırmasıyla bilinir ve ADR-0026
ile karara bağlanabilir.

## Neden

M0'da `PlaybackEngine` ve `SubtitleRenderer` portları tasarlandı ve merkezi bir
Rust application-session'ın bu portları **reverse callback** ile yöneteceği
varsayıldı. Bu varsayım hiç doğrulanmadı — oysa M3'ün tamamı ve M7'nin position
çözünürlüğü buna bağlı. Yanlışsa, öğrenmek için en ucuz an şimdi.

## Karşılaştırılacak iki yaklaşım

- **A — Core-owned session.** Rust core, `PlaybackEngine`/`SubtitleRenderer`
  callback interface'lerini çağırır ve playback session'ın sahibidir.
  *(architecture.md'deki mevcut tercih)*
- **B — Shell-owned session.** Platform shell playback motorunun sahibidir;
  core'a coarse-grained playback state, position ve track snapshot gönderir.

Fake Swift adapter zorunlu; fake Kotlin adapter **mümkünse**. NEN-011 önce
biterse Kotlin yarısı ucuzlar, ama bu task ona bağlı değildir.

## Ölçülüp belgelenecekler

| Konu | Not |
|---|---|
| reverse callback uygulanabilirliği | A yönü teknik olarak mümkün mü |
| Swift MainActor / UI thread dönüşü | callback hangi thread'e düşüyor, maliyeti ne |
| Kotlin main thread dönüşü | aynı soru JVM tarafında |
| object ownership ve lifetime | adapter nesnesini kim tutuyor, ne kadar |
| callback sonrası object release | release doğru anda mı oluyor |
| reentrancy | callback içinden core'a çağrı güvenli mi |
| event ordering | event'ler gönderim sırasında mı geliyor |
| position update sıklığı | **iki uçta ölçülecek:** ~4 Hz (UI-yeterli) ve ~60 Hz (frame-senkron) |
| seek command / seek-complete sırası | komut ve tamamlanma sinyali karışıyor mu |
| cancellation | akış ortasında iptal — I1 ve I4 geçerli |
| shutdown | motor kapanırken sızıntı/çökme var mı |
| platform lifecycle | arka plana alma / öne getirme davranışı |
| typed error aktarımı | adapter hatası core'a typed olarak dönüyor mu |
| gereksiz yüksek frekanslı FFI trafiği | çağrı hacmi ve maliyeti |

Tüm ölçümler **baseline**'dır (pass/fail eşiği değil); bağlamıyla kaydedilir:
cihaz · OS/toolchain · debug/release build.

**Invariant'lar (pass/fail):** I1 (iptal sonrası late callback yok) ·
I4 (thread/bellek sızıntısı yok).

## YAPILMAYACAK

- Gerçek libmpv, AVPlayer veya Media3 implementasyonu — **yalnız fake adapter**
- `PlaybackEngine` / `SubtitleRenderer` portlarını kaldırmak veya yeniden
  tasarlamak — portlar her iki sonuçta da kalır
- Contract test kiti yazmak → NEN-021 (ADR-0026'dan sonra)
- Spike kodunun ürüne terfisi — `core/spikes/` altında kalır

## Kanıt (DoD)

- [x] A ve B için ölçüm tablosu, her satır bağlamıyla birlikte
- [x] Position update: ~4 Hz ve ~60 Hz uçlarında FFI çağrı hacmi ve maliyeti
- [x] Event ordering ve seek/seek-complete sırası test edildi
- [x] **I1** — iptal sonrası late callback yok
- [x] **I4** — tekrarlı start/stop/shutdown sonrası sızıntı yok
- [x] **M7 etkisi raporlandı:** B seçilirse manuel sync için gereken position
      çözünürlüğü ne oluyor
- [x] ADR-0026 taslağı: önerilen yön + reddedilen yön + geri dönüş maliyeti —
      `docs/adr/0026-playback-renderer-ownership.md`, kullanıcı A yönünü
      onayladı, `status: accepted` (2026-08-24). `docs/architecture.md`'nin
      "spike bekliyor" notu da bu onayla güncellendi.

## Kanıt kaydı

`core/spikes/spike-reverse-ffi/` — NEN-009'un delivery-gate deseni
(`Mutex<Option<Arc<dyn Trait>>>`, callback kilit altında çağrılıyor,
`cancel()` aynı kilidi alıyor) tek bir crate içinde iki yöne genelleştirildi:

- **A — `ReverseEngine`.** Kendi tick thread'i olan sahte bir motor; `Arc<dyn
  PlaybackObserver>` foreign trait'ini (`on_position`/`on_state_changed`/
  `on_track_snapshot`/`on_seek_completed`/`on_error`) tekrar tekrar çağırıyor.
  `cancel()` yalnız delivery gate'i kapatıyor (I1); `shutdown()` ayrıca tick
  thread'ini ve her seek'in kendi kısa ömürlü thread'ini join ediyor (I4).
- **B — `ForwardSession`.** Thread'i yok, observer kaydı yok — Swift'in kendi
  `DispatchSourceTimer`'ı düz `report_*` çağrıları yapıyor, maliyet Swift
  tarafında ölçülüyor. Ters çağrı hiç olmadığı için ölçülecek bir thread-hop
  de yok — bu asimetri karşılaştırmanın baş bulgusu (aşağıda).

Zamanlama disiplini: her çağrı **başlatan tarafta** ölçülüyor (A için Rust
`Instant::now()`, B için Swift'in kendi saati) — iki taraf arasında saat
korelasyonu hiç gerekmiyor.

**Bulunan ve düzeltilen bir kusur:** ilk yazılan Rust testleri paralel
`cargo test` thread'leri arasında `LIVE_ENGINES`/`LIVE_FORWARD_SESSIONS`
global sayaçlarını paylaşıyordu; NEN-009'un aksine bu spike'ın tick'leri
gerçek wall-clock `sleep` kullanıyor (10-80 ms pencereler), bu da paralel
testlerin çakışmasını NEN-009'un mikrosaniye ölçekli CPU-bound busy-loop'larına
göre çok daha olası kılıyor — deterministik biçimde 2/11 test kırıldı (ilk
koşuda `left: 4, right: 0`). Düzeltme: `tests` modülüne `static TEST_SERIAL:
Mutex<()>` eklendi, her test bunu ilk satırda kilitliyor. Üç ardışık koşuda
11/11 stabil geçti.

### Rust unit testleri

```
$ cargo test --manifest-path core/Cargo.toml -p spike-reverse-ffi
running 11 tests
test tests::forward_calls_after_shutdown_return_typed_error_not_panic ... ok
test tests::engine_error_variants_are_distinguishable_without_string_parsing ... ok
test tests::engine_runs_and_shutdown_stops_it_cleanly ... ok
test tests::cancel_closes_the_gate_before_shutdown ... ok
test tests::repeated_new_shutdown_cycles_leave_zero_live_sessions ... ok
test tests::forward_session_records_arrival_order ... ok
test tests::repeated_start_shutdown_cycles_leave_zero_live_engines ... ok
test tests::seek_out_of_range_delivers_typed_error ... ok
test tests::seek_within_range_completes_with_target ... ok
test tests::seq_is_strictly_increasing_across_callback_kinds ... ok
test tests::trigger_error_delivers_engine_failure ... ok
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo clippy -p spike-reverse-ffi --all-targets -- -D warnings` → exit 0.
`cargo fmt -p spike-reverse-ffi --check` → exit 0.

### Swift invariant/ordering/typed-error testleri (release ve debug)

```
$ bash scripts/spike-reverse-ffi.sh --test-only          # release
✔ Test run with 12 tests in 3 suites passed after 0.839 seconds.
$ bash scripts/spike-reverse-ffi.sh --debug --test-only
✔ Test run with 12 tests in 3 suites passed after 0.820 seconds.
```

Üç dosya, üç suite: `InvariantTests.swift` (I1, I4×2, observer-lifetime),
`OrderingTests.swift` (event ordering, seek/seek-complete×2, reentrancy),
`TypedErrorTests.swift` (A ve B'nin typed-error yolları). Debug ve release'de
ayrı ayrı üç kere tekrarlanan koşularda hepsi kararlı geçti (0 flake).

**Reentrancy bulgusu (I1-I5 dışı, baseline gözlem):** callback içinden
`seek()` çağırmak güvenli — `seek()` yalnız kendi `seek_threads` kilidini
alıyor, delivery gate'in `sink` kilidine hiç dokunmuyor
(`reentrantSeekFromCallbackDoesNotDeadlock` testi, bounded wait ile
kanıtlandı). Callback içinden `cancel()` çağırmak ise **kod incelemesiyle
kanıtlanmış** bir self-deadlock: `deliver()` callback'i `sink` kilidi
altındayken çağırıyor, `cancel()` aynı kilidi istiyor. Bu senaryo kasıtlı
olarak **çalıştırılmadı** — gerçek bir deadlock, bu binary'nin geri kalan
testleri için `live_engines()` sayacını kalıcı olarak bozardı (bkz.
`OrderingTests.swift`'teki doc-comment). NEN-021'in gerçek kontratı bu
kısıtı (aynı thread'den gate-çakışan reentrant çağrı yasak / async dispatch
zorunlu) açıkça yazmalı.

### Baseline — 4 Hz / 60 Hz × per-call maliyet (release)

Bağlam: Apple M5 · macOS 27.0 (26A5416b) · arm64 · rustc 1.98.0 · Swift 6.4 ·
uniffi 0.32.0 · release (rust profile release, swift -c release) · tek koşu,
2026-08-24.

**A — core-owned, reverse callback:**

| hz | pozisyon sample | per-call maliyet p50/p95/max | MainActor hop p50/p95/max | callback thread'i |
|---|---|---|---|---|
| 4 | 117 | 75.04 µs/96.79 µs/316.00 µs | 58.00 µs/82.00 µs/275.83 µs | hep ana thread DIŞINDA (117/117) |
| 60 | 222 | 36.67 µs/52.12 µs/91.92 µs | 40.54 µs/49.58 µs/61.54 µs | hep ana thread DIŞINDA (222/222) |

**B — shell-owned, forward call:**

| hz | pozisyon sample | per-call maliyet (Swift ölçümü) p50/p95/max |
|---|---|---|
| 4 | 120 | 3.67 µs/4.79 µs/11.42 µs |
| 60 | 300 | 1.33 µs/2.71 µs/16.58 µs |

60 Hz'de tahmini toplam FFI-ilişkili maliyet: **A ~4.63 ms/sn** (call+hop×60),
**B ~0.08 ms/sn** (call×60, hop yok) — 1000 ms/sn'lik kare bütçesinin binde
biri mertebesinde, ikisi de.

### Baseline — 4 Hz / 60 Hz × per-call maliyet (debug)

Aynı bağlam, debug build (rust profile dev, swift -c debug):

**A:**

| hz | pozisyon sample | per-call maliyet p50/p95/max | MainActor hop p50/p95/max | callback thread'i |
|---|---|---|---|---|
| 4 | 118 | 104.67 µs/126.04 µs/355.04 µs | 59.75 µs/81.58 µs/229.21 µs | hep ana thread DIŞINDA (118/118) |
| 60 | 222 | 57.04 µs/90.54 µs/171.29 µs | 43.67 µs/57.79 µs/120.54 µs | hep ana thread DIŞINDA (222/222) |

**B:**

| hz | pozisyon sample | per-call maliyet (Swift ölçümü) p50/p95/max |
|---|---|---|
| 4 | 120 | 23.58 µs/31.33 µs/43.25 µs |
| 60 | 300 | 9.29 µs/12.92 µs/35.29 µs |

60 Hz'de tahmini toplam: **A ~6.04 ms/sn**, **B ~0.56 ms/sn** — debug build A'yı
~1.3×, B'yi ~7× yavaşlatıyor (NEN-008'in "debug 10-100× yanıltabilir" uyarısı
burada B tarafında daha belirgin çünkü B'nin mutlak maliyeti zaten çok küçük).

**Baş bulgu:** A her event için bir MainActor-hop vergisi ödüyor (release'de
~40-58 µs p50) çünkü callback her zaman Rust'ın kendi tick thread'ine düşüyor,
asla ana thread'e değil — UI durumuna dokunmak isteyen bir gerçek adapter
implementasyonu bu hop'u **açıkça** yapmak zorunda. B'de bu maliyet hiç yok
çünkü Swift zaten çağıran taraf. Ama mutlak rakamlar (60 Hz'de A ~4.6 ms/sn,
B ~0.08 ms/sn, ikisi de 1000 ms/sn'lik bütçenin küçük bir kesri) bu farkı
performans açısından **kararı belirleyici** yapmıyor — asıl ayrım aşağıdaki
backgrounding bulgusunda.

### Backgrounding proxy (yalnız A, gerçek OS lifecycle DEĞİL — bkz. `lib.rs`/`main.swift` modül dokümanı)

Swift tarafındaki callback-drain kuyruğu 2 saniye askıya alınırken Rust'ın
tick thread'i (30 Hz) çağırmaya devam etti — `deliver()` yalnız kuyruğa
ekliyor, kuyruğun askıda olması Rust'ı hiç bloklamadı. Kuyruk devam ettirilince
birikme temizlendi, kaçak kalmadı:

```
suspended 2.0s at 30 Hz (~60 ticks expected) → max backlog depth 51,
drained 150 total, pending after 0.3s settle: 0
```

**Bu, A'nın B'ye göre gerçek yapısal riski:** A'nın üreticisi (Rust tick
thread'i) tüketicinin (Swift callback drain) hazır olup olmadığından habersiz,
bağımsız yaşıyor — arka plana alma sırasında gerçek bir uygulamanın UI
thread'i uzun süre meşgul kalırsa, bu birikme sınırsız büyüyebilir (burada
sınırlı kaldı çünkü tick thread'in kendisi `run_duration_ms` sonunda doğal
olarak duruyor; gerçek bir sürekli oynatımda böyle bir doğal sınır olmaz).
B'de bu risk yapısal olarak yok: B'nin "üreticisi" zaten platformun kendi
timer'ı/callback'i, arka plana alındığında kendiliğinden durur.

### MainActor-izoleli observer uygunluğu (araç zinciri bulgusu)

`@MainActor final class MainActorObserver: PlaybackObserver` derleniyor, ama
yalnız her protokol metodu `nonisolated` işaretlenip MainActor durumuna
dokunmak için `MainActor.assumeIsolated { ... }` kullanılınca. Swift 6 strict
concurrency senkron bir foreign-trait metodunu doğrudan actor-isolated
bırakmıyor — M1'in önceki spike'larında ölçülmemiş bir araç zinciri gerçeği,
NEN-021'in gerçek Swift adapter'ı bu deseni kullanmak zorunda kalacak.

### ADR-0028 mekanik denetimi

```
$ grep -rn --include='*.rs' -E "uniffi::|extern \"C\"" core/crates | grep -v crates/nen-ffi/
(çıktı boş — temiz)
$ cargo metadata --format-version 1 --no-deps   # crates/* → spikes/* kenarı yok
bad edges: (yok — temiz)
$ grep -rn "spike" platforms/apple-shared/Package.swift
(çıktı boş — temiz)
```

### Workspace doğrulaması

```
$ cargo fmt --manifest-path core/Cargo.toml -p spike-reverse-ffi --check   → exit 0
$ cargo clippy --manifest-path core/Cargo.toml --workspace --all-targets -- -D warnings   → exit 0
$ cargo test --manifest-path core/Cargo.toml --workspace   → exit 0 (spike-reverse-ffi: 11/11)
$ bash scripts/doctor.sh M1   → SONUÇ: M1 için tüm blocker'lar hazır (exit 0)
$ bash scripts/check-docs.sh   → 8/8 denetim geçti (exit 0, docs/STATUS.md güncellendi)
$ bash scripts/test.sh   → exit 0
```

### M7 etkisi

B seçilirse, M7'nin manuel sync'i için gereken position çözünürlüğü **FFI
maliyetiyle sınırlı değil**: B'nin forward-call maliyeti release'de 60 Hz'de
p50 **1.33 µs** (debug'da 9.29 µs) — 60 Hz'in toplam FFI payı zaten
**0.08 ms/sn** (release), 1000 ms/sn'lik bütçenin binde birinden az. Yani B
altında gerçek üst sınır platform shell'in kendi playback motorunun ürettiği
position-update frekansı olur (AVPlayer periodic time observer, mpv
property-observe, Media3 `Player.Listener` — bunların hepsi zaten tipik olarak
4-60 Hz aralığında çalışır), FFI boyutu değil. B seçilseydi M7'nin çözünürlüğü
motorun native callback granularity'sine kadar FFI maliyeti düşünmeden
yükseltilebilirdi.

### Kotlin

Ertelendi. Task metni NEN-029'un NEN-011'e bağımlı olmadığını söylüyor; bu
makinede JDK kurulu değil (B2, `docs/STATUS.md`). Bu crate altında hiçbir
Kotlin scaffolding'i oluşturulmadı.

### Sonuç ve ADR-0026

Ölçümler A yönünün (merkezi Rust session, reverse callback) teknik olarak
uygulanabilir ve maliyetinin (60 Hz'de bile ~5 ms/sn mertebesinde) hiçbir
makul kare bütçesini zorlamadığını gösteriyor; I1 ve I4 her iki yönde de
sağlanıyor. B daha ucuz ve yapısal olarak daha basit (backpressure riski yok),
ama A'nın mutlak maliyeti zaten ihmal edilebilir düzeyde olduğundan bu fark
tek başına kararı değiştirmiyor. **ADR-0026 A'yı önerdi, kullanıcı onayladı**
— `docs/adr/0026-playback-renderer-ownership.md`, `status: accepted`
(2026-08-24). `docs/architecture.md`'nin "spike bekliyor" notu bu kararla
güncellendi; `NEN-021` artık başlayabilir.
