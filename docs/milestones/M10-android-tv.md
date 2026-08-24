# M10 — Android / Android TV

> **İskelet.** Task kırılımı, bir önceki milestone kapanırken üretilir
> (bkz. `tasks/README.md` → "Yeni task açma").

## Amaç

Android ve Android TV desteği. Stremio external-player senaryosunun **asıl evi** burasıdır; gerçek cihaz testi zorunludur.

## Kapsam

- Media3 (veya ADR-0025 kararı) playback adapter'ı
- ACTION_VIEW external-player Intent: video MIME, http/https/content URI, başlangıç pozisyonu, geçici URI izinleri
- Playback sonucu ve son pozisyonun Stremio'ya döndürülmesi
- TV navigasyonu (D-pad, focus)
- Keystore destekli encrypted credential storage

## Kapsam dışı

- Media URI veya token içeren extras'ı loglamak — **yasak**
- Emülatörle yetinmek — gerçek cihaz testi zorunlu

## Çıkış kriterleri

- [ ] Stremio'dan açılan medya gerçek Android TV cihazında doğru pozisyondan oynuyor
- [ ] Son pozisyon Stremio'ya dönüyor
- [ ] Negatif: extras/URI **loglanmıyor**
- [ ] TV'de altyazı menüsü D-pad ile kullanılabiliyor
- [ ] Büyük cue listesinde UI donmuyor (NEN-008 eşikleri TV donanımında)

## Task'lar

_(henüz kırılmadı)_

## Bağımlılıklar

M1 + M3 + M4

## Retro

<!-- kapanışta doldurulacak -->
