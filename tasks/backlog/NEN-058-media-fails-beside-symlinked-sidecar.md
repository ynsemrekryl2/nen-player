---
id: NEN-058
title: A medium fails to load when a symlinked sidecar sits beside it
milestone: M3
size: S
state: backlog
depends_on: [NEN-022, NEN-025]
blocks: []
adr: [12]
---

# NEN-058 — A medium fails to load when a symlinked sidecar sits beside it

## Sonuç

Yanındaki altyazı dosyası ne olursa olsun medya açılır; teşhis edilir ve
kanıtla kapanır.

## Bağlam

`NEN-025`'in kapanış acceptance'ında **gözlemlendi, teşhis edilmedi.**
`fixtures/media/contract-clip.mkv`'nin iki kopyası aynı dizine kondu:

| Dosya | Yanındaki `.srt` | Sonuç |
|---|---|---|
| `Probe.mkv` | geçerli, düz dosya | oynadı |
| `Linked.mkv` | **symlink** (geçerli bir `.srt`'ye) | `Dosya okunamadı.` — art arda 2 kez |

`cmp` iki `.mkv`'nin **byte-eş** olduğunu doğruladı, yani fark medyada değil.
Hata `FfiLoadFailure::unreadable`, yani mpv `loadfile`'ı reddediyor.

**En olası hipotez — sınanmadı:** adapter mpv'nin `sub-auto` ayarını
kapatmıyor, mpv medyanın yanındaki `.srt`'yi kendi başına yüklemeye çalışıyor
ve symlink'te takılıyor. Doğruysa asıl bulgu daha geniş: **altyazıyı hangi
katmanın yükleyeceği** ADR-0026 ve `NEN-027` ile çekiliyor, mpv'nin kendi
kendine sidecar yüklemesi o sınırı deliyor ve kataloğu atlayan görünmez bir
kaynak yaratıyor.

Komut satırından `mpv` ile izole etme denendi ve **başarısız oldu** — kullanılan
çağrı üç dosyada da (kontrol dahil) asılı kaldı, yani ölçüm aracı çalışmadı,
dosyalar hakkında bir şey söylemedi. Teşhis bu task'ın işi.

Kural 5 gereği `NEN-025`'e eklenmedi.

## Kapsam

- Semptomu tekrar üret ve **ölç** — `sub-auto` hipotezini doğrula veya çürüt
- Doğruysa: mpv'nin kendi sidecar yüklemesi kapatılır; altyazı yükleme tek
  yerden, katalogdan geçer
- Yanlışsa: gerçek kök neden bulunur ve kaydedilir

## YAPILMAYACAK

- Hipotezi ölçmeden uygulamak
- Altyazı render'ı → `NEN-027`

## Kanıt (DoD)

- [ ] Semptomun kök nedeni **ölçümle** gösterildi (kod okuması yeterli değil)
- [ ] Yanında symlink `.srt` olan medya açılıyor
- [ ] Negatif: düzeltme geri alındığında semptom aynen geri geliyor
- [ ] `sub-auto` hipotezi doğruysa: mpv'nin katalog dışı bir altyazı
      yüklemediğini gösteren test

## Kanıt kaydı

<!-- done olurken doldurulacak -->
