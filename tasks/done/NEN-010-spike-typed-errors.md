---
id: NEN-010
title: Spike - typed error mapping
milestone: M1
size: S
state: done
closed: 2026-08-24
depends_on: [NEN-007]
blocks: [NEN-011]
adr: [28]
---

# NEN-010 — Spike: typed error mapping

## Sonuç

Rust'taki her hata varyantı Swift ve Kotlin tarafında, mesaj string'i parse
edilmeden ayrıştırılabilir.

## Kapsam

- Temsili hata enum'ı (parse hatası, ağ hatası, iptal, capability yok,
  validation başarısız)
- Rust enum → Swift `Error` / Kotlin `Exception` eşlemesi
- Payload'lı varyantların redaction kurallarına uyması

## YAPILMAYACAK

- Nihai hata taksonomisi → ADR-0005, M2'de gerçek hatalarla olgunlaşır
- Kullanıcıya gösterilecek hata metinleri / lokalizasyon

## Kanıt (DoD)

- [x] Swift tarafında her varyant `switch` ile ayrıştırılıyor (string parse yok)
- [x] Payload'lı varyantın `{:?}` çıktısı yasaklı desen içermiyor
- [x] Negatif: bilinmeyen varyant sessizce yutulmuyor

## Kanıt kaydı

**`adr:` alanı `[5]` → `[28]` olarak düzeltildi** — NEN-006/008/009 ile aynı
gerekçe: `docs/adr/` altında `0005-*.md` yok, bu task hata taksonomisini
kararlaştırmıyor (o M2'de ADR-0005 ile olgunlaşacak); task'ın gerçekte
dayandığı karar `core/spikes/` altında kendi FFI kapısını açmasına izin veren
ADR-0028.

### Crate ve harness

`core/spikes/spike-typed-errors/` — kendi Swift harness'ı
(`apple-harness/`), NEN-008/009 ile aynı desen. `AppError` beş varyant
taşıyan bir UniFFI **rich** error (`#[derive(uniffi::Error)]`,
`#[uniffi(flat_error)]` yok): alanlar Swift'e string'e düzleştirilmeden,
gerçek typed veri olarak geçiyor.

**Tasarım bulgusu (spike'ın asıl öğrettiği şey):** NEN-006'nın elle yazılmış
`Debug` garantisi yalnız Rust `{:?}` çıktısını korur — FFI'yı gerçekten
geçen bir hata için bu yetmez, çünkü Swift'in kendi `String(describing:)`
basımı Rust'ın `Debug`'ını hiç görmüyor. Bu yüzden `AppError`'ın alanları
inşa **öncesinde** güvenli hale getiriliyor (`make_parse_error` yalnız
`nen_domain::redact::extension()`'ı taşıyor, ham path'i asla; ağ hatası
yalnız `redact_host()`'tan geçmiş host'u taşıyor) — ham değer FFI teline hiç
çıkmıyor, dolayısıyla Rust `Debug`'ı, wire'ı ve Swift'in varsayılan basımını
aynı anda güvenli kılıyor. `AppError`'ın `Debug`'ı yine de elle yazıldı
(derive edilmedi) — K23'ün genel kuralı ve ileride ham bir alan eklenirse
sessiz regresyona karşı savunma.

**Yan bulgu:** bu uniffi sürümünde (0.32.0) `uniffi::Error` varyant adları
Swift'te **PascalCase** (`.Parse`, `.Network`) üretiyor, sıradan
`uniffi::Enum` varyantları ise **camelCase** (`.localAsr`) — iki derive
makrosu arasında tutarsız isimlendirme. M2'nin gerçek hata taksonomisi
yazılırken hatırlanmalı.

### Kanıt — Rust (`cargo test -p spike-typed-errors`)

```
running 5 tests
test tests::parse_error_debug_has_no_forbidden_pattern_and_keeps_the_extension ... ok
test tests::bypassing_the_sanitizing_constructor_does_leak ... ok
test tests::network_error_keeps_allowlisted_host ... ok
test tests::variants_are_distinguishable_without_string_parsing ... ok
test tests::network_error_debug_has_no_forbidden_pattern_and_redacts_unknown_host ... ok
test result: ok. 5 passed; 0 failed
```

`bypassing_the_sanitizing_constructor_does_leak` — **negatif kanıt**
(DoD #2'nin vacuous olmadığını kanıtlıyor, NEN-006'daki
`BadFixtureWithDerivedDebug` ile aynı teknik): sanitizing constructor
bypass edilip ham `PRIVATE_PATH` doğrudan `extension` alanına konursa,
elle yazılmış `Debug` bunu ayırt edemez ve **gerçekten sızdırır** — yani
üstteki no-leak testleri gerçek bir korumayı ölçüyor, kendiliğinden geçen
boş bir kontrol değil.

`cargo clippy -p spike-typed-errors --all-targets -- -D warnings` → çıkış 0,
uyarı yok. `cargo fmt --check` (yalnız bu crate) → çıkış 0. Workspace geneli
`cargo fmt --check` diğer, bu task'la ilgisiz dosyalarda (ör.
`nen-subtitle`, `spike-async-cancel`) önceden var olan format sapması
gösteriyor — `spike-typed-errors` bunun dışında, kendi diff'i boş.

### Kanıt — Swift (`bash scripts/spike-typed-errors.sh --swift-only`)

```
✔ Test "every variant switches without string parsing" passed
✔ Test "caught error's default Swift printing has no forbidden pattern" passed
✔ Test run with 2 tests in 1 suite passed
```

`every variant switches without string parsing` (DoD #1) — beş `trigger*`
fonksiyonu çağrılıp yakalanan `AppError` `default:` OLMADAN exhaustive bir
`switch` ile ayrıştırılıyor; her varyantın alanları (extension, host,
capability, field) string parse edilmeden okunuyor.

`caught error's default Swift printing has no forbidden pattern` (DoD #2,
FFI'yı gerçekten geçmiş kanıt) — `triggerParseError(path: PRIVATE_PATH, ...)`
ve `triggerNetworkError(url: MEDIA_URL_WITH_TOKEN, ...)` gerçekten
çağrılıp yakalanıyor; Swift'in **kendi** `String(describing:)` basımı (Rust
`Debug`'ı burada devrede değil) ne ham path'i ne de ham URL/host'u
içeriyor.

### Kanıt — M1 baseline (varyant sayısı · eşleme maliyeti, `M1-core-spike.md`)

```
## Varyant sayısı
`AppError`: 5 varyant (Parse, Network, Cancelled, CapabilityUnavailable, Validation).

## Eşleme maliyeti — throw → catch → switch round-trip
- repeats: 10000
| p50     | p95     | max       |
| 2.04 µs | 2.17 µs | 389.46 µs |
```

Baseline'dır, eşik değil (NEN-008/009 ile aynı kural). Bağlam: Apple M5 ·
macOS 27.0 (26A5416b) · release build (rust `--release`, swift `-c
release`).

### Kanıt — negatif kontrol (DoD #3, `bash scripts/spike-typed-errors.sh --negative-only`)

Gerçek crate'e hiç dokunmadan, tamamen ayrı bir scratch cargo+swift
projesinde aynı `uniffi::Error` makine mekanizmasıyla mekanik kanıt:

```
1/2  3 varyant + exhaustive switch → swift build (beklenen: başarı)
     ok — baseline yeşil
2/2  4. varyant eklendi, switch AYNI kaldı → swift build (beklenen: kırılma)
     ok — swift build çıkış 1, derleyici exhaustiveness ihlalini bildirdi:
       .../ExhaustiveSwitchTests.swift:6:9: error: switch must be exhaustive
▶ DoD #3 kanıtlandı: yeni bir varyant, switch güncellenmeden Swift
  derlemesini kırıyor — sessizce yutulmuyor.
```

Yani "bilinmeyen varyant sessizce yutulmuyor" iddiası çalışma zamanı
davranışı değil, **derleme zamanı garantisi** olarak kanıtlanmış oluyor —
`scripts/tests/doctor.test.sh`'ın shadow-PATH tekniğiyle aynı ruh: geçici
durum, mekanik kanıt, iz bırakmadan temizlik (scratch dizin `mktemp -d` +
`trap ... RETURN`, gerçek repoya hiç dokunulmadı). Ayrıca uniffi'ın kendi
wire-protokolü de aynı disipline sahip: üretilen `FfiConverterTypeAppError`
bilinmeyen bir discriminant'ta `default: throw
UniffiInternalError.unexpectedEnumCase` ile hata fırlatıyor, sessizce
yutmuyor.

### Doğrulama

```
$ cargo test --workspace --manifest-path core/Cargo.toml
31 passed, 0 failed (26 eski + 5 yeni)          → exit 0

$ cargo clippy -p spike-typed-errors --all-targets --manifest-path core/Cargo.toml -- -D warnings
                                                  → exit 0

$ bash scripts/spike-typed-errors.sh
  cargo test -p spike-typed-errors               → 5 passed
  swift test (SpikeTypedErrorsTests)             → 2 passed
  swift build + ölçüm                            → baseline yukarıda
  negatif kontrol (scratch)                      → DoD #3 kanıtlandı
```

I3 invariant'ı (`M1-core-spike.md`: "typed error string parse gerektirmiyor")
karşılandı.
