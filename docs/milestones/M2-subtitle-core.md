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

- [x] Geçerli SRT korpusu golden testleri geçiyor
- [x] ≥20 malformed vaka beklenen typed error'ı döndürüyor; fuzz'da panik yok
- [x] Round-trip SRT → WebVTT byte-eşit
- [x] Timeline fingerprint: zaman değişince değişiyor, metin değişince değişmiyor
- [x] Cue lookup 10k rastgele seek'te lineer taramayla birebir aynı sonucu veriyor
- [x] Şartname §8'deki menü, katalog projeksiyonundan birebir üretiliyor
- [x] ≥30 dosya adı için kimlik çıkarımı golden'ı geçiyor

## Task'lar

`NEN-013` · `NEN-014` · `NEN-015` · `NEN-016` · `NEN-017` · `NEN-018` ·
`NEN-019` · `NEN-020`

## Bağımlılıklar

M1 (ADR-0002 kararı). M2, M1 sonrası başlayabilen **tek** milestone'dur.

## Retro

**Süre ve çıktı.** M2, 2026-08-25'te sekiz task (`NEN-013`–`NEN-020`) ile
kapandı. Strict parse/encoding/WebVTT, fingerprint/index, media identity,
source catalog ve dil tespiti platform bağımsız Rust core'unda tamamlandı.
Workspace regresyonu kapanışta 328 test, 0 failure; cue lookup baseline'ı
bilinçli olarak ignored kaldı ve kendi script'iyle ölçülüyor.

**Yanlış veya eksik varsayımlar.** Üç cümlelik metnin bütün dillerde `> 0.90`
güven üreteceği varsayımı yanlıştı: English ve Arabic eşik altında, Russian ise
çok düşük güvenle Bulgarian çıktı. Eşik düşürülmedi; fixture'lar sekiz çeşitli
cümleye çıkarılınca doğru dil güvenleri `1.0` oldu. Media identity golden'ları
da parser'ın elle fark edilmeyen dört kusurunu buldu; bilinen-cevap OSDb vektörü
ilk hash byte-order'ının yanlış olduğunu gösterdi. Golden/bağımsız referans
kanıtları bu milestone'da unit assertion'lardan daha fazla tasarım hatası
yakaladı.

**Kararlar.** ADR-0007 (fingerprint), ADR-0008 (encoding), ADR-0009 (media
evidence), ADR-0010 (source catalog) ve ADR-0029 (language detection) kabul
edildi; hiçbiri supersede edilmedi. Whatlang'ın transitif Zlib lisansı
implementation sırasında bulundu ve ADR-0029 Notlar + `deny.toml` içinde
kaydedildi.

**Sonraki milestone.** M3 task kırılımı hazır (`NEN-021`–`NEN-028`, `NEN-036`,
`NEN-037`), ilk iş playback port sözleşmesi. Ancak M3 kapısı bugün tam Xcode ve
libmpv eksikliği nedeniyle kapalı; `bash scripts/doctor.sh M3` bu ikisini
blocker olarak raporluyor. Kurulum yapılmadan M3 implementation'ı başlamaz.
