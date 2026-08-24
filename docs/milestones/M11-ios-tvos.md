# M11 — iOS / tvOS

> **İskelet.** Task kırılımı, bir önceki milestone kapanırken üretilir
> (bkz. `tasks/README.md` → "Yeni task açma").

## Amaç

iOS ve tvOS desteği; AVPlayer adapter'ı ve Apple timed-text renderer.

## Kapsam

- AVPlayer/AVKit playback adapter'ı (mevcut contract kitiyle)
- Apple timed-text subtitle renderer
- Sandbox dosya erişimi ve security-scoped bookmark
- Keychain credential storage

## Kapsam dışı

- Yeni contract testi yazmak
- Sideload/dağıtım süreçleri → S1 kararına bağlı

## Çıkış kriterleri

- [ ] AVPlayer adapter'ı **mevcut** contract kitini geçiyor
- [ ] Capability farkları (ör. external subtitle injection) typed error ile ele alınıyor
- [ ] macOS slice'ının beş kabul maddesi iOS/tvOS'ta geçerli

## Task'lar

_(henüz kırılmadı)_

## Bağımlılıklar

M3

## Retro

<!-- kapanışta doldurulacak -->
