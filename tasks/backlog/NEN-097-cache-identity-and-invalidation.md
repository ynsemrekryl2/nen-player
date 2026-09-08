---
id: NEN-097
title: Cache identity and invalidation
milestone: M5
size: M
state: backlog
closed:
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

- [ ] ADR-0018 `accepted`
- [ ] Unit: aynı girdi kümesi iki kez aynı kimliği veriyor (determinizm)
- [ ] Negatif: **her** bileşen için ayrı test — bileşen değişince kimlik değişiyor
      ve eski artifact eşleşmiyor (bileşen başına bir iddia)
- [ ] Negatif: kimlik hesabından bir bileşen kaldırıldığında o bileşenin testi
      kırmızıya dönüyor — kontrol sağır değil
- [ ] Unit: hiçbir bileşen değişmediğinde kimlik aynı kalıyor (gereksiz invalidasyon yok)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
