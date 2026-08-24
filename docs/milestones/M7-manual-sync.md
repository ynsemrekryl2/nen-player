# M7 — Manual Sync

> **İskelet.** Task kırılımı, bir önceki milestone kapanırken üretilir
> (bkz. `tasks/README.md` → "Yeni task açma").

## Amaç

Kullanıcının altyazıyı elle hizalayabilmesi: basit offset, replik temelli anchor ve iki anchor ile doğrusal drift düzeltmesi.

## Kapsam

- "Bu replik şimdi başlamalı" komutu → `offset = currentPlaybackTime - cue.startTime`
- İki anchor → `renderTime = cueTime × rate + offset`
- SyncProfile (media fingerprint + audio track identity + timeline fingerprint)
- Undo, reset, preview, restart sonrası persistence
- Gerekirse custom overlay renderer

## Kapsam dışı

- Otomatik/sesli senkronizasyon → M8
- Orijinal cue zamanlarını değiştirmek — **yasak**
- Tam subtitle editörü — non-goal

## Çıkış kriterleri

- [ ] Seçilen replik komuttan sonra doğru anda başlıyor
- [ ] İki anchor ile drift düzeliyor (ölçülmüş sapma)
- [ ] Orijinal cue zamanları **değişmemiş** (fingerprint aynı)
- [ ] Aynı timeline'dan türeyen AI artifact **aynı** SyncProfile'ı kullanıyor
- [ ] Undo/reset çalışıyor; profil restart sonrası korunuyor

## Task'lar

_(henüz kırılmadı)_

## Bağımlılıklar

M3 (renderer + playback pozisyonu)

## Retro

<!-- kapanışta doldurulacak -->
