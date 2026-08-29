# NEN-066 — elle koşu: altyazı ekranda mı?

Koşum: **2026-08-29** · macOS 27.0 · Apple M5 · libmpv 2.5.0 (Homebrew,
dinamik) · `.app` ad-hoc imzalı.

Ölçümün tamamı `evidence/M3/NEN-066-measurement.md`'de. Bu dosya, ölçülen
düzeltmenin gerçek `.app` penceresinde de göründüğünü kaydeder — DoD'nin
"gerçek `.app`, ekran görüntüsü" maddesi.

Medya: `fixtures/media/menu-clip.mkv`'nin bir kopyası (`Nen Demo.mkv`) ve
yanında elle yazılmış bir `.srt`. Ekran görüntülerinde tam dosya yolu, query
veya özel medya metadata'sı görünmemelidir (ADR-0031 Karar 2); replikler
uydurmadır (K23 #4).

Uygulama:

```
open '/Users/ye/Developer/nen-player/platforms/macos/.build/NenPlayer.app'
```

## Adımlar

Dördü de aynı soruyu soruyor: **replik ekranda mı, ve transport barının altında
mı kalıyor?** `NEN-027`'nin koşusunda bu dördün üçü başarısızdı.

| # | Adım | Beklenen | Sonuç |
|---|---|---|---|
| 1 | `⌘O` ile `Nen Demo.mkv` aç, oynat, işaretçiyi kıpırdatma (kontroller ~2,5 sn'de gizlenir) | Otomatik seçilen gömülü track ekranda, **barın olduğu yerde** | **Geçti.** Gömülü Türkçe replik oynarken görünür kaldı. |
| 2 | İşaretçiyi oynat, transport barı görünsün | Replik **yukarı kayıyor**, barın üstünde kalıyor; bar repliği örtmüyor | **Geçti.** Piksel bandı barın üstüne taşındı; görünür ekran kaydedildi. |
| 3 | Duraklat (kontroller görünür kalır) | Replik hâlâ görünür ve barın üstünde | **Geçti.** Duraklatılmış karede replik ve bar birlikte görünür kaldı. |
| 4 | CC → `Kullanıcı Altyazıları` → `Nen Demo.srt` | Panel açıkken de replik görünür; panel kapanınca replik yerinde | **Geçti.** Seçim `CC, Nen Demo.srt` olarak doğrulandı; panel açılıp kapanırken çizgi kaybolmadı. |
| 5 | Panel kapalı, oynat, kontroller gizlensin | Replik **aşağı iniyor**, karenin altına dönüyor | **Geçti.** Kullanıcı repliği alt banda döndü ve tamamen görünür kaldı; gizli ekran kaydedildi. |
| 6 | Tam ekrana geç (`NEN-060`), gömülü bir track seç, duraklat | Replik ekranda ve barın üstünde | **Geçti.** `F` ile sistem tam ekranına geçildi; görünür krom durumunda çizgi güvenli alandaydı. |
| 7 | Tam ekranda kontroller gizlenince | Replik aşağı iniyor, hâlâ görünür | **Geçti.** Tam ekranda 2,5 sn sonra krom gizlendi; çizgi görünür kaldı. `Esc` pencere moduna döndürdü. |
| 8 | `Kapalı` | Altyazı gidiyor | **Geçti.** Panel `Altyazılar kapalı.` ve düğme `CC, Kapalı` durumunu gösterdi; çizgi kaldırıldı. |

## Ekran görüntüleri

`evidence/M3/` altına, en az ikisi:

- `NEN-066-controls-visible.png` — kontroller görünür, replik barın üstünde
- `NEN-066-controls-hidden.png` — kontroller gizli, replik karenin altında

## Otomatik kanıt (bu koşudan bağımsız)

- Swift: `SubtitleSafeAreaTests` — 5 test. `the line the engine draws is not
  left under the transport bar` gerçek libmpv'yi offscreen sürüyor, altyazı
  bandını piksel farkından çıkarıyor ve barın kapladığı 134 pt ile
  karşılaştırıyor. Ekran kontrolü gerektirmiyor.
- Negatif kontrol (adapter `sub-pos` yazmıyor): **1 kırmızı**, mesaj kusurun
  kendisi — `lifted.bottom → 15.5 >= chrome → 134.0`.
- Negatif kontrol (görünüm geometriyi bildirmiyor): **1 kırmızı** —
  `session.subtitleBottomInsets → []`.
- Negatif kontrol (çekirdek kırpıyor, reddetmiyor): **2 kırmızı** Rust.

## Koşu sonucu

**GEÇTİ.** Sekiz adımın tamamı 2026-08-29'da ad-hoc imzalı gerçek `.app`
üzerinde koşuldu. Görünür/gizli ekranlar gerçek render yüzeyinde altyazı
bandının sırasıyla transport üstüne çıktığını ve krom kalkınca alt banda
döndüğünü gösteriyor. Kullanıcı dosyası seçimi, duraklatma, tam ekran `F`,
çıkış `Esc` ve `Kapalı` davranışı ayrıca doğrulandı.
