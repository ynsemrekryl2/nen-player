---
id: NEN-027
title: SubtitleRenderer port and libmpv injection adapter
milestone: M3
size: M
state: blocked
depends_on: [NEN-017, NEN-026, NEN-066]
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

## Engel

**Engelleyen:** `NEN-066` — motorun çizdiğini bildirdiği replik gerçek `.app`in
penceresinde görünmüyor.

Bu task'ın kodu yerinde ve ölçülü: port, engine-native adapter, session yolu,
FFI ve kabuk sadeleştirmesi bitti; Rust tarafında 4 000 momentlik parite
sweep'i, Swift tarafında gerçek libmpv üzerinde 28 moment ve paylaşılan
contract kiti geçiyor. DoD'nin ikinci, üçüncü ve dördüncü maddeleri
karşılandı.

Karşılanamayan tek madde **"seçim anında altyazı görünüyor"**. Gerçek `.app`te
belge motora ulaşıyor, seçiliyor ve motor o an çizdiğini bildiriyor
(`sub-text` dolu) — ama pencerede hiçbir şey yok. Aynı kusur **gömülü** track'i
de vuruyor, yani enjeksiyon yolundan eski ve ondan geniş; bu yüzden ayrı bir
task'a ayrıldı. Ölçümün tamamı: `evidence/M3/NEN-027-checklist.md`.

`NEN-066` kapanınca bu task'ın kalan işi yalnız görsel koşuyu yürütüp kanıt
kaydını doldurmaktır.

## Kanıt kaydı

<!-- done olurken doldurulacak -->
