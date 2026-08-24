# M5 — Translation Core

> **İskelet.** Task kırılımı, bir önceki milestone kapanırken üretilir
> (bkz. `tasks/README.md` → "Yeni task açma").

## Amaç

AI çeviri pipeline'ının tamamı: bağlam analizi, overlapping blok çevirisi, sıkı yerel doğrulama, checkpoint/cancellation ve kalıcı artifact. Bu milestone **mock provider** ile tamamlanır — gerçek sağlayıcılar M6'da.

## Kapsam

- Blok stratejisi (varsayılan 40, izin 30–60, overlap 6)
- Strict validation: exact cue count, izin verilen/unique ID, non-empty text, sıra normalizasyonu
- En fazla 2 targeted repair + 1 full-block retry
- Yalnız doğrulanmış blok checkpoint
- ValidatedSubtitleArtifact + cache identity + atomik commit
- SQLite index + content-addressed store (ADR-0017)
- Deterministic mock provider

## Kapsam dışı

- Gerçek provider entegrasyonu → M6
- Secure credential storage → M6
- Progressive/yarım subtitle yayını — **yasak**
- Kaynak seçiminin çeviri başlatması — **yasak**

## Çıkış kriterleri

- [ ] Uçtan uca: kaynak seçimi → açık çeviri komutu → doğrulanmış artifact
- [ ] Yarım/progressive çıktı hiçbir koşulda yayınlanmıyor
- [ ] İptal sonrası **late commit yok**
- [ ] Cache identity bileşenlerinden biri değişince eski artifact **kullanılmıyor**
- [ ] Cue ID/sıra/zamanlar girdiyle birebir aynı
- [ ] Çeviri sırasında kaynak değişimi işi retarget **etmiyor**

## Task'lar

_(henüz kırılmadı)_

## Bağımlılıklar

M2 + M3

## Retro

<!-- kapanışta doldurulacak -->
