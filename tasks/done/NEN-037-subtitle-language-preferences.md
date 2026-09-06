---
id: NEN-037
title: Subtitle language preference setting (primary and secondary)
milestone: M3
size: S
state: done
depends_on: [NEN-019, NEN-024, NEN-047]
blocks: []
adr: [10, 31]
---

# NEN-037 — Subtitle language preference setting (primary and secondary)

## Sonuç

Kullanıcı birinci ve ikinci tercih edilen altyazı dilini ayarlayabilir; ayar
oturumlar arasında korunur ve menü sırası ile otomatik seçim bu ayara uyar.

## Kapsam

- macOS **`Settings` scene'i** (NEN-024'ün açtığı sahne): M3'te **yalnız**
  iki dil seçici (birinci / ikinci tercih), ikisi de boş bırakılabilir —
  ADR-0031 Karar 6
- Ayarın kalıcılığı (M5'ten önce platform-yerel kalıcılık yeterli;
  cloud sync **non-goal**)
- `SubtitlePreferences` değerinin `nen-catalog` projeksiyonuna ve otomatik
  seçim politikasına (NEN-019) girdi olarak verilmesi
- Dil listesinde adların **endonim** gösterilmesi (ADR-0010 Karar 7)

## YAPILMAYACAK

- Üçüncü tercih veya sıralanabilir tam dil listesi — ADR-0010 iki slotta karar
  kıldı
- Gruplama/sıralama mantığını UI'da yeniden yazmak — çekirdek zaten sıralı
  projeksiyon döndürüyor (NEN-019)
- Cihazlar arası senkronizasyon — **non-goal** (roadmap S8)
- Aynı pencereye başka bir ayar eklemek — M3'te bu sahne iki seçiciden ibaret
  (ADR-0031 Karar 6)
- Otomatik indirme davranışı → NEN-038

## Kanıt (DoD)

- [x] İki tercih ayarlanıp uygulama yeniden başlatıldığında korunuyor
- [x] Tercih değişince altyazı menüsünün grup sırası değişiyor (checklist)
- [x] İkinci tercih birinciyle aynı seçilirse yok sayılıyor
- [x] Tercih boşken menü, ADR-0010 Karar 10'un "tercih ayarlanmamış" örneğiyle
      aynı sırada

## Kullanıcı kararları

- **İlk açılışta sistem dilinden tohumlama.** Uygulama ilk kez açıldığında
  sistemin dili birinci tercih olarak depoya yazılır — görünür, elle
  değiştirilebilir ve boşaltılabilir. Bugünkü otomatik seçim davranışı
  (sistem dilindeki gömülü track'in açılması) böylece korunur; tohumlama
  `subtitlePreferencesSeeded` bayrağıyla tam bir kez olur.
- **Dil listesi Foundation'ın adlandırabildiği tüm diller.** İkinci bir dil
  tablosu yazılmadı; `LanguageCatalog` mevcut `SubtitleMenuPresentation
  .endonym(for:)`'u (ADR-0010 Karar 7) tekrar kullanıp adlandıramadığı kodları
  düşürür.

## Kanıt kaydı

- **Uygulanan yol:** yeni `SubtitlePreferenceStore.swift`
  (`SubtitleLanguagePreferences` + `SubtitlePreferenceStoring` protokolü +
  `UserDefaultsSubtitlePreferenceStore`, `RecentMediaStore`'un deseninde) ve
  yeni `LanguageCatalog.swift`. `PlayerModel` artık sabit
  `preferredSubtitleLanguage: String?` yerine enjekte edilebilir bir
  `preferenceStore` tutuyor, `@Published subtitlePreferences` yayınlıyor ve
  yeni `updateSubtitlePreferences(_:)` menüyü yeniden sıralıyor —
  `hasAutoSelected`'a dokunmuyor (ADR-0031 Karar 4.3). `SettingsPlaceholderView`
  kaldırılıp yerine iki `Picker`'lı `SubtitlePreferencesSettingsView` geldi.
- Manuel acceptance: Apple M-serisi · arm64 · macOS 27.0 (26A5421a) ·
  Xcode 26.6 · Swift 6.3.3 · libmpv 2.5.0 · debug, ad-hoc imzalı `.app` ·
  yalnız sentetik fixture (`menu-clip.mkv`, İngilizce/Fransızca/Türkçe gömülü
  track'ler). Dört DoD maddesi de gerçek `.app` üzerinde tek tek koşuldu —
  tam ⌘Q/yeniden-açılış restart'ı ve diskteki `.plist` doğrudan okunarak dahil.
  **4/4 geçti**. Ayrıntı: `evidence/M3/NEN-037-checklist.md`.
- `bash scripts/test-macos.sh` çıkış 0: **177 test / 21 suite**, 0 failure.
  Yeni suite'ler `SubtitlePreferenceStoreTests` (7), `LanguageCatalogTests` (4);
  `SubtitleMenuTests`'e 3 yeni test. Negatif kontrol iki yönde ve ayrık —
  `refreshSubtitleMenu()` çağrısı kaldırılınca yalnız yeniden-sıralama testi
  kırmızı, tohumun subtag indirgemesi bozulunca yalnız tohum testi kırmızı.
- Rust workspace **568 passed / 1 ignored** (bu task Rust'a dokunmadı), fmt,
  clippy (`-D warnings`), cargo-deny, `bash scripts/test.sh` ve
  `check-docs.sh` yeşil.
- Yeni mimari karar veya dış bağımlılık yok; ADR-0010 Karar 4/7/10 ve
  ADR-0031 Karar 6 uygulandı, değiştirilmedi.
