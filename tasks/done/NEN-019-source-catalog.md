---
id: NEN-019
title: SubtitleSourceCatalog with grouping and dedup
milestone: M2
size: M
state: done
depends_on: [NEN-016, NEN-018]
blocks: [NEN-026, NEN-037]
adr: [10]
---

# NEN-019 — SubtitleSourceCatalog with grouping and dedup

## Sonuç

Dört kaynak türü tek katalogta toplanır ve şartname §8'deki menü yapısı — ADR-0010'un
genişlettiği haliyle — bu katalogtan **projeksiyon** olarak üretilir.

## Kapsam

- `SubtitleSourceCatalog` modeli, kaynak türleri: `embedded` · `user` ·
  `opensubtitles` · `ai`
- Gruplama: kullanıcı kaynakları kendi grubunda, diğerleri dile göre
- Dedup: aynı kaynak iki kez görünmez — **metadata kimliğiyle**, içerik
  fingerprint'iyle değil (ADR-0010 Karar 2; §7 lazy kuralı)
- `Dil Belirsiz` grubu, `Kapalı` her zaman mevcut
- AI sonucu **hedef dil grubunda**, AI rozetiyle
- **`SubtitlePreferences`** (birinci/ikinci tercih edilen dil) → grup sırası
  (ADR-0010 Karar 4)
- **Otomatik seçim politikası** — saf fonksiyon, tür önceliği
  `embedded` → `user`; `opensubtitles` basamağı **kapalı** (ADR-0010 Karar 9)
- Menü projeksiyonu **yapısal** döner; görünen ad UI'ın işi (ADR-0010 Karar 7)

## YAPILMAYACAK

- UI çizimi → NEN-026
- Tercih ayarının kullanıcı yüzeyi ve kalıcılığı → NEN-037
- Otomatik indirme (`opensubtitles` basamağı) → NEN-038, **kendi ADR'siyle**
- Dil tespiti → NEN-020 (katalog `Option<LanguageTag>`'i olduğu gibi gruplar)
- Lazy yükleme mekaniği (indirme/extract) → M3/M6 (burada yalnız model ve kurallar)
- Otomatik kaynak önceliği — şartname bunu **yasaklıyor**; tek öncelik ADR-0010
  Karar 9'un otomatik seçim tür sırasıdır ve menü sırası öncelik taşımaz

## Kanıt (DoD)

- [x] ADR-0010 Karar 10'un **tercih ayarlanmamış** menüsü katalog
      projeksiyonundan **birebir** üretiliyor (golden)
- [x] Aynı katalog, birinci tercih `tr` / ikinci tercih `en` iken Karar 10'un
      ikinci örneğini **birebir** üretiyor (golden)
- [x] Aynı kaynak iki kez eklendiğinde katalogda tek görünüyor (dört türde)
- [x] Dili bilinmeyen kaynak `Dil Belirsiz` grubunda ve **her zaman en sonda**
- [x] `Kapalı` her koşulda listede — boş katalog dahil
- [x] Ayrı "AI subtitle mode" **yok** — AI kaynağı normal grup içinde
- [x] İkinci tercih birinciyle aynıysa yok sayılıyor; kaynağı olmayan tercih
      grubu gösterilmiyor
- [x] Otomatik seçim: tercih edilen dilde `embedded` varsa o, yoksa `user`;
      `opensubtitles` ve `ai` **hiçbir koşulda** otomatik seçilmiyor (negatif test)
- [x] Otomatik seçim, tercih edilen dil dışında hiçbir dilde çalışmıyor; hiçbiri
      yoksa `Kapalı` (negatif test)
- [x] **Negatif kontrol:** `SubtitleSource`/`SubtitleSourceId` `Debug`'ı dosya
      adı, yol parçası veya private file ID sızdırmıyor; `#[derive(Debug)]`'lı
      kasıtlı bozuk ikiz bunların **hepsini** sızdırıyor (K23 #3/#8, Kural 3)

## Kanıt kaydı

### Menü projeksiyonu ve katalog kuralları

```text
$ cargo test -p nen-catalog --manifest-path core/Cargo.toml
  unit tests                    21 passed
  guard_source_debug             2 passed
  menu_projection_golden         2 passed
  toplam                         25 passed · 0 failed
```

- `menu_without_preferences_matches_adr_0010_decision_10` →
  `fixtures/catalog/menu-default.golden` ile byte-eşit.
- `menu_with_turkish_then_english_matches_adr_0010_decision_10` →
  `fixtures/catalog/menu-preferred-tr-en.golden` ile byte-eşit; aynı katalogda
  grup sırası `tr` → `en` → `fr`.
- `dedup_covers_all_four_kinds` → `embedded` · `user` · `opensubtitles` · `ai`
  kimliklerinin her biri iki kez eklendiğinde katalog uzunluğu 1.
- `unknown_language_group_is_always_last` ·
  `closed_is_present_even_for_an_empty_catalog` ·
  `ai_output_sits_in_its_target_language_group` ·
  `preferences_drop_a_secondary_equal_to_the_primary` ·
  `a_preferred_language_without_sources_shows_no_group` geçiyor.
- Otomatik seçim pozitif/negatif kanıtı:
  `embedded_wins_over_a_user_file_in_the_same_language` ·
  `a_user_file_is_picked_when_no_embedded_track_matches` ·
  `opensubtitles_is_never_selected_automatically` ·
  `ai_output_is_never_selected_automatically` ·
  `no_language_outside_the_preferences_is_selected` ·
  `nothing_is_selected_without_preferences` geçiyor.

### K23 negatif kontrolü

- `source_and_id_debug_hide_filename_path_digest_and_private_file_id` → gerçek
  `SubtitleSource`/`SubtitleSourceId` çıktısında özel tam yol, dosya adı, opak
  digest parçası ve private file ID'nin hiçbiri yok.
- `derived_debug_twin_leaks_every_value_the_real_types_hide` → aynı dört değeri
  taşıyan `#[derive(Debug)]`'lı kasıtlı bozuk ikiz **dördünü de** sızdırıyor;
  guard'ın boşta dönmediği doğrulanıyor.

### Bağımlılık ve tam doğrulama

```text
$ cargo tree -p nen-domain --edges normal --manifest-path core/Cargo.toml
nen-domain v0.1.0                         → tek düğüm, sıfır bağımlılık

$ cargo tree -p nen-catalog --edges normal --manifest-path core/Cargo.toml
nen-catalog → nen-domain · nen-identity · nen-subtitle
                                           → yeni doğrudan dış bağımlılık yok;
                                             Cargo.lock değişmedi

$ cargo fmt --all --check --manifest-path core/Cargo.toml
                                             → exit 0
$ cargo clippy --workspace --all-targets --manifest-path core/Cargo.toml -- -D warnings
                                             → exit 0, uyarı yok
$ cargo test --workspace --manifest-path core/Cargo.toml
                                             → exit 0, 0 failed, 1 ignored baseline
$ (cd core && cargo deny check)
  advisories ok · bans ok · licenses ok · sources ok
                                             → exit 0
$ bash scripts/test.sh
  check-docs.test.sh ✓ (11 doğrulama)
  doctor.test.sh     ✓ (24 doğrulama)        → exit 0
```
