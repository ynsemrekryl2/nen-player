---
id: NEN-050
title: Cover the recent-media store's transient error paths
milestone: M3
size: S
state: backlog
depends_on: [NEN-048]
blocks: []
adr: [31]
---

# NEN-050 — Cover the recent-media store's transient error paths

## Sonuç

Son açılan medya deposundan türeyen üç geçici bildirim yolunun testi vardır.

## Kapsam

- `MemoryRecentStore`'a hata enjeksiyonu (`save` ve `resolve` throw edebilsin)
- Üç yolun testlenmesi:
  - `openMedia` içinde `recentStore.save` hata verirse
    `Son açılan medya kaydedilemedi.` çıkıyor **ve medya yine de yükleniyor**
  - `openRecentMedia` içinde `resolve()` `nil` dönerse depo temizleniyor ve
    `Son açılan medya artık kullanılamıyor.` çıkıyor
  - `resolve()` throw ederse aynı davranış
- Bu metinlerin de kapalı küme negatif testine dahil edilmesi

## YAPILMAYACAK

- Metinleri değiştirmek — kapalı küme aynı kalır
- Son medya listesi UI'ı → `NEN-042`
- Bookmark saklama mekanizmasını değiştirmek

## Neden ayrı task

`NEN-048` motor hatasından (`FfiPlaybackError`) türeyen geçici sınıfı düzeltip
kanıtladı. Aynı `presentTransient` yüzeyini kullanan fakat kaynağı son-medya
deposu olan üç yol o task'ın kapsamında değildi ve testsiz kaldı;
`evidence/M3/NEN-048-checklist.md` bunu açıkça kaydediyor.

## Kanıt (DoD)

- [ ] Üç yolun her biri için geçici bildirim üreten test
- [ ] `save` hatasında medyanın yine de yüklendiğini gösteren test
- [ ] Negatif: bu üç metin de yol, motor adı ve sayısal kod içermiyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
