---
id: NEN-012
title: Spike report and core language decision
milestone: M1
size: S
state: backlog
depends_on: [NEN-011, NEN-029]
blocks: [NEN-013, NEN-018, NEN-021]
adr: [2, 27]
---

# NEN-012 — Spike report and core language decision

## Sonuç

Shared core dili ADR-0002 ile **kilitlenmiştir**; kararın arkasında NEN-008/009/
010/011/029'dan gelen gerçek sayılar ve invariant kanıtları vardır.

## Kapsam

- Spike raporu: **beş** ölçümün baseline sonuçları (bağlamlarıyla) ve
  invariant'ların (I1–I5) durumu
- ADR-0002: karar, gerekçe, reddedilen alternatifler
  (Kotlin Multiplatform · Swift core + ayrı Android · C++ core), geri dönüş maliyeti
- **NEN-029'un ownership sonucu bu kararın girdisidir** — reverse-FFI her iki
  yönde de kabul edilemezse Rust adayı `no-go` alabilir
- ADR-0027 taslağı: baseline'lardan çıkan performans bütçesi önerisi
- `no-go` çıkarsa: yeni plan taslağı ve etkilenen milestone listesi

> `no-go`, "baseline beklenenden yavaş" demek **değildir** — bütçe ADR-0027 ile
> ayarlanabilir. `no-go` yalnız bir invariant sağlanamazsa, baseline hiçbir makul
> tasarıma izin vermiyorsa, veya reverse-FFI'ın **her iki yönü** de kabul
> edilemezse verilir.

## YAPILMAYACAK

- Spike kodunu ürüne terfi ettirmek — `core/spikes/` altında kalır
- Karar öncesi M2 dışındaki milestone'lara başlamak

## Kanıt (DoD)

- [ ] ADR-0002 `accepted` ve beş ölçümün **sayıları + bağlamı** içinde
- [ ] Invariant'lar I1–I5'in her birinin durumu tabloda
- [ ] Reddedilen alternatifler bölümü dolu
- [ ] NEN-029 ownership sonucunun karara etkisi yazılmış
- [ ] go/no-go sonucu, yukarıdaki `no-go` tanımına göre açıkça gerekçelendirilmiş

## Kanıt kaydı

<!-- done olurken doldurulacak -->
