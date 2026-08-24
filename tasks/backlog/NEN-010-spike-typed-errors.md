---
id: NEN-010
title: Spike - typed error mapping
milestone: M1
size: S
state: backlog
depends_on: [NEN-007]
blocks: [NEN-011]
adr: [5]
---

# NEN-010 — Spike: typed error mapping

## Sonuç

Rust'taki her hata varyantı Swift ve Kotlin tarafında, mesaj string'i parse
edilmeden ayrıştırılabilir.

## Kapsam

- Temsili hata enum'ı (parse hatası, ağ hatası, iptal, capability yok,
  validation başarısız)
- Rust enum → Swift `Error` / Kotlin `Exception` eşlemesi
- Payload'lı varyantların redaction kurallarına uyması

## YAPILMAYACAK

- Nihai hata taksonomisi → ADR-0005, M2'de gerçek hatalarla olgunlaşır
- Kullanıcıya gösterilecek hata metinleri / lokalizasyon

## Kanıt (DoD)

- [ ] Swift tarafında her varyant `switch` ile ayrıştırılıyor (string parse yok)
- [ ] Payload'lı varyantın `{:?}` çıktısı yasaklı desen içermiyor
- [ ] Negatif: bilinmeyen varyant sessizce yutulmuyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
