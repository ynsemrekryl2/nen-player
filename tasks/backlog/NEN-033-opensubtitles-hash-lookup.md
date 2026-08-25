---
id: NEN-033
title: OpenSubtitles hash-based identity lookup
milestone: M6
size: M
state: backlog
depends_on: [NEN-018]
blocks: []
adr: []
---

# NEN-033 — OpenSubtitles hash-based identity lookup

## Sonuç

NEN-018'in hesapladığı OpenSubtitles-uyumlu hash, OpenSubtitles'ın resmi
API'sine sorulur ve birebir dosya eşleşmesi varsa kimlik **tahmin edilmeden**
çözülür.

## Kapsam

- Hash → resmi API sorgusu, typed sonuç (`Match` · `NoMatch` · `Ambiguous`)
- Sonucun `MediaEvidence` katman sırasına (ADR-0009 Karar 6) nasıl girdiği
- Hash'i olmayan medyada (boyut sınırı altı, `Range` desteklemeyen sunucu)
  akışın bozulmaması
- Deterministic fake client (CLAUDE.md kural 8 — gerçek kota kullanılamaz)

## YAPILMAYACAK

- Scraping — **non-goal**, yalnız resmi API
- Private file ID / hash / filename metadata loglama — K23 #7, #8
- Aday skorlama ve kullanıcıya gösterim → NEN-035

## Kanıt (DoD)

- [ ] Fake client'la birebir eşleşme kimliği çözüyor
- [ ] `NoMatch` durumunda alt katmanlara düşülüyor, hata üretilmiyor
- [ ] Negatif: private file ID ve hash hiçbir log/`Debug` çıktısında yok
- [ ] Negatif: testler gerçek provider kotası kullanmıyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
