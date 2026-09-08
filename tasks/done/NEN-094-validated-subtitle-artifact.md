---
id: NEN-094
title: ValidatedSubtitleArtifact and its WebVTT output
milestone: M5
size: M
state: done
closed: 2026-09-09
depends_on: [NEN-093, NEN-014]
blocks: [NEN-095, NEN-097]
adr: []
---

# NEN-094 — ValidatedSubtitleArtifact and its WebVTT output

## Sonuç

Çeviri sonucu, **yalnız belgenin tamamı doğrulandıktan sonra**, cue ID/sıra/
zamanları girdiyle birebir aynı olan bir `ValidatedSubtitleArtifact` ve onun
UTF-8 WebVTT çıktısı olarak var oluyor.

## Bağlam

Şartname §11 artifact'in taşıması gereken alanları sayıyor: artifact ID ·
source fingerprint · subtitle timeline fingerprint · source/target language ·
normalized cues · WebVTT · provider/model · pipeline versions · glossary
identity · media identity fingerprint · createdAt.

Bunların çoğu **zaten var**: `nen_subtitle::fingerprint::{SourceFingerprint,
TimelineFingerprint}` (NEN-016), `nen_subtitle::webvtt::write` (NEN-014),
`SubtitleSourceKind::Ai` (NEN-019). Bu task yeni bir fingerprint veya ikinci bir
WebVTT yazıcısı açmaz, var olanları birleştirir.

## Kapsam

- `ValidatedSubtitleArtifact` tipi ve alanlarının doldurulması
- Belge seviyesi son doğrulama: bütün blokların birleşimi girdiyle aynı cue
  kümesini veriyor mu
- WebVTT üretimi (`nen_subtitle::webvtt::write` yeniden kullanılır)
- Glossary identity için **alan** — değeri M5'te boş olabilir

## YAPILMAYACAK

- Diske yazma — `NEN-096`
- Cache identity'nin hesaplanması ve invalidasyonu — `NEN-097`
- Glossary yazma/düzenleme yüzeyi — M6; burada yalnız kimlik alanı durur
- İkinci bir WebVTT yazıcısı veya yeni fingerprint algoritması

## Kanıt (DoD)

- [x] Golden: sabit fixture'ın artifact WebVTT çıktısı snapshot ile byte düzeyinde eşleşiyor
- [x] Unit: artifact'in cue ID'leri, sırası ve `TimeSpan`'leri **girdi belgesiyle birebir aynı**
- [x] Unit: `timeline fingerprint` girdi ve artifact için aynı değeri veriyor
- [x] Negatif: bir blok doğrulanmamışken artifact **üretilmiyor** (yarım yayın yasağı)
- [x] Negatif: cue metni dışında bir alanı değiştirilmiş sahte bir sonuçtan artifact üretilemiyor
- [x] Guard: artifact'in `Debug`/`Display` gösterimi cue metnini taşımıyor (K23 #4)

## Kanıt kaydı

**`ValidatedSubtitleArtifact` ve `assemble()` `core/crates/nen-translate/src/artifact.rs`'e
eklendi (yeni modül).** Artifact şartname §11'in saydığı alanların tamamını
taşıyor: `id` (`ArtifactId` — dışarıdan verilen, opak, doğrulanmış newtype;
bu crate ID üretmiyor, çünkü onu üretecek katman — `NEN-096` CAS / `NEN-097`
cache identity — henüz `accepted` bir ADR'ye sahip değil, Kural 4) ·
`source_fingerprint`/`timeline_fingerprint` (girdi belgesinden, `nen_subtitle::
fingerprint`) · `source_language`/`target_language` · `provider`
(`TranslationProviderIdentity`, kendi redaksiyonlu `Debug`'ı devralınıyor) ·
`pipeline_version` (yeni `PIPELINE_VERSION = 1` sabiti) · `block_layout_version`
(`BlockLayout::version()`) · `glossary` (`GlossaryIdentity` — M5'te her zaman
`none()`, alan M6 için hazır) · `media_hash` (`Option<MediaHash>`) · `created_at`
(`ArtifactTimestamp` — çağıran verir; crate saat okumuyor) · `translated_document()`
· `webvtt()` (`nen_subtitle::webvtt::write` yeniden kullanılıyor, ikinci yazıcı
yok).

**`assemble()` `CompletedBlocks`'u parametre alır** — `crate::checkpoint::
BlockCheckpoints::into_completed` bunu yalnız her blok checkpoint'lendiğinde
üretebildiği için, yarım bir çalışmadan `CompletedBlocks` zaten elde edilemiyor;
bu tip düzeyinde "yarım yayın yasağı"nın artifact'e kadar taşınmasıdır (negatif
DoD #1: `a_run_that_never_fully_checkpointed_cannot_reach_assembly` bu yolun
`assemble` hiç çağrılmadan kapandığını doğruluyor). Ayrıca `assemble` girdiye
**körü körüne güvenmiyor**: her blok için `layout`'un kendi `output_positions()`
aralığındaki her `doc_position`'da kaynak belgenin `CueId`'si ve `TimeSpan`'i
`ValidatedBlock`'un karşılığıyla birebir karşılaştırılıyor — sahte/yanlış
eşleşen bir `completed` kümesi (yanlış belge, yanlış layout) `ArtifactError`
ile reddediliyor, sessizce birleştirilmiyor.

**Mutasyon kontrolü (elle ölçüldü, DoD "kontrol sağır değil" maddesi) —
`assemble()`'daki her kapı ayrı ayrı kaldırılıp ilgili negatif test(ler)in
tam olarak kırmızıya döndüğü ölçüldü:**
- Üst düzey `LayoutMismatch` (cue count) kaldırılınca: yalnız
  `a_layout_built_for_a_shorter_document_than_a_longer_one_is_rejected`
  kırmızı oldu — `LayoutMismatch` yerine daha az kesin `TimelineMismatch`
  döndü (kısa belge senaryosu döngü-içi `.get()` denetimiyle hâlâ
  yakalandığından etkilenmedi — bu da o iç denetimin gerçekten ayrı bir kapı
  olduğunu gösteriyor).
- `CueMismatch` kaldırılınca: `a_completed_run_paired_with_a_document_of_
  different_cue_ids_is_rejected` kırmızı oldu — sahte sonuç **sessizce
  `Ok(artifact)` üretti** (hiçbir başka kapı yakalamadı).
- `SpanMismatch` kaldırılınca: `a_completed_run_paired_with_a_document_of_
  shifted_timing_is_rejected` kırmızı oldu — çünkü birleştirilen belge zamanı
  her zaman *çağrıya verilen* `document`'ten alınıyor, dolayısıyla sondaki
  `TimelineMismatch` kapısı bu senaryoyu hiçbir zaman yakalayamıyor; bu kontrol
  gerçekten tekil kapı.
- `BlockCount` kaldırılınca: `a_completed_run_paired_with_a_differently_
  shaped_layout_is_rejected` kırmızı oldu — hata `BlockCount` yerine daha az
  kesin `BlockCueCount`'a düştü (yine de yakalandı, ama yanlış teşhisle).
- `BlockCueCount` kaldırılınca: `a_completed_run_paired_with_a_layout_of_the_
  same_block_count_but_different_windows_is_rejected` kırmızı oldu — hata
  `BlockCueCount` yerine `CueMismatch`'e düştü.
- Artifact'in elle yazılmış `Debug`'ı `#[derive(Debug)]`'a çevrilince:
  `no_artifact_debug_output_leaks_dialogue_text` kırmızı oldu — türetilmiş
  `Debug`, `webvtt` alanının tam metnini (sentinel dahil) bastı.
- Ayrıca kod incelemesiyle: `BlockOrder` adında planlanan bir kapı, hem
  `BlockLayout::of`'un hem `BlockCheckpoints::into_completed`'in kendi
  değişmezleri gereği (ikisi de blokları her zaman `0..n` sırasında, kendi
  indeksleriyle üretiyor) **hiçbir girdiyle asla tetiklenemeyeceği** için
  koddan tamamen çıkarıldı — sağır bir kontrolü tutmak yerine kaldırmak
  tercih edildi. `TimelineMismatch` son kapısı ise gerçekten "kuşak-ve-askı":
  yukarıdaki alan-düzeyi kontroller geçtiğinde bugün ulaşılamıyor, ama
  `BlockLayout`'un kendi "her pozisyon tam bir bloğun output'una girer"
  değişmezini ayrı bir modülden (`assemble` bunu yeniden türetmiyor) doğrulayan
  tek gerçek kapı olduğu için (gelecekte `blocks.rs`'te bir hata olursa cue
  sayısı uyuşmazlığını yakalar) tutuldu; bu, kodun kendi yorumunda açıkça
  belirtildi.
- Her mutasyon ölçümden hemen sonra geri alındı; kalıcı diff yok, kanıt yalnız
  test çıktısı.

**Metin → satır dönüşümü** (`cue_lines`): `\r\n` normalize ediliyor, `'\n'`
üzerinden bölünüyor, tamamen boş satırlar atılıyor (WebVTT'de içteki boş satır
cue'yu erken bitirir). `validate_block` metnin trim'lenmiş hâlinin boş
olmamasını zaten garanti ettiğinden en az bir satır her zaman kalıyor —
`interior_blank_line_is_dropped_but_a_wholly_blank_split_never_happens` unit
testiyle doğrulandı.

**Kanıt (gerçek çıktı):**
- `cargo test -p nen-translate` — lib **40 passed** (yeni `artifact::tests`
  4'ü dahil), `artifact_golden` **1 passed**, `artifact_negative` **7 passed**,
  `block_layout_golden` **1 passed**, `checkpoint_cancellation_negative`
  **4 passed**, `guard_artifact_debug` **2 passed**, `guard_context_debug`
  **3 passed**, `validation_negative` **4 passed** — toplam **62 passed**
- `cargo test --workspace` — **738 passed / 1 ignored**, 0 failed
- `cargo fmt --check` — temiz
- `cargo clippy --workspace --all-targets -- -D warnings` — temiz
- `cargo deny check` — advisories/bans/licenses/sources ok (yeni bağımlılık yok)
- `bash scripts/test.sh` — 4/4 dosya geçti (doctor + stremio-bridge)
- `bash scripts/check-docs.sh` — **SONUÇ: tüm denetimler geçti** (0 hata)

**Golden fixture:** `fixtures/subtitles/blocks/layout-sample.srt` (95 cue,
3 blok) — `fixtures/subtitles/blocks/layout-sample.artifact.golden` fingerprint
çiftini, dil çiftini, sabit sürüm alanlarını ve tam WebVTT gövdesini
(`UPDATE_GOLDEN=1 cargo test -p nen-translate --test artifact_golden` ile
üretilebilir) byte düzeyinde sabitliyor. Çok bloklu bir fixture kasıtlı seçildi:
byte-özdeş WebVTT gövdesi tek başına blokların birleşiminin belgeyi birebir
verdiğinin kanıtı (M5 çıkış kriteri eşlemesi: "Cue ID/sıra/zamanlar girdiyle
birebir aynı → `NEN-094` golden").

**Kapsam dışı bırakılanlar (plana göre):** diske yazma (`NEN-096`), cache
identity hesabı ve invalidasyonu (`NEN-097`), glossary yazma yüzeyi (M6),
ikinci bir WebVTT yazıcısı veya yeni fingerprint algoritması — hiçbiri
açılmadı. `nen-ffi`, macOS kabuğu, `nen-persist` dokunulmadı; değişiklik
yalnız `nen-translate` içinde (`src/artifact.rs`, `src/lib.rs`,
`tests/support/mod.rs`, üç yeni test dosyası, bir golden fixture).
