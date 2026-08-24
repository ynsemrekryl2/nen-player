---
id: NEN-017
title: Indexed cue lookup
milestone: M2
size: M
state: backlog
depends_on: [NEN-016]
blocks: [NEN-027]
adr: [7]
---

# NEN-017 — Indexed cue lookup

## Sonuç

Zamandan cue'ya erişim, doküman büyüklüğünden bağımsız olarak logaritmik sürede
doğru sonucu verir.

## Kapsam

- `CueIndex`: binary search veya boundary-event yapısı
- `activeCue(at:)` ve `cues(range:)`
- Kenar durumlar: boş doküman, tek cue, örtüşen cue'lar, boşluk (cue yok) anları
- Benchmark: 50k cue

**Eşik:** lookup logaritmik; lineer taramaya karşı ölçüm tablosu zorunlu.

## YAPILMAYACAK

- Renderer entegrasyonu → NEN-027
- Sync offset uygulaması → M7 (index ham zamanlar üzerinde çalışır)

## Kanıt (DoD)

- [ ] 10.000 rastgele seek'te index sonucu lineer tarama sonucuyla **birebir** aynı
- [ ] Benchmark: 50k cue'da lookup süresi + lineer taramayla karşılaştırma tablosu
- [ ] Kenar durumlar: boş doküman, tek cue, örtüşen cue, cue'suz an

## Kanıt kaydı

<!-- done olurken doldurulacak -->
