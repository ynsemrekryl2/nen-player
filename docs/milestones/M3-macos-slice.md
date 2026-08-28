# M3 — macOS Vertical Slice

## Amaç

İlk gerçek ürün. Kullanıcı hikâyesi:

> "macOS'ta bir video dosyası açıyorum, hemen oynuyor. Altyazı düğmesine
> bastığımda gömülü track'leri ve dosyanın yanındaki `.srt`'yi gruplu bir
> listede görüyorum. Birini seçiyorum, altyazı ekranda beliriyor. İleri
> sarıyorum, doğru replik anında görünüyor."

## Kapsam

- `PlaybackEngine` port kontratı + capability modeli + contract test kiti
- libmpv adapter (playback + track enumeration)
- macOS SwiftUI kabuk: dosya aç, video yüzeyi, transport
- Çerçevesiz pencere kromu, okunabilir cam transport ve üst medya şeridi
- Kullanıcı altyazısı yükleme + sidecar keşfi (güvenlik kapılarıyla)
- Altyazı menüsü UI ve transport üstüne hizalı panel yüzeyi
- `SubtitleRenderer` port + libmpv injection adapter

## Kapsam dışı

- OpenSubtitles → M6
- AI çeviri → M5
- Manuel/otomatik sync → M7/M8
- Stremio handoff → M4
- Persistence/cache → M5
- Diğer platformlar → M9–M11

## Ön koşul

**Tam Xcode ve libmpv kurulu olmalıdır** (`scripts/doctor.sh M3` ile
doğrulanır). Bu makinede kapı 2026-08-25'te açıldı (Xcode 26.6 · libmpv 2.5.0);
kapı açılırken doctor'da bulunan bir yanlış pozitif `NEN-041`'e ayrıldı. M3 o
tarihten beri sürüyor.

## Çıkış kriterleri (NEN-028 acceptance)

Bu beş madde kanıtlanmadan M3 kapanmaz:

- [ ] Medya, katalog taraması bitmeden oynuyor
- [ ] Bozuk bir `.srt` playback'i **durdurmuyor**, yalnız o kaynağı hatalı işaretliyor
- [ ] Menüde aynı kaynak iki kez görünmüyor; dili bilinmeyen kaynak
      `Dil Belirsiz` grubunda; `Kapalı` her zaman var
- [ ] Symlink ve path-traversal ile verilen altyazı dosyası reddediliyor
- [ ] Seek sonrası doğru cue anında görünüyor (NEN-017 benchmark'ı ile birlikte)

Ek olarak: gerçek libmpv adapter'ı, fake adapter ile **aynı** contract kitini
geçmelidir.

## Task'lar

`NEN-021` · `NEN-022` · `NEN-023` · `NEN-024` · `NEN-025` · `NEN-026` ·
`NEN-027` · `NEN-028` · `NEN-061` · `NEN-062`

İptal edilen kapsam: `NEN-063` — teknik video kalitesi rozeti
(ADR-0036 `rejected`).

## Bağımlılıklar

M2. ADR-0012 (macOS motoru, linkleme ve proje lisansı) **2026-08-26'da kabul
edildi**: motor libmpv, adapter Swift'te (`platforms/macos/`), geliştirmede
dinamik link, proje lisansı GPL-3.0-or-later. Depo köküne `LICENSE` eklendi ve
roadmap **S12** kapandı. `.app` içine gömme + notarization bu milestone'un çıkış
kriterlerinden **değil** — `NEN-043`, dağıtımdan (S11) önce.

## Retro

<!-- M3 kapanışında doldurulacak -->
