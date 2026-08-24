---
id: NEN-028
title: macOS vertical slice acceptance
milestone: M3
size: S
state: backlog
depends_on: [NEN-027]
blocks: []
adr: []
---

# NEN-028 — macOS vertical slice acceptance

## Sonuç

"Dosya aç → oynat → katalog → seç → ekranda göster" akışı uçtan uca çalışır ve
beş kabul maddesi kanıtlanmıştır.

## Kapsam

- `docs/milestones/M3-macos-slice.md` içine adım adım manuel test senaryosu
- Uçtan uca çalıştırma ve ekran kaydı
- M3 retro'sunun yazılması

## YAPILMAYACAK

- Yeni özellik eklemek — bu bir **doğrulama** task'ı
- M4+ kapsamındaki hiçbir şey

## Kanıt (DoD)

- [ ] Medya, katalog taraması bitmeden oynuyor
- [ ] Bozuk bir `.srt` playback'i **durdurmuyor**, yalnız o kaynağı hatalı işaretliyor
- [ ] Menüde aynı kaynak iki kez yok; dili bilinmeyen `Dil Belirsiz` grubunda;
      `Kapalı` her zaman var
- [ ] Symlink ve path-traversal ile verilen altyazı **reddediliyor**
- [ ] Seek sonrası doğru cue anında görünüyor (NEN-017 benchmark'ıyla birlikte)
- [ ] Ekran kaydı `evidence/M3/` altında kayıtlı

## Kanıt kaydı

<!-- done olurken doldurulacak -->
