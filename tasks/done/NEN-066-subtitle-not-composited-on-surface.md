---
id: NEN-066
title: A subtitle mpv reports drawing is missing from the video surface
milestone: M3
size: M
state: done
closed: 2026-08-29
depends_on: [NEN-024]
blocks: [NEN-027]
adr: [37]
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
- **`NEN-060` bu task'a katlandı** (2026-08-29, kullanıcı kararı): ölçüm aynı
  kök nedeni gösterdi, tam ekran adımı elle koşuya eklendi.

## YAPILMAYACAK

- Altyazının stilini, konumunu veya boyutunu ayarlamak **kullanıcı ayarı
  olarak** — M7. Kromun örttüğü bandın dışında kalmak bir yerleşim bilgisidir,
  stil tercihi değil (ADR-0037 Karar 1) ve bu task'ın işidir.
- Enjeksiyon yolunu değiştirmek: `NEN-027`'nin ölçümleri o yolun çalıştığını
  gösteriyor.
- Render API'sinden vazgeçip `wid` gömmeye dönmek — ADR-0012 ve NEN-022'nin
  kaydettiği nedenlerle kapalı; gerekirse önce ADR.

## Kanıt (DoD)

- [x] A/B ayrımı ölçümle kapandı ve kök neden yazıldı
- [x] Duraklatılmışken ve oynarken, gömülü track ve kullanıcı dosyası için
      replik ekranda (gerçek `.app`, ekran görüntüsü)
- [x] Kusuru düzeltmeden önce kırmızı olan bir test eklendi ve şimdi yeşil
- [x] `NEN-027`'nin görsel koşusu baştan sona geçiyor

## Kanıt kaydı

2026-08-29'da A/B ölçümü iki hipotezi de eledi: libmpv render API'si gömülü
ve kullanıcı track'lerini oynarken ve duraklatılmışken yüzeye çiziyordu; gerçek
kök neden 134 pt'lik transport katmanının çizilen altyazı bandını örtmesiydi.
Kök neden, negatif kontroller ve piksel ölçümleri
`evidence/M3/NEN-066-measurement.md` içinde.

ADR-0037 uyarınca kabuk, görünür kromun gerçek alt inset oranını playback
session üzerinden motora iletiyor; libmpv adapter'ı bunu `sub-pos`'a mapliyor.
Krom gizlenince inset sıfırlanıyor. `SubtitleSafeAreaTests` gerçek libmpv'yi
offscreen sürerek çizilen bandı ölçüyor: 5/5 geçti; düzeltme kaldırıldığında
ilgili test kırmızıya dönüyor. Rust workspace ve renderer kontrat testleri
yeşil; `bash scripts/test-macos.sh` 12 suite / 102 test geçti.

Ad-hoc imzalı gerçek uygulama ile pencere ve tam ekran koşusu tamamlandı:
gömülü ve kullanıcı altyazısı oynarken/duraklatılmışken görünür, transport
açıkken barın üstünde, gizliyken alt banda dönüyor; `F`, `Esc` ve `Kapalı`
regresyonları geçti. Ekranlar ve sekiz adımlık kayıt
`evidence/M3/NEN-066-checklist.md` içinde. `.app` build'i ve
`codesign --verify --deep --strict` geçti.
