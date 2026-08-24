# Durum

> **Bu dosya yalnız doğrulanmış bugünü anlatır.** Plan `roadmap.md`'de, kararlar
> `DECISIONS.md`'de, task ayrıntısı `tasks/INDEX.md`'de. Burada tekrar edilmez.
>
> Son güncelleme: **2026-08-24** (NEN-029 kapanışı, ADR-0026 accepted)

## Nerede duruyoruz

| | |
|---|---|
| **Mevcut milestone** | **M1 — Core Technical Spike** (M0 kapandı) |
| **Aktif task** | `NEN-005` — CI skeleton |
| **Son tamamlanan** | `NEN-029` — spike: playback/renderer reverse-FFI boundary |
| **Sıradaki READY** | `NEN-011` |
| **Task sayısı** | 32 · done 13 · active 1 · blocked 0 · backlog 18 |

**`NEN-029` kapandı ve `ADR-0026` accepted oldu.**
`core/spikes/spike-reverse-ffi/` — NEN-009'un delivery-gate deseni tek bir
crate içinde iki playback-ownership yönüne genelleştirildi: **A** (core-owned,
`ReverseEngine` fake motoru bir `PlaybackObserver` foreign trait'ini tekrar
tekrar çağırıyor) ve **B** (shell-owned, `ForwardSession`'a Swift kendi
`DispatchSourceTimer`'ıyla düz çağrılar yapıyor, ters çağrı yok). Release'de
60 Hz'de A'nın per-call maliyeti p50 **36.67 µs** + MainActor-hop p50
**~40 µs** (callback her zaman ana thread DIŞINDA düşüyor); B'nin maliyeti
p50 **1.33 µs**, hop hiç yok. Mutlak toplam ikisinde de küçük (A ~4.6 ms/sn,
B ~0.08 ms/sn — 1000 ms/sn'lik kare bütçesinin binde biri mertebesinde), yani
performans tek başına kararı belirlemedi. Asıl ayrım yapısaldı: bir
backgrounding proxy deneyi, A'nın Rust-taraflı üretici thread'inin Swift
tüketicisi hazır olmasa da üretmeye devam ettiğini gösterdi (Swift'in drain
kuyruğu 2 sn askıya alınırken Rust tıklamaya devam etti, sonra birikme
temizlendi) — B'de bu risk yapısal olarak yok. I1 ve I4 her iki yönde de
sağlandı; event ordering, seek/seek-complete sırası, typed error aktarımı
ikisinde de doğrulandı. Reentrancy: callback içinden `seek()` güvenli (ayrı
kilit kullanıyor), callback içinden aynı thread'den `cancel()` ise delivery
gate'in kendi kilidiyle self-deadlock — kod incelemesiyle kanıtlandı, canlı
çalıştırılmadı (gerçek bir deadlock `live_engines()` sayacını binary'nin geri
kalan testleri için kalıcı bozardı).

**`ADR-0026` A'yı önerdi, kullanıcı onayladı: merkezi Rust session (reverse
callback) kalıyor.** Gerekçe: A'nın mutlak maliyeti hiçbir makul UI bütçesini
zorlamıyor, hiçbir invariant ihlal edilmedi; merkezi session'ın asıl
gerekçesi zaten performans değil, subtitle sync/çeviri tetiklemenin tek
kaynaktan yönetilmesiydi — bu spike o gerekçeyi ölçemezdi, yalnız A'nın
uygulanabilir olduğunu doğruladı. `NEN-021`'in gerçek kontratı iki ölçülmüş
riski (backpressure/backgrounding, reentrancy disiplini) açıkça ele almak
zorunda — ADR-0026 → "Karar". `docs/architecture.md`'nin "Ownership yönü —
spike bekliyor" notu bu kararla güncellendi; `NEN-021` artık başlayabilir.

**Bulunan ve düzeltilen bir test kusuru:** ilk yazılan Rust testleri paralel
`cargo test` thread'leri arasında `LIVE_ENGINES`/`LIVE_FORWARD_SESSIONS`
global sayaçlarını paylaşıyordu. NEN-009'un aksine bu spike'ın tick'leri
gerçek wall-clock `sleep` kullanıyor (10-80 ms pencereler, NEN-009'un
mikrosaniye ölçekli busy-loop'larının aksine), bu da paralel test çakışmasını
çok daha olası kılıp 2/11 testi deterministik kırdı. Düzeltme:
`tests` modülüne bir `Mutex<()>` eklenip her test onu ilk satırda kilitledi;
üç ardışık koşuda 11/11 stabil geçti.

Kotlin ertelendi: task NEN-011'e bağımlı değil, bu makinede JDK kurulu değil
(B2). Ayrıntı, tam ölçüm tabloları (release+debug) ve ADR-0028 mekanik
denetim çıktıları task'ın kanıt kaydında.

`NEN-029`'un `adr:` alanı zaten `[26]` doğruydu — bu kez düzeltme gerekmedi.

**Eski `NEN-010` kapanışı.** `core/spikes/spike-typed-errors/` — beş varyantlı bir
`AppError` (UniFFI **rich** error, `flat_error` değil) Swift'e geçirilip
`default:` olmadan exhaustive bir `switch` ile ayrıştırıldığı gösterildi
(I3: "typed error string parse gerektirmiyor"). **Tasarım bulgusu:**
NEN-006'nın elle yazılmış `Debug` garantisi yalnız Rust `{:?}` çıktısını
korur — Swift'in kendi `String(describing:)` basımı Rust `Debug`'ını hiç
görmüyor, dolayısıyla FFI'yı geçen bir hata için koruma "inşa öncesi
sanitize et" ile sağlandı: `Parse` yalnız `nen_domain::redact::extension()`'ı
taşıyor (ham path'i asla), `Network` yalnız `redact_host()`'tan geçmiş
host'u. Bu sayede ham değer FFI teline hiç çıkmıyor; hem Rust `Debug`'ı hem
wire hem Swift'in varsayılan basımı aynı anda güvenli. Negatif kanıt
(NEN-006'daki `BadFixtureWithDerivedDebug` ile aynı teknik): sanitizing
constructor bypass edilip ham path doğrudan alana konursa, elle yazılmış
`Debug` bunu ayırt edemiyor ve gerçekten sızdırıyor — üstteki no-leak
testlerinin boşta dönmediğinin kanıtı.

**Yan bulgu:** bu uniffi sürümünde (0.32.0) `uniffi::Error` varyant adları
Swift'te PascalCase (`.Parse`), sıradan `uniffi::Enum` varyantları ise
camelCase (`.localAsr`) — iki derive makrosu arasında tutarsız isimlendirme;
M2'nin gerçek hata taksonomisi yazılırken hatırlanmalı.

**DoD #3 ("bilinmeyen varyant sessizce yutulmuyor") negatif kanıtı**
gerçek crate'e dokunmadan, tamamen ayrı bir scratch cargo+swift projesinde
aynı `uniffi::Error` mekanizmasıyla mekanik kanıtlandı: 3 varyantlı bir
enum + exhaustive switch derleniyor; switch'e dokunulmadan 4. varyant
eklenince `swift build` **"switch must be exhaustive"** ile kırılıyor.
Yani iddia çalışma zamanı davranışı değil derleme zamanı garantisi olarak
kanıtlandı — `doctor.test.sh`'ın shadow-PATH tekniğiyle aynı ruh (geçici
durum, mekanik kanıt, iz bırakmadan temizlik).

Baseline (M1-core-spike.md'nin istediği, eşik değil): 5 varyant; throw→catch→
switch eşleme maliyeti p50 **2.04 µs**, p95 **2.17 µs** (Apple M5 · macOS
27.0 · release, 10 000 tekrar).

`NEN-010`'un `adr:` alanı `[5]` → **`[28]`** olarak düzeltildi — NEN-006/
008/009'la aynı gerekçe: ADR-0005 dosyası yok, task onu kararlaştırmıyor;
gerçek dayanak spike-local FFI kapısını açan ADR-0028.

**`NEN-006` kapandı.** `nen-domain` (bağımlılıksız değer crate'i) içine bir
`redact` modülü eklendi: `Redacted<T>` (Debug/Display her koşulda
`<redacted>` basar), `extension()`, `size_class()`, `redact_host()` — hepsi
`docs/security-policy.md` §1'in "Loglanabilecekler" listesiyle sınırlı.
Guard test (`tests/guard_redaction.rs`), K23'ün 8 yasaklı deseninin
(medya URL, token query, tam özel yol, cue metni, raw provider response,
API key, OpenSubtitles private file ID, özel hash/filename) elle yazılmış
`Debug` kullanan örnek tiplerin `{:?}` çıktısında **hiç** görünmediğini
kanıtlıyor. **Negatif kanıt** (güvenlik task'ı için zorunlu):
`#[derive(Debug)]` ile yazılmış kasıtlı bozuk bir fixture aynı deseni
gerçekten sızdırıyor — yani kontrolün boşta dönmediği, gerçek bir
`derive(Debug)` hatasını yakalayacağı ayrıca kanıtlandı (NEN-032'nin
shadow-PATH kanıtıyla aynı biçim). Typed error tarafı: örnek `ErrorFixture`
enum'ının üç varyantı payload'a hiç dokunmadan, yalnız discriminant
üzerinden ayrıştırılabiliyor. `nen-domain`'in sıfır bağımlılık özelliği
korundu (`cargo tree` tek düğüm); `cargo clippy -D warnings` ve
`cargo fmt --check` (yalnız `nen-domain` kapsamında) temiz. Ayrıntı ve
tam test çıktısı task'ın kanıt kaydında.

`NEN-006`'nın `adr:` alanı `[5]` → **`[]`** olarak düzeltildi — NEN-007/008/009
ile aynı gerekçe: ADR-0005 dosyası `docs/adr/` altında henüz yok, task onu
kararlaştırmıyor, yalnız zaten kabul edilmiş `docs/security-policy.md`'yi
koda döküyor.

`NEN-032` kapandı. `doctor.sh`, `swift`i M3'ten M1'e taşıdı: NEN-007
(`test-apple.sh`) ve NEN-008 (`spike-cues.sh`) M1 içinde zaten Swift'e
bağımlıydı, ama doctor bunu M3'e kadar blocker saymıyordu — Swift'siz bir
makinede `doctor.sh M1` yanlışlıkla çıkış 0 verip hatayı ilk Swift
komutuna erteliyordu. `requirement()`'ta `swift` kendi satırına ayrıldı
(`swift:M1|swift:M3` → blocker), Xcode/libmpv'nin M3 grubu ve tam
Xcode/M1 istisnası dokunulmadan kaldı. Yeni test senaryosu (S7) "swift
yok" durumunu doğruladı — CLT kurulu bir Mac'te `/usr/bin/swift` gerçek
bir binary olduğundan, `command -v swift`'in onu bulamaması için
`/usr/bin`'in geri kalanı swift hariç bir gölge dizine bağlanıp PATH ona
yönlendirildi. `bash scripts/test.sh` ve `bash scripts/check-docs.sh`
yeşil; ayrıntı task'ın kanıt kaydında.

**`NEN-009` kapandı.** FFI sınırından geçen bir işin kooperatif iptali,
`Mutex<Option<Arc<dyn ProgressSink>>>` "delivery gate" tasarımıyla ölçüldü:
worker her checkpoint'te callback'i **kilit altında** çağırıyor, `cancel()`
aynı kilidi alıp sink'i temizliyor — böylece `cancel()` bir callback'in
ortasında dönemiyor ve döndükten sonra hiçbir checkpoint artık sink
bulamıyor. Üç invariant (I1 geç callback yok, I2 geç commit engellendi, I4
kaynak sızıntısı yok) hem release hem debug build'de Swift testleriyle
deterministik kanıtlandı — `core/spikes/spike-async-cancel/`.

Baseline (release · Apple M5 · macOS 27.0 · block_micros=20 µs sabit ·
200 koşu/satır): latency checkpoint aralığına neredeyse birebir bağlı —
checkpoint_every=1 → p50 **33 µs**, checkpoint_every=100 → p50 **2.02 ms**;
kapı maliyeti (kilit + çağrı) aralığın yanında ölçülemeyecek kadar küçük.

**Kanıt sürecinde bulunan bir kusur, sürecin kendisini de düzeltti:** ilk
yazılan Swift testi "iptal isteği" bayrağını `cancel()` çağrılmadan ÖNCE
işaretliyordu; 800 koşuluk sweep'te debug build'de 1 kaçak callback ortaya
çıktı (checkpoint_every=1'de). Bu I1'in ihlali değildi — "kullanıcı iptale
karar verdi" ile "Swift'in `cancel()`'ı fiilen çağırması" arasındaki dispatch
penceresinde meşru bir kaçaktı; testin ölçtüğü sınır I1'in gerçek sınırıyla
(cancel() **döndükten** sonra) örtüşmüyordu. Düzeltme: bayrak artık
`cancel()` döndükten SONRA işaretleniyor — kilit tasarımı gereği yapısal
olarak imkânsız bir kaçağı test ediyor, artık deterministik. Ayrıntı ve
release/debug tam tabloları task'ın kanıt kaydında.

`NEN-009`'un `adr:` alanı `[4]` → **`[28]`** olarak düzeltildi — NEN-007/008
ile aynı gerekçe: ADR-0004 (async/cancellation kontratı) ölçümler
(NEN-009+NEN-011+NEN-029) bitmeden `accepted` olamaz; task'ın gerçekte
dayandığı karar spike-local FFI kapısını açan ADR-0028.

**`NEN-008` kapandı — M1'in ilk sayıları var.** 50 000 cue'luk bir doküman
(3.1 MiB) Swift'e iki yoldan geçirilip ölçüldü (release · Apple M5 · macOS 27.0):

| | tam liste | pencere/handle |
|---|---|---|
| p50 | 37.7 ms | **44.3 µs** (40 cue'luk pencere) |
| en uzun main-thread bloğu | **48.4 ms** (~3 kare @60fps) | **0.08 ms** |
| peak RSS | 19.5 MiB | 11.7 MiB |

**Öneri: pencereli erişim** — ama iki kayıtla: (1) pencere cue **başına** %47
daha pahalı, kazancı hızdan değil ödemediği cue'lardan geliyor; (2) dokümanın
*tamamı* gerçekten gerekiyorsa tek çağrı %35 daha ucuz (37.7 ms'e karşı
50.7 ms), yani "her şey pencereli olsun" kuralı yanlış olur. `activeCue`
**1.50 µs** — playback sırasında cue aramanın FFI maliyeti pratikte yok; bu
sayı NEN-029'a ve M7'nin position çözünürlüğüne girdi. Ayrıntı, debug/release
karşılaştırması ve doğrulama çıktıları task'ın kanıt kaydında.
**Bunlar baseline'dır, eşik değil** — bütçe ADR-0027 ile kabul edilecek.

Ön koşul olarak **ADR-0028 kabul edildi**: ADR-0006 kural 2 ("`nen-ffi` tek dış
kapıdır"), kural 3 ve CLAUDE.md kural 7 birlikte M1'in FFI ölçümlerine yer
bırakmıyordu. ADR-0028 kural 2'nin kapsamını **ürün koduyla** sınırlıyor;
`core/spikes/*` altındaki bir spike crate yalnız ölçüm için kendi atılabilir
kapısını açabiliyor. Üç sınır grep + `cargo metadata` ile mekanik doğrulanıyor
ve çıktıları kanıt kaydında. ADR-0006 **düzenlenmedi** — yalnız "Notlar"ına
işaret eklendi (ADR-0001'in izin verdiği istisna).

`NEN-008`'in `adr:` alanı `[3]` → **`[28]`** olarak düzeltildi; NEN-007'de
yapılan düzeltmenin aynısı (ADR-0003 spike ölçümleri olmadan `accepted` olamaz,
üstelik dosyası da yok).

`NEN-031` kapandı: `scripts/tests/check-docs.test.sh` artık **kendi task
fixture'ını kuruyor** — canlı `tasks/` ve `INDEX.md` okunmuyor. Denetim 8'in her
iki dalı (ready listesi dolu / boş) ayrı ayrı doğrulanıyor; boş-ready dalı bugüne
kadar hiç test edilmemişti. Yan etki: koşu 16.4 s → 1.6 s (eski fixture tüm
repo'yu, `core/target` dahil 1.2 GB, tarlıyordu). Bu, **NEN-005'in (CI) ön
koşuluydu**.

Ondan önce `NEN-007` ile repository kod içermeye başlamıştı: Cargo workspace,
ADR-0006'nın tarif ettiği 11 crate ve `nen-ffi` üzerinden Swift'e geçen bir
`version()` fonksiyonu ayakta. Milestone sırası önerisi (`M1-core-spike.md`)
**NEN-009 ve NEN-010** ile devam ediyor — ikisi de aynı spike zeminini
kullanacak.

**UniFFI hâlâ aday.** NEN-007 binding teknolojisini seçmedi; ADR-0003
NEN-011/NEN-012'de karara bağlanacak. Kabul edilen mimari karar **ADR-0006** —
monorepo yapısı, crate sınırları ve `nen-ffi`'ın tek dış kapı olması.

NEN-007'nin `adr:` alanı `[3, 6]` → **`[6]`** olarak düzeltildi: ADR-0003 spike
ölçümleri olmadan `accepted` olamaz, yani task'ın kapanışını kilitliyordu.
Aynı kusur NEN-008'de bu kapanışta giderildi; geriye **`NEN-011`** kaldı.

## Toolchain

`bash scripts/doctor.sh M1` (2026-08-24, bu makine — macOS 27.0):

| Araç | Durum | M1'deki seviyesi |
|---|---|---|
| `cargo` / `rustc` | ✅ 1.98.0 (2026-08-18) | blocker — **karşılandı** |
| `cargo-deny` | ❌ eksik | soon — NEN-005 öncesi |
| JDK | ❌ eksik | soon — NEN-011 (Kotlin/JVM parity) öncesi |
| `swift` | ✅ Apple Swift 6.4 | M1 (blocker) — NEN-007 testi ve NEN-008 harness'ı kullanıyor |
| Tam Xcode | ❌ yalnız `/Library/Developer/CommandLineTools` | M3 — **M1 blocker'ı değil** |
| libmpv | ❌ eksik | M3 — **M1 blocker'ı değil** |
| Gradle | ❌ eksik | hiçbir milestone'da blocker değil (wrapper) |
| Android SDK | ❌ eksik | M10 |

Rust `rustup` ile kuruldu (2026-08-24). Workspace `core/rust-toolchain.toml` ile
**1.98.0'a pinli** — M1 ölçümlerinin başka makinede karşılaştırılabilmesi için.

`doctor.sh M1` → **çıkış 0** · `doctor.sh` (parametresiz) →
**çıkış 0** (bilgilendirici). Kurulum komutları çıktıda; script **hiçbir şey
kurmaz** — bu, `scripts/tests/doctor.test.sh` S7 ile mekanik olarak kanıtlanıyor.

## Blocker'lar

| # | Blocker | Kimi durduruyor | Çözüm |
|---|---|---|---|
| ~~B1~~ | ~~Rust kurulu değil~~ | — | ✅ **çözüldü** 2026-08-24 — rustup, 1.98.0 |
| B2 | JDK yok | `NEN-011` (M1 içinde, sıra gelmedi) | `brew install --cask temurin` |
| B3 | Tam Xcode + libmpv yok | M3; Swift testleri CommandLineTools'ta ek bayrak istiyor (`scripts/test-apple.sh` hallediyor) | App Store'dan Xcode + `brew install mpv` |
| ~~B4~~ | ~~`check-docs.test.sh` fixture'ı canlı repo durumuna bağlı~~ | — | ✅ **çözüldü** 2026-08-24 — `NEN-031` |

**Gerçek blocker kalmadı.** B2 ve B3 sıradaki task'ları engellemiyor.

## Kullanıcı kararı bekleyenler

| # | Konu | Ne zaman gerekiyor |
|---|---|---|
| **S3** | Çeviri kalite hedefinin operasyonel ölçütü | M5 |
| **S4** | Offline/uçak modu birinci sınıf mı? S8'in "cloud sync non-goal" cevabı bunu doğrudan etkiliyor — sync yoksa offline davranış tamamen yerel cache'e bağlı | M5–M6 |
| **S6** | Local ASR modeli ve cihaz kaynak bütçesi *(privacy kısmı cevaplandı)* | M8 |
| **S7** | Android TV minimum API seviyesi ve hedef cihaz sınıfı | M10 |
| **S9** | Birden fazla AI artifact'in UI'da gösterimi | M5 |
| **S11** | İleride public dağıtım | M3 sonrası |
| **S12** | Gerçek lisans seçimi | ADR-0012 sonrası |

Hiçbiri sıradaki task'ları bloke etmiyor.

## Son doğrulama

2026-08-24, tümü bu makinede çalıştırıldı (Apple M5 · arm64 · macOS 27.0
26A5416b · rustc/cargo 1.98.0 · Swift 6.4 · uniffi 0.32.0):

```
$ bash scripts/doctor.sh M1
SONUÇ: M1 için tüm blocker'lar hazır.            → exit 0

$ cargo test --manifest-path core/Cargo.toml --workspace
spike_cue_transfer 8 · spike_async_cancel 6 · spike_typed_errors 5 ·
spike_reverse_ffi 11 · nen-app 1 · nen-ffi 1 ·
nen-domain 4 (unit) + 6 (guard_redaction) = 10
42 passed, 0 failed                              → exit 0

$ cargo tree -p nen-domain --edges normal
nen-domain v0.1.0                                → tek düğüm, sıfır bağımlılık

$ cargo clippy -p nen-domain --all-targets --manifest-path core/Cargo.toml -- -D warnings
                                                  → exit 0, uyarı yok (NEN-006)

$ bash scripts/build-apple.sh                     → binding temiz üretildi
$ bash scripts/test-apple.sh
✔ Test run with 2 tests in 1 suite passed        → exit 0

$ bash scripts/spike-cues.sh                      → NEN-008 release baseline
$ bash scripts/spike-cues.sh --debug              → NEN-008 debug karşılaştırması

$ bash scripts/spike-async.sh --test-only         → NEN-009 invariant'lar (release)
$ bash scripts/spike-async.sh --debug --test-only → NEN-009 invariant'lar (debug)
✔ Test run with 3 tests in 1 suite passed        → exit 0 (ikisinde de)
$ bash scripts/spike-async.sh --measure-only      → NEN-009 release baseline
$ bash scripts/spike-async.sh --debug --measure-only → NEN-009 debug karşılaştırması

$ bash scripts/spike-typed-errors.sh
✔ Test run with 2 tests in 1 suite passed        → exit 0 (switch + redaction)
  eşleme maliyeti p50 2.04 µs · p95 2.17 µs        → NEN-010 baseline
  negatif kontrol (scratch): 4. varyant swift build'i "switch must be
  exhaustive" ile kırdı                            → DoD #3 kanıtlandı

$ bash scripts/spike-reverse-ffi.sh --test-only         → NEN-029 (release)
$ bash scripts/spike-reverse-ffi.sh --debug --test-only → NEN-029 (debug)
✔ Test run with 12 tests in 3 suites passed      → exit 0 (ikisinde de)
$ bash scripts/spike-reverse-ffi.sh                     → NEN-029 release baseline (A+B)
$ bash scripts/spike-reverse-ffi.sh --debug              → NEN-029 debug karşılaştırması
  A p50 36.67 µs + hop ~40 µs · B p50 1.33 µs (60 Hz, release)
  → NEN-029 baseline, ADR-0026'nın dayanağı

$ bash scripts/check-docs.sh
  8/8 denetim geçti                              → exit 0

$ bash scripts/test.sh
  check-docs.test.sh ✓ (11 doğrulama)
  doctor.test.sh     ✓ (24 doğrulama)            → exit 0
```

**Ölçüm build tipi artık kayıt altında.** NEN-007'nin kanıtı debug'dı; NEN-008
ikisini de koştu ve farkı ölçtü: debug, tam liste geçişini **1.75×**, pencere
erişimini **1.24×** yavaşlatıyor. İki koşunun Swift tarafındaki checksum'ları
birebir aynı — fark yalnız hızda, veride değil.

`scripts/test.sh` **`tasks/active/` dolu ve boşken ayrı ayrı** koşuldu; iki
koşunun `check-docs.test.sh` çıktısı birebir aynı (NEN-031 kanıt kaydı).

**B4 kapandı — kaydedilmiş gerekçesi de hatalıydı.** B4, `check-docs.test.sh`'ın
"bir task `active` olduğu anda" kırıldığını söylüyordu. Gerçek tetikleyici bu
değil: **ready listesinin boşalması**. NEN-031 `active/`'e alındığında listede
NEN-005/006/008/009/010 kaldı ve `test.sh` yeşil kaldı — yani "bir sonraki task
başlatıldığında yeniden kırmızıya dönecek" beklentisi de yanlıştı. NEN-007'de
kırılmasının sebebi, o an her backlog task'ının NEN-007'ye bağlı olmasıydı.
`check-docs.sh`'ın kendisi her iki durumda da doğru çalışıyordu (exit 0); kusur
yalnız fixture'daydı ve `NEN-031` ile kapandı.

## Repository'nin gerçek durumu

- **Kod var** (NEN-007 ile): `core/` altında Cargo workspace + 11 crate,
  `core/rust-toolchain.toml`, `Cargo.lock`.
- `core/crates/nen-domain/src/redact.rs` — NEN-006'nın redaction yardımcıları
  (`Redacted<T>`, `extension()`, `size_class()`, `redact_host()`); crate hâlâ
  bağımlılıksız. Guard test `core/crates/nen-domain/tests/guard_redaction.rs`.
- `core/spikes/spike-cue-transfer/` — NEN-008'in ölçüm crate'i ve içinde kendi
  Swift harness'ı (`apple-harness/`). **Ürün kodu değil, terfi etmez**; kendi
  FFI kapısını ADR-0028 sayesinde açıyor. Üretilen binding commit edilmiyor.
- `core/spikes/spike-async-cancel/` — NEN-009'un ölçüm crate'i; kendi Swift
  harness'ı hem CLI ölçüm hedefi hem swift-testing invariant test hedefi
  içeriyor (`apple-harness/Tests/`). Aynı ADR-0028 kapısı, aynı terfi yasağı.
- `core/spikes/spike-typed-errors/` — NEN-010'un ölçüm crate'i; `nen-domain`'e
  bağımlı (redaction yardımcıları), kendi Swift harness'ı hem switch/redaction
  test hedefi (`apple-harness/Tests/`) hem baseline ölçüm hedefi
  (`apple-harness/Sources/`) içeriyor. Aynı ADR-0028 kapısı, aynı terfi yasağı.
- `core/spikes/spike-reverse-ffi/` — NEN-029'un ölçüm crate'i; A (`ReverseEngine`,
  foreign `PlaybackObserver` trait'ini tekrar çağıran fake motor) ve B
  (`ForwardSession`, düz forward çağrılar) tek crate'te. Swift harness'ı hem
  baseline ölçüm hedefi (`apple-harness/Sources/`) hem üç ayrı swift-testing
  hedefi (`InvariantTests`/`OrderingTests`/`TypedErrorTests`) içeriyor. Aynı
  ADR-0028 kapısı, aynı terfi yasağı.
- `platforms/apple-shared/` — SwiftPM paketi (`Package.swift` + swift-testing
  test target'ı). Üretilen binding `generated/` altında ve **commit edilmiyor**.
- Diğer `platforms/*` dizinleri hâlâ boş iskelet.
- Var olan: 6 ana doküman · 12 milestone dosyası · **4 accepted ADR**
  (0001, 0006, 0026, 0028) · 32 task · 10 script + 2 shell testi ·
  `fixtures/` iskeleti.
- Depo kökünde **`LICENSE` dosyası bilerek yok** — bkz. [`licensing.md`](licensing.md).
- Git: `main` branch. **Bu dosya commit hash'i tutmaz** — commit geçmişi
  kanonik kayıttır ve elle tutulan hash satırı her kapanışta bayatlar
  (CLAUDE.md → "Commit politikası").

## Bu dosyayı kim günceller

Her task kapanışında (`/finish-task`) ve her milestone geçişinde.
`scripts/check-docs.sh` **denetim 8**, buradaki "Sıradaki READY" satırının
`tasks/INDEX.md` ile uyuşmasını mekanik olarak zorlar — bayatlarsa CI kırılır.
