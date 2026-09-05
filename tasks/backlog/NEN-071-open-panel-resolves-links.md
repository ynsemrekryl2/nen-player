---
id: NEN-071
title: Decide what the subtitle file picker does with a link
milestone: M3
size: S
state: backlog
depends_on: [NEN-025]
blocks: []
adr: [31]
---

# NEN-071 — Altyazı seçicisinin kısayolla ne yapacağına karar verilir

## Sonuç

`Altyazı Dosyası Yükle…` panelinden seçilen bir kısayolun (symlink / alias)
nasıl ele alındığı **bilinçli bir karardır** ve kararın gerektirdiği kanıt
vardır — bugünkü gibi platformun varsayılanına bırakılmış değildir.

## Gözlem (NEN-028 kabul koşusu, 2026-09-05)

`NSOpenPanel` kısayolu **kendisi çözüyor**: panele `Kisayol2.srt` (hedefi
`Baska.srt` olan bir symlink) seçildiğinde uygulamaya gelen yol hedefin
yoluydu ve menüde `Baska.srt` satırı belirdi. Yani:

- `NEN-025`'in symlink kapısı bu yüzeyden **hiç çağrılmıyor**; kapının ürettiği
  `Bu bir kısayol; altyazı olarak açılamıyor.` metni panel üzerinden
  erişilemez durumda.
- Kapının kendisi sağlam: aynı symlink medyanın yanında sidecar olarak
  durduğunda **sessizce reddediliyor** (`evidence/M3/NEN-028-symlink-refused.jpg`)
  — tehdit modelinin asıl hâli budur, çünkü orada yolu uygulamanın kendisi
  türetir.

## Karar verilecek

Kullanıcının **açıkça seçtiği** bir kısayol için doğru davranış hangisi:

1. Bugünkü hâl — platform çözer, hedef dosya normal bir kullanıcı dosyası
   olarak kataloğa girer. Kullanıcının kendi seçimi; sürpriz yok, red yok.
   Gerekçe belgelenir, `explicitRefusalIsAnnounced` testinin hangi yüzeyi
   temsil ettiği netleşir.
2. `panel.resolvesAliases = false` — kapı çağrılır, kullanıcı reddi görür.
   Güvenlik açısından ek bir şey kazandırmaz (yolu kullanıcı verdi), okunabilir
   bir dosya için ret üretir.

## YAPILMAYACAK

- Sidecar taramasının symlink kapısını gevşetmek — orada red doğrudur ve
  `NEN-025`'in negatif testleriyle korunuyor.
- Karar verilmeden panel bayrağını değiştirmek.

## Kanıt (DoD)

- [ ] Karar yazılı: hangi seçenek, neden (gerekirse ADR-0031'e not)
- [ ] Seçilen davranışı **ürün yüzeyinde** doğrulayan bir test —
      bugünkü boşluk tam olarak buydu
- [ ] `bash scripts/test-macos.sh` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
