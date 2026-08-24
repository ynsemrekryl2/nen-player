---
id: NEN-024
title: macOS SwiftUI shell with transport controls
milestone: M3
size: M
state: backlog
depends_on: [NEN-022]
blocks: [NEN-025]
adr: []
---

# NEN-024 — macOS SwiftUI shell with transport controls

## Sonuç

macOS uygulaması açılır, dosya seçtirir ve videoyu transport kontrolleriyle
oynatır.

## Kapsam

- SwiftUI uygulama iskeleti, pencere ve video yüzeyi
- Dosya seçici (sandbox izinleri / security-scoped bookmark)
- Transport: oynat/duraklat, seek bar, süre göstergesi, ses seviyesi
- Medyanın **hiçbir şeyi beklemeden** oynaması

## YAPILMAYACAK

- Altyazı menüsü → NEN-026
- Stremio handoff → M4
- Tercihler/ayarlar ekranı → sonraki milestone'lar
- Motor adının UI'da görünmesi — **yasak**

## Kanıt (DoD)

- [ ] Uygulama açılıp seçilen klibi oynatıyor (ekran kaydı)
- [ ] Seek bar ve süre göstergesi gerçek pozisyonu yansıtıyor
- [ ] Sandbox izniyle seçilen dosya yeniden başlatmada erişilebilir (bookmark)
- [ ] UI'da motor adı geçmiyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
