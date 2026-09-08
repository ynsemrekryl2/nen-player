---
id: NEN-089
title: Translation block layout and whole-document context
milestone: M5
size: M
state: done
closed: 2026-09-08
depends_on: [NEN-016]
blocks: [NEN-090, NEN-103]
adr: [0015]
---

# NEN-089 — Translation block layout and whole-document context

## Sonuç

Bir `SubtitleDocument` her zaman aynı, yeniden üretilebilir overlapping bloklara
ayrılıyor ve belgenin tamamından çıkarılan bağlam her bloğa aynı biçimde giriyor.

## Bağlam

`core/crates/nen-translate/src/lib.rs` bugün üç satırlık boş bir iskelet.
Şartname (`docs/product-spec.md` §10) blok boyutunu (varsayılan **40**, izin
verilen **30–60**) ve overlap'i (**6**) sabitliyor, ama blok sınırının nereye
düştüğünü, overlap'in sonuçta nasıl birleştiğini ve "bütün subtitle dokümanı
için bağlam analizi"nin ne ürettiğini söylemiyor. Bunlar ADR-0015 ile
kararlaştırılır.

## Ön koşul — ADR-0015

`proposed` yazılır, kullanıcı onaylar, `accepted` olur. En az şunları
kararlaştırır: blok sınırı kuralı · overlap penceresinin çakışan cue'larda hangi
bloğun sözünün geçerli olduğu · bağlam analizinin çıktısı (ne tutulur, ne kadar
büyür) · cue kimliğinin provider'a hangi biçimde gösterildiği · blok düzeni
versiyonunun (`block-layout version`, §11) nasıl türetildiği.

## Kapsam

- `nen-translate` içinde blok düzeni: deterministik, aynı belge → aynı düzen
- Belge tümü üzerinden bağlam çıkarımı (yalnız yerel, provider çağrısı yok)
- `block-layout version` üretimi — cache identity bunu tüketecek (`NEN-097`)
- Blok boyutunun izin verilen aralık dışına çıkmasının reddi

## YAPILMAYACAK

- Provider çağrısı veya prompt metni — `NEN-090`
- Doğrulama, repair, checkpoint — `NEN-091` · `NEN-092` · `NEN-093`
- Bağlam kalitesinin dilsel ölçümü — M5'in ölçütü yapısal doğruluk (S3, 2026-09-08)

## Kanıt (DoD)

- [x] ADR-0015 `accepted`
- [x] Golden: sabit bir fixture belgesinin blok düzeni snapshot'la byte düzeyinde eşleşiyor
- [x] Unit: aynı belge iki kez bölündüğünde düzen birebir aynı (determinizm)
- [x] Unit: overlap penceresindeki cue'lar her iki blokta da görünüyor ve sınır kuralı ADR-0015'in yazdığı gibi çözülüyor
- [x] Negatif: aralık dışı blok boyutu (29 ve 61) tipli hata ile reddediliyor
- [x] Negatif: cue sayısı bir bloktan küçük belge tek blok üretiyor, boş blok üretmiyor

## Kanıt kaydı

**ADR-0015 `accepted` (2026-09-08).** Taslağın iki eksik maddesi (bağlam
çıkarım kuralı, overlap doğrulama kapısı) kullanıcı kararıyla tamamlandı;
`docs/adr/README.md` ve `docs/DECISIONS.md` §6 sayımı (28→29 accepted,
5→4 proposed) güncellendi. Kendi commit'i: `3980c1e`.

**`core/crates/nen-translate/` dolduruldu** — `blocks.rs`
(`BlockLayoutConfig`, `BlockLayoutError`, `TranslationBlock`, `BlockLayout`)
ve `context.rs` (`ContextTerm`, `DocumentContext`). `nen-subtitle`'ın panik
kaçış yollarını kapatan `deny` bloğu birebir alındı; yeni dış bağımlılık yok.

**Golden** — `fixtures/subtitles/blocks/layout-sample.srt` (95 cue, üretken
script'le oluşturuldu) → `layout-sample.blocks.golden`.
`cargo test -p nen-translate --test block_layout_golden` (1 passed). Golden
elle doğrulandı: `block_size=40 overlap=6` için pencereler `0..40`, `34..74`,
`55..95` (son pencere `95-40=55`'e sabitlenmiş — Karar 1); çıktı sınırları
`floor` orta noktasıyla `37`, `64`; bağlam terimleri `Killua 48 · Gon 29 ·
Kurapika 19` — `Leorio` her zaman cümle başında geçtiği için (hiç orta
konumda görünmüyor) elenmiş, kuralın kör olmadığının kanıtı.

**Unit — determinizm:** `blocks::tests::same_document_splits_identically_twice`.
**Unit — overlap/sınır:**
`blocks::tests::neighbouring_windows_always_overlap_and_share_boundary_cues_as_context_only`,
`blocks::tests::output_ranges_partition_the_whole_document_in_order` (8 cue
sayısı için: 41,50,79,80,81,95,121,240).
**Negatif (zorunlu) — blok boyutu:** `blocks::tests::rejects_block_size_out_of_range`
(29 ve 61 → `BlockSizeOutOfRange`), sınır değerleri (30, 60) ayrı testte kabul
ediliyor (`accepts_block_size_boundaries`).
**Negatif — overlap:** `blocks::tests::rejects_overlap_out_of_range` (overlap
0 ve `block_size/2`=20 → `OverlapOutOfRange`).
**Negatif — küçük belge:** `blocks::tests::document_no_larger_than_a_block_is_a_single_block`
(25 cue → tek blok, boş blok yok) ve `blocks::tests::rejects_empty_document`
(0 cue → `EmptyDocument`).
**K23 guard (context):** `guard_context_debug.rs` — 3 test; sentinel
(`Zzqxvunlogged`) gerçekten tutulan bir terim oluyor
(`the_sentinel_actually_becomes_a_kept_term`, guard'ın kör olmadığının
kanıtı), `DocumentContext`/`ContextTerm`'ın `Debug`'ı sızdırmıyor
(`no_context_debug_output_leaks_the_term_text`), `BlockLayout`/
`TranslationBlock`/`BlockLayoutError` zaten metin taşımıyor (yapısal
kontrol, `no_block_layout_debug_or_display_output_leaks_document_text`).

`cargo test -p nen-translate`: **25 passed** (21 unit + 1 golden + 3 guard).

**Üç negatif kontrol ayrık ölçüldü** (kanıtın kör olmadığının kanıtı):
overlap sınırı sabit `sₖ₊₁`'e (orta nokta yerine) çekilince yalnız golden
test kırmızı (`every_fixture_matches_its_golden`, çıktı `0..34/34..55/55..95`
bekleneni `0..37/37..64/64..95` yerine üretti; unit `neighbouring_windows_*`
testi bu değişikliği yakalamadı çünkü onun aralığı gevşek — bu, planın
öngördüğünden farklı ama gerçek ölçülmüş sonuç); overlap doğrulama kapısı
kaldırılınca yalnız `rejects_overlap_out_of_range` kırmızı (20/21); K23
guard'ı `#[derive(Debug)]`'a çevrilince yalnız
`no_context_debug_output_leaks_the_term_text` kırmızı ve gerçek terim metnini
(`Zzqxvunlogged`) basılı gösterdi. Üçünde de değişiklik geri alındı, tüm
takım yeniden yeşile döndü.

`cargo test --workspace`: **688 passed / 1 ignored**, hepsi yeşil.
`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo deny check` (advisories/bans/licenses/sources ok, yeni dış bağımlılık
yok) temiz. `bash scripts/test.sh`: 4/4 dosya geçti. `bash
scripts/check-docs.sh` çıkış 0 (bu kapanıştan sonra).

Dokunulmayanlar: `nen-ffi`, macOS kabuğu, `nen-persist`. Değişiklik yalnız
`core/crates/nen-translate/`, `fixtures/subtitles/blocks/` ve doküman
tarafında ADR-0015/`docs/DECISIONS.md`/`docs/adr/README.md`/
`docs/milestones/M5-translation-core.md`.
