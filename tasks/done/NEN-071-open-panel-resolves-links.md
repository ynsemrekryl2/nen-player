---
id: NEN-071
title: Decide what the subtitle file picker does with a link
milestone: M3
size: S
state: done
closed: 2026-09-07
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

## Karar

Kullanıcının **açıkça seçtiği** bir kısayol için doğru davranış hangisi:

**Seçenek 1 kabul edildi:** panel kısayolu çözer, hedef dosya normal bir
kullanıcı dosyası olarak kataloğa girer. Kullanıcının kendi seçimi olduğu için
sürprizsizdir; güvenlik kapısı uygulamanın kendisinin keşfettiği veya doğrudan
kapıya ulaşan symlink yollarında aynen kalır. `NSOpenPanel`'in varsayılanına
bağlanmamak için `resolvesAliases = true` açıkça ayarlanır.

Seçenek 2 (`panel.resolvesAliases = false`) reddedildi: kullanıcı tarafından
açıkça seçilmiş, okunabilir hedefi engeller ve güvenlik açısından ek bir kazanım
sağlamaz.

## YAPILMAYACAK

- Sidecar taramasının symlink kapısını gevşetmek — orada red doğrudur ve
  `NEN-025`'in negatif testleriyle korunuyor.
- Karar verilmeden panel bayrağını değiştirmek.

## Kanıt (DoD)

- [x] Karar yazılı: hangi seçenek, neden (ADR-0031'e not)
- [x] Seçilen davranışı **ürün yüzeyinde** doğrulayan bir test —
      bugünkü boşluk tam olarak buydu
- [x] `bash scripts/test-macos.sh` yeşil

## Kanıt kaydı

2026-09-07'de gerçek `.app` fixture koşusu `evidence/M3/NEN-071-checklist.md`
ile kaydedildi: kullanıcı `Kisayol.srt` symlink'ini `⇧⌘O` panelinden seçti;
uygulama reddetmeden oynatmaya döndü ve menüde `Kullanıcı Altyazıları 1`
göründü. Özel fixture yolu ve cue metni kanıta yazılmadı.

Otomatik kanıt:

- `bash scripts/test-macos.sh` — Swift paketi **198 test / 22 suite geçti**.
- `bash scripts/build-macos-app.sh` — ad-hoc `NenPlayer.app` üretildi.
- `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app` —
  paket geçerli ve Designated Requirement'ı karşılıyor.
- `PlayerModelTests.subtitlePanelResolvesAliasesExplicitly`, paneli önce
  `resolvesAliases = false` yapıp yapılandırmadan sonra `true` olduğunu
  doğruluyor; atama kaldırılırsa test kırmızı olur.
- `directSymlinkRefusalIsAnnounced` ve sidecar-symlink negatifleri yeşil;
  uygulamanın keşfettiği veya doğrudan aldığı symlink yolları reddedilmeye
  devam ediyor.
