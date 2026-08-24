---
id: NEN-027
title: SubtitleRenderer port and libmpv injection adapter
milestone: M3
size: M
state: backlog
depends_on: [NEN-017, NEN-026]
blocks: [NEN-028]
adr: [13]
---

# NEN-027 — SubtitleRenderer port and libmpv injection adapter

## Sonuç

Seçilen altyazı video üzerinde görünür ve seek sonrası doğru cue **anında**
gösterilir.

## Kapsam

- `SubtitleRenderer` portu ve capability'leri
- libmpv external subtitle injection adapter'ı
- `CueIndex` (NEN-017) ile seek sonrası anlık doğru cue
- UI'ın renderer implementasyonunu **bilmemesi**

## YAPILMAYACAK

- Custom overlay renderer → manuel sync/inspector ihtiyacında (M7)
- Stil/konum ayarları
- Sync offset uygulaması → M7

## Kanıt (DoD)

- [ ] Seçim anında altyazı görünüyor (ekran kaydı)
- [ ] Rastgele seek sonrası gösterilen cue, `CueIndex` sonucuyla birebir aynı
- [ ] Cue'suz bir ana seek → altyazı gösterilmiyor (boş, artık cue kalmıyor)
- [ ] UI kodunda renderer implementasyon adı geçmiyor (grep kanıtı)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
