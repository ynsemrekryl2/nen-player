# M8 — AI Audio-Assisted Sync

> **İskelet.** Task kırılımı, bir önceki milestone kapanırken üretilir
> (bkz. `tasks/README.md` → "Yeni task açma").

## Amaç

Kullanıcının açık komutuyla, ses analizine dayanarak altyazıyı otomatik hizalamak. Sonuç **hiçbir güven seviyesinde** onaysız uygulanmaz.

## Kapsam

- Seçili audio track'ten gerekli segmentlerin alınması
- VAD → timestamp'li speech recognition → normalization
- Semantic/cross-lingual alignment, anchor üretimi, outlier eleme
- Modeller: constant offset · linear drift · piecewise mapping
- Confidence: high/medium/low/rejected
- Kullanıcı önizlemesi ve açık onay

## Kapsam dışı

- Kullanıcı komutu olmadan audio analizi — **yasak**
- Raw audio loglama — **yasak**
- Varsayılan olarak buluta ses göndermek — **yasak** (varsayılan `localOnly`)
- Protected/DRM audio extraction — **yasak**

## Çıkış kriterleri

- [ ] Auto-sync yalnız açık komutla başlıyor
- [ ] **High** confidence sonuç bile onaysız kalıcı uygulanmıyor
- [ ] Low/rejected sonuç otomatik uygulanmıyor
- [ ] Geçici audio dosyaları işlem sonrası siliniyor
- [ ] Remote gönderim yalnız açık izinle; varsayılan localOnly
- [ ] Farklı intro/recap içeren örnekte piecewise model devreye giriyor

## Task'lar

_(henüz kırılmadı)_

## Bağımlılıklar

M7 + M5

## Retro

<!-- kapanışta doldurulacak -->
