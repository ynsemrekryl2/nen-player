---
id: NEN-049
title: Serialize real libmpv platform tests
milestone: M3
size: S
state: backlog
depends_on: [NEN-022, NEN-045]
blocks: []
adr: [12, 33]
---

# NEN-049 — Serialize real libmpv platform tests

## Sonuç

`scripts/test-macos.sh` içindeki gerçek libmpv testleri birbirinin zamanlamasını
bozmadan deterministik geçer; ürünün seek kontratı gevşetilmez.

## Kapsam

- Swift Testing'in tam paket koşusunda gerçek libmpv suite'lerini aynı anda
  çalıştırmaması için en dar test-runner veya suite izolasyonu
- `aSeekIsAnsweredThroughTheSession` testinin `SeekCompleted` içindeki gerçek
  konumu sınamaya devam etmesi; toleransı genişletmeme ve `0 ms`'yi kabul etmeme
- Seri/paralel farkının tekrarlı koşuyla kanıtlanması

## YAPILMAYACAK

- Playback adapter davranışını yalnız testi yeşile çevirmek için değiştirmek
- Seek UI uzlaştırması; transport sıçraması ayrı ürün işi
- Platform testlerini atlamak veya contract kitini daraltmak

## Neden ayrı task

NEN-046 kapanışında tam paralel macOS paketi iki ardışık koşuda aynı şekilde
kırmızı oldu: `aSeekIsAnsweredThroughTheSession`, beklenen `12_000 ms` yerine
`SeekCompleted(0 ms)` gördü. Test tek başına **5/5**, tüm paket açıkça
`--no-parallel` ile **31/31** geçti. NEN-046 yalnız pencere yaşam döngüsüdür;
gerçek motor testlerinin runner izolasyonu bu kapsama karıştırılmaz.

## Kanıt (DoD)

- [ ] `bash scripts/test-macos.sh` art arda en az 3 kez çıkış 0
- [ ] `aSeekIsAnsweredThroughTheSession` hâlâ `12_000 ± 100 ms` bekliyor
- [ ] Değişiklik öncesi paralel kırmızı ve sonrası tekrarlı yeşil sayıları
      bağlamıyla kaydedilmiş
- [ ] Ürün kaynak kodunda değişiklik yok

## Kanıt kaydı

<!-- done olurken doldurulacak -->
