---
id: NEN-015
title: Encoding detection and sanitization
milestone: M2
size: M
state: done
depends_on: [NEN-013]
blocks: [NEN-025]
adr: [8]
---

# NEN-015 — Encoding detection and sanitization

## Sonuç

UTF-8 olmayan veya kontrol karakteri içeren altyazı dosyaları doğru okunur ve
zararlı içerikten arındırılır; okunamayanlar typed error ile reddedilir.

## Kapsam

- BOM tespiti (UTF-8, UTF-16LE/BE)
- Legacy code page tespiti (CP1254 Türkçe, CP1252) ve dönüşüm
- Sanitization: kontrol karakterleri, bidi override, sıfır genişlikli karakterler
- Aşırı boyut reddi
- `fixtures/subtitles/encodings/` korpusu

## YAPILMAYACAK

- Otomatik dil tahmini → NEN-020
- Dosya sistemi güvenlik kontrolleri (symlink, traversal) → NEN-025

## Kanıt (DoD)

- [x] Her encoding fixture'ı doğru metne çözülüyor (golden)
- [x] Negatif: bidi override / sıfır genişlikli karakter içeren girdi temizleniyor
- [x] Negatif: boyut sınırını aşan dosya okunmuyor, typed error dönüyor
- [x] Negatif: çözülemeyen encoding sessizce mojibake üretmiyor, hata veriyor

## Kanıt kaydı

**Ön koşul: ADR-0008 kabul edildi** (`docs/adr/0008-encoding-detection-and-sanitization.md`,
`proposed` → `accepted`, kullanıcı onayı bu task'ın kapanışında alındı). Karar:
`encoding_rs` bağımlılığı (WHATWG Encoding Standard implementasyonu), BOM'suz
durumda tek legacy fallback olarak **yalnız Windows-1254** (CP1252 otomatik
ayrımı yapılmıyor — bu NEN-020'nin dil tahmini kapsamına taşar), bidi
override/zero-width karakterlerin **silinmesi** (escape değil), 10 MiB boyut
sınırı. `encoding_rs`'in lisansı `(Apache-2.0 OR MIT) AND BSD-3-Clause` çıktı;
`core/deny.toml`'a `BSD-3-Clause` eklendi.

**Yeni modül:** `core/crates/nen-subtitle/src/encoding.rs` —
`decode(&[u8]) -> Result<String, EncodingError>` ve `sanitize(&str) -> String`.
BOM sniff sırası UTF-8 → UTF-16LE → UTF-16BE, sonra sıkı UTF-8 denemesi, sonra
Windows-1254 fallback'i. `EncodingError` (`TooLarge { size_class }` ·
`UndecodableBytes { declared }`) `SrtError`'la aynı disiplinde: hiçbir varyant
ham byte/metin taşımıyor (`size_class`, `nen_domain::redact::size_class`
üzerinden — tam boyut değil).

**Fixture korpusu** (`fixtures/subtitles/encodings/`, 8 dosya, byte-precise
üretildi): `utf8-bom.srt` · `utf16le-bom.srt` · `utf16be-bom.srt` ·
`cp1254-turkish-no-bom.srt` · `cp1252-no-bom.srt` ·
`bidi-override-injection.srt` · `zero-width-injection.srt` (7 pozitif, her
biri `.decoded.golden` snapshot'ıyla) · `undecodable-garbage.srt` (1 negatif —
UTF-16LE BOM + eşleşmeyen yüksek surrogate). Golden'lar `UPDATE_GOLDEN=1
cargo test -p nen-subtitle --test encoding_golden` ile üretildi, elle
doğrulandı (örn. CP1254 fixture'ı İ/ş/ü'yü doğru çözüyor: `c4b0`/`c59f`/`c3bc`
UTF-8 baytları).

**Test dosyaları:** `tests/encoding_golden.rs` (4 test — golden eşleşme,
determinizm, korpus/golden tutarlılık denetimleri) ve
`tests/encoding_negative.rs` (7 test — boyut sınırı, undecodable, bidi/
zero-width/kontrol karakteri sanitization, `EncodingError`'ın Debug/Display
çıktısında sentinel sızmadığı). `tests/support/mod.rs`'e `read_bytes()`
eklendi (mevcut `read()` yalnız UTF-8 varsayıyordu). `src/encoding.rs`
içinde 10 birim testi.

**Tam test çıktısı** (2026-08-24, Apple M5 · macOS 27.0 · rustc/cargo 1.98.0):

```
$ cargo test --workspace --manifest-path core/Cargo.toml
nen-app 1 · nen-domain 10 (unit) + 9 (guard_redaction) = 19 ·
nen-subtitle 21 (unit) + 4 (encoding_golden) + 7 (encoding_negative) +
             4 (fuzz_smoke) + 3 (golden_valid) + 4 (guard_error_debug) +
             3 (malformed) + 3 (webvtt_roundtrip) = 49 ·
spike_async_cancel 6 · spike_cue_transfer 8 · spike_reverse_ffi 11 ·
spike_typed_errors 5
100 passed, 0 failed                             → exit 0 (82 → 100)

$ cargo clippy --workspace --all-targets --manifest-path core/Cargo.toml -- -D warnings
                                                  → exit 0, uyarı yok
$ cargo fmt --all --check --manifest-path core/Cargo.toml → exit 0
$ cargo deny check --manifest-path core/Cargo.toml
  advisories ok · bans ok · licenses ok (BSD-3-Clause eklendi) · sources ok → exit 0

$ cargo tree -p nen-domain --edges normal --manifest-path core/Cargo.toml
nen-domain v0.1.0                                → tek düğüm, sıfır bağımlılık (değişmedi)
$ cargo tree -p nen-subtitle --edges normal --manifest-path core/Cargo.toml
nen-subtitle → encoding_rs → cfg-if
nen-subtitle → nen-domain                        → tek yeni dış kenar (ADR-0008)
```

**Negatif test kanıtı (K23, `EncodingError` Debug/Display sızıntısı yok):**
`no_error_variant_leaks_input_through_debug_or_display` — 44 karakterlik bir
sentinel dize, hem `TooLarge` hem `UndecodableBytes` varyantlarının `Debug`/
`Display` çıktısında aranıyor, hiçbirinde bulunmuyor (yapısal olarak da
imkânsız — iki varyant da yalnız `size_class`/`declared` taşıyor).
