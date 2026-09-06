# Mimari

> Bu dosya yaşayan mimaridir. Buradaki bir kural değişiyorsa önce ADR yazılır
> (bkz. `docs/adr/0001-adr-process.md`).

> **Karar statüsü — okumadan önce.** Bu belgedeki geri kalan **teknoloji
> adları**, ilgili ADR'ler kabul edilene kadar **adaydır**, kabul edilmiş
> karar değildir. **Rust artık aday değil** — shared core dili olarak
> [ADR-0002](adr/0002-core-language.md) ile kilitlendi (2026-08-24).
>
> | Aday | Rol | Kararı verecek |
> |---|---|---|
> | UniFFI | Swift/Kotlin binding | ADR-0003 |
> | SwiftUI | macOS UI | ADR-0011 dönemi |
> | Rust HTTP / rustls | gelecekteki paylaşılan HTTP adapter | ADR-0019 dönemi |
> | SQLite + content-addressed files | persistence adapter | ADR-0017 |
> | C ABI | Windows/Linux binding | ADR-0003 |
>
> **Aday olmayanlar:** port sınırları, capability modeli, politika sahipliği ve
> `nen-domain`'in I/O'suzluğu. Bunlar mimari yönün kendisidir ve spike sonucundan
> bağımsızdır. "Teknoloji adaydır" ifadesi "mimari açıktır" anlamına gelmez.

## Katmanlar

```
┌─────────────────────────────────────────────────────────────┐
│  Platform UI   (aday) SwiftUI · Compose · (Win/Linux TBD)    │  platform
│  lifecycle, dosya seçici, sandbox, secure storage, handoff   │
└───────────────────────────┬─────────────────────────────────┘
                            │  application API (binding adayı: UniFFI)
┌───────────────────────────▼─────────────────────────────────┐
│  Application / Use-Case katmanı        (shared core içinde)  │
│  PlaybackSession · CatalogService · TranslationOrchestrator  │
│  SyncService · SourceSelection                               │
├─────────────────────────────────────────────────────────────┤
│  Domain (saf, I/O'suz)                                       │
│  SubtitleDocument · Cue · MediaEvidence · MediaIdentity      │
│  SubtitleSource · ValidatedSubtitleArtifact · SyncProfile    │
│  CacheKey · Fingerprint · TypedError                         │
├─────────────────────────────────────────────────────────────┤
│  Ports (trait)                                               │
│  PlaybackEngine · SubtitleRenderer · SecureCredentialStore   │
│  MediaFileAccess · EmbeddedTrackExtractor · AudioSampleSource│
│  HttpClient · Persistence · Clock · LogSink                  │
├─────────────────────────────────────────────────────────────┤
│  Varsayılan adapter adayları                                 │
│  HTTP (aday: rustls) · persistence (aday: SQLite + CAS)      │
└───────────────────────────┬─────────────────────────────────┘
                            │  platform adapters
┌───────────────────────────▼─────────────────────────────────┐
│  libmpv · AVPlayer · Media3 · Keychain · Keystore · DPAPI    │
└─────────────────────────────────────────────────────────────┘
```

## Sınır kuralı

Bir yeteneğin core'da mı platformda mı olduğunun **tek testi**:

> Cihaz/OS API'sine dokunuyor mu?

- **Dokunuyorsa** → port + platform adapter.
  Playback · dosya erişimi ve sandbox · secure storage · audio decode ·
  subtitle renderer · Stremio handoff · lifecycle.
- **Dokunmuyorsa** → core.
  Parse · model · fingerprint · katalog · orchestration · validation ·
  cache identity · sync matematiği · hata modeli.

### Politika core'a, implementasyon adaya aittir

**HTTP güvenlik politikası ve persistence semantiği shared core'a aittir.**
Core'un sahibi olduğu politikalar:

- **HTTP / network:** approved host · bounded redirect · maksimum boyut ·
  retry politikası · archive reddi · privacy/redaction · response validation
- **Persistence:** artifact semantiği · cache identity · atomik commit ·
  schema/version davranışı · retention kuralları · migration gereksinimleri

`HttpClient` ve `Persistence` **portları korunur.** Varsayılan güçlü adaylar:
Rust HTTP/rustls adapter ve SQLite + content-addressed artifact store. Platform
zorunluluğu ortaya çıkarsa (ör. bir iOS/Android ağ veya depolama kısıtı) aynı
portlara platform-native adapter bağlanabilir — **politika değişmeden**.

ADR-0039 ile ilk uzak medya evidence adapter'ı macOS'ta Foundation
`URLSession` olarak kabul edilmiştir. Bu, Rust HTTP/rustls adayını silmez;
yalnız ilk platform tesliminin adapter'ını sabitler.

Bu esnekliğin bedeli, platform başına ayrışan güvenlik politikası riskidir. İki
kural bunu kapatır:

1. **Politika, adapter'ın içinde değil core'da veri olarak yaşar.** Approved
   host listesi, boyut ve redirect bütçeleri, retention süreleri bir *policy*
   nesnesidir; adapter onu tüketir, kendi kopyasını taşımaz.
2. **`HttpClient` ve `Persistence` de contract test kiti alır.** İleride
   bağlanacak herhangi bir native adapter, aynı güvenlik negatif testlerini
   geçmek zorundadır (bkz. `docs/testing-strategy.md`).

Platform ayrıca iki şeyi enjekte eder: **yazılabilir dizin yolu** (sandbox
kuralları böyle karşılanır) ve **secure storage'dan gelen credential**.

**Embedded text extraction porttur**, çünkü çıkarımı yapan zaten playback
motorudur. Core yalnız çıkan metni parse eder.

## Portlar

| Port | Kim implemente eder | Neden port |
|---|---|---|
| `PlaybackEngine` | libmpv / AVPlayer / Media3 adapter | Cihaz medya API'si. **Capability tabanlı** — bkz. aşağıda |
| `SubtitleRenderer` | Engine-native adapter (core'da, motorun enjeksiyon capability'sine delege eder) · ileride custom overlay | Çizim stratejisi değişebilir; çağrı yeri değişmesin (ADR-0013) |
| `SecureCredentialStore` | Keychain / Keystore / CredMan / Secret Service | OS güvenlik API'si |
| `MediaFileAccess` | Platform dosya seçici + sandbox/bookmark | İzin modeli platforma özgü |
| `EmbeddedTrackExtractor` | Playback motoru | Çıkarımı motor yapar |
| `AudioSampleSource` | Platform decoder | M8 audio auto-sync için |
| `HttpClient` | Core adapter (aday: rustls); gerekirse native | Politika core'da, implementasyon adapter'da |
| `Persistence` | Core adapter (aday: SQLite + CAS) | Semantik core'da; yol platformdan enjekte edilir |
| `Clock` | Core adapter / test fake | Deterministik test |
| `LogSink` | Platform log sistemi | Redaction core'da uygulanır, yazma platformda |

## Capability modeli

`PlaybackEngine` bir yetenek kümesi bildirir. Application katmanı **motor adına
göre asla dallanmaz**:

```
// YANLIŞ
if engine.name == "mpv" { ... }

// DOĞRU
if engine.capabilities.contains(.externalSubtitleInjection) { ... }
```

Capability'si olmayan bir operasyon çağrılırsa **typed error** döner, panik veya
sessiz no-op olmaz.

Kullanıcı motor adını görmez ve motor seçmez (şartname §4).

### Altyazı çizimi — engine-native, ayrı port (ADR-0013)

`SubtitleRenderer` kendi trait'i, kendi capability kümesi ve kendi contract
kitiyle `nen-ports` içinde durur; ilk adapter'ı **core'da** yaşar ve
`PlaybackEngine`'in `ExternalSubtitleInjection` capability'sine delege eder.
Belge motora WebVTT olarak `memory://` üzerinden geçer, diske yazılmaz.

Bunun iki sonucu var: platform tarafında yeni bir callback yüzeyi açılmıyor —
bir motor `inject_subtitle`'ı implemente edince renderer'ı hazır oluyor — ve
M7'nin manuel sync overlay'i çağrı yerlerine dokunmadan ikinci adapter olarak
girebiliyor. Hangi satırın nasıl gösterileceği (gömülü track mi, kullanıcı
dosyası mı) kabuğun değil **core session'ın** kararıdır; altyazı diyaloğu
FFI'dan kabuğa hiç geçmez.

### Ownership yönü — core-owned session (ADR-0026)

Merkezi Rust application-session, platform playback/renderer adapter'larını
**reverse callback** ile yönetir — `NEN-029`'un fake-adapter ölçümüyle
doğrulandı, `ADR-0026` ile kabul edildi. Karşılaştırılan iki yönden (A:
core-owned/reverse callback, B: shell-owned/forward call) **A** seçildi:
ölçülen maliyeti (60 Hz'de bile ~5 ms/sn mertebesinde) hiçbir makul UI
bütçesini zorlamıyor, I1/I4 dahil hiçbir invariant ihlal edilmedi.

`PlaybackEngine` ve `SubtitleRenderer` portları değişmedi — A/B ayrımı yalnız
session'ın sahibini etkiliyordu.

**ADR-0026'nın NEN-021'e bıraktığı iki ölçülmüş risk `ADR-0011` ile kapandı**
(NEN-021, `core/crates/nen-ports/src/playback/`):

- **Backpressure/backgrounding** → **bounded kuyruk + sınıfa göre coalescing.**
  `PositionChanged` tek bekleyen örnek tutuyor (mutlak değer, en yenisi doğru
  olan); state değişimi, seek-complete, hata gibi kritik olaylar sıra koruyor ve
  sessizce düşmüyor. Taşma olursa kuyruk `EventsLost { dropped }` ile kapanıyor
  ve tüketici resync ediyor — akış ya eksiksiz ya da eksik olduğunu söylüyor.
- **Reentrancy disiplini** → **yasak.** Bir callback içinden aynı thread'den
  yapılan senkron port çağrısı deadlock'a girmiyor, `ReentrantCall` ile hemen
  dönüyor. Kural adapter'da değil portta: teslimat sırasında thread'i işaretleyen
  `deliver_all` tek yerde duruyor, hiçbir adapter unutamıyor.

Tam ölçüm tablosu ve gerekçe: `docs/adr/0026-playback-renderer-ownership.md`;
kontratın kendisi `docs/adr/0011-playback-port-contract.md`.

## FFI kontratı

M1 spike'ının doğrulayacağı beş madde. Ölçüm **baseline'ları ve invariant'ları**
`docs/milestones/M1-core-spike.md` içinde — bu tablodaki maddeler kontrat, sayılar
değil.

| Konu | Kontrat |
|---|---|
| **Binding** | *Aday:* UniFFI (Swift + Kotlin tek tanımdan). Windows/Linux için *değerlendirme adayı* C ABI + generated header. `nen-ffi` **tek dış kapıdır**; başka crate FFI yüzeyi açmaz |
| **Async** | Core `async`; Swift'te `async/await`, Kotlin'de `suspend` |
| **Progress** | Callback interface: `onProgress(jobId, phase, done, total)`. **Cue içeriği FFI sınırından geçmez** — ilerleme yalnız sayıdır (redaction gereği) |
| **Cancellation** | `JobHandle.cancel()` → checkpoint noktalarında kooperatif iptal. **İptalden sonra hiçbir callback gelmez, late commit yasaktır** |
| **Typed error** | Rust enum → Swift `Error` / Kotlin `Exception` hiyerarşisi. String mesaj parse edilmez. Hata payload'ı redaction kurallarına uyar |
| **Büyük listeler** | 50k cue tam liste olarak FFI'dan geçmez. UI'a **pencere/handle** verilir: `cues(range:)`, `activeCue(at:)`. Tam liste yalnız render adapter'ına zorunluysa geçer |

## Ana veri akışı

```
handoff / dosya seçimi
  → PlaybackSession.load()              [playback HEMEN başlar]
  → MediaEvidence toplanır              [async, playback'i bloklamaz]
  → CatalogService
      ├─ embedded track metadata        (lazy: metin çıkarımı yok)
      ├─ sidecar tarama (aynı basename)
      └─ OpenSubtitles sorgusu          (lazy: indirme yok)
  → SubtitleSourceCatalog               (gruplu, dedup'lu)
  → kullanıcı source seçer  ──────────► SubtitleRenderer
                            └─────────► AI çeviri komutunun KAYNAĞI olur
  → kullanıcı açıkça "AI ile çevir" der
      → TranslationOrchestrator(job)
      → context analizi
      → overlapping blocklar (40 / overlap 6)
      → her blok: strict validation → checkpoint
      → TÜM belge doğrulandı
      → atomik artifact commit
  → artifact hedef dil grubuna eklenir  (zorla geçiş YOK)
```

**Değişmez:** hiçbir ok playback'i bekletmez. Subtitle tarafındaki hata yalnız
ilgili kaynağı hatalı işaretler.

## Crate sınırları

> Bu tablo **ADR-0006** ile kabul edildi ve `core/Cargo.toml` altında uygulandı
> (NEN-007). Grafiğin belgeye uygunluğu `cargo tree` / `cargo metadata` ile
> mekanik olarak doğrulanabilir.

| Crate | Sorumluluk | Bağımlı olabileceği |
|---|---|---|
| `nen-domain` | Saf model, I/O yok | — |
| `nen-subtitle` | SRT/WebVTT parse-write, encoding, timeline, dil tespiti | domain |
| `nen-identity` | Evidence, hash, release-name parse | domain |
| `nen-catalog` | Source catalog, gruplama, dedup | domain, subtitle, identity |
| `nen-translate` | Blok pipeline, validation, checkpoint | domain, subtitle, ports |
| `nen-sync` | Offset, drift, piecewise, SyncProfile | domain, subtitle |
| `nen-persist` | persistence adapter (aday: SQLite + CAS) | domain, ports |
| `nen-ports` | Trait tanımları + contract test kitleri | domain |
| `nen-providers` | mock · OpenAI · OpenRouter · OpenSubtitles | domain, ports |
| `nen-app` | Use-case katmanı | hepsi |
| `nen-ffi` | FFI yüzeyi (aday: UniFFI) — **tek dış kapı** | app |

Ok yönü tek yönlüdür; `nen-domain` hiçbir şeye bağlı değildir.
