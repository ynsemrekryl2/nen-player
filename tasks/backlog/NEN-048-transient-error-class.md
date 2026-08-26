---
id: NEN-048
title: Fix the spurious transient error and prove the transient error class
milestone: M3
size: S
state: backlog
depends_on: [NEN-024]
blocks: []
adr: [31]
---

# NEN-048 — Fix the spurious transient error and prove the transient error class

## Sonuç

Geçici hata bildirimi yalnız kullanıcının reddedilen bir eyleminden sonra
çıkar, ve bu davranışın testi vardır.

## Kapsam

- `resynchronize()`'ın medya yokken geçici bildirim üretmemesi — resync
  kullanıcının eylemi değildir, reddedilmesi de haber değildir
- Reddedilen kullanıcı eylemlerinin (seek, ses, oynat/durakla) geçici
  bildirim üretmeye **devam etmesi**
- `FakeSession`'a hata enjeksiyonu eklenmesi ve geçici sınıfın testlenmesi:
  mesaj kapalı kümeden geliyor, yol/motor adı/kod taşımıyor, fatal yüzeyi
  açmıyor, kendiliğinden temizleniyor
- `PlaybackPresentation.errorMessage`'ın her `FfiPlaybackError` varyantı için
  ayrı metin döndürdüğünün testlenmesi

## YAPILMAYACAK

- Yeni bir hata sınıfı veya yeni yüzey — ADR-0031 Karar 1 üç sınıfta karar kıldı
- Metinlerin yeniden yazılması — kapalı küme aynı kalır
- Kaynak düzeyi hatalar → `NEN-026`
- `EventsLost` davranışının değiştirilmesi — sessiz resync doğru (Karar 3)

## Neden ayrı task

`NEN-024` incelemesinde çalışan `.app` üzerinde görüldü: uygulama medya
yokken **her öne geldiğinde** boş durumda `Önce bir medya açın.` bildirimi
çıkıyor — kullanıcı hiçbir şey yapmamış olmasına rağmen. Sebep zincirinin
tamamı okundu: `applicationBecameActive()` → `resynchronize()` →
`positionMs()` medya yokken `requireMedia()` üzerinden `NotLoaded` atıyor →
`catch` bunu `presentTransient` ile bildirime çeviriyor.

Bu doğrudan ADR-0031 Karar 1'e aykırı: geçici sınıfın tanımı "kullanıcının
doğrudan eylemi reddedildi". Kusurun görülmemiş olmasının sebebi de aynı
yerde: `FakeSession` hiçbir metodunda throw etmiyor, dolayısıyla dört
`presentTransient` yolunun hiçbiri testte çalışmıyor ve
`evidence/M3/NEN-024-checklist.md` içinde geçici bildirimle ilgili tek madde
yok. Fatal sınıf kanıtlı, geçici sınıf kanıtsız kapandı.

## Kanıt (DoD)

- [ ] Medya yokken uygulama arka plana alınıp öne getirildiğinde hiçbir
      bildirim çıkmıyor (checklist)
- [ ] Uygulama ilk açılışta boş durumda bildirimsiz duruyor (checklist)
- [ ] Reddedilen seek/ses/oynat eylemi geçici bildirim üretiyor ve fatal
      yüzeyi **açmıyor** (test)
- [ ] Geçici mesaj yol, motor adı ve sayısal kod içermiyor (test — negatif)
- [ ] Geçici mesaj kendiliğinden temizleniyor (test)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
