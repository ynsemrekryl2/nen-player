---
id: NEN-022
title: libmpv playback adapter for macOS
milestone: M3
size: L
state: backlog
depends_on: [NEN-021, NEN-004]
blocks: [NEN-023, NEN-024]
adr: [11, 12]
---

# NEN-022 — libmpv playback adapter for macOS

## Sonuç

Gerçek libmpv adapter'ı, fake adapter ile aynı contract test kitini geçer.

## Kapsam

- libmpv bağlama ve yaşam döngüsü (init/shutdown)
- load · play · pause · stop · absolute/relative seek · position · duration ·
  rate · volume
- State eşlemesi: buffering / ready / ended / failure
- Event stream

## YAPILMAYACAK

- Track enumeration/seçimi → NEN-023
- Altyazı render → NEN-027
- Dağıtım/notarization/lisans kararı → ADR-0012 (bu task'ta yalnız geliştirme
  ortamında çalışır hale getirilir; **karar ayrı**)

## Kanıt (DoD)

- [ ] Aynı contract kiti gerçek libmpv ile geçiyor
- [ ] Yerel klip fixture'ıyla oynat/duraklat/seek doğrulandı
- [ ] Duration'ın ötesine seek → `ended` state (kenar durum)
- [ ] Bozuk/eksik dosyada `failure` state, panik yok
- [ ] Negatif: adapter loglarında medya yolu/URL **yok**

## Kanıt kaydı

<!-- done olurken doldurulacak -->
