# Mimari

> Bu dosya yaşayan mimaridir. Buradaki bir kural değişiyorsa önce ADR yazılır
> (bkz. `docs/adr/0001-adr-process.md`).

> **Karar statüsü — okumadan önce.** Bu belgedeki **teknoloji adları**, M1
> spike'ı ve ADR-0002/0003 tamamlanana kadar **adaydır**, kabul edilmiş karar
> değildir:
>
> | Aday | Rol | Kararı verecek |
> |---|---|---|
> | Rust | shared core dili | ADR-0002 (NEN-012) |
> | UniFFI | Swift/Kotlin binding | ADR-0003 |
> | SwiftUI | macOS UI | ADR-0011 dönemi |
> | Rust HTTP / rustls | paylaşılan HTTP adapter | ADR-0019 dönemi |
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
| `SubtitleRenderer` | libmpv / Apple timed-text / Media3 / custom overlay | Ekrana çizim platforma ait |
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

### Ownership yönü — spike bekliyor

> **Bu belge tercih edilen yönü gösterir, doğrulanmış yönü değil.** Tercih:
> merkezi Rust application-session, platform playback/renderer adapter'larını
> **reverse callback** ile yönetir. Rust core'un Swift/Kotlin adapter'larını bu
> şekilde geri arayabildiği **henüz ölçülmedi**.
>
> Karşılaştırılan iki yön (NEN-029 → ADR-0026):
>
> - **A —** Core, `PlaybackEngine`/`SubtitleRenderer` callback interface'lerini
>   çağırır ve session'ın sahibidir. *(şu anki tercih)*
> - **B —** Platform shell motorun sahibidir; core'a coarse-grained state,
>   position ve track snapshot gönderir.
>
> `PlaybackEngine` ve `SubtitleRenderer` portları **her iki sonuçta da kalır**;
> değişebilecek olan yalnızca session'ın sahibidir. `NEN-021` (port contract),
> ADR-0026 kabul edilmeden başlamaz.

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
| `nen-subtitle` | SRT/WebVTT parse-write, encoding, timeline | domain |
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
