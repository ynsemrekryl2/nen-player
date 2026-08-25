---
id: NEN-039
title: Subtitle menu language grouping and matching use the primary subtag
milestone: M3
size: S
state: backlog
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

- [ ] `en-us` metadata'lı bir kaynak ile `en` metadata'lı bir kaynak menüde
      **tek** dil grubunda (unit)
- [ ] Tercihi `tr` olan bir katalogda yalnız `tr-tr` etiketli bir kaynak
      varken `auto_selection` onu buluyor (unit)
- [ ] Mevcut iki golden (`menu-default.golden`,
      `menu-preferred-tr-en.golden`) **değişmeden** geçiyor — fixture'da
      region yok, davranış geriye dönük uyumlu
- [ ] Yeni golden: `pt-br` ve `pt-pt` etiketli iki kaynak tek "pt" grubunda,
      ikisi de listede, giriş sırası korunuyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
