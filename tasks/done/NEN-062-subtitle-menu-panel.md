---
id: NEN-062
title: Subtitle menu panel above the transport bar
milestone: M3
size: M
state: done
depends_on: [NEN-026, NEN-061]
blocks: []
adr: [10, 31, 35]
---

# NEN-062 — Altyazı menüsü transport barının üstünde açılıyor

## Sonuç

Altyazı menüsü popover yerine CC düğmesine hizalı, barın üstünde sabit bir cam
panel olarak açılır.

## Kapsam

- Panel state'i `PlayerRootView`'e taşınır; `TransportControls` binding ile CC
  düğmesini sürer ve `.popover` kaldırılır.
- Panel barın trailing kenarına hizalanır, 12 pt üstünde, 440×260 pt ve 190/250
  pt iki kolon olarak kalır; kolon içi scroll korunur.
- Bar ile aynı cam yüzey kullanılır; aktif nokta 7 pt olur.
- `PlayerModel.setControlsPinned(_:)` panel açıkken otomatik gizlemeyi durdurur;
  kapanınca oynatmanın normal 2,5 saniyelik döngüsü geri başlar.
- CC düğmesi, panel dışına tıklama, medya değişimi, fatal durum ve shutdown
  paneli kapatır.
- ADR-0031 Karar 4.2/5 ile NEN-026'nın gruplama, seçim, scroll, kusurlu satır ve
  boş durum semantiği değişmez.

## YAPILMAYACAK

- Gruplama, sıra, dedup, endonim metin ve sebep etiketlerini Swift'te yeniden
  üretmek
- Mockup'ın AI çeviri/gecikme/senkronizasyon üçüncü kolonu
- `Esc` davranışı → `NEN-047`
- Tam ekran çizim kusuru → `NEN-060`

## Kanıt (DoD)

- [x] `setControlsPinned` testleri: panel açıkken kontroller gizlenmiyor,
      kapandıktan sonra normal döngü sürüyor
- [x] Panel açıkken 2,5 saniye sonra bar ve panel görünür
- [x] CC düğmesi ve dış tık paneli kapatıyor; medya değişimi stale panel bırakmıyor
- [x] `NEN-026` checklist'i yeni yüzeyde aynı sonuçlarla geçiyor; sidecar gelişi
      seçim ve scroll'u değiştirmiyor
- [x] Mevcut `SubtitleMenuTests` ve `bash scripts/test-macos.sh` yeşil
- [x] Parlak fixture karesinde panel metni ve aktif işaretler okunuyor

## Kanıt kaydı

**2026-08-29 — tamamlandı.** Popover kaldırıldı; panel state'i root'a ve CC
kontrolü binding'e taşındı. Pencere içi panel barla aynı `GlassSurface` üstünde,
trailing hizalı ve 12 pt aralıklı; ölçüsü 440×260 pt, kolonları 190/250 pt.
Panel içi seçim semantiği değişmedi, panel dışı video/transport tıklamaları
paneli kapatırken transport eylemini koruyor.

`setControlsPinned(_:)` oynarken hide task'ını askıya alıyor ve unpin'de 2,5
sn döngüsünü yeniden kuruyor; pause halinde chrome görünür kalıyor. Medya,
fatal ve shutdown sınırları pini temizliyor. Root ayrıca uygulama pasifleşmesi
ve disappear sırasında paneli kapatıyor. Aynı basename taşıyan iki farklı
medya için monoton `mediaPresentationRevision` testi stale panel sinyalini
koruyor.

Gerçek `.app` acceptance'ı sentetik `menu-clip.mkv`, `glass-dark-clip.mkv` ve
`broken-clip.mkv` ile yapıldı. Panel 3,2 sn sonra açık kaldı; CC, video alanı ve
transport kapanışları; kaynak seçiminin paneli açık tutması; kusurlu satır,
fatal ve shutdown yolları doğrulandı. NEN-026'nın 16 adımı aynı sonuçlarla
geçti. Geçici `menu-clip.srt` kapanışta kaldırıldı.

Otomatik kapılar: `bash scripts/test-macos.sh` **91 test / 10 suite, 0
failure**; mevcut `SubtitleMenuTests` değiştirilmeden yeşil. `.app` build ve
strict codesign doğrulaması çıkış 0. Tam acceptance ve ekran görüntüleri:
[NEN-062 checklist](../../evidence/M3/NEN-062-checklist.md) ·
[parlak panel](../../evidence/M3/NEN-062-panel.jpg) ·
[kusurlu kaynak](../../evidence/M3/NEN-062-defective-source.jpg).
