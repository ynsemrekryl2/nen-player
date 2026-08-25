---
id: NEN-020
title: Subtitle language detection with confidence threshold
milestone: M2
size: S
state: done
depends_on: [NEN-013]
blocks: []
adr: [10, 29]
---

# NEN-020 — Subtitle language detection with confidence threshold

## Sonuç

Altyazı metninden dil tespit edilir; güven eşiğini geçemeyen kaynak yanlış bir
dile atanmak yerine `Dil Belirsiz` grubuna düşer.

## Kapsam

- Metin tabanlı dil tespiti
- Güven eşiği ve eşik altı davranışı
- Metadata'da dil etiketi varsa onun önceliği ve çelişki durumu
- Çok dilli fixture seti

## YAPILMAYACAK

- Otomatik çeviri tetikleme — dil tespiti çeviri **başlatmaz**
- Ağ tabanlı dil servisi

## Kanıt (DoD)

- [x] Çok dilli fixture'larda doğru dil tespiti (golden)
- [x] Kısa/karışık metinde eşik altı → `Dil Belirsiz`
- [x] Metadata etiketi ile tespit çeliştiğinde tanımlı davranış test edildi

## Kanıt kaydı

### DoD 1 — Çok dilli golden

- `fixtures/subtitles/languages/` telif-temiz, üretilmiş 9 belgeli korpus:
  Arabic → `ar`, English → `en`, French → `fr`, German → `de`, Japanese →
  `ja`, Russian → `ru`, Spanish → `es`, Turkish → `tr`; kısa/karışık belge →
  `unknown`. Dört yazı sistemi var: Latin, Kiril, Arap, Japon.
- `cargo test -p nen-subtitle --test language_detection_golden` → **2 passed,
  0 failed**. `every_language_fixture_matches_its_golden` her SRT'yi strict
  parse edip `.language.golden` sonucuyla byte-eşit karşılaştırıyor;
  `language_corpus_has_no_orphan_goldens` korpus/golden bütünlüğünü koruyor.
- İlk üç cümlelik fixture denemesi ölçüm olarak kullanıldı: English `0.8374`,
  Arabic `0.7494`; Russian yanlış fakat düşük güvenli Bulgarian `0.0586`
  döndürdü. Eşik gevşetilmedi. Belgeler gerçek subtitle örneğine daha yakın
  sekiz farklı cümleye çıkarılınca üçünün doğru dil güveni de `1.0` oldu.
  Bu ara sayılar acceptance eşiği değil, fixture tasarımının nedenidir.

### DoD 2 — Güven eşiği ve `Dil Belirsiz`

- `language::tests::threshold_is_strict_and_unknown_is_safe_fallback`:
  `0.90` tam sınırda `Unknown`, sınırın üstünde `Text`.
- `short-mixed.srt` (`Okay` / `Merci` / `Tamam`) detector'ın düşük güvenli
  Tagalog adayını kabul etmiyor; golden sonucu `unknown`.
- `language_resolution::short_mixed_text_is_unknown_without_metadata_and_does_not_conflict_with_it`
  aynı belgenin metadata yokken `Unknown`, metadata varken metadata dili
  olduğunu ve zayıf adayın conflict üretmediğini doğruluyor.

### DoD 3 — Metadata önceliği ve çelişki

- `language_resolution::conflicting_metadata_wins_and_exposes_reliable_text_candidate`:
  English belge + `tr` metadata → nihai `tr`; `en` adayı ve `> 0.90` güveni
  typed conflict içinde korunuyor.
- `language_resolution::matching_primary_preserves_metadata_region_without_conflict`:
  English belge + `en-us` metadata → `en-us`; primary aynı olduğu için conflict
  yok ve region kaybolmuyor.
- Unit sınır testleri ayrıca eşik altı farklı metnin metadata conflict'i
  üretmediğini kanıtlıyor.

### Kapsam ve güvenlik kanıtı

- Whatlang'ın 70 dilinin tamamı exhaustive `Lang` eşlemesinden kanonik,
  birbirinden farklı ve geçerli iki harfli `LanguageTag` alıyor; yeni enum
  varyantı eşlenmeden derleme geçemez.
- `language_results_never_retain_or_debug_subtitle_dialogue`, K23 özel marker'lı
  cue metninin `LanguageResolution` `Debug` çıktısına girmediğini doğruluyor.
- `cargo tree -p nen-domain --depth 1` → yalnız `nen-domain`; sıfır bağımlılık
  değişmedi. `nen-subtitle` doğrudan yeni dış bağımlılık olarak yalnız
  `whatlang 0.18.0` aldı.
- `cargo deny check` → advisories **ok**, bans **ok**, licenses **ok**, sources
  **ok**. Whatlang'ın `foldhash 0.1.5` transitif Zlib lisansı gerekçeli olarak
  `core/deny.toml`'a eklendi; duplicate sürümler politika gereği yalnız warning.

### Tam regresyon

- `cargo test -p nen-subtitle` → **81 passed, 0 failed, 1 ignored** (yalnız
  mevcut cue-lookup baseline'ı); NEN-020 toplam **11** yeni test ekledi.
- `cargo test --workspace` → çıkış **0**, bütün crate/spike/doc testleri yeşil.
  Repo test sayısı **317 → 328**.
- `cargo clippy --workspace --all-targets -- -D warnings` → çıkış **0**.
- `cargo fmt --all -- --check` → çıkış **0**.
- `bash scripts/test.sh` → **2 test dosyasının hepsi geçti**.
- `bash scripts/check-docs.sh` → çıkış **0** (kapanış sonrası).

### Karar

- `ADR-0010` ve `ADR-0029` `accepted`.
- Tespit offline ve yan etkisiz; kaynak seçimi, indirme veya AI çevirisi
  başlatan hiçbir yol eklenmedi.
