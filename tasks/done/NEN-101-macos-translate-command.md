---
id: NEN-101
title: macOS translate command and target language setting
milestone: M5
size: M
state: done
closed: 2026-09-10
depends_on: [NEN-100, NEN-037]
blocks: [NEN-102]
adr: []
---

# NEN-101 — macOS translate command and target language setting

## Sonuç

Kullanıcı macOS'ta hedef çeviri dilini bir ayardan seçebiliyor ve seçili
altyazı kaynağı için "AI ile çevir" komutunu verebiliyor.

## Bağlam

Şartname §9: "Kullanıcı ayrıca **açıkça** 'AI ile <hedef dile> çevir' komutunu
verir. Hedef dil kullanıcı ayarıdır; varsayılan Türkçe olabilir. Kaynak zaten
hedef dildeyse translation başlatılmaz."

`NEN-037` Settings sahnesini ve `SubtitlePreferenceStore` + `LanguageCatalog`
desenini zaten kurdu; hedef dil ayarı bu desenin üzerine gelir, ikinci bir dil
tablosu açılmaz. Komut yeri `PlayerCommands` (`NEN-042`'nin "Son Açılanları
Temizle" maddesini eklediği yer).

## Kapsam

- Settings'e hedef çeviri dili tercihi (varsayılan: sistem dilinden bir kez
  tohumlanır — `UserDefaultsSubtitlePreferenceStore.seedIfNeeded` deseninin
  aynısı)
- Yeni `CommandMenu("Altyazı")` içine "AI ile çevir" komutu (kabuk şekli
  kararı — mimari değil; kapı model durumu, `NEN-042`'nin "Son Açılanları
  Temizle" idiyomu, odak kapısı değil)
- Komutun etkin/devre dışı koşulları: kaynak seçili değilse, kaynak zaten hedef
  dildeyse, ya da iş zaten koşuyorsa devre dışı
- Kaynak seçmenin komutu **tetiklememesi**
- Komut işi başlatır, `join()` eder ve sonucu kataloğa ekler (arka plan
  task'ında) — ekranda ilerleme/iptal yok, bu `NEN-102`'nin kapsamı; ekrandaki
  seçili altyazı zorla değiştirilmez
- Artifact deposunun kökü `~/Library/Application Support/NenPlayer/`
  (ADR-0034 gereği uygulama sandbox'sız)

## YAPILMAYACAK

- İlerleme ve iptal yüzeyi — `NEN-102`
- Provider veya model seçtirme — M6
- Glossary düzenleme yüzeyi — M6
- Teknik ID girişi — non-goal

## Kanıt (DoD)

- [x] Swift testi: hedef dil ayarı yazılıp okunuyor ve uygulama yeniden başlatıldığında kalıcı
- [x] Swift testi: kaynak seçiliyken komut etkin, seçili değilken devre dışı
- [x] Negatif: kaynak zaten hedef dildeyken komut devre dışı ve iş başlatılamıyor
- [x] Negatif: kaynak seçmek tek başına hiçbir çeviri işi başlatmıyor
- [x] Gerçek `.app` checklist: ayar değiştirilip komut veriliyor, iş başlıyor — `evidence/M5/NEN-101-checklist.md`

## Kanıt kaydı

**Kullanıcı artık macOS'ta hedef çeviri dilini Settings'ten seçebiliyor ve
seçili bir altyazı kaynağı için menü çubuğundaki yeni `Altyazı ▸ AI ile
<hedef dile> Çevir` komutuyla açıkça çeviri başlatabiliyor.** Rust
çekirdeğine hiç dokunulmadı — `NEN-100`'ün ürettiği `FfiTranslationEngine` /
`FfiTranslationJob` yüzeyi olduğu gibi çağrıldı. Yeni
`TranslationPreferenceStore.swift` `SubtitlePreferenceStore.swift`'in
birebir deseni: `UserDefaultsTranslationPreferenceStore` sistem dilinden
**bir kez** tohumlanan, `translationTargetSeeded` bayrağıyla korunan tek bir
`String?` tutuyor — ikinci bir dil tablosu açılmadı, seçici
`LanguageCatalog.allCodes` + `SubtitleMenuPresentation.endonym(for:)`'u
kullanıyor.

**Komutun tek kapısı `PlayerModel.canTranslateSelectedSubtitle`.** Sırasıyla:
iş koşmuyor · hedef dil seçili · bir kaynak seçili · o kaynağın menü satırı
bulunuyor · `defect == nil` · `translatable` · kaynağın dili biliniyor ·
kaynağın ve hedefin **birincil subtag'leri** farklı (`nen_domain::source::
LanguageTag::primary_tag`'in ADR-0030 granülerliğiyle birebir — ikisi de
`split(separator: "-", maxSplits: 1)` ile indirgeniyor). Komut
`PlayerCommands`'te yeni bir `CommandMenu("Altyazı")` içinde, `NEN-042`'nin
"Son Açılanları Temizle" idiyomuyla **model durumuna** bağlı — odak kapısı
değil, çünkü test edilebilir olan bu.

**`translateSelectedSubtitle()` işi başlatır, `join()` eder ve sonucu
kataloglar; ekranı asla değiştirmez.** `startSidecarScan`'in
`Task.detached` deseniyle iş ana actor dışında çalışıyor; başarıyla dönen
`FfiTranslationJob` ana actor'a geri gelip `catalogInto(library:)` çağrılıyor
— **yalnız** medya o sırada değişmediyse (`mediaPresentationRevision`
karşılaştırması, aynı mekanizmanın var olan kullanım amacı — bir çeviri işi
önceki medyanın kataloğuna sessizce satır eklemesin diye). `selectSubtitle`
hiçbir zaman çağrılmıyor: §9'un "zorla AI çıktısına geçilmez" kuralı ve
ADR-0031 Karar 4.3 ailesi.

**Her iki kapı elle mutasyona uğratıldı, her birinde tam olarak beklenen
test(ler) kırmızıya döndü, sonra geri alındı:**
1. `canTranslateSelectedSubtitle`'ın dil-eşitliği koşulu `return true`'ya
   çevrildiğinde — yalnız `alreadyTargetLanguageDisablesAndRefuses` ve
   `regionQualifiedTargetLanguageAlsoRefuses` kırmızı oldu (3 ayrı
   `#expect` başarısızlığı, ikisi de tam beklenen satırlarda).
2. `translateSelectedSubtitle()`'ın `canTranslateSelectedSubtitle` guard'ı
   kaldırıldığında (hedef/token bağlamaları kaldı) — yalnız
   `alreadyTargetLanguageDisablesAndRefuses`'ın "hiçbir `artifacts/` dizini
   oluşmadı" iddiası kırmızı oldu; bu da Swift'in kendi kapısının
   `FfiTranslationEngine`'i hiç kurmadığını, yalnız Rust'un
   `AlreadyTargetLanguage` reddine güvenmediğini kanıtlıyor — kontrol
   sağır değil.

**Yol üstünde gerçek bir kusur bulundu ve aynı task içinde düzeltildi
(gerçek `.app` koşusunda ölçüldü, Swift testleri görmedi).**
`nen-persist::FilesystemArtifactStore::new` kökü `fs::canonicalize` ile
açıyor — kökün **zaten var olmasını** şart koşuyor, yalnız kendi
`artifacts/` alt dizinini yaratıyor. `defaultTranslationStoreRoot()`'un
döndürdüğü `~/Library/Application Support/NenPlayer/` hiçbir yerde
yaratılmıyordu; temiz bir kurulumda ilk komut her zaman
`StoreUnavailable`'a düşerdi. Swift testleri bunu yakalayamadı çünkü
`TempFixture` kendi kökünü zaten yaratıyor. Düzeltme:
`translateSelectedSubtitle()` motoru kurmadan önce
`FileManager.default.createDirectory(at:withIntermediateDirectories:)`
çağırıyor, başarısızlık `.StoreUnavailable` ile aynı yüzeye düşüyor.

**Gerçek `.app` kabulü — `evidence/M5/NEN-101-checklist.md`.**
`bash scripts/build-macos-app.sh` ile üretilen ad-hoc imzalı `.app`,
`fixtures/media/contract-clip.mkv` (gömülü Türkçe altyazı) ve
`fixtures/subtitles/languages/english.srt` ile: hedef dil Almanca'ya
değiştirildi, İngilizce kaynak seçildi, `Altyazı ▸ AI ile Deutsch Çevir`
verildi — menüde `Deutsch ▸ AI çevirisi (de)` (AI rozetli) belirdi, seçili
altyazı ve badge (`english.srt`) **değişmeden** kaldı. Negatif: hedef dil
Türkçe'ye çevrilip gömülü Türkçe altyazı seçilince `AI ile Türkçe Çevir`
soluk/devre dışı görüldü. Diskte tek bir `<64hex>.json` artifact dosyası
oluştu (ADR-0017 Karar 2), kanıt toplandıktan sonra geliştirme dizini
temizlendi.

**Testler:** iki yeni suite — `TranslationPreferenceStoreTests` (4 test:
kalıcılık, tek seferlik tohumlama, sistem dili yokken boş kalma, bölge
indirgeme) ve `TranslationCommandTests` (6 test: etkin/devre dışı, zaten
hedef dilde reddediliyor, bölge varyantı da reddediliyor, seçim tek başına
başlatmıyor, mutlu yol + retarget yok, hedef dil değişimi menüyü/seçimi
bozmuyor). `bash scripts/test-macos.sh` **249 passed / 0 failed / 30 suites**
(bu task'ın 10 testi dahil). `cargo test --workspace` **831 passed / 1
ignored** (`NEN-100` baseline'la aynı — Rust'a dokunulmadı); `cargo fmt
--check`, `cargo clippy --workspace --all-targets -- -D warnings` ve `bash
scripts/test.sh` **4/4** yeşil.

**ADR-0031'e ayrı, önceden push edilen bir Notlar girdisi eklendi
(`8d0fc77`).** Karar 6'nın "M3'te tek ayar yüzeyi" kapsamının M5'te üçüncü
bir satırla genişlediği kayıt altına alındı; yasaklanan genel ayarlar ekranı
değil, Karar 6'nın gövdesi değişmedi (ADR-0043/`NEN-081` emsali).

`nen-persist`, `nen-translate`, `nen-app`, `nen-ffi` dokunulmadı — yalnız
macOS kabuğu (`platforms/macos/Sources/NenPlayerShell/`).
