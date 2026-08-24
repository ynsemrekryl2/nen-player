# M9 — Windows / Linux

> **İskelet.** Task kırılımı, bir önceki milestone kapanırken üretilir
> (bkz. `tasks/README.md` → "Yeni task açma").

## Amaç

Aynı shared core'un Windows ve Linux'ta libmpv adapter'ı ve platform secure storage ile çalışması.

## Kapsam

- libmpv adapter (mevcut contract kitini geçerek)
- Windows Credential Manager, Linux Secret Service
- Platform dosya seçici ve UI kabuğu

## Kapsam dışı

- Yeni contract testi yazmak — mevcut kit çalıştırılır
- Platform-özgü özellik eklemek

## Çıkış kriterleri

- [ ] libmpv adapter'ı **mevcut** contract kitini her iki platformda geçiyor
- [ ] Secret'lar platform secure storage'ında
- [ ] macOS slice'ının beş kabul maddesi her iki platformda geçerli

## Task'lar

_(henüz kırılmadı)_

## Bağımlılıklar

M3

## Retro

<!-- kapanışta doldurulacak -->
