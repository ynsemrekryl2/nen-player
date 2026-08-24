---
id: NEN-006
title: Log redaction helpers and guard test
milestone: M1
size: M
state: done
depends_on: [NEN-001, NEN-007]
blocks: []
adr: []
---

# NEN-006 — Log redaction helpers and guard test

## Sonuç

Gizli veri taşıyan tiplerin hata, log ve `Debug` çıktısı yasaklı bilgilerin
hiçbirini içermez ve bu, test ile mekanik olarak korunur.

## Kapsam

- `nen-domain` içinde redaction yardımcıları: `Redacted<T>`, güvenli türev
  yardımcıları (`extension()`, `size_class()`, host allowlist eşlemesi)
- Hassas tiplerde `#[derive(Debug)]` **yasağı**, elle yazılmış `Debug`
- `docs/security-policy.md` §1'deki 8 yasak için guard test: hassas tiplerin
  `{:?}` çıktısı yasaklı desen içermiyor
- Typed error payload'larının aynı kurala uyması

## YAPILMAYACAK

- Platform log sink adapter'ları → ilgili platform milestone'ları
- Telemetri/crash raporlama → S10 yanıtlanmadan tasarlanmaz

## Kanıt (DoD)

- [x] Guard test: URL, tam yol, cue metni, API key, private file ID içeren
      örnek tiplerin `{:?}` çıktısında yasaklı desen **yok**
- [x] Negatif: `derive(Debug)` eklenmiş bir tip guard testi **kırıyor**
      (testin gerçekten koruduğunun kanıtı)
- [x] Typed error varyantları payload'sız ayrıştırılabiliyor

## Kanıt kaydı

`nen-domain` içine bağımlılıksız bir `redact` modülü eklendi: `Redacted<T>`
(Debug/Display her zaman `<redacted>` basar), `extension()`, `size_class()`,
`redact_host()` — `core/crates/nen-domain/src/redact.rs`.

Guard test `core/crates/nen-domain/tests/guard_redaction.rs`'de
`docs/security-policy.md` §1'deki 8 yasaklı desenin (medya URL, token query
string, tam özel yol, cue metni, raw provider response, API key, OpenSubtitles
private file ID, özel hash/filename) hiçbiri, elle yazılmış `Debug`
kullanan örnek tiplerin (`MediaRefFixture`, `ProviderResponseFixture`,
`ApiKeyFixture`, `ErrorFixture`) `{:?}` çıktısında bulunmuyor.

**Negatif kanıt:** `BadFixtureWithDerivedDebug` (`#[derive(Debug)]` ile,
K23'ün yasakladığı şekilde) kasıtlı olarak API key'i sızdırıyor;
`derived_debug_on_a_sensitive_type_is_caught_by_the_pattern_check` testi bunu
doğruluyor — yani aynı desen kontrolü gerçek bir `derive(Debug)` hatasına
uygulansaydı yakalayacaktı (NEN-032'nin shadow-PATH kanıtıyla aynı biçim).

**Typed error / payload:** `ErrorFixture` enum'ının üç varyantı
(`ParseFailed`, `ProviderRejected`, `NotFound`) `classify()` ile yalnız
discriminant üzerinden, hiçbir string parse etmeden ayrıştırılıyor
(`error_variants_are_distinguishable_without_string_parsing`).

```
$ cargo test -p nen-domain --manifest-path core/Cargo.toml
running 4 tests (unit, redact::tests::*)
test result: ok. 4 passed; 0 failed

running 6 tests (tests/guard_redaction.rs)
test derived_debug_on_a_sensitive_type_is_caught_by_the_pattern_check ... ok
test error_variants_are_distinguishable_without_string_parsing ... ok
test error_payload_debug_has_no_forbidden_pattern ... ok
test provider_response_debug_has_no_forbidden_pattern ... ok
test media_ref_debug_has_no_forbidden_pattern ... ok
test api_key_debug_has_no_forbidden_pattern ... ok
test result: ok. 6 passed; 0 failed

$ cargo test --workspace --manifest-path core/Cargo.toml
→ exit 0, tüm crate'ler (nen-domain dahil) yeşil

$ cargo tree -p nen-domain --edges normal --manifest-path core/Cargo.toml
nen-domain v0.1.0                → tek düğüm, sıfır bağımlılık (korunuyor)

$ cargo clippy -p nen-domain --all-targets --manifest-path core/Cargo.toml -- -D warnings
→ exit 0, uyarı yok

$ cargo fmt --check -p nen-domain   (core/ içinden)
→ temiz

$ bash scripts/test.sh    → exit 0 (check-docs.test.sh + doctor.test.sh yeşil)
$ bash scripts/check-docs.sh   → exit 0 (bu kapanıştan sonra, STATUS.md
  güncellendikten sonra doğrulandı)
```

Ölçüm: Apple M5 · macOS 27.0 · rustc/cargo 1.98.0 · 2026-08-24.

**`adr:` alanı `[5]` → `[]` olarak düzeltildi.** ADR-0005 ("Typed error
taksonomisi ve redaction kuralları") henüz `docs/adr/` altında bir dosya
olarak yok — yalnız `adr/README.md`'nin "Planlanan" listesinde. `NEN-006`
bu ADR'yi **kararlaştırmıyor**: burada uygulanan kurallar zaten
`docs/security-policy.md` §1'de kabul edilmiş politika, task yalnız onu
koda döküyor. DECISIONS.md → "Task'ın `adr:` alanı, kararın verildiği
task'a yazılır — kullandığı task'a değil" ilkesiyle aynı düzeltme
(NEN-007/008/009'da uygulanan) burada da geçerli; `check-docs.sh` denetim
6 bunu mekanik olarak zorladı.
