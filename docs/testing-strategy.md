# Test Stratejisi

> **Test kanıtı olmayan task DONE değildir.** Her task dosyasının "Kanıt kaydı"
> bölümü gerçek çıktıyla doldurulur.

## Test katmanları

| Katman | Neyi doğrular | Nerede | Kanıt formatı |
|---|---|---|---|
| **Unit** | Tek fonksiyon/tip davranışı | crate içi `#[cfg(test)]` | Geçen test adı |
| **Golden** | Parse/format çıktısının byte düzeyinde sabitliği | `fixtures/` + snapshot | Fixture adı + snapshot dosyası |
| **Contract** | Bir portun **tüm** adapter'larının aynı sözleşmeyi sağladığı | `nen-ports` içinde paylaşılan kit | Aynı kitin fake + gerçek adapter'da geçmesi |
| **Integration** | Crate'ler arası akış (katalog → seçim → render) | `core/` workspace testleri | Senaryo adı + çıktı |
| **Security** | Reddedilmesi gerekenin reddedildiği | ilgili crate | Negatif test adı + reddetme sebebi |
| **Platform** | Adapter'ın gerçek OS API'siyle davranışı | `platforms/*/Tests` | Test adı + platform/sürüm |
| **Cihaz acceptance** | Gerçek cihazda uçtan uca kullanıcı senaryosu | elle, senaryo dosyasıyla | Adım listesi + ekran/video kaydı yolu |

## Kanıt formatı — task tipine göre

**Her task'ın benchmark veya ekran kaydına ihtiyacı yoktur.** Yanlış kanıt
talebi iki kötü sonuç üretir: ya sahte kanıt yazılır, ya task hiç kapanmaz.
Task'ın tipine uygun kanıt yeterlidir:

| Task tipi | Yeterli kanıt |
|---|---|
| domain / logic | unit veya contract test |
| serialization / format | golden fixture |
| **security / validation** | **negatif test — zorunlu** |
| performans riski taşıyan | benchmark (baseline raporu, bağlamıyla) |
| UI | screenshot **veya** kısa manuel checklist |
| lifecycle / handoff | gerçek cihaz veya manuel acceptance checklist |
| architecture spike | ölçüm raporu + ADR |
| dokümantasyon / tooling | link ve tutarlılık kontrolü, script çıktısı |

Korunan sertlik: **kanıtsız implementation task'ı `done` olamaz**, ve
güvenlik/validation task'larında negatif test tartışmasız zorunludur.

Örnek kayıtlar (biçim örneğidir, eşik değildir):

```
- `nen_subtitle::srt::tests::rejects_overlapping_cues` geçiyor
  (cargo test -p nen-subtitle — 42 passed, 0 failed)
- Benchmark (örnek): 50k cue, lookup p99 = 31 µs, lineer tarama 2.4 ms
  — release build, M4 Pro / macOS 27, fixture: 50 000 cue / 3.1 MB
- Cihaz: Nvidia Shield / Android 11 — evidence/M10/stremio-handoff.mp4
- Manuel checklist: docs/milestones/M3-macos-slice.md §Çıkış kriterleri, 5/5
```

Yeterli **değil**: "test edildi", "çalışıyor", "manuel doğrulandı".

## Deterministic fake kuralı

**Testler gerçek provider kredisi veya kotası kullanmaz.** İstisnası yoktur.

- Her provider portunun bir `mock` implementasyonu vardır (`nen-providers`).
- Mock deterministiktir: aynı girdi → aynı çıktı, rastgelelik yok, ağ yok.
- Gerçek provider cevapları `fixtures/providers/` altında **redakte edilerek**
  kaydedilir ve replay edilir. Kaydedilen cevaplarda API key ve private ID
  bulunmaz.
- Hata senaryoları (5xx, timeout, bozuk JSON, eksik cue, tekrar eden ID) mock ile
  üretilir — gerçek servisten beklenmez.

Validation'ın olgunluğu mock ile ölçülür: **provider'ın kötü davranışını taklit
edemiyorsak, validation'ı test etmiş sayılmayız.**

## `fixtures/` düzeni

```
fixtures/
├─ subtitles/
│  ├─ valid/          # temiz SRT örnekleri (kısa, telif-temiz, üretilmiş)
│  ├─ malformed/      # her biri TEK bir bozukluğu izole eder
│  └─ encodings/      # BOM, UTF-16LE/BE, CP1254, CP1252, karışık
├─ media/             # küçük üretilmiş klipler (çok track'li, bitmap sub'lı)
└─ providers/         # redakte edilmiş kayıtlı provider cevapları
```

**Kural:** fixture'lar telif-temiz olmalıdır — gerçek film altyazısı veya klip
repoya konmaz. Klipler `ffmpeg` ile sentetik üretilir; üretim komutu fixture'ın
yanında `.txt` olarak saklanır.

**Kural:** `malformed/` altındaki her dosya tek bir hatayı izole eder (eksik
zaman satırı, ters zaman aralığı, tekrar eden index, boş metin, kırık BOM…).
Böylece hangi typed error'ın döndüğü kesin test edilir.

## Contract test yaklaşımı

`PlaybackEngine` gibi çok adapter'lı portlar için test kiti **portun yanında**
yaşar ve her adapter tarafından çalıştırılır:

```
nen-ports/src/playback/contract.rs   → paylaşılan senaryolar
  ├─ fake adapter        (core testlerinde, hızlı)
  ├─ libmpv adapter      (macOS platform testinde)
  ├─ AVPlayer adapter    (M11)
  └─ Media3 adapter      (M10)
```

Yeni bir adapter eklendiğinde **yeni test yazılmaz** — mevcut kit çalıştırılır.
Kit geçmiyorsa adapter eksiktir, kit değiştirilmez.

Capability'si olmayan operasyonun typed error döndüğü de kitin parçasıdır.

**`HttpClient` ve `Persistence` de contract kiti alır.** Bu iki portun
implementasyonu bugün core-owned adaylar (rustls, SQLite + CAS), ama
`docs/architecture.md` platform zorunluluğu halinde native adapter'a izin
veriyor. Kit, güvenlik politikasının adapter değişse de aynı kalmasını mekanik
olarak zorlar:

- **HttpClient kiti:** approved liste dışı host reddi · redirect bütçesi aşımı ·
  boyut sınırı aşımı · archive içerik reddi · düz HTTP reddi · redaction
- **Persistence kiti:** atomik commit (yarım dosya asla görünmez) · iptal sonrası
  late commit reddi · cache identity bileşeni değişince eski artifact'in
  kullanılmaması · schema sürüm uyumsuzluğu davranışı

## Negatif test zorunluluğu

Güvenlik veya validation ile ilgili her task, **reddedilmesi gerekeni** test
etmek zorundadır (bkz. `docs/security-policy.md` §9). Yalnız mutlu yol test
edilmiş bir güvenlik task'ı `done` olamaz.

## Baseline ve invariant ayrımı

**Ölçmeden eşik konmaz.** M0'da yazılan `< 100 MB` ve `< 250 ms` gibi sayılar
hiçbir ölçüme dayanmıyordu; kaldırıldılar. Yerlerine iki ayrı kategori geçti.

**Baseline (pass/fail değil).** Raporlanır, karşılaştırılır, bütçe sonradan
ADR-0027 ile kabul edilir. Her baseline şu bağlamla birlikte kaydedilir:
fixture boyutu · cihaz · OS/toolchain · debug/release build. Bağlamsız sayı
kanıt sayılmaz.

**Invariant (pass/fail — gevşetilemez).** Bunlar performans değil, doğruluk ve
güvenlik taahhütleridir (K15, K20):

| # | Invariant |
|---|---|
| I1 | Cancellation sonrası late callback yok |
| I2 | Cancellation sonrası late commit yok |
| I3 | Typed error string parse gerektirmiyor |
| I4 | Resource/thread sızıntısı yok |
| I5 | Semantic sonuçlar Swift ve Kotlin arasında aynı |

| Ne | Kategori | Task |
|---|---|---|
| Cue lookup (50k cue) | Baseline: logaritmik davranış + lineer taramaya karşı tablo | NEN-017 |
| FFI cue erişimi (50k) | Baseline: süre, p50/p95, peak RSS, tam liste vs. pencereli | NEN-008 |
| Cancellation | Baseline: latency · **Invariant: I1, I2, I4** | NEN-009 |
| Reverse-FFI A/B | Baseline: thread dönüşü, event ordering, position frekansı · **Invariant: I1, I4** | NEN-029 |

## CI'da çalışanlar (M1'den itibaren)

```
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo deny check
bash scripts/task-index.sh --check
bash scripts/check-docs.sh
```

Platform ve cihaz testleri CI'da çalışmaz; kanıtları elle kaydedilir.
