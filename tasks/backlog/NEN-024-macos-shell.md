---
id: NEN-024
title: macOS SwiftUI shell with transport controls
milestone: M3
size: M
state: backlog
depends_on: [NEN-022]
blocks: [NEN-025, NEN-037, NEN-042]
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

- [ ] Uygulama açılıp seçilen klibi oynatıyor (ekran kaydı)
- [ ] Seek bar ve süre göstergesi gerçek pozisyonu yansıtıyor
- [ ] Sandbox izniyle seçilen dosya yeniden başlatmada erişilebilir; boş
      durumdaki "son açılan" satırından tekrar açılıyor (bookmark)
- [ ] Yedi kısayolun her biri manuel checklist'te tek tek doğrulanmış
- [ ] Oynarken kontroller gizleniyor, duraklıyken kalıcı görünüyor (checklist)
- [ ] Uygulamayı arka plana alıp öne getirdikten sonra pozisyon ve durum doğru
      (`EventsLost` resync'i — ADR-0011'in backgrounding senaryosu)
- [ ] Açılamayan dosya video yüzeyini hata durumuna çeviriyor, uygulama
      çökmüyor ve "Başka dosya aç" ile devam edilebiliyor
- [ ] UI'da motor adı geçmiyor
- [ ] Pencere başlığı ve hata metni yalnız **dosya adı** gösteriyor; tam yol
      hiçbir yüzeyde yok
- [ ] `evidence/M3/` altına giren kayıt yalnız `fixtures/` medyasıyla üretilmiş

## Kanıt kaydı

<!-- done olurken doldurulacak -->
