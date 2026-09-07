---
id: NEN-039
title: Subtitle menu language grouping and matching use the primary subtag
milestone: M3
size: S
state: done
closed: 2026-08-25
depends_on: [NEN-019]
blocks: [NEN-026]
adr: [30]
---

# NEN-039 — Subtitle menu language grouping and matching use the primary subtag

## Sonuç

`en` ve `en-us` menüde tek grup olur; tercihi `tr` olan kullanıcı `tr-tr`
etiketli bir kaynak için de otomatik seçim alır.

## Kapsam

- `nen-domain::source::LanguageTag`'e `primary()` (`&str`) ve `primary_tag()`
  (region'sız `LanguageTag`) eklenir
- `nen-subtitle::language`'ın zaten var olan özel `primary_subtag()`
  yardımcısı **kaldırılır**, yerine `LanguageTag::primary()` kullanılır —
  aynı ayrımı iki kod yolunda ayrı ayrı elle yazmamak için
- `nen-catalog::menu::project`'in `BTreeMap` gruplama anahtarı primary
  subtag'e döner; temsilci `MenuGroup::Language` `primary_tag()`'ten kurulur
- `nen-catalog::auto_select::auto_selection`'ın dil eşleşmesi
  `LanguageTag::primary()` üzerinden yapılır

## YAPILMAYACAK

- `LanguageTag`'in kendisini region'sız yapmak — ADR-0029 Karar 5'i geri
  almaz, region saklanmaya devam eder
- Girişte region'ı görünür kılmak ("English (US)") — NEN-026
- Grup içi sıralamaya dokunmak — ADR-0010 Karar 5 değişmiyor

## Kanıt (DoD)

- [x] `en-us` metadata'lı bir kaynak ile `en` metadata'lı bir kaynak menüde
      **tek** dil grubunda (unit)
- [x] Tercihi `tr` olan bir katalogda yalnız `tr-tr` etiketli bir kaynak
      varken `auto_selection` onu buluyor (unit)
- [x] Mevcut iki golden (`menu-default.golden`,
      `menu-preferred-tr-en.golden`) **değişmeden** geçiyor — fixture'da
      region yok, davranış geriye dönük uyumlu
- [x] Yeni golden: `pt-br` ve `pt-pt` etiketli iki kaynak tek "pt" grubunda,
      ikisi de listede, giriş sırası korunuyor

## Kanıt kaydı

Ortam: Apple M5 · macOS 27.0 · rustc/cargo 1.98.0 (2026-08-25).

### Değişen kod

| Dosya | Değişiklik |
|---|---|
| `nen-domain/src/source.rs` | `LanguageTag::primary()` (`&str`) + `primary_tag()` (region'sız `LanguageTag`) |
| `nen-subtitle/src/language.rs` | özel `primary_subtag()` yardımcısı **silindi**, `resolve_candidate` artık `language.primary()` kullanıyor |
| `nen-catalog/src/menu.rs` | `BTreeMap` anahtarı `&LanguageTag` → sahipli `LanguageTag` (primary); tercih eşleşmesi `preferred.primary_tag()` |
| `nen-catalog/src/auto_select.rs` | `s.language() == Some(language)` → `l.primary() == language.primary()` |

`SubtitleSource::language()`, `SubtitlePreferences` ve
`AUTO_SELECTABLE_KINDS` **değişmedi** (ADR-0030 Karar 3, ADR-0010 Karar 9).

### DoD 1 — `en` ve `en-us` tek grup

`menu::tests::regions_of_one_language_share_a_single_group`: iki embedded
kaynak (`en-us`, `en`) → gruplar `[Closed, Language(en)]`; grubun entry'leri
kendi tam etiketlerini (`en-us`, `en`) katalog sırasında koruyor.

### DoD 2 — `tr` tercihi `tr-tr` kaynağı buluyor

`auto_select::tests::a_preference_matches_a_track_tagged_with_a_region`:
katalogta yalnız `tr-tr` embedded varken tercih `tr` → seçim yapılıyor,
seçilen kaynağın `.language()`'i hâlâ `Some(tr-tr)`.

Ters yön ayrıca kanıtlandı —
`a_preference_carrying_a_region_matches_a_plain_track`: tercih `tr-tr`
(sistem dilinden gelen biçim), kaynak düz `tr`. Bu yön ADR-0030'un
"Gerekçe"sinde eklenen asıl kırılma senaryosu: NEN-026 tercihi macOS
locale'inden alırsa (`tr-TR`), tam eşitlikle **hiçbir** track eşleşmezdi.

### DoD 3 — mevcut iki golden değişmeden geçiyor

`fixtures/catalog/menu-default.golden` ve `menu-preferred-tr-en.golden`
dosyalarına dokunulmadı; `git status` ikisini de değişmemiş gösteriyor,
`menu_projection_golden` 2/2 yeşil.

### DoD 4 — yeni golden

`fixtures/catalog/menu-pt-regions.golden` (yeni) —
`regions_of_one_language_share_one_heading_and_keep_their_tags`:

```
group	closed
group	language:pt
entry	embedded	pt-br	Português (BR)
entry	embedded	pt-pt	Português (PT)
```

Tek `language:pt` başlığı, iki entry giriş sırasında, her biri **kendi tam
etiketiyle** — Karar 1 ve Karar 3 aynı snapshot'ta.

### Negatif kontrol — testler boşta dönmüyor

Kanıtın kendisi iki gerçek test kusuru buldu. `menu.rs` ve `auto_select.rs`
geçici olarak eski (tam etiket) hâline döndürülüp koşuldu:

- **İlk koşu 4/6 kırmızı.** Geçen iki menü testi boştaydı: (1) tercih
  `en-us` + kaynak `en` senaryosunda beklenen sıra `[Closed, en, fr]`'ydi,
  ama `en` alfabetik kuyruğun zaten başında olduğu için eski kod da aynı
  sırayı üretiyordu — tercih **hoist edilmese de** test geçiyordu. Düzeltme:
  tercih `fr-ca` yapıldı, `fr` alfabetik olarak `en`'den sonra geldiği için
  hoist artık sırada görünüyor. (2) `(en, en-us)` tercih çifti testinde
  katalogta yalnız `en` kaynağı vardı; eski kodda ikinci tercih de hiçbir şey
  bulamadığı için sonuç aynıydı. Düzeltme: katalog `en` + `en-us` yapıldı,
  eski kod artık iki ayrı başlık üretiyor.
- **Düzeltme sonrası 6/6 kırmızı**, kod geri alınınca 6/6 yeşil.

### Doğrulama çıktıları

```
$ cargo test --manifest-path core/Cargo.toml --workspace
337 passed, 0 failed, 1 ignored              → exit 0 (NEN-039 ile 328 → 337)
  nen-domain 20 (unit, +2) · nen-catalog 27 (unit, +6) +
  menu_projection_golden 3 (+1)
  ignored = cue_lookup_baseline (NEN-017 baseline'ı)

$ cargo fmt --check                          → exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
                                             → exit 0, uyarı yok

$ cargo tree -p nen-catalog --edges normal
nen-catalog → nen-domain · nen-identity · nen-subtitle
                                             → yeni dış bağımlılık YOK

$ bash scripts/check-docs.sh                 → SONUÇ: tüm denetimler geçti
```

### Kapsam dışı bırakılanlar (ADR-0030 Notlar'a yazıldı)

- **Grup içi region tiebreak'i.** Tercihi `pt-br` olan kullanıcı `pt-pt`
  yerine `pt-br`'yi isteyemiyor; seçim tür önceliğine ve katalog sırasına
  düşüyor. Bu davranış `auto_select::tests::region_is_not_a_tiebreak_inside_a_language`
  ile **kilitlendi** — ileride değiştirilirse sessizce değil, kırmızı testle
  değişir. ADR-0010 Karar 5'e dokunacağı için kendi kararını istiyor.
- **`SubtitlePreferences::new`'in tam etiket dedupe'u.** `(en, en-us)` çifti
  artık aynı dile işaret ediyor: mükerrer menü bölümü oluşmuyor
  (`two_preferences_of_one_language_do_not_open_two_groups`), ama kullanıcının
  ikinci tercih slotu fiilen boşa gidiyor. Çözüm çekirdekte değil NEN-026'da.
