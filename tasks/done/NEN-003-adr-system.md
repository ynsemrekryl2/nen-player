---
id: NEN-003
title: ADR system
milestone: M0
size: S
state: done
closed: 2026-08-24
depends_on: [NEN-001]
blocks: []
adr: [1]
---

# NEN-003 — ADR system

## Sonuç

Mimari kararlar `docs/adr/` altında tanımlı bir süreçle yazılır; bir task'ın
referans verdiği ADR `accepted` olmadan o task `done` olamaz.

## Kapsam

- `docs/adr/0000-template.md` — Bağlam / Karar / Gerekçe / Reddedilen
  alternatifler / Sonuçlar / Geri dönüş maliyeti
- `docs/adr/0001-adr-process.md` — süreç, `proposed → accepted` akışı, ADR ne
  zaman gerekir/gerekmez, numaralandırma, değiştirme kuralı
- `docs/adr/README.md` — yazılmış + planlanan ADR listesi (0002–0025)
- `check-docs.sh` içinde ADR-durum denetimi

## YAPILMAYACAK

- ADR-0002…0025'in içeriğini şimdiden yazmak — her biri kendi milestone'unda,
  gerçek ölçüm ve bağlamla yazılır
- ADR onay akışını otomatikleştirmek

## Kanıt (DoD)

- [x] ADR-0001 `accepted` durumunda ve reddedilen alternatifler bölümü dolu
- [x] Şablon "Reddedilen alternatifler" ve "Geri dönüş maliyeti" bölümlerini içeriyor
- [x] `check-docs.sh`, `adr:` alanı dolu `done` task'ın ADR'si `accepted` değilse hata veriyor

## Kanıt kaydı

Tarih: 2026-08-24 · Kanıt tipi: dosya denetimi + negatif test

- `docs/adr/0000-template.md` — "Reddedilen alternatifler" ve "Geri dönüş
  maliyeti" bölümlerini içeriyor; şablonda "Bu bölüm boş bırakılamaz" kuralı yazılı
- `docs/adr/0001-adr-process.md` — `status: accepted`, üç reddedilen alternatif
  tablo halinde (CLAUDE.md'de kararlar · GitHub Discussions · kod yorumları)
- `docs/adr/README.md` — ADR-0002…0025 planlanan liste, her biri milestone
  etiketiyle; link denetimi 3/3 geçti
- `check-docs.sh` denetim 6 ADR durumunu doğruluyor.
  Negatif test: ADR-0001 geçici olarak `status: proposed` yapıldığında →
  `HATA tasks/done/NEN-003-adr-system.md: done, ama ADR-0001 durumu 'proposed'
   (accepted olmalı).` — çıkış kodu 1. Değişiklik geri alındı.
