---
id: NEN-060
title: A selected embedded track is not drawn while the window is full screen
milestone: M3
size: S
state: canceled
depends_on: [NEN-022, NEN-026]
blocks: []
adr: [12, 31]
---

# NEN-060 — Seçili gömülü track tam ekranda çizilmiyor

## Sorun

`NEN-026`'nın elle koşusunda gözlendi: menüden bir gömülü altyazı track'i
seçildiğinde **seçim gerçekleşiyor** — menüde aktif işareti yerine geçiyor,
motor komutu kabul ediyor — fakat pencere **tam ekrandayken** ekranda hiçbir
altyazı belirmiyor.

**Gözlendi, teşhis edilmedi.** `NEN-058`'in açıldığı andaki durumla aynı: tek
bir değişken denendi, kök neden ölçülmedi.

## Ölçülmüş olan

- **Seçim motor seviyesinde çalışıyor.** `MenuFixtureTests`
  (`everySubtitleTrackCanBeSelectedAndReadsBack`) `menu-clip.mkv`'nin dört
  subtitle track'inin her birini seçiyor ve `selectedTrack` ile aynı
  `ff-index`'i geri okuyor; `Kapalı` da geri okunuyor. Yani ff-index → `sid`
  eşlemesi doğru.
- **Kabuk komutu gönderiyor.** `selectSubtitle` yalnız `selectTrack` hata
  vermediğinde seçimi kaydediyor, ve menüde işaret yerine geçti.
- **Çizim en az bir kez çalıştı.** Aynı `.app`'te, **pencere modunda**,
  oynatma başlarken otomatik seçilen Türkçe track ekranda göründü
  ("Öğleden sonranın ışığı yerde ilerliyordu.").

## Ölçülmemiş olan

- Tam ekran gerçekten değişken mi, yoksa gözlemin kendisi mi kusurlu
  (yanlış konum, `Bitti` durumu, duraklatılmışken yeniden çizim yok).
- mpv duraklatılmışken `sid` değişince kareyi yeniden çiziyor mu.
- Tam ekran geçişinde `MPVVideoView`'ün yüzey boyutu motora bildiriliyor mu.

## Kapsam

- Önce **ölçüm**: pencere ve tam ekran modunda, oynatılırken ve
  duraklatılmışken, seçim öncesi/sonrası — hangi kombinasyonda çiziliyor.
- Kök neden bulunduktan sonra düzeltme; gerekirse ayrı ADR.

## YAPILMAYACAK

- Harici belge render'ı → `NEN-027`
- Kör deneme: ölçüm yapılmadan yüzey boyutu / OSD ayarı değiştirmek

## Kanıt (DoD)

- [ ] Dört kombinasyonun (pencere/tam ekran × oynatılıyor/duraklatıldı) ölçüm
      kaydı, `evidence/M3/` altında
- [ ] Kök neden yazılı ve bir testle bağlı
- [ ] Negatif kontrol: düzeltme geri alınınca test kırmızıya dönüyor
- [ ] Ekran kaydı `fixtures/` medyasıyla (ADR-0031 Karar 2)

## İptal kaydı

**Tarih:** 2026-08-29 · **Karar:** kullanıcı (bu task `NEN-066`'ya katlandı).

**Gerekçe: aynı kusur.** `NEN-066` ölçtü ve kök neden çizimde değil, örtmede
çıktı: render yolu altyazıyı dört durumun dördünde de (gömülü/enjekte ×
duraklatılmış/oynarken) kareye bileştiriyor, ama motor onu pencerenin altından
**15–32,5 pt** bandına koyuyor ve `TransportControls` + `.padding(.bottom, 20)`
o bandın üstünü **20–134 pt** ile örtüyor. Tam ekranda da bar aynı sabit
yüksekliği kaplıyor, ve **kontroller yalnız oynarken gizleniyor** — bu task'ın
"tam ekranda hiç çizilmedi, pencere modunda bir kez çizildi" gözlemi tam olarak
bu iki kuralın bileşimi. Ölçüm: `evidence/M3/NEN-066-measurement.md`.

Bu task'ın "Ölçülmemiş olan" listesinin üç maddesi de `NEN-066`'da yanıtlandı:
tam ekran değişken değil; mpv duraklatılmışken `sid` değiştiğinde kareyi
yeniden çiziyor (offscreen ölçümde duraklatılmış hücre çiziyor); yüzey boyutu
motora her render çağrısında veriliyor ve bant yüzey yüksekliğiyle orantılı
ölçekleniyor.

Düzeltme `ADR-0037` ile `NEN-066`'da yapıldı ve tam ekran adımı `NEN-066`'nın
elle koşusuna eklendi (`evidence/M3/NEN-066-checklist.md`). Bu task ayrı bir
kanıt üretmez; `done` sayılmaz ve hiçbir bağımlılığı tamamlamaz (`blocks: []`
olduğu için bir şeyi de engellemiyor).
