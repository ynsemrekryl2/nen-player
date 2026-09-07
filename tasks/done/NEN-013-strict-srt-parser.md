---
id: NEN-013
title: Strict SRT parser
milestone: M2
size: M
state: done
closed: 2026-08-24
depends_on: [NEN-012]
blocks: [NEN-014, NEN-015, NEN-016, NEN-020, NEN-025]
adr: []
---

# NEN-013 — Strict SRT parser

## Sonuç

Geçerli SRT dosyaları doğru `SubtitleDocument`'a dönüşür; bozuk her dosya
**typed error** ile reddedilir, hiçbiri sessizce kabul edilmez.

## Kapsam

- Strict SRT parse: index, zaman satırı, çok satırlı metin, blok ayrımı
- Her bozukluk için ayrı typed error varyantı
- `fixtures/subtitles/valid/` ve `malformed/` korpusu (her dosya **tek** bozukluk)
- Fuzz smoke testi (panik yok, `unwrap` yok)

## YAPILMAYACAK

- Encoding tespiti → NEN-015 (bu task UTF-8 girdi varsayar)
- WebVTT yazımı → NEN-014
- Toleranslı/kurtarıcı parse — strict, bilinçli tercih

## Kanıt (DoD)

- [x] Geçerli korpus golden testleri geçiyor
- [x] ≥20 malformed vaka, her biri **beklenen typed error** varyantını döndürüyor
- [x] Fuzz smoke: rastgele/kesik girdide panik yok
- [x] Kod içinde `unwrap`/`expect` yok (lint kanıtı)

## Kanıt kaydı

Ortam: Apple M5 · arm64 · macOS 27.0 26A5416b · rustc/cargo 1.98.0 (workspace
`rust-toolchain.toml` ile pinli) · debug build.

### Ne yazıldı

| Dosya | İçerik |
|---|---|
| `core/crates/nen-domain/src/subtitle.rs` | `CueId` · `TimeSpan` (invariant tipte) · `Cue` · `SubtitleDocument` · `TimeSpanError` |
| `core/crates/nen-subtitle/src/srt.rs` | `parse(&str) -> Result<SubtitleDocument, SrtError>` · 17 varyantlı `SrtError` · `variant_name()` |
| `core/crates/nen-subtitle/src/lib.rs` | Panik lint kapısı (aşağıda) |
| `fixtures/subtitles/valid/` | 7 fixture + 7 `.golden` snapshot |
| `fixtures/subtitles/malformed/` | **25 fixture**, her biri tek bozukluk |
| `core/crates/nen-subtitle/tests/` | `golden_valid` · `malformed` · `fuzz_smoke` · `guard_error_debug` + `support/` |

`nen-domain` sıfır bağımlılıklı kaldı; `nen-subtitle` yalnız `nen-domain`'e
bağlı — ADR-0006'nın izin verdiği tam grafik:

```
$ cargo tree -p nen-domain --edges normal
nen-domain v0.1.0                       → tek düğüm

$ cargo tree -p nen-subtitle --edges normal
nen-subtitle v0.1.0
└── nen-domain v0.1.0                   → tek kenar
```

Yeni dış bağımlılık eklenmedi (fuzz üreteci crate içinde ~15 satırlık
xorshift64*), dolayısıyla `cargo deny check` → `advisories ok, bans ok,
licenses ok, sources ok`.

### DoD #1 — geçerli korpus golden testleri

```
$ cargo test -p nen-subtitle --test golden_valid
test the_valid_corpus_has_no_orphan_goldens ... ok
test parsing_is_deterministic_across_runs ... ok
test every_valid_fixture_matches_its_golden ... ok
test result: ok. 3 passed; 0 failed
```

Golden biçimi (`support::render_golden`): başlık satırı `cues\t<n>`, ardından
cue başına `id \t start_ms \t end_ms \t line_count \t metin` — metinde `\`,
`\n`, `\t` escape'li, böylece her cue tam olarak bir satır. `UPDATE_GOLDEN=1`
ile yeniden üretilir. NEN-014/015/016 aynı biçimi kullanacak.

Korpusun kanıtladıkları: `minimal` · `multiline` (satır yapısı korunuyor) ·
`crlf` (CRLF ve LF **aynı** dokümanı veriyor) · `hour-boundary`
(`99:59:59,999` = 359 999 999 ms, `MAX_TIMESTAMP_MS` ile birebir) ·
`overlapping-cues` (çakışma **kabul ediliyor**) · `trailing-blank-lines` ·
`unicode` (Türkçe + emoji).

### DoD #2 — malformed korpus

```
$ cargo test -p nen-subtitle --test malformed
test the_corpus_exercises_a_broad_spread_of_variants ... ok
test the_table_covers_the_whole_malformed_corpus ... ok
test every_malformed_fixture_returns_its_expected_variant ... ok
test result: ok. 3 passed; 0 failed
```

**25 vaka** (istenen ≥20) ve `SrtError`'ın **17 varyantının hepsi** en az bir
fixture'la kapsanıyor. Fixture → beklenen varyant tablosu `tests/malformed.rs`'te;
`the_table_covers_the_whole_malformed_corpus` iki yönlü denetliyor — diskte
olup tabloda olmayan da, tabloda olup diskte olmayan da testi kırar.

Tam eşleme:

| Fixture | Varyant | Fixture | Varyant |
|---|---|---|---|
| `empty-input` | `EmptyInput` | `period-instead-of-comma` | `MalformedTimestamp` |
| `leading-bom` | `LeadingBom` | `two-digit-millis` | `MalformedTimestamp` |
| `missing-index-line` | `MissingIndexLine` | `missing-hours` | `MalformedTimestamp` |
| `non-numeric-index` | `NonNumericIndex` | `minutes-out-of-range` | `TimestampFieldOutOfRange` |
| `index-overflow` | `IndexOverflow` | `seconds-out-of-range` | `TimestampFieldOutOfRange` |
| `duplicate-index` | `NonSequentialIndex` | `non-numeric-timestamp` | `NonNumericTimestampField` |
| `out-of-order-index` | `NonSequentialIndex` | `negative-timestamp` | `NonNumericTimestampField` |
| `skipped-index` | `NonSequentialIndex` | `end-before-start` | `EndBeforeStart` |
| `missing-time-line` | `MissingTimeLine` | `zero-duration` | `ZeroDuration` |
| `missing-arrow` | `MissingArrow` | `non-monotonic-cue` | `NonMonotonicCue` |
| `short-arrow` | `MalformedArrow` | `empty-text` | `EmptyText` |
| `long-arrow` | `MalformedArrow` | `trailing-content-on-time-line` | `TrailingContentOnTimeLine` |
| `doubled-arrow` | `MalformedArrow` | | |

### DoD #3 — fuzz smoke

```
$ cargo test -p nen-subtitle --test fuzz_smoke
test the_generator_is_deterministic ... ok
test truncating_a_valid_fixture_anywhere_never_panics ... ok
test random_noise_never_panics ... ok
test mutating_a_valid_fixture_never_panics ... ok
test result: ok. 4 passed; 0 failed; finished in 0.07s
```

`cargo-fuzz` **kullanılmadı** — nightly toolchain gerektirir, workspace
1.98.0'a pinli ve doctor.sh'a yeni bir blocker eklerdi. Yerine sabit tohumlu
(`0x4E45_4E30_3133`) deterministik smoke: **16 281 vaka**, 0.07 s, her
`cargo test` koşusunda ve dolayısıyla CI'da otomatik.

| Aile | Vaka | Neyi taklit ediyor |
|---|---|---|
| Truncation | 781 (her geçerli fixture'ın her karakter sınırı) | Yarım yazılmış / boyut sınırında kesilmiş indirme |
| Mutation | 10 500 (7 fixture × 1500) | Gramerle ilgili alfabeden ekle/sil/değiştir |
| Noise | 5 000 | Aynı alfabeden tamamen rastgele dize |

İddia yalnız "panik yok" değil: `Ok` dönen **her** vakada doküman kendi
invariant'larından geçiriliyor (index dizisi 1..n, `start < end`, zaman
geriye gitmiyor, boş metin satırı yok, boş doküman `Ok` olarak dönmüyor) —
"çöp sessizce kabul edildi" durumu da yakalanıyor.

### DoD #4 — `unwrap`/`expect` yok (lint kanıtı)

`core/crates/nen-subtitle/src/lib.rs`:

```rust
#![cfg_attr(not(test), deny(
    clippy::unwrap_used, clippy::expect_used,
    clippy::panic, clippy::unreachable, clippy::indexing_slicing,
))]
```

`not(test)` sayesinde `#[cfg(test)]` modülünde `unwrap` serbest kalıyor;
`tests/` zaten ayrı crate. CI'ın mevcut `cargo clippy -D warnings` adımı
kapıyı otomatik zorluyor — yeni CI adımı gerekmedi.

```
$ cargo clippy --workspace --all-targets -- -D warnings   → exit 0
$ cargo fmt --check                                        → exit 0
```

Parser'da hiç panik yolu yok: aritmetik `checked_*`, dizin erişimi
`first()`/`get()`, `Vec` dilimleme `get(2..).unwrap_or(&[])`.

### Negatif kanıtlar

Üçü de kanıtın boşta dönmediğini gösteriyor (NEN-006/NEN-010/NEN-032'nin
tekniği: geçici bozma, mekanik kanıt, izsiz temizlik).

**1 — lint kapısı gerçekten kırıyor.** `parse()`'ın ilk satırına kasıtlı bir
`unwrap()` konuldu:

```
error: used `unwrap()` on a `Result` value
   --> crates/nen-subtitle/src/srt.rs:194:35
    |
194 |     let _deliberate_defect: u32 = "1".parse().unwrap();
    = note: if this value is an `Err`, it will panic
error: could not compile `nen-subtitle` (lib) due to 1 previous error
```

Geri alındı; `cargo clippy -p nen-subtitle --all-targets -- -D warnings`
yeniden exit 0.

**2 — cue metni sızıntısı gerçekten yakalanıyor.** İki *kalıcı* negatif test:
`nen-domain/tests/guard_redaction.rs::derived_debug_over_cue_lines_is_caught_by_the_pattern_check`
(`#[derive(Debug)]` ile yazılmış bir cue tipi K23 #4 desenini gerçekten
sızdırıyor — `Cue`'nun elle yazılmış `Debug`'ının aksine) ve
`nen-subtitle/tests/guard_error_debug.rs::an_error_that_carries_cue_text_is_caught_by_the_same_check`
(metin taşıyan varsayımsal bir hata varyantı sızdırıyor).

**3 — fixture tablosu tamlık denetimi gerçekten kırıyor.** Tabloya
eklenmemiş bir `untabled-fixture.srt` korpusa konuldu:

```
test the_table_covers_the_whole_malformed_corpus ... FAILED
untabled-fixture.srt: fixture exists on disk but no expected variant is
declared in EXPECTED
test result: FAILED. 2 passed; 1 failed
```

Dosya silindi; 3/3 yeniden yeşil.

### Güvenlik (K23)

`Cue` ve `SubtitleDocument` cue metni taşıyor → `#[derive(Debug)]`
kullanılmadı; elle yazılmış `Debug` yalnız `line_count` / `cue_count` basıyor
(`docs/security-policy.md` §1). `SrtError`'ın **hiçbir varyantı** metin
taşımıyor — yalnız `block`, `line` ve sayısal beklenen/bulunan değerler, yani
K23'ün "Loglanabilecekler" listesindeki *"hata varyantı (payload'sız) · blok
indeksi"*; bu yüzden `derive(Debug)` burada güvenli.

`guard_error_debug.rs` bunu üç açıdan kanıtlıyor: (1) fixture kümesi **17
varyantın hepsine** ulaşıyor — varyant eklenip fixture eklenmezse test kırılır;
(2) hiçbir varyantın `Debug`'ı veya `Display`'i sentinel diyaloğu ya da onun
parçalarını içermiyor; (3) malformed korpusunun **kendi** metni de hiçbir
hata çıktısında görünmüyor.

`nen-domain` guard testi 6 → **9** doğrulamaya çıktı (`Cue`, `SubtitleDocument`
ve derived-Debug negatifi eklendi).

### Tasarım kararları ve gerekçeleri

| Karar | Gerekçe |
|---|---|
| Çakışan cue'lar **kabul** | Farklı konuşmacı SRT'lerinde meşru ve yaygın; reddetmek NEN-025'te kullanıcının geçerli dosyasını kırardı. Sıra yine zorlanıyor (`NonMonotonicCue`) — eşit başlangıç serbest, geriye gitmek değil |
| Index dizisi **tam** 1,2,3,… | Atlama/tekrar/sıra bozukluğu tek varyantla (`NonSequentialIndex{expected,found}`) ifade edilebiliyor; strict'in en test edilebilir hâli |
| `end == start` reddediliyor | Sıfır süreli cue render edilemez; `TimeSpan` tipte reddediyor, aşağı akış (NEN-016/017) invariant'ı yeniden doğrulamak zorunda değil |
| Zaman satırında fazla token reddediliyor | Bazı üreticilerin eklediği `X1:… Y1:…` konum alanları — "toleranslı parse yok" kuralının doğrudan sonucu |
| BOM **reddediliyor**, atlanmıyor | Encoding NEN-015'in işi; sessizce yutmak byte katmanının fark edilmeden geçmesi demek olurdu |
| Girdi `&str`, `&[u8]` değil | UTF-8 garantisi NEN-015'ten geliyor; kapsam sınırı tipte |
| `lines: Vec<String>` | NEN-014'ün "round-trip byte-eşit" çıkış kriteri satır yapısının korunmasını gerektiriyor |
| `SrtError::variant_name()` eklendi | K23'ün loglanmasına izin verdiği tek projeksiyon; test tablosunu da string-parse'sız yazılabilir kılıyor |

### Yan bulgu (bu task'ın kapsamı değil)

Depoda iki **untracked** yol var: `AGENTS.md` (CLAUDE.md'nin birebir kopyası)
ve `.agents/skills/` (`.claude/skills/`'in kopyası). Bu commit'e dahil
edilmedi; commit mi / `.gitignore` mı / symlink mi olacağı ayrı bir karar.

### Doğrulama (tümü bu makinede, bu ağaçta)

```
$ cargo test --workspace
nen-domain 10 (unit) + 9 (guard_redaction) ·
nen-subtitle 7 (unit) + 3 (golden_valid) + 3 (malformed) +
             4 (fuzz_smoke) + 4 (guard_error_debug) ·
nen-app 1 · nen-ffi 1 · spike_async_cancel 6 · spike_cue_transfer 8 ·
spike_reverse_ffi 11 · spike_typed_errors 5
72 passed, 0 failed                              → exit 0

$ cargo clippy --workspace --all-targets -- -D warnings   → exit 0
$ cargo fmt --check                                        → exit 0
$ cargo deny check      advisories/bans/licenses/sources ok → exit 0
$ cargo tree -p nen-domain --edges normal   → tek düğüm, sıfır bağımlılık
$ bash scripts/check-docs.sh                8/8 denetim geçti → exit 0
$ bash scripts/test.sh                      doctor + check-docs → exit 0
```

M1'in 42 testi 72'ye çıktı; hiçbiri kırılmadı.
