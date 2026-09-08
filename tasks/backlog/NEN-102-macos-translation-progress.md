---
id: NEN-102
title: macOS translation progress and cancellation surface
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-101]
blocks: [NEN-104]
adr: []
---

# NEN-102 — macOS translation progress and cancellation surface

## Sonuç

Koşan bir çeviri işinin ilerlemesi ekranda görünüyor, kullanıcı onu iptal
edebiliyor ve iptal edilen iş ekranda yarım bir sonuç bırakmıyor.

## Bağlam

Şartname §10 progressive/yarım yayını **yasaklıyor** — yani ilerleme
göstergesi kullanıcıya "ne kadarı bitti"yi söyler, ama ekranda çizilen altyazıyı
blok blok değiştirmez. Bu ayrım bu task'ın en kolay kaçırılan yeri.

Geçici bildirim yüzeyi `PlaybackPresentation` üzerinden zaten var (`NEN-048`,
`NEN-050`), hata sunumu `ADR-0031` ile karara bağlı. Krom `NEN-067`'nin ultra
ince tasarımı — yeni bir kalıcı çubuk eklenmez.

## Kapsam

- İlerleme göstergesi (blok/yüzde) ve iptal eylemi
- İşin bitişinde sonucun hedef dil grubunda menüye düşmesi
- Hata durumunun `ADR-0031`'in sunum kurallarına bağlanması

## YAPILMAYACAK

- Çevrilen metnin blok blok ekrana yansıtılması — **yasak** (§10)
- Kullanıcı başka kaynak izlerken zorla AI çıktısına geçme — **yasak** (§9)
- Arka planda birden fazla iş kuyruğu — kapsam dışı
- Yeni kalıcı krom öğesi — `NEN-067`'nin tasarımı korunur

## Kanıt (DoD)

- [ ] Swift testi: ilerleme olayları göstergeye sırayla yansıyor
- [ ] Swift testi: iptal eylemi işi durduruyor ve gösterge kayboluyor
- [ ] Negatif: iş yarıdayken ekrandaki altyazı **değişmiyor** (progressive yayın yok)
- [ ] Negatif: iptal edilen iş menüye hiçbir `Ai` kaynağı eklemiyor
- [ ] Gerçek `.app` checklist: ilerleme görülüyor, iptal ediliyor, ekran ve menü temiz kalıyor — `evidence/M5/NEN-102-checklist.md`

## Kanıt kaydı

<!-- done olurken doldurulacak -->
