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
- Kullanıcı altyazısı yükleme + sidecar keşfi (güvenlik kapılarıyla)
- Altyazı menüsü UI
- `SubtitleRenderer` port + libmpv injection adapter

## Kapsam dışı

- OpenSubtitles → M6
- AI çeviri → M5
- Manuel/otomatik sync → M7/M8
- Stremio handoff → M4
- Persistence/cache → M5
- Diğer platformlar → M9–M11

## Ön koşul

**Tam Xcode ve libmpv kurulu olmalıdır.** Bu makinede ikisi de eksik
(`scripts/doctor.sh` ile doğrulanır). M3 bunlar kurulmadan başlayamaz.

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
`NEN-027` · `NEN-028`

## Bağımlılıklar

M2. Ayrıca ADR-0012 (libmpv dağıtım/lisans) bu milestone'da karara bağlanır —
`LICENSE.md` ve roadmap S1 buna bağlıdır.

## Retro

<!-- M3 kapanışında doldurulacak -->
