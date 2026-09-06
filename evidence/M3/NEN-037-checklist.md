# NEN-037 — Altyazı dili tercihi ayarı checklist

Tarih: 2026-09-06

Ortam: Apple M-serisi · arm64 · macOS 27.0 (26A5421a) · Xcode 26.6
(17F113) · Swift 6.3.3 · libmpv 2.5.0 · debug, ad-hoc imzalı `.app`.
Sistem dili `tr_US` (`AppleLanguages`: `tr-US`, `en-US`).

Kanıt medyası yalnız telif-temiz sentetik fixture'dır:
`fixtures/media/menu-clip.mkv` (İngilizce/Fransızca/Türkçe gömülü altyazı
track'leri + dilsiz bir track — §8'in kanonik örneğiyle aynı küme).

Uzaktan `computer-use` aracıyla, gerçek `.app` üzerinde çalıştırıldı
(`bash scripts/build-macos-app.sh` çıkış 0; `platforms/macos/.build/NenPlayer.app`
`open` ile başlatıldı/kapatıldı).

## Kullanıcı kararları (bu task için alındı)

1. **İlk açılışta sistem dilinden tohumlama.** `UserDefaultsSubtitlePreferenceStore`
   birinci tercihi yalnız **ilk açılışta** sistem dilinin primary subtag'i ile
   yazar (`subtitlePreferencesSeeded` bayrağı bir daha tetiklenmiyor); kullanıcı
   boşalttıktan sonra bir sonraki açılışta geri gelmez.
2. **Dil listesi Foundation'ın adlandırabildiği tüm diller.** İkinci bir dil
   tablosu yazılmadı; `LanguageCatalog` `Locale.LanguageCode.isoLanguageCodes`'u
   mevcut `SubtitleMenuPresentation.endonym(for:)` ile adlandırıp adlandıramadığı
   düşürür.

## Manuel acceptance

1. **DoD #1 — iki tercih ayarlanıp yeniden başlatıldığında korunuyor.**
   İlk açılışta `Ayarlar…` → `Birinci tercih edilen dil: Türkçe` (tohum, sistem
   dilinden), `İkinci tercih edilen dil: Yok` — sistem dilinin doğru tohumlandığı
   doğrulandı. Birinci `Français`, ikinci `English` seçildi. `⌘Q` ile uygulama
   tamamen sonlandırıldı (`pgrep` boş döndü) ve `~/Library/Preferences/player.nen.macos.plist`
   üzerinde `subtitlePrimaryLanguage = fr`, `subtitleSecondaryLanguage = en`,
   `subtitlePreferencesSeeded = true` doğrudan diskte görüldü. Uygulama yeniden
   `open` ile başlatıldı; `Ayarlar…` **Français / English**'i aynen gösterdi —
   tohum ikinci açılışta geri gelmedi. **Geçti.**
2. **DoD #2 — tercih değişince menünün grup sırası değişiyor, ekrandaki altyazı
   değişmiyor.** `menu-clip.mkv` açıldığında birinci tercih `Türkçe` iken otomatik
   seçim Türkçe track'i açtı (transport etiketi `Türkçe`); CC menüsü
   `Kapalı, Türkçe, English, Français, Dil Belirsiz` sırasında. Tercih
   `Français`/`Yok`'a çekilip yeni bir medya açılışında (`menu-clip.mkv` tekrar
   açıldı — otomatik seçim medya başına bir kez çalıştığı için mevcut oturumda
   sıra değişse de seçim sabit kalmalıydı; yeni açılışta yeni "başlangıç" ölçüldü)
   otomatik seçim bu kez Fransızca'yı açtı, CC menüsü
   `Kapalı, Français, English, Türkçe, Dil Belirsiz` sırasında — birinci ve
   ikinci tercih üstte, kalan diller alfabetik. **Geçti** (otomatik testte ayrıca
   *aynı oturumda* tercih değişip seçimin sabit kaldığı da kanıtlı — bkz. Otomatik
   kanıt, `changingPreferenceReordersWithoutMovingSelection`).
3. **DoD #3 — ikinci tercih birinciyle aynı seçilirse yok sayılıyor.** Birinci
   `Türkçe` iken ikinci tercih de `Türkçe` seçildi; `İkinci tercih edilen dil`
   seçici anında **`Yok`**'a döndü (kullanıcı arayüzünde görünür geri bildirim).
   **Geçti.**
4. **DoD #4 — tercih boşken menü Karar 10'un ilk örneğiyle aynı sırada.** İki
   tercih de `Yok`'a çekildi; CC menüsü
   `Kapalı, English, Français, Türkçe, Dil Belirsiz` — ADR-0010 Karar 10'un
   "tercih ayarlanmamış" örneğiyle (alfabetik `en < fr < tr`) birebir aynı.
   Ekrandaki altyazı (o ana kadar seçili `Türkçe`) değişmedi — otomatik seçim
   yeniden tetiklenmedi. **Geçti.**

Sonuç: **4/4 geçti.**

## Otomatik kanıt

`bash scripts/test-macos.sh` çıkış 0 — **177/177** (21 suite, yeni
`SubtitlePreferenceStoreTests` (7), `LanguageCatalogTests` (4) ve
`SubtitleMenuTests`'e eklenen 3 test dahil). Öne çıkanlar:

- `preferencesSurviveRestart` / `systemLanguageSeedsOnlyOnce` /
  `noSystemLanguageSeedsNothing` / `matchingSecondaryIsDroppedOnSave` — DoD #1
  ve #3'ün depolama katmanı.
- `noPreferenceSortsAlphabetically` — DoD #4.
- `changingPreferenceReordersWithoutMovingSelection` — DoD #2'nin "ekrandaki
  altyazı değişmiyor" yarısı, aynı oturumda.
- `matchingSecondaryHasNoEffect` — DoD #3'ün menü etkisi.

**Negatif kontrol iki yönde ve ayrık** (CLAUDE.md Kural 3):
- `updateSubtitlePreferences` içindeki `refreshSubtitleMenu()` çağrısı elle
  kaldırıldığında yalnız `changingPreferenceReordersWithoutMovingSelection`
  kırmızı oldu (1 kırmızı, `Suite "Subtitle menu"` içinde) — düzeltme geri
  alındı.
- Tohumun `Locale.preferredLanguages`'ı primary subtag'e indirgeyen
  `.split(separator: "-", ...)` çağrısı bozulunca (`"-XXX"` yapılınca) yalnız
  `systemLanguageSeedsOnlyOnce` kırmızı oldu ve gerçek değeri (`tr-tr`)
  söyledi — düzeltme geri alındı.

Rust workspace **568 passed / 1 ignored** (bu task Rust'a dokunmadı), fmt,
clippy (`-D warnings`), cargo-deny, shell testleri (`bash scripts/test.sh`)
ve `check-docs.sh` yeşil.
