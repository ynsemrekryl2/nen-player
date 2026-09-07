---
id: NEN-065
title: Stabilize the pinned-controls timing test
milestone: M3
size: S
state: done
closed: 2026-08-31
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

- [x] Testler yük altında da geçiyor: tam paket ardışık 10 koşumda yeşil
- [x] Zamanlamaya bağlı sabit uyku kalmadı (grep kanıtı)

## Kanıt kaydı

### Önce — kusur deterministik hale gelmişti

2026-08-30 ölçümü, `NEN-067`'nin son kapısında:

| Koşum | Sonuç |
|---|---|
| `bash scripts/test-macos.sh` (paralel) | 107 test, **1 kırmızı**, 12,5 s |
| `swift test --package-path platforms/macos --no-parallel` | 107/107 yeşil, 33,2 s |

Kırmızı olan tek test `pinned controls stay visible and unpin restores the hide
timer`. Kayıtlı üç paralel koşum + bu koşum, dördü de aynı satırda kırmızı —
yani artık ara sıra kırılan bir test değil.

### Kusur: iki gerçek-saat uykusu yarışıyordu

`PlayerModel.scheduleControlsHide()` zamanlayıcıyı gerçek saatte
`Task.sleep(2 ms)` ile kuruyor, test ise gerçek saatte 10 ms bekleyip
gizlenmenin **çoktan olmuş** olmasını bekliyordu. Pay 5×, ve paralel koşumda
aynı makinede gerçek libmpv testleri var (`ContractTests` tek başına 12,5 s).
Davranış doğru, ölçüm kırıktı.

### Düzeltme: iddianın tipine göre iki farklı yol

İki iddianın **olumsuz** olduğu görüldü — "pin'liyken gizlenmiyor" ve "duraklıyken
gizlenmiyor". Uyku bunları zaten kanıtlayamaz, yalnız örnekler: yüklü makinede
eski 10 ms penceresi, zamanlayıcı *henüz çalışmadığı* için de geçebilirdi, ki bu
pin hakkında hiçbir şey söylemez. Bu ikisi `hideControlsNow()` doğrudan
çağrılarak yapısal hale getirildi — zamanlayıcının çağırdığı şeyin ta kendisi,
yani guard'ın kendisi sınanıyor. Kapsam bu yüzden **daralmadı, genişledi**.

Kalan tek **olumlu** iddia — "pin bırakılınca zamanlayıcı yeniden kuruluyor" —
5 saniye bütçeli koşula beklemeye çevrildi; aynı dosyada zaten iki örneği olan
desen (satır ~401 ve ~539).

Modelin enjekte edilebilir saatini gizleme zamanlayıcısına da bağlamak
incelendi ve **gerekmedi**: olumsuz iddialar yapısallaşınca ve olumlu iddia
koşula beklenince ürün kodunda değişiklik olmadan tam determinizm elde edildi.
Task'ın *YAPILMAYACAK*'ı da davranışa dokunulmamasını istiyordu.

### Sonra — 10 ardışık paralel koşum

`swift test --package-path platforms/macos` (paralel, NEN-067 çalışma ağacıyla
birlikte 107 test):

| Koşum | Sonuç |
|---|---|
| 1–7 | 107/107 yeşil, 14,4–16,6 s |
| 8 | **6 meşgul-döngü süreci altında** 107/107 yeşil, 16,6 s |
| 9–10 | 107/107 yeşil, 14,5 / 16,5 s |

Bu commit'in gösterdiği ağaç ayrıca **tek başına** doğrulandı (NEN-067'nin
çalışma ağacı stash'lenip yalnız bu değişiklik uygulanarak): `test-macos.sh`
dahil dört koşum, **102/102 yeşil**, 14,5–16,6 s.

### Grep kanıtı

`PlayerModelTests.swift`'te kalan üç `Task.sleep`'in üçü de deadline
döngüsünün *poll aralığı*; sabit zamanlama uykusu kalmadı:

```
265-        while model.controlsVisible, Date() < deadline {
266:            try await Task.sleep(nanoseconds: 1_000_000)
401-        while (second.playCount == 0 || model.playbackState != .playing), Date() < deadline {
402:            try await Task.sleep(nanoseconds: 1_000_000)
539-        while model.transientMessage != nil, Date() < deadline {
540:            try await Task.sleep(nanoseconds: 5_000_000)
```
