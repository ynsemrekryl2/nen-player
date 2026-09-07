---
id: NEN-050
title: Cover the recent-media store's transient error paths
milestone: M3
size: S
state: done
closed: 2026-09-06
depends_on: [NEN-048]
blocks: []
adr: [31]
---

# NEN-050 — Cover the recent-media store's transient error paths

## Sonuç

Son açılan medya deposundan türeyen üç geçici bildirim yolunun testi vardır.

## Kapsam

- `MemoryRecentStore`'a hata enjeksiyonu (`save` ve `resolve` throw edebilsin)
- Üç yolun testlenmesi:
  - `openMedia` içinde `recentStore.save` hata verirse
    `Son açılan medya kaydedilemedi.` çıkıyor **ve medya yine de yükleniyor**
  - `openRecentMedia` içinde `resolve()` `nil` dönerse depo temizleniyor ve
    `Son açılan medya artık kullanılamıyor.` çıkıyor
  - `resolve()` throw ederse aynı davranış
- Bu metinlerin de kapalı küme negatif testine dahil edilmesi

## YAPILMAYACAK

- Metinleri değiştirmek — kapalı küme aynı kalır
- Son medya listesi UI'ı → `NEN-042`
- Bookmark saklama mekanizmasını değiştirmek

## Neden ayrı task

`NEN-048` motor hatasından (`FfiPlaybackError`) türeyen geçici sınıfı düzeltip
kanıtladı. Aynı `presentTransient` yüzeyini kullanan fakat kaynağı son-medya
deposu olan üç yol o task'ın kapsamında değildi ve testsiz kaldı;
`evidence/M3/NEN-048-checklist.md` bunu açıkça kaydediyor.

## Kanıt (DoD)

- [x] Üç yolun her biri için geçici bildirim üreten test
- [x] `save` hatasında medyanın yine de yüklendiğini gösteren test
- [x] Negatif: bu üç metin de yol, motor adı ve sayısal kod içermiyor

## Kanıt kaydı

`MemoryRecentStore` (`ShellTestSupport.swift`) `FakeSession`'ın deseninde hata
enjeksiyonu ve sayaç kazandı: `errors: [RecentStoreCall: Error]` (`.save` ·
`.resolve`) ve `clearCount` — ikincisi olmadan "depo temizlendi" ile "zaten
boştu" ayrışmazdı, çünkü `resolve() == nil` yolunda `url` başından beri `nil`.

Üç yeni test `PlayerModelTests.swift`'e, `recentMediaReopens()`'in yanına
eklendi:

- `recentMediaSaveFailurePresentsTransientAndStillLoads` — `save` atınca
  `transientMessage == "Son açılan medya kaydedilemedi."` **ve**
  `fixture.loadedLocators == [url.path]` (medya yine de yüklendi)
- `recentMediaResolveNilClearsStoreAndPresentsTransient` — `resolve()` `nil`
  dönünce `store.clearCount == 1`, `recentMediaName == nil`, doğru metin,
  `loadedLocators` boş
- `recentMediaResolveThrowsClearsStoreAndPresentsTransient` — `resolve()`
  atınca (kayıt **dolu** olsa bile) aynı üç iddia

Üç metin `PlayerModel`'den `PlaybackPresentation`'a taşındı
(`recentMediaSaveFailedMessage`, `recentMediaUnavailableMessage`) —
`subtitleRejectionMessage(for:)` deseninde; metin birebir aynı kaldı, çağrı
yeri sabitlere bağlandı. `transientCopyIsClosed`'ın gövdesi `expectClosedCopy`
yardımcısına ayrıldı ve yeni `recentMediaCopyIsClosed` testi iki metni de
aynı kontrolden geçiriyor (yol/motor adı/sayısal kod yok).

**Negatif kontrol dört yönde ve ayrık**, hepsi ölçülüp geri alındı:

1. `:267`'deki `presentTransient` çağrısı kaldırılınca → **yalnız**
   `recentMediaSaveFailurePresentsTransientAndStillLoads` kırmızı (137 testten 1)
2. `openRecentMedia`'nın throw dalındaki `recentStore.clear()` kaldırılınca →
   **yalnız** `recentMediaResolveThrowsClearsStoreAndPresentsTransient` kırmızı
   — `store.clearCount → 0` beklenen `1`'e karşı, yani sayaç gerçekten ayırt
   edici
3. Save hatası yükü de bloklayacak şekilde `return` eklenince → **yalnız**
   "medya yine de yükleniyor" iddiası (`loadedLocators`) kırmızı
4. Kapalı küme kontrolünün sağır olmadığı: `recentMediaUnavailableMessage`'a
   geçici bir rakam (`"(kod 42)"`) eklenince **yalnız**
   `recentMediaCopyIsClosed` kırmızı; metne doğrudan bağlı iki senaryo testi
   sabite işaret ettiği için etkilenmedi

macOS Swift paketi **182 → 186** (paralel koşu, tek koşuda), fmt/clippy'ye
dokunmadı. Rust workspace **568 passed / 1 ignored** (bu task Rust'a
dokunmadı), `cargo fmt --check` ve `cargo clippy --workspace --all-targets -D
warnings` temiz, `scripts/test.sh` ve `scripts/check-docs.sh` yeşil.

## YAPILMAYACAK maddesine sadakat

`UserDefaultsRecentMediaStore.resolve()`'daki bilinen kusur (bayat bookmark'ı
scope açmadan tazeleme, tazeleme hatasında başarıyla çözülmüş kaydı silme)
bilinçli olarak dokunulmadı — DoD'u ve kapsamı `NEN-042`'de.
