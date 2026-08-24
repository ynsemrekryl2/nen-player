---
id: NEN-019
title: SubtitleSourceCatalog with grouping and dedup
milestone: M2
size: M
state: backlog
depends_on: [NEN-016, NEN-018]
blocks: [NEN-026]
adr: [10]
---

# NEN-019 — SubtitleSourceCatalog with grouping and dedup

## Sonuç

Dört kaynak türü tek katalogta toplanır ve şartname §8'deki menü yapısı bu
katalogtan **projeksiyon** olarak üretilir.

## Kapsam

- `SubtitleSourceCatalog` modeli, kaynak türleri: `embedded` · `user` ·
  `opensubtitles` · `ai`
- Gruplama: kullanıcı kaynakları kendi grubunda, diğerleri dile göre
- Dedup: aynı kaynak iki kez görünmez
- `Dil Belirsiz` grubu, `Kapalı` her zaman mevcut
- AI sonucu **hedef dil grubunda**, AI rozetiyle

## YAPILMAYACAK

- UI çizimi → NEN-026
- Lazy yükleme mekaniği (indirme/extract) → M3/M6 (burada yalnız model ve kurallar)
- Otomatik kaynak önceliği — şartname bunu **yasaklıyor**

## Kanıt (DoD)

- [ ] Şartname §8'deki örnek menü, katalog projeksiyonundan **birebir** üretiliyor (golden)
- [ ] Aynı kaynak iki kez eklendiğinde katalogda tek görünüyor
- [ ] Dili bilinmeyen kaynak `Dil Belirsiz` grubunda
- [ ] `Kapalı` her koşulda listede
- [ ] Ayrı "AI subtitle mode" **yok** — AI kaynağı normal grup içinde

## Kanıt kaydı

<!-- done olurken doldurulacak -->
