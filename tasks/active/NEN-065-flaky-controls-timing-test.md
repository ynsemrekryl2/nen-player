---
id: NEN-065
title: Stabilize the pinned-controls timing test
milestone: M3
size: S
state: active
depends_on: [NEN-061]
blocks: [NEN-067]
adr: []
---

# NEN-065 — Kontrol gizleme testinin zamanlamaya bağlı kırılganlığı

## Sonuç

`pinnedControlsSuspendAndRestoreAutomaticHiding` makine yüküne bakmadan aynı
sonucu verir.

## Bağlam

NEN-027'nin koşumları sırasında bir kez kırmızıya döndü:

```
✘ "pinned controls stay visible and unpin restores the hide timer"
   PlayerModelTests.swift:256 — Expectation failed: !(model.controlsVisible)
```

Aynı test tek başına üç kez, tüm paket ardından üç kez yeşil geçti — yani
davranış değil, ölçüm kırılgan. Test 2 ms'lik gizleme gecikmesi kuruyor ve
`Task.sleep(10 ms)` sonrasında kontrolün gizlenmiş olmasını bekliyor; paralel
koşan gerçek libmpv testleri makineyi meşgul ettiğinde bu pencere yetmiyor.
NEN-027 gerçek motorla koşan beş test daha ekledi, yani pencere bundan sonra
daha sık kaçırılacak.

Kırılgan test, kırık koddan daha pahalıdır: kırmızıyı "yine o test" diye
okumaya başlarsak testin söylediği hiçbir şeye güvenilmez.

## Kapsam

- `pinnedControlsSuspendAndRestoreAutomaticHiding` ve aynı desendeki komşuları
  (`pausedControlsRemainVisibleAfterUnpinning`, ...) uykuya değil **koşula**
  beklesin — mevcut `waitUntil` deseninin kabuk testlerindeki karşılığı.
- Gizleme gecikmesinin gerçek zamandan bağımsız sürülüp sürülemeyeceğine
  bakılsın: model zaten enjekte edilebilir bir saat tutuyor.

## YAPILMAYACAK

- Testi silmek veya `disabled` işaretlemek.
- Gizleme davranışının kendisini değiştirmek — ölçüm kırık, davranış değil.

## Kanıt (DoD)

- [ ] Testler yük altında da geçiyor: tam paket ardışık 10 koşumda yeşil
- [ ] Zamanlamaya bağlı sabit uyku kalmadı (grep kanıtı)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
