---
id: NEN-091
title: Strict local validation of a translated block
milestone: M5
size: M
state: done
closed: 2026-09-08
depends_on: [NEN-090]
blocks: [NEN-092]
adr: [16]
---

# NEN-091 — Strict local validation of a translated block

## Sonuç

Provider'ın çevrilmiş blok cevabı, structured output olsa bile, **yerel
doğrulama** geçmeden hiçbir yere teslim edilmiyor.

## Bağlam

Şartname §10: provider cevabı **untrusted**, structured output olsa bile
**local validation authoritative**. Doğrulama kümesi şartnamede sayılı: exact
cue count · yalnız izin verilen cue ID · unique cue ID · non-empty text ·
beklenen cue sırasına normalization.

Bu M5'in **yapısal doğruluk** ölçütünün kalbi (S3 kararı, 2026-09-08) ve
`tasks/README.md`'nin security/validation satırına girer — **negatif test
zorunlu**.

## Ön koşul — ADR-0016

`proposed` yazılır, kullanıcı onaylar, `accepted` olur. En az şunları
kararlaştırır: doğrulama hata taksonomisi · hangi ihlalin targeted repair ile
onarılabilir, hangisinin doğrudan blok başarısızlığı olduğu · sıra
normalizasyonunun sınırı (neyin düzeltme, neyin ihlal sayıldığı) · zaman ve ID
alanlarının provider tarafından **hiç** değiştirilemeyeceği.

## Kapsam

- Blok cevabı doğrulayıcısı ve tipli ihlal listesi
- Beklenen cue sırasına normalizasyon
- Cue ID / `TimeSpan` alanlarının girdiden aynen korunduğunun zorlanması

## YAPILMAYACAK

- Onarım denemesi — `NEN-092`
- Dilsel kalite değerlendirmesi (anlam, üslup) — M5'in ölçütü değil (S3)
- Belge seviyesinde birleştirme ve WebVTT — `NEN-094`

## Kanıt (DoD)

- [x] ADR-0016 `accepted`
- [x] Negatif: eksik cue (`count-mismatch` + `missing-cue`) reddediliyor
- [x] Negatif: fazladan/bloğa ait olmayan cue ID reddediliyor
- [x] Negatif: tekrar eden cue ID reddediliyor ve tüm kopyalar repair kümesine giriyor
- [x] Negatif: boş veya yalnız boşluktan oluşan metin reddediliyor
- [x] Negatif: değiştirilmiş cue ID unknown + missing olarak reddediliyor; provider zamanı taşıyamadığı için başarılı çıktıda kaynak `TimeSpan` aynen korunuyor
- [x] Unit: karışık sırayla gelen geçerli cevap normalize ediliyor, dış boşluk kırpılıyor ve reddedilmiyor
- [x] Guard: ihlal/başarı tipleri **cue metnini taşımıyor** (K23 #4) — negatif kontrolle

## Kanıt kaydı

`docs/adr/0016-translation-validation-and-repair.md` kabul edildi ve ADR-only
commit `94c7e10` origin/main'e gönderildi; GitHub Actions CI run
`34265789758` başarıyla tamamlandı.

`nen-translate::validation::validate_block` typed `TranslationResponse`'ı
yerel olarak doğruluyor. `ValidatedBlock` yalnız eksiksiz ve güvenilir çıktı
varsa üretilebiliyor; başarılı cue'lar kaynak `SubtitleDocument`'in
`TimeSpan`'ini aynen taşıyor. Cevap sırası kaynak output sırasına normalize
ediliyor ve yalnız dış Unicode boşlukları kırpılıyor. Hata raporu
`CountMismatch`, `MissingCue`, `DuplicateCue`, `UnknownCue` ve `EmptyText`
varyantlarını deterministik sırada taşıyor; repair ID'leri kaynak sırasını
koruyor. Yalnız fazladan bilinmeyen ID için repair kümesi boş kalıyor ve
full-block retry tüketicisine bırakılıyor. Ham structured-output parse hataları
M6 provider adapter kararına ertelendi.

Kanıt:

- `cargo test -p nen-translate`: **32 passed, 0 failed**; unit normalization/
  span preservation ve `validation_negative` testleri missing/count,
  unknown-extra, changed-ID, duplicate, empty-text ve repair routing'i geçti.
- K23 guard: `failure_and_success_debug_surfaces_hide_translated_text` hata ve
  başarı debug/display yüzeylerinde sentinel bulmadı; kasıtlı derive ikizi
  `the_guard_sentinel_would_be_visible_in_a_derived_debug_twin` sentinel'ı
  gerçekten sızdırdı.
- `cargo test --workspace`: **702 passed, 0 failed, 1 ignored**.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo deny check`: başarılı (advisories/bans/licenses/sources ok).
- `bash scripts/test.sh`: **4/4** test dosyası geçti; `bash scripts/check-docs.sh`:
  tüm 9 denetim geçti.

Yeni bağımlılık, fixture, provider, FFI veya platform değişikliği yok.
