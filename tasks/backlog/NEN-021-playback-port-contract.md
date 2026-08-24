---
id: NEN-021
title: PlaybackEngine port contract and capability model
milestone: M3
size: M
state: backlog
depends_on: [NEN-012]
blocks: [NEN-022]
adr: [11, 26]
---

# NEN-021 — PlaybackEngine port contract and capability model

## Sonuç

`PlaybackEngine` portu, tüm adapter'ların geçmek zorunda olduğu bir contract
test kitiyle birlikte tanımlıdır; capability'si olmayan operasyon typed error
döner.

> **Ön koşul: ADR-0026 (ownership yönü) kabul edilmiş olmalı.** Port'un imzası,
> session'ın sahibinin core mu platform shell mi olduğuna göre değişir
> (NEN-029'un A/B karşılaştırması). Bu karar verilmeden contract yazmak, büyük
> olasılıkla yeniden yazılacak bir contract üretir.

## Kapsam

- Port trait'i: load · play · pause · stop · absolute/relative seek · position ·
  duration · state (buffering/ready/ended/failure) · rate · volume · audio &
  subtitle track enumeration · track selection · embedded text extraction
  capability · external subtitle injection capability · event stream · shutdown
- `Capability` kümesi ve capability sorgulama
- Contract test kiti (`nen-ports` içinde, paylaşılan)
- Fake adapter — kiti geçen referans implementasyon

## YAPILMAYACAK

- Gerçek motor bağlama → NEN-022
- Renderer → NEN-027
- Motor adının UI'a sızması — **yasak** (§4)

## Kanıt (DoD)

- [ ] Fake adapter contract kitinin tamamını geçiyor
- [ ] Negatif: capability'si olmayan operasyon typed error dönüyor (panik/no-op yok)
- [ ] Application katmanında motor **adına** göre dallanma yok (grep kanıtı)
- [ ] Event stream sıralama garantisi test edildi

## Kanıt kaydı

<!-- done olurken doldurulacak -->
