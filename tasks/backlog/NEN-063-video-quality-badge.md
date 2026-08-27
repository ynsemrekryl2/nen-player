---
id: NEN-063
title: Video presentation metadata and quality badge
milestone: M3
size: L
state: backlog
depends_on: [NEN-045, NEN-061]
blocks: []
adr: [11, 31, 33, 36]
---

# NEN-063 — Video presentation metadata ve kalite rozeti

## Sonuç

Üst şerit, metadata hazır olduğunda videonun yaygın çözünürlük sınıfını ve
biliniyorsa SDR/HDR dinamik aralığını doğru gösterir.

## Kapsam

- `VideoPresentationMetadata` ve kapalı `DynamicRange` playback portu, session,
  UniFFI ve Swift shell yüzeyinden geçer (ADR-0036).
- Optional `video_metadata()` sorgusu ve coalescing `VideoMetadataChanged`
  olayı eklenir; `EventsLost` resync metadata'yı da yeniler.
- macOS adapter'ı `video-params/dw`, `dh` ve `gamma` değerlerini ürün tipine
  çevirir; ham motor string'i adapter dışına çıkmaz.
- PQ/HLG → HDR; bilinen SDR transferleri → SDR; eksik/tanınmayan → unknown.
- Yaygın sınıf uzun/kısa kenarı 1280/1920/2560/3840 eşiklerine map eder;
  `1920×800` → `1080p`.
- Metadata hazır değilse rozet yok; dynamic range unknown ise yalnız `1080p`,
  biliniyorsa `1080p · SDR/HDR`.
- Telif/gizli veri taşımayan sentetik SDR ve HDR fixture'ları ve üretim
  reçeteleri eklenir.

## YAPILMAYACAK

- Raw codec/transfer/primaries metnini UI'a veya log'a çıkarmak
- Bitrate, audio layout, chapter, buffering veya decode/render istatistikleri
- Kullanıcıya engine seçtirmek veya motor adına göre UI dallandırmak

## Kanıt (DoD)

- [ ] Fake ve gerçek adapter aynı genişletilmiş playback contract kitini geçiyor
- [ ] Port/session/UniFFI round-trip metadata alanlarını ve change event'ini
      eksiksiz taşıyor
- [ ] PQ, HLG, bilinen SDR ve unknown exhaustive testleri geçiyor
- [ ] 720p/1080p/1440p/2160p ve `1920×800 → 1080p` presentation testleri geçiyor
- [ ] Sentetik SDR/HDR fixture'ları gerçek libmpv adapter'ında doğru sınıflanıyor
- [ ] Metadata gecikirse rozet gizli; unknown range'de yalnız çözünürlük görünüyor
- [ ] `bash scripts/test-macos.sh` ve workspace testleri yeşil

## Kanıt kaydı

<!-- Kapanışta gerçek test ve acceptance çıktısıyla doldurulur. -->
