---
id: NEN-097
title: Cache identity and invalidation
milestone: M5
size: M
state: done
closed: 2026-09-09
depends_on: [NEN-094]
blocks: [NEN-098]
adr: [0018]
---

# NEN-097 — Cache identity and invalidation

## Sonuç

Bir artifact'in kimliği şartnamenin saydığı bütün bileşenlerden türüyor ve
bileşenlerden **biri** değiştiğinde eski artifact artık kullanılmıyor.

## Bağlam

Şartname §11 cache identity'yi tek tek sayıyor: source fingerprint · source
language · target language · provider/model · media context · glossary · block
size/overlap · pipeline version · prompt version · schema version · block-layout
version · translation-session version. Ve şunu şart koşuyor: "Prompt/schema/
pipeline semantiği değişince **uyumsuz cache kullanılmamalıdır**."

Bu M5'in dördüncü çıkış kriterinin tamamı, ve `tasks/README.md`'nin
validation satırına girdiği için **negatif test zorunlu**.

`block-layout version` `NEN-089`'dan, `source`/`timeline fingerprint`
`NEN-016`'dan, provider kimliği `NEN-090`'dan gelir — bu task yeni kaynak
üretmez, hepsini tek bir kimliğe bağlar.

## Ön koşul — ADR-0018

`proposed` yazılır, kullanıcı onaylar, `accepted` olur. En az şunları
kararlaştırır: kimliğin tam bileşen listesi ve kanonik sırası · versiyon
alanlarının ne zaman elle artırıldığı · S9 kararının (2026-09-08: diskte çoklu,
menüde hedef dil başına en yeni) kimliğe yansıması · glossary alanı boşken
kimliğin nasıl hesaplandığı.

## Kapsam

- Cache identity hesabı ve kanonik serileştirmesi
- Bileşen değişiminin yeni kimlik ürettiğinin zorlanması
- Versiyon alanlarının tek bir yerde toplanması

## YAPILMAYACAK

- Disk sorgusu ve saklama — `NEN-096` · `NEN-098`
- Kullanıcıya artifact seçtirme yüzeyi — S9 gereği M5'te menüde yalnız en yeni
- Glossary içeriğinin yazılması — M6

## Kanıt (DoD)

- [x] ADR-0018 `accepted`
- [x] Unit: aynı girdi kümesi iki kez aynı kimliği veriyor (determinizm)
- [x] Negatif: **her** bileşen için ayrı test — bileşen değişince kimlik değişiyor
      ve eski artifact eşleşmiyor (bileşen başına bir iddia)
- [x] Negatif: kimlik hesabından bir bileşen kaldırıldığında o bileşenin testi
      kırmızıya dönüyor — kontrol sağır değil
- [x] Unit: hiçbir bileşen değişmediğinde kimlik aynı kalıyor (gereksiz invalidasyon yok)

## Kanıt kaydı

**ADR-0018 `accepted` (2026-09-09).** Ayrı commit'te: `docs/adr/README.md` ve
`docs/DECISIONS.md`'nin ADR indeksi sayaçları güncellendi (34 accepted, 0
proposed — M5'in beş ADR'sinin tamamı kapandı), `docs/milestones/
M5-translation-core.md`'nin kapanış notu düzeltildi. Yol üstünde ADR-0017'nin
kendi bağımsız kusuru bulundu — `NEN-095` (2026-09-09, `8b145ef`) ADR'yi kabul
etmişti ama `docs/adr/README.md`'nin durum sütununu ve `docs/DECISIONS.md`'nin
sayacını hiç güncellememişti; Kural 5 gereği ayrı bir "doküman düzeltmesi"
commit'i olarak atıldı (önceki `3dcfed2` emsaliyle aynı desen).

**Uygulama tamamı `core/crates/nen-translate/` içinde** — `nen-ports`,
`nen-persist`, `nen-ffi`, macOS kabuğu dokunulmadı (kullanıcı kararı: hesaplanan
kimlik bu task'ta `ArtifactRecord`'a yazılmıyor, nereye yazılacağı `NEN-098`).

- `src/versions.rs` (yeni): beş versiyon sabiti (`PIPELINE_VERSION` ·
  `PROMPT_VERSION` · `SCHEMA_VERSION` · `BLOCK_LAYOUT_VERSION` ·
  `TRANSLATION_SESSION_VERSION`) ve bunları paketleyen `PipelineVersions::CURRENT`
  tek yerde (ADR-0018 Karar 4). `artifact::PIPELINE_VERSION` ve
  `blocks::BLOCK_LAYOUT_VERSION` eski yollarında `pub use` re-export olarak
  kaldı — mevcut çağrı yerleri (`artifact.rs`, `blocks.rs`) değişmedi.
- `src/identity.rs` (yeni): `CacheIdentityInput` (yedi alan: source
  fingerprint, source/target language, provider/model, media hash, glossary,
  block layout) ve `CacheIdentity::of` — `nen_subtitle::fingerprint`'in
  desenini birebir izleyen açık, sabit sıralı byte kodlaması + BLAKE3
  (ADR-0018 Karar 1/2). `GlossaryIdentity` 0/1 elemanlı koleksiyon olarak
  kodlanıyor (Karar 3) — "glossary yok" ile "boş glossary" ayrı temsil değil.
  `media context` yalnız `MediaHash` (Karar 5, kullanıcı onayı). Provider ve
  model iki ayrı uzunluk-önekli alan (`SourceFingerprint`'in satır-çakışma
  önleme gerekçesiyle aynı).
- `Cargo.toml`: `blake3.workspace = true` — yeni dış bağımlılık değil (zaten
  `nen-subtitle`/`nen-persist`/`nen-app`'in workspace dep'i); `Cargo.lock`
  diff'i tek satır (`+ "blake3"` `nen-translate`'in dependency listesinde),
  yeni `[[package]]` yok.

**K23 (`docs/security-policy.md` §1 #4/#8).** `CacheIdentityInput`'un elle
yazılmış `Debug`'ı glossary adını ve provider/model kimliğini basmıyor
(yalnız dil, `has_media_hash`, `has_glossary`, blok boyutu/overlap).
`CacheIdentity`'nin `Debug`/`Display`'i `ContentAddress` emsaliyle
`<redacted>` basıyor; gerçek hex yalnız adlandırılmış `to_hex()`/`as_bytes()`
ile ulaşılıyor. `tests/guard_cache_identity_debug.rs` (4 test): sentinel
glossary adında ve provider/model'de `Debug`'a sızmıyor, `CacheIdentity`'nin
kendi hex'i kendi `Debug`/`Display`'inde geçmiyor, ve kasıtlı
`#[derive(Debug)]` ikizi aynı sentinel'ı sızdırarak guard'ın sağır olmadığını
gösteriyor (`checkpoint_cancellation_negative.rs`'in aynı deseni).

**Negatif test — 14 mutasyon, bileşen/alan başına bir test** (ADR-0018 Karar
2'nin 13 adlı bileşeninden `provider/model` iki ayrı kodlanmış alan olduğu
için iki ayrı test aldı): `tests/cache_identity_negative.rs`'te 11'i (source
fingerprint · source language · target language · provider · model · media
hash ×2 · glossary ×2 · block size · overlap, `HashMap` lookup miss
iddiasıyla; dosyadaki 12. test — `two_unmutated_baselines_share_an_identity`
— negatif değil, `Baseline`'ın kendisinin deterministik olduğunu doğrulayan
bir sağlık kontrolü), `src/identity.rs`'in kendi `#[cfg(test)]` modülünde
5'i (pipeline/prompt/schema/block-layout/translation-session version bump —
crate-private `of_with_versions` gerektirdiği için burada). Her test hem
`assert_ne!` hem de eski kimlikle kurulmuş bir `HashMap<CacheIdentity, _>`
lookup'ının yeni kimlikle **miss** verdiğini iddia ediyor — "eski artifact
eşleşmiyor" DoD maddesinin doğrudan karşılığı.

**Sağırlık kontrolü — 14 mutasyonun her biri elle ölçüldü** (`encode`'daki
ilgili satır/blok geçici olarak kaldırıldı, `cargo test -p nen-translate`
koşuldu, tam olarak beklenen test(ler) kırmızıya döndüğü doğrulandı, orijinal
geri yüklendi):

| Kaldırılan | Kırmızıya dönen |
|---|---|
| source fingerprint push | `source_fingerprint_change_invalidates` (yalnız) |
| source language push | `source_language_change_invalidates` (yalnız) |
| target language push | `target_language_change_invalidates` (yalnız) |
| provider adı push | `provider_name_change_invalidates` (yalnız) |
| provider model push | `provider_model_change_invalidates` (yalnız) |
| media hash push bloğu | `media_hash_none_to_some_invalidates` + `media_hash_different_value_invalidates` (yalnız bu ikisi) |
| glossary match bloğu | `glossary_none_to_named_invalidates` + `glossary_different_name_invalidates` (yalnız bu ikisi) |
| block size push | `block_size_change_invalidates` (yalnız) |
| overlap push | `overlap_change_invalidates` (yalnız) |
| pipeline version push | `pipeline_version_bump_invalidates` (yalnız) |
| prompt version push | `prompt_version_bump_invalidates` (yalnız) |
| schema version push | `schema_version_bump_invalidates` (yalnız) |
| block-layout version push | `block_layout_version_bump_invalidates` (yalnız) |
| translation-session version push | `translation_session_version_bump_invalidates` (yalnız) |

14 mutasyonun **hepsinde** kaldırılan satır tam olarak beklenen testi/testleri
kırdı, başka hiçbir test etkilenmedi — kontrol sağır değil.

**Determinizm / gereksiz invalidasyon yok** (`src/identity.rs`, 4 unit):
`same_input_produces_the_same_identity_twice` (aynı örnek iki kez), 
`independently_built_equal_inputs_produce_the_same_identity` (her biri kendi
`LanguageTag::parse`/`TranslationProviderIdentity::new` çağrısından kurulan
iki bağımsız girdi — işaretçi/örnek kimliğine değil değere dayandığını
kanıtlıyor), `same_media_hash_value_from_two_builds_produces_the_same_identity`
(`Some(MediaHash)` yolu için aynı), `to_hex_is_lowercase_and_64_characters`.

`cargo test -p nen-translate`: **89 passed** (51 lib unit + 1 golden + 7
artifact negatif + 1 block-layout golden + 12 cache-identity negatif + 4
checkpoint negatif + 2 artifact guard + 4 cache-identity guard + 3 context
guard + 4 validation negatif). `cargo test --workspace`: **793 passed / 1
ignored** (NEN-096 baseline 768'in üstüne bu task'ın 25 yeni testi — 9 unit +
12 negatif + 4 guard). `cargo fmt --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo deny check` (advisories/bans/licenses/
sources ok, yeni paket yok), `bash scripts/test.sh` (4/4) ve `bash
scripts/check-docs.sh` hepsi yeşil. `nen-ffi`, macOS kabuğu, `nen-persist`
dokunulmadı.
