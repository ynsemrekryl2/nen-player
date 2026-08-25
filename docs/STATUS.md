# Durum

> **Bu dosya yalnız doğrulanmış bugünü anlatır.** Plan `roadmap.md`'de, kararlar
> `DECISIONS.md`'de, task ayrıntısı `tasks/INDEX.md`'de. Burada tekrar edilmez.
>
> Son güncelleme: **2026-08-25** (NEN-019 kapanışı — subtitle source catalog)

## Nerede duruyoruz

| | |
|---|---|
| **Mevcut milestone** | **M2 — Subtitle Core** (M1 kapandı) |
| **Aktif task** | *yok* — `tasks/active/` boş |
| **Son tamamlanan** | `NEN-019` — SubtitleSourceCatalog with grouping and dedup |
| **Sıradaki READY** | `NEN-020`, `NEN-021`, `NEN-033`, `NEN-034`, `NEN-035`, `NEN-036` |
| **Task sayısı** | 38 · done 23 · active 0 · blocked 0 · backlog 15 |

**`NEN-019` kapandı — dört kaynak tek katalog ve tek menü projeksiyonunda.**
`nen-domain`'e `SubtitleSourceKind` · `SubtitleSourceId` · `SubtitleSource` ·
`LanguageTag` · `SubtitlePreferences`; `nen-catalog`'a metadata-kimlikli upsert,
dil/kullanıcı gruplama, yapısal menü projeksiyonu ve saf otomatik seçim
politikası eklendi. Otomatik seçim yalnız tercih edilen dillerde ve bugün yalnız
`embedded` → `user`; `opensubtitles` basamağı NEN-038'e kadar kapalı, `ai` hiçbir
koşulda otomatik seçilmiyor.

**ADR-0010 Karar 10'un iki menüsü byte-eşit golden.** Aynı katalog tercih yokken
`en` → `fr` → `tr`, birinci `tr` / ikinci `en` tercihinde `tr` → `en` → `fr`
grup sırasını üretiyor; kullanıcı grubu `Kapalı`'nın hemen altında, bilinmeyen
dil her zaman en sonda. `Debug` guard'ı özel tam yol, dosya adı, digest ve
private file ID'yi gizliyor; `#[derive(Debug)]`'lı bozuk ikiz dördünü de
sızdırarak negatif kontrolü kanıtlıyor. Yeni doğrudan dış bağımlılık yok,
`nen-domain` hâlâ tek düğüm. Test sayısı **284 → 317**. Tam kanıt:
`tasks/done/NEN-019-*.md`.

**`NEN-018` kapandı — `nen-identity` artık dolu.** Dokuz modül eklendi
(`os_hash` · `release_name` · `url_hints` · `dir_hints` · `declared_name` ·
`siblings` · `nfo` · `container` · `evidence`), **yeni dış bağımlılık yok** —
`nen-identity` tek kenar (`nen-domain`), `nen-domain` hâlâ tek düğüm,
`deny.toml`'a dokunulmadı. Test sayısı **121 → 284**.

**Golden'lar dört gerçek parser kusuru buldu, hiçbiri elle fark edilmemişti:**
(1) `Blade.Runner.2049.2017…` başlığı "Blade Runner", yılı **2049** sanıyordu —
arka arkaya iki yıl-benzeri token varsa artık ilki başlığın parçası; (2)
`[SubGroup] Steins Gate` başlığa fansub grubunu katıyordu; (3) `Unknown` bir
sonuç yarım başlık bırakıyordu — artık `Unknown ⇒ title = None`; (4)
`Severance (2022)/Season 02/` `movie` dönüyordu, sezon da artık dizi işareti.

**Wire biçimi kusuru elle doğrulanabilir vektörle yakalandı.** Hash ilk sürümde
`to_le_bytes()` ile saklanıyordu; OpenSubtitles `%016x` ile **big-endian**
basıyor. Tamamı sıfır olan 131 072 byte'lık dosyanın hash'inin kendi boyutu
olması gerektiği (`0000000000020000`) ilk koşuda uyuşmazlığı gösterdi — yani
gönderilecek hash baştan yanlış olacaktı. DoD #1 üç ayrı kanıt taşıyor: elle
doğrulanabilir bilinen cevap · kasıtlı olarak farklı yazılmış **bağımsız
referans implementasyonu** (açık indeks aritmetiği + elle bit kaydırma) ·
commit edilmiş golden.

**Negatif kontrol iki biçimde.** Kalıcı olanı `DerivedEvidence`: aynı değerleri
taşıyan `#[derive(Debug)]`'lı kasıtlı bozuk ikiz, 10 yasak desenin **hepsini**
sızdırdığı sürekli doğrulanıyor — sızdırmazsa test kırılır, yani "yasak desen
yok" iddiası boşta dönemez. Tek seferlik mekanik doğrulama da yapıldı:
`MediaEvidence`'ın elle yazılmış `Debug`'ına kasıtlı sızıntı sokulunca guard
2/7 testte kırmızıya döndü, dosya geri alındı.

**Üç tasarım kararı kapanışta kayda geçti.** (1) Boşluk doldurma yalnız
sayısal alanlarda — kazanan katman başlığı ve türü sahiplenir, alt katmanlar
yalnız eksik yıl/sezon/bölüm verir. (2) Bölüm **veya sezon** türü kesinleştirir;
sezonlu bir `Movie` döndürmek tutarsız olurdu. (3) **Kardeş mutabakatı
daraltıldı:** task açılışında "yanlış `SxxEyy`'leri eler" yazıyordu, bu
ADR-0009'un "düşürmez" ilkesiyle çelişiyordu — artık yalnız verdict döndürüyor
ve eksik bir dizi adını dolduruyor. Tam kanıt: `tasks/done/NEN-018-*.md`.

**Eski `NEN-018` açılışı ve `ADR-0009`.** Medya kimliği **katmanlı bir
kanıt modeliyle** çözülecek: beyan katmanları (handoff metadata · `.nfo`
sidecar · container metadata · dosya adı beyanı) tahmin katmanlarının
(üst klasör adları · kardeş dosya teyidi · URL path segmentleri) üstünde;
ilk `Unknown` olmayan katman kazanır. Gerekçe kullanıcı kararında: gömülü
altyazı yoksa altyazılar OpenSubtitles'tan gelecek ve bu ancak kimlik
çözülürse mümkün — **kullanıcıya aday listesi göstermek son çaredir**, hedef
medyaların büyük çoğunluğunda hiç sormamak.

ADR-0009 şartname §6'nın kanıt listesini **iki yönde genişletti**: (1) uzak
medyada URL'in yalnız basename'i değil **tüm path segmentleri** ipucu sayılıyor
— Stremio/debrid URL'lerinde basename çoğu zaman anlamsız (`stream.mkv`, hash
adı), anlamlı ad üst segmentte; (2) sunucunun beyan ettiği ad
(`Content-Disposition`, yönlendirme zincirinin sonu) ayrı bir kanıt katmanı
oldu. Query, fragment ve host **hiçbir koşulda** kimliğe girmiyor (§6 yasağı +
K23 #1/#2 aynen korundu). `docs/product-spec.md` §6'ya ADR'ye işaret eden bir
not düşüldü; şartname yeniden yazılmadı (ADR-0001'in izin verdiği biçim).

Kapsam `size: M` → **`L`** oldu (offline kanıt katmanları eklendi). Dört takip
task'ı açıldı: **NEN-033** (OSDb hash → IMDb ID, M6) · **NEN-034** (AI ile
release-name normalizasyonu, M6 — kendi gizlilik ADR'sini ister) ·
**NEN-035** (güven skoru ve aday sıralama, M6 — gösterim oranı ölçülebilir
kabul kriteri) · **NEN-036** (uzak kanıt portu: HEAD/`Content-Disposition`/
bounded redirect/`Range`, M3).

**Torrent/debrid non-goal olarak kaldı** — kullanıcı kararı, doküman
değişikliği yapılmadı. ADR-0009 Notlar bölümü bunun kimlik çıkarımını neden
zayıflatmadığını kaydediyor: Stremio senaryosunda URL infohash tabanlı ve opak
olsa bile kimlik handoff extras'tan ve `Range` ile okunan container başlığı +
hash'ten gelebilir.

**Eski `NEN-017` kapanışı — M2'nin cue lookup çıkış kriteri karşılandı.**
`nen-subtitle`'a `CueIndex` eklendi (`index.rs`): ödünç alınmış bir
`SubtitleDocument` üzerinde `active_cues(at)` · `active_cue(at)` ·
`cues_in(range)`. **Yeni bağımlılık yok.**

**API çakışan cue'ları düşürmüyor.** NEN-013 çakışmayı kabul ettiği için bir
`t` anında birden fazla cue aktif olabilir; birincil sorgu `active_cues`
hepsini doküman sırasında döndürüyor. Tekil `active_cue` kolaylık
sarmalayıcısı olarak kaldı ama artık *ilk* cue'yu döndürüp diğerlerini attığı
doküman yorumunda açık — NEN-027 istiflenmiş konuşmacıyı sessizce
kaybetmesin diye.

**Yapı: sıralı dizi + `max_end_prefix` artırımı** (iki `partition_point`).
Reddedilen alternatif boundary-event segment dizisiydi: sorgusu en kötü
durumda da `O(log n + k)` olurdu ama tamamı çakışan `n` cue'da `O(n²)` bellek
tüketirdi — bu crate güvenilmeyen girdi ayrıştırdığı için (10 MiB sınırı
~100k cue'ya izin veriyor) belleğin tükenmesi, tek bir sorgunun yavaşlamasından
kötü bir başarısızlık biçimi. Seçilen yapı girdi ne olursa olsun `O(n)`
bellekte. Bedeli kayıtta: aday penceresi çakışma derinliğiyle büyür; dokümanı
baştan sona kaplayan tek cue şekli parity korpusunda var, doğruluk bozulmuyor.

**Baseline** (Apple M5 · release · 3 koşunun aralığı): 50k cue'da index
lookup p50 **48–57 ns**, lineer tarama p50 **15.6–17.6 µs** → **~290–344×**.
Ölçüm `#[ignore]`'lu (`scripts/bench-cue-lookup.sh`), CI yavaşlamıyor.
**Ölçüm yönteminin kendisi bir kusur buldu:** ilk sürümde hangi tablo satırı
önce koşarsa ~2.5× yavaş çıkıyordu (sıra ters çevrilince sapma da tersine
döndü) — sebep lookup değil, taze 50k cue'luk dokümanın ilk dokunuş
page-fault'larıydı; her satır iki kez ölçülüp yalnız ikincisi raporlanarak
düzeltildi.

**Negatif kontrol:** parity testinin boşta dönmediği, `index.rs`'e iki yönde
(fazla cue döndüren / eksik cue döndüren) kasıtlı hata sokularak kanıtlandı —
ikisinde de 3/3 parity testi kırmızıya döndü, sonra dosya geri alındı.

Test sayısı 108 → **121** (+10 index unit, +3 `lookup_parity`; benchmark
`ignored`). **ADR yazılmadı** — yapı `nen-subtitle` içinde kalıyor, crate
sınırı/bağımlılık grafiği değişmiyor, product-spec §14 zaten lineer taramayı
yasaklıyor; karar task'ın kanıt kaydında. `adr:` alanı `[7]` → **`[]`**
düzeltildi (NEN-013/014/015 precedent'i). Tam kanıt:
`tasks/done/NEN-017-*.md`.

**Eski `NEN-016` kapanışı — `ADR-0007` accepted oldu.** `nen-subtitle`'a
(`nen-domain`'e değil — `docs/architecture.md`'nin crate tablosu "timeline"ı
zaten `nen-subtitle`'a veriyor ve bu, `nen-domain`'in her kapanışta
doğrulanan sıfır-bağımlılık özelliğini korur) bir `fingerprint` modülü
eklendi: `TimelineFingerprint::of` yalnız cue zamanlarını (`blake3` ile, `u32
LE` cue sayısı + sıralı `start_ms`/`end_ms`), `SourceFingerprint::of` aynısını
+ cue başına satır sayısı + uzunluk-önekli satır byte'larını hash'liyor.
`CueId` hiçbir hash'e dahil değil — sıra zaten doküman-sırası iterasyonuyla
kodlanıyor. `ADR-0007` ayrıca `subtitle.rs`'nin NEN-013'ten beri açık bıraktığı
"stable, cross-source cue identity" sorusunu kapattı: yeni bir kimlik tipi
**yok**, kaynaklar arası eşleştirme doküman-düzeyi fingerprint üzerinden
yapılacak (M5/M7). `blake3`'ün lisansı (`CC0-1.0 OR Apache-2.0`) `deny.toml`
değişikliği gerektirmedi — `Apache-2.0` kolu zaten izinli listedeydi. 8 yeni
unit test (DoD'un 4 maddesi + `CueId` etkisizliği + satır bölünmesi
ayrışması + boş doküman paniksizliği), test sayısı 100 → **108**. Tam kanıt:
`tasks/done/NEN-016-*.md`.

**Eski `NEN-015` kapanışı — encoding detection and sanitization.** `nen-subtitle`'a `srt::parse`'ın önünde çalışan bir
encoding/sanitization katmanı (`encoding::decode`) eklendi: BOM sniff (UTF-8 →
UTF-16LE → UTF-16BE), BOM yoksa önce sıkı UTF-8 denemesi, başarısız olursa
**Windows-1254**'e (tek legacy fallback) düşüş; ardından kontrol karakteri
(`\n`/`\r` hariç), bidi override ("Trojan Source" sınıfı) ve zero-width
karakter temizliği. 10 MiB boyut sınırı decode denenmeden önce uygulanıyor.
Workspace'in ilk gerçek dış bağımlılığı **`encoding_rs`** (WHATWG Encoding
Standard implementasyonu, Firefox/Servo) eklendi — lisansı
`(Apache-2.0 OR MIT) AND BSD-3-Clause` çıktığı için `core/deny.toml`'a
`BSD-3-Clause` eklendi, `cargo deny check` yeşil. Fixture korpusu
(`fixtures/subtitles/encodings/`) 8 byte-precise dosya: 7 pozitif (BOM'lu
UTF-8/UTF-16LE/UTF-16BE, BOM'suz CP1254 Türkçe metin, BOM'suz CP1252 Batı
Avrupa metni, bidi-override enjeksiyonu, zero-width enjeksiyonu) + 1 negatif
(UTF-16LE BOM + eşleşmeyen surrogate). Test sayısı 82 → **100**.

**`ADR-0008` kabul edildi** (`Karar 1`: `encoding_rs`; `Karar 2`: BOM'suz
durumda CP1252 ile otomatik ayrım **yapılmıyor** — yalnız Windows-1254,
CP1254/CP1252 ayrımı BOM'suz genel durumda çözülemez ve dil tahmini
NEN-020'nin kapsamı; `Karar 3`: bidi/zero-width karakterler escape değil
**silinir**). `docs/adr/README.md` "Yazılmış" tablosuna taşındı.

**Eski `NEN-014` kapanışı — WebVTT writer.** `nen-subtitle`'a bir WebVTT writer (`webvtt::write`)
eklendi: `SubtitleDocument` → `WEBVTT` başlığı, cue başına identifier
(`CueId`) + `HH:MM:SS.mmm --> HH:MM:SS.mmm` zaman satırı + `&`/`<`/`>` kaçışlı
metin satırları, BOM hiç yazılmıyor. Fixture korpusu 7 → **8** geçerli dosyaya
çıktı (`html-special-chars.srt`, kaçış kapsamını kanıtlamak için eklendi);
8 dosyanın hepsi için SRT → doc → WebVTT round-trip `.vtt` golden'ı commit
edildi. Test sayısı 72 → **82**. `adr:` alanı `[7]` → **`[]`** düzeltildi —
NEN-013'teki aynı gerekçe: ADR-0007'nin konusu NEN-016'nın kararı. Tam kanıt:
`tasks/done/NEN-014-*.md`.

**Eski `NEN-013` kapanışı — M2'nin ilk ürün kodu.** `nen-domain`'e subtitle
değer tipleri (`CueId` · `TimeSpan` · `Cue` · `SubtitleDocument`),
`nen-subtitle`'a strict SRT parser'ı ve **17 varyantlı** `SrtError` eklendi.
Fixture korpusu: 7 geçerli dosya + 7 `.golden` snapshot, **25 malformed**
dosya (her biri tek bozukluk), ve 17 varyantın hepsi en az bir fixture'la
kapsanıyor. Test sayısı 42 → **72**; `nen-domain` sıfır bağımlılıklı kaldı,
`nen-subtitle` yalnız `nen-domain`'e bağlı (ADR-0006 grafiği). Tam kanıt:
`tasks/done/NEN-013-*.md`.

**Üç tasarım kararı** kapanışta kayda geçti: (1) **çakışan cue'lar kabul
ediliyor** — farklı konuşmacı SRT'lerinde meşru, reddetmek NEN-025'te
kullanıcının geçerli dosyasını kırardı; sıra yine zorlanıyor
(`NonMonotonicCue`, eşit başlangıç serbest). (2) Index dizisi **tam** 1,2,3,…
olmak zorunda — atlama/tekrar/sıra bozukluğu tek varyantla ifade ediliyor.
(3) BOM **reddediliyor, atlanmıyor**; encoding NEN-015'in işi ve parser'ın
girdisi `&str`, yani kapsam sınırı tipte duruyor.

**Fuzz `cargo-fuzz` ile değil, deterministik smoke ile yapıldı** — nightly
toolchain gerektirirdi ve `doctor.sh`'a yeni bir gereksinim eklerdi. Sabit
tohumlu (`0x4E45_4E30_3133`) xorshift64* üreteci **16 281 vaka** üretiyor
(781 truncation · 10 500 mutation · 5 000 noise), 0.07 s'de koşuyor ve her
`cargo test` ile CI'da otomatik tekrarlanıyor. İddia yalnız "panik yok"
değil: `Ok` dönen her vakada doküman kendi invariant'larından geçiriliyor.

`NEN-013`'ün `adr:` alanı `[7]` → **`[]`** olarak düzeltildi — NEN-006/007/
008/009/010/011 ile aynı precedent: ADR-0007'nin konusu (cue kimliği ve
timeline fingerprint algoritması) **NEN-016**'nın kararı; NEN-013 yalnız
ADR-0006'nın zaten çizdiği crate sınırlarını dolduruyor.

**Eski yan bulgunun bugünkü durumu:** `AGENTS.md` artık izlenen giriş noktası ve
kanonik `CLAUDE.md`'ye yönlendiriyor. Yalnız `.agents/` untracked; task
commit'lerine dahil edilmiyor.

**Eski `NEN-012` kapanışı — M1 kilitlendi.** Beş spike'ın (NEN-008/009/010/011/029)
ölçümleri `ADR-0002`'de sentezlendi: **Rust shared core dili olarak kabul
edildi** (go), I1–I5 invariant'larının hepsi kanıtlı, M1'in üç no-go
koşulundan hiçbiri tetiklenmedi. Reddedilen alternatifler (Kotlin
Multiplatform · Swift core + ayrı Android · C++ core) ayrı spike edilmedi —
zaten kanıtlanmış bir adaydan geçmenin ölçülmüş gerekçesi yoktu. Aynı task
kapsamında `ADR-0027` (dört FFI performans bütçesi, ölçülen p50/p95'in
üzerine 4–11× marj) de `accepted` oldu. `docs/architecture.md`,
`docs/DECISIONS.md` ve `docs/roadmap.md` güncellendi (Rust artık "aday"
değil; M1 → kapandı, M2 → sıradaki). Tam kanıt: `tasks/done/NEN-012-*.md`.

**Yan bulgu (ayrı arka plan görevine yönlendirildi, NEN-012 kapsamı değil):**
`docs/DECISIONS.md`'nin "Ertelenmiş kararlar" tablosu `ADR-0026`'yı hâlâ
bekleyen olarak listeliyor — `NEN-029` kapanışında güncellenmemiş bir kusur.

**Eski `NEN-011` kapanışı.** NEN-008/009/010'un ölçtüğü üç Rust crate'ine
dokunulmadan, her birine Swift'teki `apple-harness/`'in eşi bir
`jvm-harness/` (Gradle wrapper tabanlı Kotlin/JVM projesi) eklendi — binding
üretimi aynı `spike-*-uniffi-bindgen` binary'lerinden, yalnız
`--language kotlin` ile (`scripts/spike-cues-jvm.sh`,
`spike-async-jvm.sh`, `spike-typed-errors-jvm.sh`).

**Ön koşul: B2 kapandı.** Bu makinede gerçek bir JDK yoktu (yalnız CLT'nin
çalışmayan `/usr/bin/java` stub'ı). `brew install --cask temurin` sudo şifresi
istediği için bu ortamda başarısız oldu; `brew install openjdk` (formula,
sudo gerektirmez) kullanıldı, `/opt/homebrew/opt/openjdk/bin` `~/.zshrc`'ye
PATH eklendi. `bash scripts/doctor.sh M1` artık JDK'yi ✓ gösteriyor. Bootstrap
için `brew install gradle` (9.7.1) geçici kuruldu — yalnız üç `jvm-harness/`'ta
bir kere `gradle wrapper` çalıştırıp kendi `gradlew`'lerini üretmek için;
bundan sonra hiçbir geliştiricinin sistem Gradle'ı kurmasına gerek yok.

**Kotlin'e özgü iki isimlendirme sapması kaynak incelemesiyle doğrulandı**
(`uniffi_bindgen` 0.32 `bindings/kotlin/gen_kotlin/mod.rs`): (1) adı "Error"
ile biten bir `uniffi::Error` tipi Kotlin'de otomatik "Exception" ile
değiştiriliyor — Rust/Swift `AppError`, Kotlin'de **`AppException`**; (2) düz
`uniffi::Enum` varyantları Kotlin'de **SCREAMING_SNAKE_CASE** (Swift'te
camelCase'ti) — Swift'in zaten bulduğu "hata PascalCase / enum camelCase"
asimetrisine üçüncü bir kural ekleniyor.

**Cue-transfer checksum'ları Swift'le birebir aynı** (`75001045577800` /
`34337591381145`, 50 000 cue, NEN-008 ile aynı fixture) — I5'in ("semantik
sonuçlar Swift ve Kotlin arasında aynı") doğrudan kanıtı. Coroutine iptali
gerçek Rust `JobHandle.cancel()`'ı tetikliyor (Rust tarafı `async fn` değil,
`JobHandleCoroutines.kt`'nin `suspendCancellableCoroutine` +
`invokeOnCancellation` sarmalayıcısı üzerinden) — I1/I2/I4 invariant'ları
Kotlin/JVM tarafında da (3/3 test) deterministik kanıtlandı. En büyük ölçüm
sapması: `checkpoint_every=1`'de dispatch-penceresi kaçağı Swift'te 0/800,
Kotlin/JVM'de 193/800 (I1'in kendisini etkilemiyor — yalnız "kullanıcı
iptale karar verdi" ile "cancel() fiilen çağrıldı" arasındaki gevşek
pencere, JVM thread scheduling + GC nedeniyle daha geniş). Typed-error eşleme
maliyeti Kotlin/JVM'de Swift'in p50'de ~5.5×, p95'te ~23×'ü — JIT ısınması +
GC, bug değil. Tüm sapmalar ve M10 etkileri task'ın kanıt kaydında "Metodolojik
sapmalar" tablosunda.

**Yan bulgu (ayrı backlog task'ına yönlendirildi, NEN-011 kapsamı değil):**
`scripts/doctor.sh`'ın `run_timeout` helper'ı `java -version`'ın (stderr'e
yazan) çıktısını kendi içindeki `2>/dev/null` ile siliyor — `doctor.sh`
raporunda "✓ JDK" satırının sürüm detayı hep boş kalıyor. Kozmetik (M1
gate'inin exit kodunu etkilemiyor); önceki hiçbir makinede gerçek bir JDK
çalışmadığı için şimdiye kadar ortaya çıkmamıştı.

`NEN-011`'in `adr:` alanı `[3]` → **`[28]`** olarak düzeltildi — NEN-006/
007/008/009/010 ile aynı gerekçe: ADR-0003 henüz `accepted` değil ve bu task
onu kararlaştırmıyor (sonuçları NEN-012 üzerinden ADR-0003'e girdi olacak);
gerçek dayanak spike-local FFI kapısını açan ADR-0028.

**Eski `NEN-005` kapanışı.** `.github/workflows/ci.yml` — macOS runner'da her
push/PR'da `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test
--workspace`, `cargo deny check` (yeni `core/deny.toml`), `scripts/test.sh`,
`task-index.sh --check`, `check-docs.sh` koşuyor; cargo registry+target
cache'leniyor, aynı branch'te eski koşu iptal ediliyor. Bu task için
`github.com/ynsemrekryl2/nen-player` adında private bir repo kuruldu (bu
repoda daha önce remote yoktu).

İlk gerçek koşu, yerelde tekrarlanamayan **3 gerçek kusur** buldu — üçü de bu
task kapsamında düzeltildi: (1) `rust-toolchain.toml`'ın 1.98.0 pin'i yalnız
cwd `core/` altındayken tetikleniyor, `--manifest-path` yetmiyor — fmt/clippy/
test adımları `working-directory: core` ile düzeltildi; (2)
`EmbarkStudios/cargo-deny-action` bir Docker container action, macOS
runner'da çalışmıyor — `taiki-e/install-action` + düz `cargo deny check`'e
geçildi; (3) **`scripts/doctor.sh`'ın `detect_jdk`'sında gerçek bir doğruluk
hatası bulundu**: diğer tüm `detect_*`'lerin aksine `command -v java`
guard'ı yoktu, `run_timeout`'un dayandığı perl `exec LIST` PATH'te java hiç
yokken sessizce exit 0 dönüyordu — yani "JDK yok" "JDK var" raporlanıyordu.
Yerel Mac'lerde bu gizli kaldı (CLT'li Mac'te JDK kurulu olmasa da
`/usr/bin/java` çalıştırılınca gerçekten hata veren bir stub'tır); GitHub'ın
macOS runner image'ında `/usr/bin/java` gerçek ve çalışan bir JDK olduğundan
kusur ilk kez CI'da ortaya çıktı. Düzeltildi; `doctor.test.sh`'ın S4 senaryosu
S7'nin (swift) kullandığı shadow-PATH tekniğiyle güncellendi.

DoD'un 4 kasıtlı ihlal kanıtı zincirleme commit'lerle toplandı (pipeline
sıralı olduğundan her biri bir öncekini düzeltip bir sonrakini bozuyor):
kasıtlı `cargo fmt` ihlali → 🔴 24s, clippy ihlali (`bool_comparison`) → 🔴
35s, bayat `INDEX.md` → 🔴 1m9s (bu koşu `scripts/test.sh`'ın CI'da fiilen
koştuğunu da doğruladı), `doctor.test.sh`'ta kasıtlı bozuk assertion → 🔴
45s, temiz duruma dönüş → 🟢. Yeşil koşu süresi: **3m58s** (soğuk cache, ilk
koşu) → **52s** (sıcak cache, sonraki koşular). Ayrıntı ve tüm run linkleri
task'ın kanıt kaydında.

Ayrıca bu task için `cargo-deny` bu makineye Homebrew ile kuruldu (0.20.2) —
toolchain tablosundaki B2 dışı "soon" eksiği kapandı.

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
| `cargo-deny` | ✅ 0.20.2 (Homebrew, NEN-005) | soon — **karşılandı** |
| JDK | ✅ OpenJDK 26.0.2.1 (Homebrew `openjdk`, NEN-011) | soon — **karşılandı** |
| `swift` | ✅ Apple Swift 6.4 | M1 (blocker) — NEN-007 testi ve NEN-008 harness'ı kullanıyor |
| Tam Xcode | ❌ yalnız `/Library/Developer/CommandLineTools` | M3 — **M1 blocker'ı değil** |
| libmpv | ❌ eksik | M3 — **M1 blocker'ı değil** |
| Gradle | ✅ 9.7.1 (Homebrew, yalnız wrapper bootstrap için — NEN-011) | hiçbir milestone'da blocker değil (wrapper) |
| Android SDK | ❌ eksik | M10 |

Rust `rustup` ile kuruldu (2026-08-24). Workspace `core/rust-toolchain.toml` ile
**1.98.0'a pinli** — M1 ölçümlerinin başka makinede karşılaştırılabilmesi için.

JDK, `brew install --cask temurin` sudo istediği için `brew install openjdk`
(formula) ile kuruldu; `/opt/homebrew/opt/openjdk/bin` `~/.zshrc`'ye PATH
eklendi (Homebrew'ün kendi "keg-only" uyarısının önerdiği sudo'suz yol).
Gradle yalnız üç `jvm-harness/`'ın kendi `gradlew` wrapper'ını üretmek için
geçici bootstrap amacıyla kuruldu — NEN-011 kapandıktan sonra hiçbir
geliştiricinin sistem Gradle'ı kurmasına gerek yok.

`doctor.sh M1` → **çıkış 0** · `doctor.sh` (parametresiz) →
**çıkış 0** (bilgilendirici). Kurulum komutları çıktıda; script **hiçbir şey
kurmaz** — bu, `scripts/tests/doctor.test.sh` S7 ile mekanik olarak kanıtlanıyor.

## Blocker'lar

| # | Blocker | Kimi durduruyor | Çözüm |
|---|---|---|---|
| ~~B1~~ | ~~Rust kurulu değil~~ | — | ✅ **çözüldü** 2026-08-24 — rustup, 1.98.0 |
| ~~B2~~ | ~~JDK yok~~ | — | ✅ **çözüldü** 2026-08-24 — `brew install openjdk`, NEN-011 |
| B3 | Tam Xcode + libmpv yok | M3; Swift testleri CommandLineTools'ta ek bayrak istiyor (`scripts/test-apple.sh` hallediyor) | App Store'dan Xcode + `brew install mpv` |
| ~~B4~~ | ~~`check-docs.test.sh` fixture'ı canlı repo durumuna bağlı~~ | — | ✅ **çözüldü** 2026-08-24 — `NEN-031` |

**Gerçek blocker kalmadı.** B3 sıradaki task'ları engellemiyor.

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

2026-08-25, tümü bu makinede çalıştırıldı (Apple M5 · arm64 · macOS 27.0
26A5416b · rustc/cargo 1.98.0 · Swift 6.4 · uniffi 0.32.0):

```
$ bash scripts/doctor.sh M1
SONUÇ: M1 için tüm blocker'lar hazır.            → exit 0

$ cargo test --manifest-path core/Cargo.toml --workspace
spike_cue_transfer 8 · spike_async_cancel 6 · spike_typed_errors 5 ·
spike_reverse_ffi 11 · nen-app 1 · nen-ffi 1 ·
nen-domain 10 (unit) + 9 (guard_redaction) = 19 ·
nen-subtitle 21 (unit) + 8 (fingerprint) + 10 (index) + 4 (encoding_golden) +
             7 (encoding_negative) + 4 (fuzz_smoke) + 3 (golden_valid) +
             4 (guard_error_debug) + 3 (lookup_parity) + 3 (malformed) +
             3 (webvtt_roundtrip) = 70 ·
nen-identity 125 (unit) + 7 (guard_evidence_debug) + 8 (nfo_and_container) +
             6 (os_hash_reference) + 4 (release_name_golden) +
             13 (resolution_layers) = 163
284 passed, 0 failed, 1 ignored                  → exit 0 (NEN-018 ile 121 → 284)
  ignored = lookup_bench (baseline; scripts/bench-cue-lookup.sh ile koşar)

$ cargo tree -p nen-domain --edges normal
nen-domain v0.1.0                                → tek düğüm, sıfır bağımlılık
$ cargo tree -p nen-subtitle --edges normal
nen-subtitle → encoding_rs → cfg-if
nen-subtitle → blake3 → ...
nen-subtitle → nen-domain                        → NEN-017 kenar EKLEMEDİ
$ cargo tree -p nen-identity --edges normal
nen-identity → nen-domain                        → tek kenar, NEN-018 dış
                                                   bağımlılık EKLEMEDİ

$ cargo clippy --workspace --all-targets --manifest-path core/Cargo.toml -- -D warnings
                                                  → exit 0, uyarı yok
$ cargo fmt --all --check --manifest-path core/Cargo.toml → exit 0
$ (cd core && cargo deny check)
  advisories ok · bans ok · licenses ok (blake3'ün Apache-2.0 kolu zaten
  izinliydi, deny.toml'a dokunulmadı) · sources ok → exit 0
  NEN-018 de deny.toml'a dokunmadı — yeni bağımlılık yok

$ cargo test -p nen-identity --manifest-path core/Cargo.toml
  release_name_golden  4 ✓  42 ad (8'i kasıtlı Unknown), .golden byte-eşit
  resolution_layers   13 ✓  ADR-0009 Karar 6 sırası + 17 URL + 12 yol korpusu
  os_hash_reference    6 ✓  bağımsız referans implementasyonuyla 7 boyutta
                            birebir; sınır vakaları ve tek-bit duyarlılığı
  nfo_and_container    8 ✓  4 sidecar fixture'ı; malformed olan hata değil
                            "tanınmadı" dönüyor
  guard_evidence_debug 7 ✓  10 yasak desen (K23 #1/#2/#3/#8) hiçbir çıktıda yok;
                            DerivedEvidence negatif kontrolü hepsini sızdırıyor

$ cargo test -p nen-subtitle --manifest-path core/Cargo.toml
  golden_valid   3 ✓   7 geçerli fixture, .golden snapshot'larıyla byte-eşit
  malformed      3 ✓   25 vaka, 17 varyantın hepsi kapsanıyor
  fuzz_smoke     4 ✓   16 281 vaka (781 trunc · 10 500 mut · 5 000 noise), 0.07 s
  guard_error_debug 4 ✓ hiçbir SrtError varyantı cue metni sızdırmıyor
  encoding_golden   4 ✓ 7 pozitif fixture, .decoded.golden'larıyla byte-eşit
  encoding_negative 7 ✓ boyut sınırı · undecodable · bidi/zero-width/kontrol
                        karakteri sanitization · EncodingError sızıntı yok
  lookup_parity     3 ✓ 10 000 seek + 10 000 aralık sorgusu + her cue sınırı,
                        14 dokümanlık korpusta lineer taramayla birebir

$ bash scripts/bench-cue-lookup.sh                → NEN-017 release baseline
$ bash scripts/bench-cue-lookup.sh --debug        → NEN-017 debug karşılaştırması
  50k cue: index p50 48–57 ns · lineer p50 15.6–17.6 µs → ~290–344×
  (3 koşunun aralığı; baseline'dır, eşik değil)

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
  bağımlılıksız. Guard test `core/crates/nen-domain/tests/guard_redaction.rs`
  (NEN-013 ile `Cue`/`SubtitleDocument` kapsamı eklendi).
- `core/crates/nen-domain/src/subtitle.rs` — NEN-013'ün subtitle değer tipleri:
  `CueId` · `TimeSpan` (invariant tipte: `start < end`) · `Cue` ·
  `SubtitleDocument`. `Cue` ve `SubtitleDocument` cue metni taşıdığı için
  `Debug` **elle yazılmış** (K23 #4).
- `core/crates/nen-subtitle/src/srt.rs` — NEN-013'ün strict SRT parser'ı ve 17
  varyantlı `SrtError`'ı. Crate `lib.rs`'inde panik lint kapısı
  (`unwrap_used`/`expect_used`/`panic`/`unreachable`/`indexing_slicing`,
  `cfg_attr(not(test))`). Testler: `golden_valid` · `malformed` · `fuzz_smoke`
  · `guard_error_debug`.
- `core/crates/nen-subtitle/src/webvtt.rs` — NEN-014'ün WebVTT writer'ı
  (`write(&SubtitleDocument) -> String`). `WEBVTT` başlığı, cue identifier +
  `HH:MM:SS.mmm` zaman satırı, `&`/`<`/`>` kaçışlı metin, BOM'suz. Test:
  `webvtt_roundtrip` (SRT → doc → WebVTT round-trip golden).
- `core/crates/nen-subtitle/src/encoding.rs` — NEN-015'in encoding/sanitization
  katmanı (ADR-0008), `srt::parse`'ın önünde çalışır. `decode(&[u8]) ->
  Result<String, EncodingError>`: BOM sniff (UTF-8/UTF-16LE/UTF-16BE) → sıkı
  UTF-8 denemesi → Windows-1254 fallback (tek legacy code page); `sanitize`
  kontrol karakteri/bidi override/zero-width temizliyor. `encoding_rs`'e
  bağımlı (workspace'in ilk gerçek dış bağımlılığı). Testler:
  `encoding_golden` · `encoding_negative` + 10 birim testi.
- `core/crates/nen-subtitle/src/fingerprint.rs` — NEN-016'nın timeline/source
  fingerprint'i (ADR-0007). `TimelineFingerprint::of`/`SourceFingerprint::of`
  `[u8; 32]` (`blake3`) döndürür; ikisi de `Display`/`Debug`'ı küçük harf hex
  olarak basar. `CueId` hiçbir hash'e dahil değil. `blake3`'e bağımlı
  (crate'in ikinci dış bağımlılığı, `encoding_rs`'ten sonra). 8 unit test.
- `core/crates/nen-identity/src/` — NEN-018'in dokuz modülü (ADR-0009):
  `os_hash.rs` (OSDb hash, I/O'suz `of(file_size, head, tail)`; `OsHash` `Debug`/
  `Display`'de `<redacted>`, gerçek değer yalnız `to_hex()`/`as_bytes()` ile) ·
  `release_name.rs` (`parse` → `ParsedName`, hata tipi **yok**, çözülemeyen ad
  `Unknown`) · `url_hints.rs` (path segmentleri; query/fragment/host tipe hiç
  girmez) · `dir_hints.rs` · `declared_name.rs` (RFC 6266/5987 + sanitization) ·
  `siblings.rs` (rapor eder, ezmez) · `nfo.rs` (Kodi/Plex sidecar, XML + tek
  satır URL) · `container.rs` (şekil + tag → identity; demuxer M3'te) ·
  `evidence.rs` (`MediaEvidence`, Karar 6 katman yürüyüşü, §6 aday üretimi).
  Crate'in `lib.rs`'inde `nen-subtitle`'ınkiyle aynı panik lint kapısı.
  **Yeni dış bağımlılık yok.**
- `core/crates/nen-domain/src/source.rs` — NEN-019'un kaynak değer tipleri
  (ADR-0010): dört `SubtitleSourceKind`, metadata-kimlikli `SubtitleSourceId`,
  normalize `LanguageTag`, redakte `SubtitleSource` ve iki dilli
  `SubtitlePreferences`. `nen-domain` sıfır bağımlılıklı kaldı.
- `core/crates/nen-catalog/src/` — NEN-019'un katalog, menü projeksiyonu ve
  otomatik seçim modülleri. Dedup upsert ile kimliğe göre; grup içi sıra ekleme
  sırası, dil grupları tercihlerden sonra tag sırası. Testler: 21 unit + 2
  golden + 2 negatif kontrollü `Debug` guard.
- `fixtures/catalog/` — ADR-0010 Karar 10'un tercihsiz ve `tr`/`en` tercihli
  yapısal menü golden'ları.
- `fixtures/media/` — NEN-018'in korpusu: `release-names.tsv` (42 ad, 8'i
  kasıtlı çözülemeyen) · `url-hints.tsv` (17 URL, token'lı query ve fragment
  dahil) · `dir-layouts.tsv` (12 yol) · her birinin `.golden`'ı ·
  `os-hash.golden` (7 sentetik boyut) · `nfo/` (4 sidecar, biri malformed).
  Golden'lar `UPDATE_GOLDEN=1 cargo test -p nen-identity` ile yenilenir.
- `core/crates/nen-subtitle/src/index.rs` — NEN-017'nin `CueIndex<'a>`'i:
  sıralı cue dizisi + monoton `max_end_prefix` artırımı üzerinde iki
  `partition_point`. `active_cues(at)` çakışan cue'ların **hepsini** doküman
  sırasında döndürür (yarı açık: `start_ms <= t < end_ms`); `active_cue(at)`
  ilkini döndüren kolaylık sarmalayıcısı; `cues_in(range)` pencere sorgusu.
  Yeni bağımlılık yok. Testler: 10 unit + `lookup_parity` (10 000 seek,
  lineer taramayla birebir) + `lookup_bench` (`#[ignore]`, baseline).
- `fixtures/subtitles/valid/` — 8 SRT fixture + 8 `.golden` (SRT parse)
  snapshot + 8 `.vtt` (WebVTT yazım, NEN-014) snapshot;
  `fixtures/subtitles/malformed/` — 25 fixture, her biri tek bozukluk;
  `fixtures/subtitles/encodings/` — 8 byte-precise fixture (NEN-015): 7
  pozitif + `.decoded.golden` snapshot'ları, 1 negatif
  (`undecodable-garbage.srt`). Golden biçimi (`valid/`): `cues\t<n>` başlığı +
  cue başına `id \t start_ms \t end_ms \t line_count \t escape'li metin`;
  `encodings/`'in golden'ı ise decode edilmiş tam UTF-8 metin (cue yapısı
  değil, decode doğruluğu kanıtlanıyor).
- `core/spikes/spike-cue-transfer/` — NEN-008'in ölçüm crate'i; kendi Swift
  harness'ı (`apple-harness/`) VE kendi Kotlin/JVM harness'ı (`jvm-harness/`,
  NEN-011 — Gradle wrapper tabanlı, `scripts/spike-cues-jvm.sh` üretir).
  **Ürün kodu değil, terfi etmez**; kendi FFI kapısını ADR-0028 sayesinde
  açıyor. Üretilen binding'ler commit edilmiyor.
- `core/spikes/spike-async-cancel/` — NEN-009'un ölçüm crate'i; Swift
  harness'ı hem CLI ölçüm hedefi hem swift-testing invariant test hedefi
  içeriyor (`apple-harness/Tests/`); Kotlin/JVM harness'ı (`jvm-harness/`,
  NEN-011) aynı invariant'ları `kotlin.test` ile ve bir
  `suspendCancellableCoroutine` sarmalayıcısıyla (coroutine iptali → gerçek
  `JobHandle.cancel()`) kanıtlıyor. Aynı ADR-0028 kapısı, aynı terfi yasağı.
- `core/spikes/spike-typed-errors/` — NEN-010'un ölçüm crate'i; `nen-domain`'e
  bağımlı (redaction yardımcıları), Swift harness'ı hem switch/redaction
  test hedefi (`apple-harness/Tests/`) hem baseline ölçüm hedefi
  (`apple-harness/Sources/`) içeriyor; Kotlin/JVM harness'ı (`jvm-harness/`,
  NEN-011) aynı ölçümü `AppException` (Kotlin'in "Error"→"Exception" son ek
  kuralı) ile tekrarlıyor. Aynı ADR-0028 kapısı, aynı terfi yasağı.
- `core/spikes/spike-reverse-ffi/` — NEN-029'un ölçüm crate'i; A (`ReverseEngine`,
  foreign `PlaybackObserver` trait'ini tekrar çağıran fake motor) ve B
  (`ForwardSession`, düz forward çağrılar) tek crate'te. Swift harness'ı hem
  baseline ölçüm hedefi (`apple-harness/Sources/`) hem üç ayrı swift-testing
  hedefi (`InvariantTests`/`OrderingTests`/`TypedErrorTests`) içeriyor. Aynı
  ADR-0028 kapısı, aynı terfi yasağı.
- `platforms/apple-shared/` — SwiftPM paketi (`Package.swift` + swift-testing
  test target'ı). Üretilen binding `generated/` altında ve **commit edilmiyor**.
- Diğer `platforms/*` dizinleri hâlâ boş iskelet.
- `.github/workflows/ci.yml` — NEN-005'in CI skeleton'ı; `core/deny.toml` —
  cargo-deny lisans/advisory/kaynak kapısı.
- Var olan: 6 ana doküman · 12 milestone dosyası · **10 accepted ADR**
  (0001, 0002, 0006, 0007, 0008, 0009, 0010, 0026, 0027, 0028) · 38 task ·
  14 script + 2 shell testi · `fixtures/subtitles|media|catalog/` dolu,
  `fixtures/providers/` hâlâ iskelet.
- Depo kökünde **`LICENSE` dosyası bilerek yok** — bkz. [`licensing.md`](licensing.md).
- Git: `main` branch. Remote: `github.com/ynsemrekryl2/nen-player` (private —
  NEN-005 ile kuruldu, CI'ın koşabilmesi için gerekliydi). **Bu dosya commit
  hash'i tutmaz** — commit geçmişi kanonik kayıttır ve elle tutulan hash
  satırı her kapanışta bayatlar (CLAUDE.md → "Commit politikası").

## Bu dosyayı kim günceller

Her task kapanışında (`/finish-task`) ve her milestone geçişinde.
`scripts/check-docs.sh` **denetim 8**, buradaki "Sıradaki READY" satırının
`tasks/INDEX.md` ile uyuşmasını mekanik olarak zorlar — bayatlarsa CI kırılır.
