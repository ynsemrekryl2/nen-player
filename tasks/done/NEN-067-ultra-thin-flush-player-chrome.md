---
id: NEN-067
title: Ultra-thin flush player chrome
milestone: M3
size: L
state: done
closed: 2026-08-31
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
- [x] `bash scripts/test-macos.sh`, uygulama build'i, strict codesign ve
      doküman kapıları geçti.

## Kanıt kaydı

<!-- done olurken doldurulacak -->

Manuel kabul, ölçüm ve görsel fixture'lar: `evidence/M3/NEN-067-checklist.md`
ile `evidence/M3/NEN-067-{bright,dark}-{1280x720,720x450}.png`.

### Otomatik kapılar (2026-08-31)

| Kapı | Sonuç |
|---|---|
| `bash scripts/test-macos.sh` (paralel) | **107/107 yeşil**, 16,5 s |
| `bash scripts/build-macos-app.sh` | exit 0 |
| `codesign --verify --deep --strict .build/NenPlayer.app` | exit 0 |
| `bash scripts/check-docs.sh` | exit 0 |
| `bash scripts/test.sh` | exit 0 |

107 testin içinde beş hız seçeneği, başarılı ve reddedilmiş hız çağrısı, kapalı
Türkçe hata, shutdown'da `1×` sıfırlaması, geçen/kalan/toplam ve bir saati aşan
süre biçimleri, panel dışlanması ve ADR-0037'nin gerçek 57 pt yerleşimi ölçen
libmpv piksel testi var.

### Engel ve kalkışı

Son kapı bir süre `NEN-065` nedeniyle kapalı kaldı: paralel koşum `pinned
controls stay visible and unpin restores the hide timer` testinde deterministik
kırmızıydı, aynı 107 test `--no-parallel` ile yeşildi. Kusur bu task'ın kodunda
değil, o testin iki gerçek-saat uykusunu yarıştıran ölçümündeydi. `NEN-065`
kapandı, ürün kodu değişmeden, ve kapı yukarıdaki hâliyle yeniden koşuldu.
