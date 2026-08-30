---
id: NEN-067
title: Ultra-thin flush player chrome
milestone: M3
size: L
state: blocked
depends_on: [NEN-061, NEN-062, NEN-065, NEN-066]
blocks: []
adr: [31, 37]
---

# NEN-067 — Ultra-ince, zemine-bitişik oynatıcı kromu

## Sonuç

Mevcut oynatma ve altyazı davranışları, pencerenin altına ve yanlarına sıfır
boşlukla oturan 57 pt yüksekliğinde tek satırlı ultra-ince cam transport
yüzeyinde çalışır. Hız, tam ekran ve buffering yüzeyleri mevcut playback
altyapısını kullanır.

## Kapsam

- Transport: 57 pt sabit yükseklik, 22 pt yatay iç boşluk, 0,5 pt üst saç
  çizgisi; çevresel çerçeve, gölge ve yuvarlatılmış alt köşe yok.
- Tek sıra: oynat/duraklat, ±5 sn, geçen süre, esnek seek, kalan/toplam süre,
  hoparlör + 72–92 pt ses slider'ı, ayraç, CC, hız ve tam ekran.
- 720×450 minimum pencerede taşmayan, geniş ekranda seek'i büyüyen yerleşim.
- Sağ süre düğmesi kalan/toplam arasında geçer; mevcut 5 sn atlama ve 2,5 sn
  otomatik gizleme korunur.
- `PresentedPanel` ile birbirini dışlayan altyazı/hız panelleri; panel açıkken
  pin ve dış tık/medya/fatal/pasif/kapanış temizliği.
- Altyazı paneli 440×326 pt, barın sağ içinden 26 pt hizalı, dikey boşluksuz;
  üst köşeler yuvarlak, alt köşeler kare, alt çizgi yok.
- Hız paneli 334×94 pt ve `0.5×`, `0.75×`, `1×`, `1.5×`, `2×` seçenekleri.
- Videonun üstünde tıklamayı engellemeyen ultra-ince cam buffering göstergesi
  ve hafif karartma.
- Tam ekran düğmesi mevcut macOS geçişini çağırır; `F`/`Esc` korunur.
- ADR-0037 güvenli alanı transport'un gerçek SwiftUI yüksekliğinden ölçülür.

## Arayüz değişiklikleri

- `PlaybackSessionClient.setRate(rate:)` mevcut FFI komutuna bağlanır.
- `PlayerModel.playbackRate` ve `setPlaybackRate(_:)`; yalnız başarıdan sonra
  güncelleme, kapalı Türkçe geçici hata ve shutdown'da `1×` sıfırlama.
- Test fake'i hız çağrılarını ve hatalarını kaydeder.
- `GlassSurface` transport ve bağlı panel için ayrı dahili varyantlar taşır.

## YAPILMAYACAK

- Kalite/HDR, AirPlay, PiP, çeviri, senkronizasyon veya chapter yüzeyi.
- Oynatma hızını Settings penceresine taşımak.
- Yeni bir mimari karar; ADR-0031 ve ADR-0037 korunur.

## Kanıt (DoD)

- [x] Beş hız seçeneği; başarı, reddedilme, hata mesajı ve shutdown sıfırlaması
      model testleriyle yeşil.
- [x] Geçen/kalan/toplam ve bir saatten uzun süre biçimleri sunum testlerinde.
- [x] ADR-0037 testi gerçek 57 pt yerleşimi ölçüyor ve altyazı bandı barın
      üstünde kalıyor.
- [x] 1280×720 ve 720×450, parlak/koyu fixture ekranlarında tek sıra,
      kenara bitişiklik, okunabilirlik ve taşmama doğrulandı.
- [x] Panel hizası/dışlanması/pin; buffering; ses pointer/klavye/VoiceOver;
      tam ekran düğmesi ve `F`/`Esc` elle doğrulandı.
- [ ] `bash scripts/test-macos.sh`, uygulama build'i, strict codesign ve
      doküman kapıları geçti.

## Kanıt kaydı

<!-- done olurken doldurulacak -->

Ara kabul kaydı: `evidence/M3/NEN-067-checklist.md`.

## Engel

**Engelleyen task: `NEN-065`.** Son DoD maddesi (`bash scripts/test-macos.sh`)
NEN-065 düzelmeden yeşile dönemez ve bu artık ara sıra kırılan bir test değil,
paralel koşumda deterministik kırmızı — kayıtlı üç koşum ve 2026-08-30'daki
doğrulama, dördü de aynı yerde.

Ölçüm (2026-08-30):

| Koşum | Sonuç |
|---|---|
| `bash scripts/test-macos.sh` (paralel) | 107 test, 1 kırmızı, 12,5 s |
| `swift test --package-path platforms/macos --no-parallel` | 107/107 yeşil, 33,2 s |

Kırmızı olan tek test `pinned controls stay visible and unpin restores the hide
timer`; NEN-067'nin dokunduğu hiçbir test kırmızı değil. Kusur ölçümde:
`PlayerModel.scheduleControlsHide()` gizleme zamanlayıcısını gerçek saatte
kuruyor, test 2 ms'lik gecikmeye karşı gerçek saatte 10 ms bekliyor, ve paralel
koşumdaki gerçek libmpv testleri (`ContractTests` tek başına 12,5 s) o 5×'lik
payı yiyor.

NEN-065 kapandığında bu task yeniden `active` olur ve yalnız son DoD maddesi
doğrulanır; kalan beş madde kanıtlanmış durumda.
