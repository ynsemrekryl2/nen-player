---
id: NEN-048
title: Fix the spurious transient error and prove the transient error class
milestone: M3
size: S
state: done
closed: 2026-08-26
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

- [x] Medya yokken uygulama arka plana alınıp öne getirildiğinde hiçbir
      bildirim çıkmıyor (checklist)
- [x] Uygulama ilk açılışta boş durumda bildirimsiz duruyor (checklist)
- [x] Reddedilen seek/ses/oynat eylemi geçici bildirim üretiyor ve fatal
      yüzeyi **açmıyor** (test)
- [x] Geçici mesaj yol, motor adı ve sayısal kod içermiyor (test — negatif)
- [x] Geçici mesaj kendiliğinden temizleniyor (test)

## Kanıt kaydı

Kanıt: `evidence/M3/NEN-048-checklist.md`.

**Düzeltme.** `resynchronize()` artık hatayı bildirime çevirmiyor; payload'suz
bir `debug` satırı bırakıyor. Resync kullanıcının eylemi değildir, dolayısıyla
reddedilmesi ADR-0031 Karar 1'in geçici sınıfına girmez. `guard hasMedia`
eklemekle yetinilmedi — o yalnız görülen semptomu kapatır, sınıfı düzeltmez.
Kullanıcı eyleminden türeyen üç yol (`seek`, `setVolume`, `togglePlayback`)
olduğu gibi duruyor. Geçici bildirimin ömrü `init` parametresi oldu
(`transientMessageDurationNanoseconds`, varsayılan 3 sn) — "kendiliğinden
temizleniyor" testi aksi halde 3 sn uyumak zorundaydı.

**Test tohumu.** `FakeSession` çağrı bazında hata enjekte edebiliyor
(`errors[.seek] = .NotLoaded`). NEN-024'te bu sınıfın kanıtsız kalmasının
sebebi buydu: fake hiçbir metodunda throw etmiyordu.

**Otomatik kanıt.** `swift test --package-path platforms/macos --no-parallel`
çıkış 0, art arda 2/2 — **39 test / 6 suite**, 0 failure (31 → 39, sekiz yeni
test). Paralel koşuların birinde `NEN-049`'un bilinen izolasyon kusuru
tekrarlandı (`aSeekIsAnsweredThroughTheSession`, seri 2/2 yeşil · paralel 3/4
yeşil); o test başka bir hedefte gerçek libmpv ile çalışıyor ve bu task'ın
sekiz testi her koşuda yeşildi. Ayrıntı kanıt dosyasında.

**Testlerin boş olmadığı iki kez sınandı.** (1) Düzeltme geri alınınca iki
sessizlik testi tam olarak bildirilen semptomu üretti:
`(model.transientMessage → "Önce bir medya açın.") == nil` başarısız. (2)
`errorMessage`'a bilinçli sızıntı (`"Medya oynatılamadı (mpv 4242)."`)
enjekte edilince negatif test hem geçici hem fatal metinde yakaladı. Her iki
sonda da değişiklik geri alındı; `PlaybackPresentation.swift` üründe
değişmedi.

**Manuel acceptance 4/4.** Ad-hoc imzalı `.app` üzerinde: ilk açılışta boş
durum bildirimsiz; Finder ile arka plana alınıp başlık çubuğuyla yeniden
etkinleştirildiğinde hem hemen ardından hem 1 sn sonra bildirimsiz.

**Kapsamda bırakılan boşluk.** Son-medya deposundan türeyen üç geçici yol
(`recentStore.save`/`resolve`) hâlâ testsiz — bu task motor hatasından
türeyen sınıfı hedefliyordu. `NEN-050` olarak backlog'a alındı.
