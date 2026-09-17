---
id: NEN-140
title: Show translation duration in pipeline event log
milestone: M6
size: S
state: done
closed: 2026-09-17
depends_on: [NEN-139]
blocks: []
adr: []
---

# NEN-140 — Show translation duration in pipeline event log

## Sonuç

Olaylar penceresindeki çeviri satırı (tamamlandı/iptal/hata), o çeviri
işinin ne kadar sürdüğünü gösterir.

## Bağlam

Süre, `NEN-138`/`NEN-139`'un token/maliyet verisinden farklı olarak core/FFI
tarafında hiç üretilmiyor ve üretilmesine gerek yok: `PipelineEventLog`'un
her satırı zaten kendi `timestamp`'ini taşıyor (`NEN-131`). Bu task tamamen
`platforms/macos` içinde kalır — Rust tarafında değişiklik yok, yeni bir
`bash scripts/test.sh`/`cargo` kapısı gerekmiyor.

Ölçülen, sağlayıcının sunucu-taraflı işlem süresi değil, `translationStarted`
ile terminal olay (`translationFinished`/`Cancelled`/`Failed`) arasındaki
duvar-saati süresidir — checkpoint/resume, retry bekleme süreleri ve UI
thread zamanlaması dahil, kullanıcının gördüğü gerçek bekleme.

## Kapsam

- `PlayerModel.swift`'te çeviri işi başlarken (`events.record(.translationStarted(...))`
  ile aynı noktada) bir başlangıç zamanı yakalamak; iş bittiğinde
  (`.succeeded`/`.joinFailed`/`.prepareFailed`/`.startFailed`) geçen süreyi
  hesaplamak.
- `PipelineEventKind`'in üç terminal case'ine (`translationFinished`,
  `translationCancelled`, `translationFailed`) `NEN-139`'daki `usage`
  alanının yanına bir `duration: TimeInterval` eklemek.
- `PipelineEventPresentation.details(for:)`'a "Süre" satırını eklemek;
  insan-okur biçim (ör. `12,4s` veya `1dk 03s`), ham `TimeInterval` değil.

## YAPILMAYACAK

- Sağlayıcı tarafında sunucu-taraflı işlem süresi/latency ölçümü — yalnız
  istemcinin gördüğü duvar-saati süresi.
- Blok başına veya faz başına (hazırlanıyor/çevriliyor/tamamlanıyor) ayrı
  süre kırılımı — yalnız işin toplam süresi.
- Geçmiş oturumlar arası kalıcı süre geçmişi — pencere zaten kalıcı değil.

## Kanıt (DoD)

- [x] `PipelineEventLogTests`'te süre formatlamasını (saniye/dakika sınırı)
      ve terminal event'lerin süre taşıdığını doğrulayan testler.
- [x] `bash scripts/test-macos.sh`, `bash scripts/check-docs.sh`, `bash
      scripts/task-index.sh --check`, `git diff --check` çıkış 0.

## Kanıt kaydı

2026-09-17 doğrulama kaydı:

- **Formatlama testi (yeni):** `durationIsFormattedForEveryTerminalKind` —
  `12.36` → `"12,4s"` (bir dakikanın altı, ondalık virgülle), `63.7` →
  `"1dk 04s"` (bir dakika ve üstü), `0` → `"0,0s"` (başlamadan başarısız
  olan bir işte bile süre satırı var, gizlenmiyor) — üç terminal case'in
  (`translationFinished`/`Cancelled`/`Failed`) üçünde de doğrulandı.
- **Uçtan uca gerçek akış:** mevcut end-to-end testte artık `translationFinished`
  satırının `duration >= 0` taşıdığı da doğrulanıyor; iptal testinde süre
  gerçek (deterministik olmayan, küçük) duvar-saati değeri olduğu için
  yalnız `usage == .zero` ve case eşleşmesi pinlendi, `duration` değeri
  sabitlenmedi.
- **Kapılar:** `bash scripts/test-macos.sh` çıkış 0 — `NenPlayerShellTests`
  265 test / 26 suite (önceki 264'ten +1), `NenPlaybackMPV` 57 test, Keychain
  4 test, hepsi geçti. `bash scripts/build-macos-app.sh` çıkış 0, gerçek
  `NenPlayer.app` üretildi. `bash scripts/check-docs.sh` 10/10, `bash
  scripts/task-index.sh --check` ve `git diff --check` çıkış 0.
- **Kapsam notu:** Rust/FFI tarafında hiçbir değişiklik yapılmadı (task
  dosyasının kendi Bağlam'ının öngördüğü gibi) — `cargo test --workspace`
  bu task için ayrıca çalıştırılmadı, önceki (NEN-138) doğrulamasından beri
  `core/` değişmedi.
