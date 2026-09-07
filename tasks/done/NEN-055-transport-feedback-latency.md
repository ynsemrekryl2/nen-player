---
id: NEN-055
title: A transport command is shown without waiting for the next poll tick
milestone: M3
size: S
state: done
closed: 2026-08-27
depends_on: [NEN-053]
blocks: []
adr: [31, 33]
---

# NEN-055 — A transport command is shown without waiting for the next poll tick

## Sonuç

Play/pause ikonu ve konum, komut verildiği anda döner; kabuk elinin altında
hazır bekleyen olayı bir sonraki poll tik'ine kadar bekletmez.

## Kapsam

- `togglePlayback` ve `seek(to:)` komutu verdikten hemen sonra oturum
  olaylarını bir kez boşaltır
- Aynı davranışın klavye yolunda da geçerli olması (kısayollar bu iki
  fonksiyona giriyor)

## YAPILMAYACAK

- Poll aralığını kısaltmak — CPU'yu artırmadan kazanılacak gecikme bu değil
- Pump aralığına dokunmak → ADR-0033 Karar 2, aralık çekirdekte
- Durumu iyimser yazmak (motorun söylemediği bir durumu göstermek) — gerek
  yok, olay zaten hazır bekliyor
- Ses düzeyi yolu → `NEN-054`

## Neden ayrı task

Bildirilen semptom: play/pause bazen gecikmeli tepki veriyor. Ölçülmeden önce
sanılan neden iki duraklı olay yolunun toplamıydı (33 ms pump + 50 ms poll).
Kod okunduğunda öyle olmadığı görüldü: `ShellEngineBridge` **her komuttan
sonra** `pull()` çağırıyor ve `MPVPlaybackEngine.pause()` durum olayını
dönmeden önce kuyruğa koyuyor. Yani komut dönerken olay Rust kuyruğunda hazır;
gecikme yalnız kabuğun kendi 50 ms poll'u, çünkü kabuk elinin altındaki olaya
bir sonraki tik'e kadar bakmıyor.

`NEN-053`'ten sonra yapılır: anında boşaltma seek'in hemen ardından bayat
`positionChanged` olaylarını da çeker, guard yoksa ışınlanma daha da erken
görünür.

## Kanıt (DoD)

- [x] `FakeSession` gerçekteki gibi play/pause'da durum olayını kuyruğa koyuyor
- [x] `togglePlayback` çağrısından sonra poll beklemeden `isPlaying` dönüyor
      (test)
- [x] Negatif: anında boşaltma kaldırılınca bu test kırmızı
- [x] Elle: boşluk tuşu ve düğme, ikon tıklamayla aynı anda dönüyor (checklist)

## Kanıt kaydı

Tam kayıt: `evidence/M3/NEN-055-checklist.md`.

**Teşhis uygulanmadan önce ölçüldü** (`NEN-053`'te teşhisin ölçümle değişmesi
üzerine). Geçici ölçüm testi (commit edilmedi) gerçek libmpv oturumunda
komuttan hemen sonra drain etti: durum olayı **50–180 mikrosaniye** sonra
görülebiliyor. İddia doğrulandı — gecikmede motorun ve çekirdeğin payı yok,
kabuk elindeki cevaba bir sonraki poll tik'ine kadar bakmıyordu. Poll aralığı
bu yüzden değiştirilmedi.

`bash scripts/test-macos.sh` çıkış 0: **48 test / 6 suite**, 0 failure.
Yeni test: `the icon turns on the click, not on the next poll tick`.

**Negatif kontrol semptomun ikinci yüzünü ortaya çıkardı.** `drainSessionEvents()`
kaldırılınca test iki beklentiyle kırmızı: durum `.playing` kalıyor **ve**
`playCount` 2 yerine 1. İkincisi şu demek: durum geç döndüğü için ikinci
tıklama `togglePlayback`'i yanlış dalda buluyor ve duraklatılmış medyayı
oynatmak yerine yeniden duraklatmaya çalışıyor. Gecikme yalnız görsel değildi.

**Kayda geçen kırmızı koşu:** drain geri konduktan sonraki ilk tam koşu tek bir
issue ile kırmızı oldu, ardından 4 koşu 4/4 yeşil. Hangi test olduğu
yakalanamadı; `test-macos.sh` paralel koşuyor ve bu `NEN-049`'un açık
konusudur. Atıf çıkarımdır, kanıt değildir (kanıt kaydı §3b).

**Elle acceptance:** kullanıcı düzeltilmiş yapıda oynat/duraklat denedi ve
"güzel duruyor, herhangi bir sorun göremedim" diye bildirdi.
