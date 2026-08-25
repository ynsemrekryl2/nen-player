---
id: NEN-019
title: SubtitleSourceCatalog with grouping and dedup
milestone: M2
size: M
state: backlog
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

- [ ] ADR-0010 Karar 10'un **tercih ayarlanmamış** menüsü katalog
      projeksiyonundan **birebir** üretiliyor (golden)
- [ ] Aynı katalog, birinci tercih `tr` / ikinci tercih `en` iken Karar 10'un
      ikinci örneğini **birebir** üretiyor (golden)
- [ ] Aynı kaynak iki kez eklendiğinde katalogda tek görünüyor (dört türde)
- [ ] Dili bilinmeyen kaynak `Dil Belirsiz` grubunda ve **her zaman en sonda**
- [ ] `Kapalı` her koşulda listede — boş katalog dahil
- [ ] Ayrı "AI subtitle mode" **yok** — AI kaynağı normal grup içinde
- [ ] İkinci tercih birinciyle aynıysa yok sayılıyor; kaynağı olmayan tercih
      grubu gösterilmiyor
- [ ] Otomatik seçim: tercih edilen dilde `embedded` varsa o, yoksa `user`;
      `opensubtitles` ve `ai` **hiçbir koşulda** otomatik seçilmiyor (negatif test)
- [ ] Otomatik seçim, tercih edilen dil dışında hiçbir dilde çalışmıyor; hiçbiri
      yoksa `Kapalı` (negatif test)
- [ ] **Negatif kontrol:** `SubtitleSource`/`SubtitleSourceId` `Debug`'ı dosya
      adı, yol parçası veya private file ID sızdırmıyor; `#[derive(Debug)]`'lı
      kasıtlı bozuk ikiz bunların **hepsini** sızdırıyor (K23 #3/#8, Kural 3)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
