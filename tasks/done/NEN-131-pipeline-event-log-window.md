---
id: NEN-131
title: Pipeline event log window
milestone: M6
size: L
state: done
closed: 2026-09-17
depends_on: [NEN-123, NEN-129]
blocks: []
adr: [31]
---

# NEN-131 — Pipeline event log window

## Sonuç

macOS menü çubuğundaki `Olaylar` menüsü (⌥⌘L) ayrı bir pencerede, bir medya
açıldığında arka planda dönen zinciri — hash → kimlik araması → yan dosya ve
gömülü tarama → OpenSubtitles aday araması → otomatik seçim/indirme → çeviri —
tek satırlık, tıklayınca aşağı doğru açılan olaylar halinde canlı gösteriyor;
denenen ama sonuçsuz kalan yöntemler de görünüyor ve hiçbir satır K23 yasaklı
veri taşımıyor.

## Bağlam

Bu adımların hepsi `PlayerModel` içinde sessiz (`try?`, evidence-only) çalışıyor;
sonuç yalnız menü ve kimlik şeridi üzerinden dolaylı görülüyor, denenip
başarısız olan yöntemler hiç görünmüyor. Kullanıcı 2026-09-17'de canlı,
okunabilir bir olay günlüğü istedi; yer olarak menü çubuğu → ayrı pencere,
geçmiş olarak oturum boyu birikim seçildi. `NEN-126` kabul koşusunda "aday
görüldü, indirme olmadı" gözlemi de bu pencereden yapılabilecek.

Bugün uygulamada kablolu tek kimlik yöntemi OpenSubtitles hash eşleşmesidir
(uzak URL için önce remote evidence); günlük yalnız gerçekten deneneni gösterir.

## Kapsam

- Core: `ProviderIdentityOutcome::NoHash` (hash hesaplanamadı ≠ sağlayıcı
  eşleşme bulamadı); `ProviderCandidateOutcome::Candidates` artık denenen
  yöntemleri (`attempted`) ve bulan yöntemi (`found_by`) taşır; FFI
  `search_opensubtitles_candidates` → `FfiCandidateSearchReport` (durum, aday
  sayısı, denenen/bulan yöntem). Private file ID / URL yine dışarı çıkmaz.
- Shell: `PipelineEventLog` (bellekte, 500 satır tavan, yerinde güncelleme,
  temizleme) ve tipli `PipelineEventKind`; `PipelineEventPresentation` tek
  satır özet + detay çiftleri; `PlayerModel` medya açılışı, oynatma hazır/hata,
  kimlik araması (başladı/bitti/hata), yan dosya ve gömülü sayımı, aday araması
  (başladı/bitti/hata), otomatik seçim kararı ve atlanma sebebi, seçim/kapatma,
  indirme (başladı/bitti/hata) ve çeviri (başladı/faz canlı/bitti/iptal/hata)
  olaylarını kaydeder.
- UI: `Olaylar` menüsü (`Olay Günlüğünü Göster` ⌥⌘L, `Günlüğü Temizle`),
  `Window("Olaylar", id: "events")`, `PipelineEventLogView` — saat · ton
  noktası · tek satır özet; tıklayınca detay aşağı açılır; yeni satırda sona
  kayar; boş durum metni; `Temizle`.

## YAPILMAYACAK

- Günlüğü diske yazmak, panoya kopyalamak, dışa aktarmak — K23; istenirse ayrı
  task + ADR
- Yeni kimlik yöntemi bağlamak (`nen-identity` yerel çıkarımları)
- Ağ istek sayacı (`NEN-126` günlükten dolaylı yararlanır)
- `os_log`/`Logger` çıktısı eklemek — günlük yalnız UI'da yaşar
- Tam yol, URL, hash, private file ID, anahtar — hiçbir olay alanında

## Kanıt (DoD)

- [x] Rust: `NoHash` HTTP isteği olmadan dönüyor; aday araması hash→kimlik
      fallback'inde `attempted == [Hash, VerifiedIdentity]`,
      `found_by == Some(VerifiedIdentity)`; hash bulunca kimlik denenmiyor
- [x] Swift `PipelineEventLogTests`: sıra, 500 tavanı, yerinde güncelleme,
      temizleme, tek satır özet
- [x] Akış testleri: kimlik eşleşmesi → aday araması → indirme dizisi; anahtar
      yok; indirme başarı/başarısızlık; çeviri başla→faz→bitti ve iptal
- [x] **Negatif (K23):** sentinel içeren yol, `?token=` taşıyan uzak URL, sahte
      anahtar ile hiçbir özet/detay tam yol, URL, token, hash veya anahtar
      içermiyor
- [x] UI checklist `evidence/M6/NEN-131-checklist.md`: menü var, ⌥⌘L açıyor,
      satırlar tek satır, tıklayınca detay açılıyor, `Temizle` boşaltıyor,
      ikinci medyada önceki satırlar kalıyor
- [x] `cargo test --workspace`, fmt, clippy, deny; `bash scripts/test-macos.sh`;
      `bash scripts/check-docs.sh`; `bash scripts/task-index.sh --check`;
      `git diff --check` — çıkış 0

## Kanıt kaydı

- Rust (`core/`): `cargo test --workspace` çıkış 0 — yeni/uyarlanan testler
  `opensubtitles_candidates.rs`: `a_hash_hit_reports_hash_and_never_asks_by_identity`
  (`attempted == [Hash]`, `found_by == Some(Hash)`, 1 istek),
  `an_identity_fallback_hit_reports_both_attempts_and_the_identity_finder`
  (`[Hash, VerifiedIdentity]`, `Some(VerifiedIdentity)`, 2 istek),
  `an_exact_hash_miss_falls_back_to_verified_identity_in_order` (`found_by: None`);
  `nen-app` `missing_hash_does_not_read_or_call_provider` ve `nen-ffi`
  `missing_hash_returns_before_credential_or_http_access` → `NoHash`, 0 HTTP,
  0 credential okuması. `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo deny check` çıkış 0.
- Swift: `bash scripts/test-macos.sh` çıkış 0 — player-shell **258/258**
  (26 suite; yeni `PipelineEventLogTests` **17/17**), gerçek libmpv **57/57**,
  Keychain **4/4**, remote-evidence 6/6. Yeni suite: sıra/tavan/yerinde
  güncelleme/temizleme (5), sunum tek satır + aday raporu + kimlik sözleri +
  `errorName` yalnız case adı (4), akış: kimlik→arama zinciri ve sırası,
  anahtarsız sessiz sonuçlar + `automaticDownloadDisabled`, tipli worker
  hataları satır (oynatma hatası değil), indirme başarısız/başarılı, çeviri
  başla→tek canlı faz→bitti, başlamadan iptal (7), **K23 negatif**
  `noRowCarriesForbiddenData`: `SECRETDIR` yolu, `?token=SECRETTOKEN` uzak URL,
  `fixture-key`, `NSError` yol payload'ı — 8 yasaklı desen, hiçbir özet/detayda
  yok (1).
- UI: `evidence/M6/NEN-131-checklist.md` — 7/7; gerçek `.app`'te ilk koşuda
  bulunan çift "Gömülü altyazı izleri" satırı medya başına bire indirildi ve
  regresyon testine bağlandı.
- `bash scripts/check-docs.sh`, `bash scripts/task-index.sh --check`,
  `git diff --check` — çıkış 0.
