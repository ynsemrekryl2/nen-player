---
id: NEN-009
title: Spike - async progress and cancellation
milestone: M1
size: M
state: done
depends_on: [NEN-007]
blocks: [NEN-011]
adr: [28]
---

# NEN-009 — Spike: async progress and cancellation

## Sonuç

FFI sınırından geçen uzun bir iş iptal edilebilir, iptalden sonra **hiçbir
callback gelmez** ve kaynak sızmaz.

## Kapsam

- Sahte uzun iş (bloklara bölünmüş, checkpoint'li)
- `onProgress(jobId, phase, done, total)` callback akışı — **cue içeriği yok**
- `JobHandle.cancel()` ile kooperatif iptal
**Baseline olarak raporlanacaklar** (pass/fail eşiği **değil**):

- Cancellation latency (p50 / p95), ölçüm bağlamıyla birlikte
- Checkpoint aralığı ile latency arasındaki ilişki
- Fixture boyutu · cihaz · OS/toolchain · debug/release build

**Invariant'lar (pass/fail — gevşetilemez, K15'in doğrudan karşılığı):**

| # | Invariant |
|---|---|
| I1 | İptalden sonra **hiçbir callback gelmez** (sayı = 0) |
| I2 | İptalden sonra **late commit yok** |
| I4 | Tekrarlı iptal ve shutdown sonrası **thread/bellek sızıntısı yok** |

## YAPILMAYACAK

- Gerçek çeviri pipeline'ı → M5
- Progress'in UI'da gösterimi → M3+

## Kanıt (DoD)

- [x] **I1** — Swift testi: işi ilk ilerleme bildirimi geldiğinde (iş henüz
      sürerken) iptal ediyor, `cancel()` döndükten sonra late callback yok
- [x] **I2** — Negatif: iptal sonrası commit denemesi engelleniyor
- [x] **I4** — Tekrarlı iptal (100 kez) sonrası thread/bellek sızıntısı yok
- [x] Baseline: cancellation latency p50/p95 + ölçüm bağlamı kayıtlı

## Kanıt kaydı

**Tasarım — "delivery gate":** callback sink `Mutex<Option<Arc<dyn
ProgressSink>>>` içinde tutuluyor; worker her checkpoint'te callback'i **kilit
altında** çağırıyor, `cancel()` da aynı kilidi alıp sink'i `None`'a çeviriyor.
Sonuç: `cancel()` bir callback'in ortasında dönemez (kilidi bekler) ve
döndükten sonra hiçbir checkpoint artık sink bulamaz. I1 buradan geliyor.
Ayrıntı: `core/spikes/spike-async-cancel/src/lib.rs` modül dokümanı.

**Önemli düzeltme (kanıt sürecinde bulundu):** İlk yazılan Swift invariant
testi `cancelRequested` bayrağını `cancel()` çağrılmadan ÖNCE işaretliyordu.
`SpikeAsyncCancel` ölçüm harness'ı ile 800 koşuluk bir sweep çalıştırılınca
`--debug` build'inde 1 kaçak callback bulundu (checkpoint_every=1, en sıkı
aralıkta) — bu I1'in ihlali değil, "kullanıcı iptale karar verdi" ile
"Swift'in `cancel()`'ı fiilen çağırması" arasındaki dispatch penceresinde
meşru bir kaçak. Test I1'in gerçek sınırını (cancel() **döndükten** sonra)
ölçmüyordu; bu yüzden kırılgandı. Düzeltme: `InvariantTests.swift`'te
`markCancelRequested()` artık `cancel()` döndükten SONRA çağrılıyor — bu
sınır kilit tasarımı gereği yapı olarak imkânsız bir kaçağı test ediyor,
deterministik. `main.swift` bu iki ölçümü artık ayrı etiketliyor (dispatch
penceresi ≠ I1) — bkz. her iki dosyanın doküman yorumları.

### Rust unit testleri

```
$ cargo test --manifest-path core/Cargo.toml -p spike-async-cancel
running 6 tests
test tests::cancel_before_start_yields_zero_delivered_and_no_commit ... ok
test tests::late_commit_attempt_is_blocked_by_the_gate ... ok
test tests::is_running_reflects_completion ... ok
test tests::join_is_idempotent ... ok
test tests::uncancelled_job_completes_and_commits_exactly_once ... ok
test tests::repeated_cancel_cycles_leave_no_live_jobs ... ok
test result: ok. 6 passed; 0 failed
```

### Swift invariant testleri — release ve debug, ikisi de yeşil

```
$ bash scripts/spike-async.sh --test-only          # release
$ bash scripts/spike-async.sh --debug --test-only  # debug
✔ Test "cancel() returns → zero further callbacks, ever" passed
✔ Test "cancelled job's late commit attempt is blocked" passed
✔ Test "100 start+cancel+join cycles leave zero live jobs" passed
✔ Test run with 3 tests in 1 suite passed
```

### Baseline — checkpoint aralığı × cancellation latency

`bash scripts/spike-async.sh` (release · Apple M5 · macOS 27.0 26A5416b ·
rustc 1.98.0 · Swift 6.4 · uniffi 0.32.0 · block_micros=20 µs sabit ·
repeats=200/satır):

| checkpoint_every | ~aralık | p50 | p95 | max | dispatch penceresi kaçağı |
|---|---|---|---|---|---|
| 1 | 0.020 ms | 0.033 ms | 0.037 ms | 0.048 ms | 0 |
| 5 | 0.100 ms | 0.110 ms | 0.122 ms | 0.138 ms | 0 |
| 20 | 0.400 ms | 0.413 ms | 0.428 ms | 0.465 ms | 0 |
| 100 | 2.000 ms | 2.019 ms | 2.055 ms | 2.104 ms | 0 |

`bash scripts/spike-async.sh --debug` (aynı makine/fixture, debug build):

| checkpoint_every | ~aralık | p50 | p95 | max | dispatch penceresi kaçağı |
|---|---|---|---|---|---|
| 1 | 0.020 ms | 0.037 ms | 0.043 ms | 0.149 ms | **1** |
| 5 | 0.100 ms | 0.115 ms | 0.121 ms | 0.131 ms | 0 |
| 20 | 0.400 ms | 0.416 ms | 0.426 ms | 0.477 ms | 0 |
| 100 | 2.000 ms | 2.028 ms | 2.052 ms | 2.084 ms | 0 |

Latency, checkpoint aralığına neredeyse birebir bağlı — kapı maliyeti
(kilit alma/çağrı) checkpoint aralığının yanında ölçülemeyecek kadar küçük.
"dispatch penceresi kaçağı" I1 **değil** (yukarıdaki düzeltme notuna bakın);
release'de 800/800 koşuda sıfır, debug'da 800/800'de 1 — debug'ın Swift
scheduling gecikmesi bu penceredeki tek gözlenen kaçağın sebebi.

### Kaynak bağlamı (I4 desteği)

| Ölçüm | release | debug |
|---|---|---|
| `live_jobs()` sweep sonrası | 0 | 0 |
| thread sayısı, başlangıç → bitiş | 3 → 3 | 3 → 3 |
| RSS, başlangıç → şu an | 9.75 → 10.14 MiB | 9.73 → 10.81 MiB |
| peak RSS | 10.17 MiB | 10.83 MiB |

### ADR-0028 sınır kontrolleri (üçü de boş — ihlal yok)

```
$ grep -rn --include='*.rs' -E "uniffi::|extern \"C\"" core/crates | grep -v crates/nen-ffi/
(boş)
$ cd core && cargo metadata --format-version 1 --no-deps   # crates/* → spikes/* kenarı
(boş)
$ grep -rn "spike" platforms/apple-shared/Package.swift
(boş)
```

### Tüm workspace

```
$ cargo test --manifest-path core/Cargo.toml --workspace
16 passed, 0 failed (nen-app 1 · nen-ffi 1 · spike-async-cancel 6 · spike-cue-transfer 8)
```

```
$ bash scripts/check-docs.sh   → SONUÇ: tüm denetimler geçti (exit 0)
$ bash scripts/test.sh         → exit 0
```
