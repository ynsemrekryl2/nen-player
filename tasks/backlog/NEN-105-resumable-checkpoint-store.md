---
id: NEN-105
title: Resumable checkpoint store for interrupted translation runs
milestone: M6
size: M
state: backlog
closed:
depends_on: [NEN-096]
blocks: []
adr: [0017]
---

# NEN-105 — Resumable checkpoint store for interrupted translation runs

## Sonuç

Çeviri ortasında kapanan uygulama yeniden açıldığında `NEN-093`'ün ürettiği
`BlockCheckpoints`'i diskten okuyup kaldığı bloktan devam ediyor; hiçbir
koşulda yarıda kalmış bir resume kaydı okunabilir bir artifact olarak
görünmüyor.

## Bağlam

ADR-0017 Karar 5 (2026-09-09, `NEN-095`) checkpoint'in kalıcı olacağını
kararlaştırdı ama implementasyonu bilerek M6'ya erteledi: M5 mock provider ile
çalışıyor, baştan başlamanın maliyeti bugün ölçülemeyecek kadar düşük. M6'da
gerçek sağlayıcı geldiğinde yarıda kalan bir çeviri gerçek kredi/kota kaybı
demek — erteleme burada biter.

`NEN-093`'ün `BlockCheckpoints::into_completed`'ı bugün "her blok
checkpoint'li değilse `CompletedBlocks` üretilmez" değişmezini tip düzeyinde
zaten garanti ediyor; bu task o garantiyi bozmadan checkpoint'lerin **diske**
yazılmasını ekliyor.

## Kapsam

- `BlockCheckpoints`'in ADR-0017 Karar 5'in tanımladığı **ayrı resume
  alanına** (artifact deposunun dışında) yazılması ve okunması
- Aynı çeviri run'ının kimliğiyle (cache identity, `NEN-097`) resume alanının
  eşleştirilmesi
- Yeniden başlatmada zaten checkpoint'li blokların provider'a hiç
  gönderilmemesi (mevcut bellek-içi davranışın disk üzerinde de korunması)
- Tamamlanan run'ın resume kaydının temizlenmesi (artifact commit edildikten
  sonra resume alanı çöp bırakmaz)

## YAPILMAYACAK

- Artifact deposunun kendisi — `NEN-096` (zaten done/yapılıyor)
- Kullanıcıya "yarım çeviriler" listesi veya UI — M6/UI işi, burada yalnız
  API sınırı
- Cache identity hesabı — `NEN-097` (zaten var)

## Kanıt (DoD)

- [ ] Unit: kaydedilen `BlockCheckpoints` aynı run kimliğinden birebir
      okunuyor (resume)
- [ ] Unit: zaten checkpoint'li bloklar yeniden başlatmada provider'a
      gitmiyor (çağrı sayacıyla ölçülür)
- [ ] Negatif (zorunlu — security/validation satırı): yarıda kesilmiş bir
      resume yazımı sonrası **okunabilir hiçbir kısmi artifact yok** — ne
      resume alanından ne artifact deposundan
- [ ] Negatif: atomik yazım düz yazımla değiştirildiğinde yukarıdaki test
      kırmızıya dönüyor — kontrol sağır değil
- [ ] Unit: artifact commit edildikten sonra resume kaydı temizleniyor
- [ ] Guard: resume alanının dosya yolu hiçbir log yüzeyine düşmüyor (K23 #3)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
