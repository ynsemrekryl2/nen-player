---
id: NEN-090
title: Translation provider port and deterministic mock
milestone: M5
size: M
state: done
closed: 2026-09-08
depends_on: [NEN-089]
blocks: [NEN-091]
adr: []
---

# NEN-090 — Translation provider port and deterministic mock

## Sonuç

Bir çeviri sağlayıcısı `nen-ports` içinde tanımlı tek bir port arkasında duruyor
ve deterministic mock bu portun contract kitini geçiyor.

## Bağlam

M5 **yalnız mock provider** ile tamamlanır; gerçek sağlayıcılar M6'da
(`docs/milestones/M5-translation-core.md` → Kapsam dışı). Kural 8 gereği testler
gerçek provider kredisi kullanamaz, dolayısıyla mock isteğe bağlı bir kolaylık
değil, M5'in tek çalışan implementasyonudur.

`docs/architecture.md` → Portlar tablosunda çeviri sağlayıcısı için bir satır
**yok**; bu task onu ekler. Emsal: `nen-ports/src/playback` ve
`nen-ports/src/renderer` — trait + paylaşılan contract kiti + fake, aynı dizinde.

## Kapsam

- `nen-ports` içinde `TranslationProvider` trait'i ve paylaşılan contract kiti
- `nen-providers` içinde deterministic mock (aynı girdi → aynı çıktı, ağ yok)
- İptal ve ilerleme sözleşmesi — ADR-0004'ün late-commit yasağıyla uyumlu
- Transient hata sınıfının porta girmesi (bounded retry'ın **politikası** core'da)
- `docs/architecture.md` Portlar tablosuna satır eklenmesi

## YAPILMAYACAK

- OpenAI / OpenRouter / gerçek HTTP — M6 (`ADR-0019`)
- Capability preflight — M6
- Cevabın doğrulanması — `NEN-091`; port cevabı **untrusted** teslim eder
- Secure credential storage — M6 (`ADR-0020`)

## Kanıt (DoD)

- [x] Contract kiti mock adapter'da geçiyor (test adı + `cargo test -p nen-providers` çıktısı)
- [x] Unit: aynı istek iki kez çağrıldığında mock birebir aynı cevabı veriyor
- [x] Negatif: iptal edilen çağrı **sonuç teslim etmiyor** (late commit yok)
- [x] Negatif: kit, bozuk/eksik cevap veren kasıtlı bir `Defect` adapter'ında kırmızıya dönüyor — kit sağır değil
- [x] Guard: provider isteği ve cevabı hiçbir log yüzeyine düşmüyor (K23 #4, #5)

## Kanıt kaydı

`ADR-0004` önce planın kullanıcı onayıyla `accepted` oldu ve ayrı
`31619b8` commit'i olarak push edildi; GitHub Actions CI `34255180527` tamamen
yeşil tamamlandı. Port senkron/object-safe ve runtime'sız kaldı; caller-owned
`TranslationCall` aynı kilit altında cancellation, monoton progress ve sonuç
teslimini kapatıyor. Provider DTO'ları yalnız `CueId + text` taşıyor;
`TimeSpan` porta girmiyor. `Cancelled`/`Transient`/`Permanent` hata sınıfları
payload'sız; retry politikası eklenmedi.

**Contract + deterministic mock:**

- `mock_passes_the_shared_contract_kit` — ortak kit
  `MockTranslationProvider` üzerinde geçti.
- `same_request_has_the_same_identity_response_and_progress` — aynı isteğin
  iki çağrısı provider/model kimliği, cevap ve progress dizisinde birebir
  eşit; çağrı sayacı `2`.
- `contract_kit_goes_red_for_a_missing_cue_defect` — iki cue beklenen istekte
  yalnız ilk cue'yu döndüren kasıtlı adapter
  `ContractViolation::ResponseCueIds` üretti; kit eksik cevaba sağır değil.

**Cancellation negatif kanıtı:**

`cancelling_an_in_flight_call_delivers_no_late_result_or_progress`, mock'u
`MockCallGate` bariyerinde bekletti; `cancel()` döndükten sonra bariyeri açtı.
Worker `Err(Cancelled)` döndü ve progress sayısı iptal anındaki değerde kaldı:
geç sonuç ve callback sayısı **0**. Ek olarak
`a_panicking_progress_sink_poison_closes_the_gate`, callback paniğinin kilidi
zehirlemesinden sonra `checkpoint()`in `Cancelled` döndüğünü kanıtladı;
gate fail-closed.

**K23 guard:** `request_response_and_errors_never_print_provider_payloads` ve
`sensitive_values_are_shape_only_in_debug_output`, istek/cevap/cue/context/
provider kimliği sentinel'larının `Debug`/`Display` çıktılarında bulunmadığını
doğruladı. Negatif kontrol `derived_debug_would_really_expose_the_guard_sentinel`,
aynı sentinel'ı `#[derive(Debug)]` kullanan kasıtlı ikizde buldu.

**Doğrulama:**

```text
cargo test -p nen-ports              106 passed, 0 failed
cargo test -p nen-providers           18 passed, 0 failed
cargo test --workspace               695 passed, 0 failed, 1 ignored
cargo fmt --all --check              exit 0
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo deny check                     advisories/bans/licenses/sources ok
bash scripts/test.sh                 4/4 shell test files passed
bash scripts/check-docs.sh           exit 0
```

Yeni dış bağımlılık veya ağ çağrısı yok. Değişiklik `nen-ports`,
`nen-providers` ve ilgili mimari/task durum belgeleriyle sınırlı; gerçek
provider, validation/repair, FFI ve platform kabuğu dokunulmadı.
