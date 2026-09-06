---
id: NEN-042
title: Recent media list in the empty state
milestone: M3
size: S
state: done
depends_on: [NEN-024]
blocks: []
adr: [31]
---

# NEN-042 — Recent media list in the empty state

## Sonuç

Boş durum, son açılan tek medya yerine son N medyayı listeler ve her satır
security-scoped bookmark'ıyla yeniden açılabilir.

## Kapsam

- Son açılan medyaların sıralı listesi (N sabit, küçük)
- Her kayıt için security-scoped bookmark saklama ve tazeleme — bayat
  bookmark **scope açıkken** tazelenir, ve tazeleme başarısız olursa
  başarıyla çözülmüş URL düşürülmez
- Erişilemeyen kaydın listeden düşürülmesi (taşınmış/silinmiş dosya)
- Listeyi temizleme komutu

## YAPILMAYACAK

- Pozisyon hatırlama / kaldığı yerden devam — ayrı iş, M5 persistence'a bağlı
- Küçük resim / poster üretimi — kapsam dışı
- Tam yolun listede gösterilmesi — **yasak** (ADR-0031 Karar 2); yalnız dosya adı
- Uzak medyanın (http/https) geçmişe yazılması — URL token taşıyabilir;
  ayrı karar gerektirir

## Neden ayrı task

`NEN-024` boş durumda **tek** son açılan satırını üretiyor (ADR-0031
beyin fırtınası, B7). Tam liste + bookmark tazeleme + erişilemeyen kayıt
temizliği ayrı bir yaşam döngüsü işi; M3 slice'ının kabul kriterlerinden
hiçbiri buna bağlı değil (CLAUDE.md kural 5).

`NEN-024` incelemesinde bu store'da bir kusur bulundu ve buraya eklendi:
`UserDefaultsRecentMediaStore.resolve()` bayat bookmark'ı security scope
başlatılmadan tazelemeye çalışıyor; `save` atarsa `resolve()` throw ediyor
ve `openRecentMedia` kaydı **siliyor** — URL başarıyla çözülmüş olmasına
rağmen. Store zaten bu task'ın konusu olduğu için ayrı task açılmadı.

## Kanıt (DoD)

- [x] N medya açıldıktan sonra liste doğru sırada görünüyor
- [x] Yeniden başlatmada tüm kayıtlar bookmark ile açılabiliyor
- [x] Taşınan/silinen dosya listeden düşüyor, uygulama hata vermiyor
- [x] Bayat bookmark tazeleniyor; tazeleme başarısız olsa bile kayıt
      korunuyor ve medya açılabiliyor
- [x] Listede tam yol görünmüyor (yalnız dosya adı)
- [x] Temizleme komutu listeyi ve saklanan bookmark'ları siliyor

## Kanıt kaydı

Kullanıcı kararları (2026-09-06): N=5 · temizleme komutu Dosya menüsünde
(`Son Açılanları Temizle`) · bugünkü tek kayıt bir kez göç ediyor.

`RecentMediaStore.swift` yeniden yazıldı: `RecentMediaEntry` (`id` +
`displayName`, yol taşımıyor — ADR-0031 Karar 2), `UserDefaultsRecentMediaStore`
5 kayıtlık, dedup'lı, tek seferlik göçlü bir liste. `NEN-050`'de ertelenen
`resolve()` kusuru bu geçişte kapandı: bayat bookmark artık security scope
**açıkken** tazeleniyor, tazeleme hatası zaten çözülmüş URL'yi düşürmüyor.
`PlayerModel.openRecentMedia(_:)` çözülemeyen kaydı yalnız **kendisi** olarak
düşürüyor (`recentStore.remove(id)`, önceki `clear()` tüm depoyu siliyordu).
`PlayerCommands`'e `Son Açılanları Temizle` maddesi eklendi (liste boşken
devre dışı).

Otomatik kanıt: `RecentMediaStoreTests.swift` (yeni, 11 test, gerçek
`UserDefaults` suite + `fixtures/media/*.mkv`) ve `PlayerModelTests.swift`'teki
güncellenmiş/yeni testler. Beş ayrı negatif kontrol (tazeleme-hatası yutma,
dedup, kapasite kırpması, `isFileURL` kapısı, per-entry `remove`) her biri
kendi başına geri alınıp **yalnız kendi testini** kırmızıya çevirdi, sonra
geri kondu. Swift paketi paralel ve seri **195/195**, Rust workspace
**568 passed / 1 ignored** (bu task Rust'a dokunmadı), fmt/clippy temiz,
`.app` build ve strict codesign yeşil.

Gerçek `.app`te ad-hoc imzalı build, yalnız `fixtures/media/*.mkv` ile: altı
fixture sırayla açılıp `⌘Q` ile tam kapatılıp yeniden açıldığında 5 satır
doğru sırada (en yeniden eskiye), yalnız dosya adlarıyla; gerçekten silinen
bir dosyanın satırı tek başına düştü, `Son açılan medya artık kullanılamıyor.`
transient'i göründü, kalan dört satır ve uygulama sağlam kaldı; `Son
Açılanları Temizle` listeyi anında boşalttı ve yeniden başlatmada boş kaldı.
Yol üstünde bir OS gözlemi: aynı birim içinde yeniden adlandırılan bir dosya
security-scoped bookmark tarafından hâlâ çözülüyor (Apple'ın kasıtlı
dayanıklılığı) — "taşınan" DoD maddesi bu yüzden dosyanın gerçekten
kaldırılmasıyla sınandı. Tam kayıt: `evidence/M3/NEN-042-checklist.md`,
iki ekran görüntüsü.
