# M2 — Subtitle Core

## Amaç

Altyazıyla ilgili tüm saf mantığı — parse, yazma, encoding, kimlik, katalog —
platform bağımsız ve headless test edilmiş halde hazırlamak. M3'ün macOS
slice'ı bu core üzerine oturur.

## Kapsam

- Strict SRT parse ve UTF-8 WebVTT yazımı
- Encoding tespiti ve sanitization
- `SubtitleDocument`, timeline/source fingerprint ayrımı
- İndeksli cue lookup
- Media evidence, OpenSubtitles-uyumlu hash, release-name parser
- `SubtitleSourceCatalog` gruplama/dedup
- Dil tespiti ve güven eşiği

## Kapsam dışı

- Translation pipeline → M5 (bilinçli olarak macOS slice'ından **sonra**;
  gerekçe `docs/roadmap.md` → "Şartnameden sapma")
- OpenSubtitles ağ çağrıları → M6
- Herhangi bir UI → M3
- Dosya sistemi güvenlik kapıları → NEN-025 (M3)

## Çıkış kriterleri

- [ ] Geçerli SRT korpusu golden testleri geçiyor
- [ ] ≥20 malformed vaka beklenen typed error'ı döndürüyor; fuzz'da panik yok
- [ ] Round-trip SRT → WebVTT byte-eşit
- [ ] Timeline fingerprint: zaman değişince değişiyor, metin değişince değişmiyor
- [ ] Cue lookup 10k rastgele seek'te lineer taramayla birebir aynı sonucu veriyor
- [ ] Şartname §8'deki menü, katalog projeksiyonundan birebir üretiliyor
- [ ] ≥30 dosya adı için kimlik çıkarımı golden'ı geçiyor

## Task'lar

`NEN-013` · `NEN-014` · `NEN-015` · `NEN-016` · `NEN-017` · `NEN-018` ·
`NEN-019` · `NEN-020`

## Bağımlılıklar

M1 (ADR-0002 kararı). M2, M1 sonrası başlayabilen **tek** milestone'dur.

## Retro

<!-- M2 kapanışında doldurulacak -->
