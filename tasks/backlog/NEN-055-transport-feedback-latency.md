---
id: NEN-055
title: A transport command is shown without waiting for the next poll tick
milestone: M3
size: S
state: backlog
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

- [ ] `FakeSession` gerçekteki gibi play/pause'da durum olayını kuyruğa koyuyor
- [ ] `togglePlayback` çağrısından sonra poll beklemeden `isPlaying` dönüyor
      (test)
- [ ] Negatif: anında boşaltma kaldırılınca bu test kırmızı
- [ ] Elle: boşluk tuşu ve düğme, ikon tıklamayla aynı anda dönüyor (checklist)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
