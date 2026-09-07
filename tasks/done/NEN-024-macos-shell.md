---
id: NEN-024
title: macOS SwiftUI shell with transport controls
milestone: M3
size: M
state: done
closed: 2026-08-26
depends_on: [NEN-022, NEN-045]
blocks: [NEN-025, NEN-037, NEN-042, NEN-043]
adr: [31]
---

# NEN-024 — macOS SwiftUI shell with transport controls

## Sonuç

macOS uygulaması açılır, dosya seçtirir ve videoyu transport kontrolleriyle
oynatır.

## Kapsam

- SwiftUI uygulama iskeleti, **tek pencere** ve video yüzeyi — yeni medya aynı
  pencerede açılır (M4 handoff'un devralacağı model)
- Dosya seçici (sandbox izinleri / security-scoped bookmark)
- Transport: oynat/duraklat, seek bar, süre göstergesi, ses seviyesi
- **Kontrollerin otomatik gizlenmesi:** oynarken fare ~2.5 sn hareketsizse
  kontroller ve imleç gizlenir, harekette geri gelir; **duraklıyken kalıcı
  görünür**
- **Boş durum:** sürükle-bırak alanı + "Aç…" düğmesi + **son açılan tek medya**
  satırı (saklanan bookmark'ın kullanıcıya açılan yüzü, yalnız dosya adıyla)
- **Klavye kısayolları:** `Space` oynat/durakla · `←`/`→` ±5 sn ·
  `⇧←`/`⇧→` ±30 sn · `↑`/`↓` ses · `F` tam ekran · `Esc` çık · `⌘O` aç
- Süre göstergesi `geçen / toplam`, tıklayınca kalana döner
- **Hata sunumu (ADR-0031 Karar 1):** fatal hata video yüzeyini boş durumla
  aynı yüzeyde hata durumuna çevirir; geçici hata transport üzerinde
  kendiliğinden kaybolan bildirimdir; kaynak düzeyi hata buraya **çıkmaz**
- **`EventsLost` sonrası sessiz resync (ADR-0031 Karar 3)** — tam durum yeniden
  sorgulanır, kullanıcıya bildirim yok, `dropped` **sayısı** loglanır
- **Ayar sahnesi:** macOS `Settings` scene'i açılır; içeriğini `NEN-037` doldurur
- Medyanın **hiçbir şeyi beklemeden** oynaması

## YAPILMAYACAK

- Altyazı menüsü → NEN-026
- Stremio handoff → M4
- **Genel** ayarlar/tercihler ekranı → sonraki milestone'lar. `NEN-037`'nin iki
  dil seçicisi bu yasağın dışındadır (ADR-0031 Karar 6)
- Tam "son açılanlar" listesi → NEN-042
- Motor adının UI'da görünmesi — **yasak**
- Tam dosya yolunun veya URL query'sinin herhangi bir yüzeyde görünmesi —
  **yasak** (ADR-0031 Karar 2)

## Kanıt (DoD)

- [x] Uygulama açılıp seçilen klibi oynatıyor (ekran kaydı)
- [x] Seek bar ve süre göstergesi gerçek pozisyonu yansıtıyor
- [x] Sandbox izniyle seçilen dosya yeniden başlatmada erişilebilir; boş
      durumdaki "son açılan" satırından tekrar açılıyor (bookmark)
- [x] Yedi kısayolun her biri manuel checklist'te tek tek doğrulanmış
- [x] Oynarken kontroller gizleniyor, duraklıyken kalıcı görünüyor (checklist)
- [x] Uygulamayı arka plana alıp öne getirdikten sonra pozisyon ve durum doğru
      (`EventsLost` resync'i — ADR-0011'in backgrounding senaryosu)
- [x] Açılamayan dosya video yüzeyini hata durumuna çeviriyor, uygulama
      çökmüyor ve "Başka dosya aç" ile devam edilebiliyor
- [x] UI'da motor adı geçmiyor
- [x] Pencere başlığı ve hata metni yalnız **dosya adı** gösteriyor; tam yol
      hiçbir yüzeyde yok
- [x] `evidence/M3/` altına giren kayıt yalnız `fixtures/` medyasıyla üretilmiş

## Kanıt kaydı

- Otomatik platform testi: `bash scripts/test-macos.sh` → **29 test / 6 suite,
  0 failure**. Gerçek libmpv adapter contract testiyle; shell modeli
  Ready→play, transport sınırları, kontrol görünürlüğü, fatal redaction ve
  `EventsLost` tam resync testleriyle; bookmark ise gerçek Foundation API'siyle
  doğrulandı.
- Geliştirme paketi: `bash scripts/build-macos-app.sh` çıkış 0;
  `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`
  çıkış 0. İmzadan okunan üç entitlement `app-sandbox`,
  `bookmarks.app-scope` ve `files.user-selected.read-only` için `true`.
- Repo tooling: `bash scripts/test.sh` → **2 test dosyası geçti**;
  `bash -n scripts/build-macos-app.sh` ve `git diff --check` çıkış 0.
- Manuel acceptance: macOS 27.0 · Xcode 26.6 · Swift 6.3.3 · libmpv 2.5.0;
  yalnız sentetik `fixtures/media/contract-clip.mkv` ve
  `fixtures/media/broken-clip.mkv`. Yedi kısayol, ±5/±30 saniye seek, kalan
  süre modu, ses, 2.5 saniyelik gizlenme, pause görünürlüğü, fullscreen,
  Settings, background/foreground resync, fatal yüzey ve tam uygulama restart'ı
  sonrası bookmark'tan yeniden açma tek tek geçti. Ayrıntılı adımlar:
  `evidence/M3/NEN-024-checklist.md`.
- Görsel kanıt: `evidence/M3/NEN-024-playback.mp4` — ffprobe ile H.264,
  **1080×680 · 5 fps · 10.000 s · 302344 byte**; 50 fixture karesindeki sayaç
  hareketi oynatmayı gösteriyor. `evidence/M3/NEN-024-transport.jpg` — yalnız
  uygulama penceresi, seek/süre/volume kontrolleri. İki kayıtta da tam yol,
  query veya motor adı yok.
