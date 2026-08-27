---
id: NEN-062
title: Subtitle menu panel above the transport bar
milestone: M3
size: M
state: backlog
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

- [ ] `setControlsPinned` testleri: panel açıkken kontroller gizlenmiyor,
      kapandıktan sonra normal döngü sürüyor
- [ ] Panel açıkken 2,5 saniye sonra bar ve panel görünür
- [ ] CC düğmesi ve dış tık paneli kapatıyor; medya değişimi stale panel bırakmıyor
- [ ] `NEN-026` checklist'i yeni yüzeyde aynı sonuçlarla geçiyor; sidecar gelişi
      seçim ve scroll'u değiştirmiyor
- [ ] Mevcut `SubtitleMenuTests` ve `bash scripts/test-macos.sh` yeşil
- [ ] Parlak fixture karesinde panel metni ve aktif işaretler okunuyor

## Kanıt kaydı

<!-- Kapanışta gerçek test ve acceptance çıktısıyla doldurulur. -->
