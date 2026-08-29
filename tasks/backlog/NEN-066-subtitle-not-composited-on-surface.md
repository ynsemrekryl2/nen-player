---
id: NEN-066
title: A subtitle mpv reports drawing is missing from the video surface
milestone: M3
size: M
state: backlog
depends_on: [NEN-024]
blocks: [NEN-027]
adr: []
---

# NEN-066 — Motorun çizdiğini söylediği altyazı ekranda yok

## Sonuç

Motorun o an çizdiğini bildirdiği replik, gerçek `.app`in penceresinde de
görünür.

## Bağlam

`NEN-027` altyazıyı motora veriyor, motor onu seçiyor ve **çizdiğini
bildiriyor** — ama pencerede hiçbir şey yok. Gerçek `.app`te ölçüldü
(2026-08-29, macOS 27.0, libmpv 2.5.0; tam kayıt
`evidence/M3/NEN-027-checklist.md`):

| Gözlem | Sonuç |
|---|---|
| Kullanıcı altyazısı seçildi; aynı anda motor durumu | `sid=5` · `track-list/count=7` · `sub-text` **dolu** (an 2,0 s, cue'nun içi) |
| Ekran | Boş |
| Pencere yeniden boyutlandırılarak tam yeniden çizim zorlandı | Hâlâ boş |
| Dosya seçiliyken cue'ların üzerinden **oynatılarak** geçildi | Hiçbiri çizilmedi |
| Aynı oturumda **gömülü** track'e dönüldü, duraklatılmış | O da çizilmedi |
| Aynı gömülü track, koşunun başında, **oynarken** | Çizildi |

İki şey bu kusurun yerini daraltıyor:

- **Kusur kullanıcı dosyasına özgü değil.** Gömülü track de duraklatılmışken
  çizilmedi, yani `NEN-027`'nin enjeksiyon yolundan eski ve ondan geniş.
- **mpv'nin altyazı boru hattı sağlam.** Aynı belge `vo=image` ile alınan
  gerçek karelere kusursuz basılıyor
  (`evidence/M3/NEN-027-frame-*.jpg`). Ayakta kalan tek fark, libmpv render
  API'siyle sürülen OpenGL yüzeyi (`MPVVideoView`).

Ayrılması gereken iki açıklama:

- **A —** render yolu duraklatılmışken OSD'yi kareye bileştirmiyor.
- **B —** dışarıdan eklenen altyazı bu yolda hiç bileştirilmiyor.

Bunlar tek bir ölçülü koşuyla ayrılır: enjeksiyondan sonra **oynatırken**,
`sub-text` ile ekran eşzamanlı kaydedilir.

Ölçüm sırasında üründe geçici bir satır kullanıldı ve **kaldırıldı**; kalıcı
bir enstrüman gerekiyorsa bu task'ın işidir.

## Kapsam

- A/B ayrımını yapan ölçüm ve kaydı.
- `MPVVideoView`'ın render parametrelerinin ve yeniden çizim tetiklemesinin
  gözden geçirilmesi (güncelleme callback'i, duraklatılmış kare, FBO boyutu).
- Kusurun kalıcı bir testle kapatılması: ekrandaki pikselin altyazıyı
  içerdiğini gösteren, ekran kontrolü gerektirmeyen bir doğrulama.

## YAPILMAYACAK

- Altyazının stilini, konumunu veya boyutunu ayarlamak — M7.
- Enjeksiyon yolunu değiştirmek: `NEN-027`'nin ölçümleri o yolun çalıştığını
  gösteriyor.
- Render API'sinden vazgeçip `wid` gömmeye dönmek — ADR-0012 ve NEN-022'nin
  kaydettiği nedenlerle kapalı; gerekirse önce ADR.

## Kanıt (DoD)

- [ ] A/B ayrımı ölçümle kapandı ve kök neden yazıldı
- [ ] Duraklatılmışken ve oynarken, gömülü track ve kullanıcı dosyası için
      replik ekranda (gerçek `.app`, ekran görüntüsü)
- [ ] Kusuru düzeltmeden önce kırmızı olan bir test eklendi ve şimdi yeşil
- [ ] `NEN-027`'nin görsel koşusu baştan sona geçiyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
