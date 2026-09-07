# NEN-071 — Altyazı panelinde kısayol çözümleme checklist'i

Tarih: 2026-09-07
Ortam: macOS 27.0 · Xcode 26.6 · Swift 6.3.3 · libmpv 2.5.0
Yapı: `platforms/macos/.build/NenPlayer.app` (ad-hoc)

## Gerçek `.app` koşusu

Fixture geçici bir dizinde hazırlandı; depo dışındadır ve kanıt metnine özel
makine yolu yazılmadı:

- `Film.mkv` — test videosu
- `Chosen.srt` — geçerli SRT hedefi
- `Kisayol.srt` — `Chosen.srt`'ye işaret eden symlink

Adımlar ve gözlenen sonuçlar:

1. `Film.mkv`, Nen Player'ın `Dosya → Aç…` akışıyla açıldı ve oynatıldı.
2. `⇧⌘O` ile altyazı paneli açıldı; `Kisayol.srt` açıkça seçildi. Panelde
   symlink satırı ve etkin `Yükle` düğmesi görüldü.
3. Onaydan sonra uygulama doğrudan-symlink reddi göstermeden oynatmaya döndü.
   Altyazı menüsünde `Kullanıcı Altyazıları 1` kaynağı göründü; seçilen hedef
   SRT katalogda kullanılabilir bir kullanıcı kaynağı olarak yer aldı.

Bu koşu, panelin kullanıcı seçimini hedefe çözerek uygulamaya verdiğini; panel
dışından gelen doğrudan symlink kapısının bu yüzeyde çalıştırılmadığını doğrular.
Cue çizimi ve katalog kaynağı için otomatik ürün-yüzeyi testi ile birlikte
değerlendirilir; özel fixture yolu veya cue metni bu kayda alınmaz.

## Otomatik kanıt

```text
$ bash scripts/test-macos.sh
198 tests in 22 suites passed

$ bash scripts/build-macos-app.sh
ad-hoc NenPlayer.app üretildi

$ codesign --verify --deep --strict --verbose=2 platforms/macos/.build/NenPlayer.app
valid on disk; satisfies its Designated Requirement
```

`PlayerModelTests.subtitlePanelResolvesAliasesExplicitly` paneli önce
`resolvesAliases = false` yapıp ürün yapılandırmasından sonra `true` olduğunu
doğrular. `directSymlinkRefusalIsAnnounced` ve mevcut sidecar-symlink negatif
testleri yeşildir; bunlar panel dışı güvenlik yüzeyinin reddini korur.
