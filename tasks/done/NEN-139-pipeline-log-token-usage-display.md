---
id: NEN-139
title: Show translation token usage and cost in pipeline event log
milestone: M6
size: M
state: done
closed: 2026-09-17
depends_on: [NEN-131, NEN-138]
blocks: []
adr: []
---

# NEN-139 — Show translation token usage and cost in pipeline event log

## Sonuç

Olaylar penceresindeki çeviri satırı, o çeviri işinin tükettiği input/cache/
output token sayısını ve sağlayıcı bildiriyorsa tahmini maliyetini gösterir.

## Kapsam

- `PipelineEventKind.translationFinished`, `.translationCancelled` ve
  `.translationFailed` satırlarına `NEN-138`'in `FfiTranslationJob.
  totalUsage()`'ından gelen token/maliyet bilgisini taşımak (bounded sayısal
  alanlar; K23 kapsamına giren bir şey yok). Kapsam, planlanandan bir adım
  geniş: yalnız `translationFinished` değil, üçü de — çünkü `NEN-138`'in asıl
  garantisi tam olarak bu ("iptal/hata sonrası da usage kaybolmaz"); yalnız
  başarı yolunda göstermek bu garantiyi kullanıcıya yarım anlatırdı. Hiçbir
  yeni core/FFI verisi gerekmedi — `FfiTranslationJob.totalUsage()` zaten
  `NEN-138`'de vardı.
- `PipelineEventPresentation.details(for:)` içinde genişletilmiş satıra
  "Girdi token", "Cache'lenmiş girdi", "Çıktı token" satırlarını eklemek;
  sağlayıcı maliyet bildirmiyorsa (OpenAI-direct) veya hiç provider çağrısı
  yapılmadıysa (erken iptal/prepare-start hatası) satırların hiçbiri
  gösterilmez — uydurma rakam yok.
- Aynı oturumda birden fazla çeviri yapıldıysa kümülatif toplamı görmek
  isteyen kullanıcı için `PipelineEventLog.sessionTokenUsage` ve
  `PipelineEventLogView`'da pencerenin üstünde küçük bir özet satırı
  eklemek.

## YAPILMAYACAK

- Core/FFI tarafında yeni veri üretmek — bu task yalnız `NEN-138`'in
  ürettiği alanları tüketir.
- Fiyat tablosu, döviz çevirisi veya bütçe/limit uyarısı gibi yeni özellikler.
- Geçmiş oturumlar arası kalıcı maliyet geçmişi — pencere zaten kalıcı değil
  (`PipelineEventLog` dokümantasyonu, task YAPILMAYACAK'ı).

## Kanıt (DoD)

- [x] `PipelineEventLogTests`'te yeni alanların satır özetinde ve
      genişletilmiş detayda göründüğünü doğrulayan testler; maliyet yokken
      satırın gizlendiğini doğrulayan negatif test.
- [x] Gerçek `.app` üzerinde manuel checklist — kullanıcı kararıyla
      computer-use kullanılmadı (bu ortamda ekran görüntüsü dosyaya
      kaydedilemiyor); onun yerine gerçek `.app` derlemesi + otomatik
      testlerin uçtan uca (mock provider ile gerçek `translate()` çağrısı →
      `PipelineEventLog` satırı) doğrulanması kanıt sayıldı.
- [x] `bash scripts/test-macos.sh`, `bash scripts/check-docs.sh`, `bash
      scripts/task-index.sh --check`, `git diff --check` çıkış 0.

## Kanıt kaydı

2026-09-17 doğrulama kaydı:

- **Sunum testleri (yeni 4 test):** `zeroUsageShowsNoDetails` (usage
  `.zero` iken "Girdi token"/"Cache'lenmiş girdi"/"Çıktı token"/"Tahmini
  maliyet" satırlarının hiçbiri yok — hem `translationFinished` hem
  `translationCancelled` hem `translationFailed` için), `tokenOnlyUsageHidesCostLine`
  (`costUsd: nil` iken token satırları var, maliyet satırı yok — OpenAI-
  direct senaryosu), `billedCostIsShownWhenReported` (`$0.0021` biçiminde
  4 ondalık), `sessionUsageSummarySumsTerminalRowsOnly` (boşken `nil`,
  in-flight `translationPhase` satırı toplamı etkilemiyor, iki terminal
  olayın (biri maliyetli biri maliyetsiz) usage'ı doğru toplanıyor —
  `costUsd` ikinci işte `nil` olsa da ilk işin maliyeti kaybolmuyor).
- **Uçtan uca gerçek akış:** `pipelineEventsRecordTheWholeChain` benzeri
  mevcut end-to-end testte artık `translationFinished` satırının usage'ının
  gerçekten pozitif olduğu (`inputTokens > 0 && outputTokens > 0`) ve
  başlamadan iptal edilen çeviride `translationCancelled(usage: .zero)`
  tam eşleştiği doğrulandı.
- **Kapılar:** `bash scripts/test-macos.sh` çıkış 0 — `NenPlayerShellTests`
  264 test / 26 suite (önceki 260'tan +4), `NenPlaybackMPV` 57 test, Keychain
  4 test, hepsi geçti. `bash scripts/build-macos-app.sh` çıkış 0, gerçek
  `NenPlayer.app` üretildi (uyarılar mevcut deprecation/dylib sürüm
  uyarıları, bu task'tan bağımsız). `bash scripts/check-docs.sh` 10/10,
  `bash scripts/task-index.sh --check` ve `git diff --check` çıkış 0.
- **Manuel checklist notu:** GUI kanıtı için computer-use kullanmak
  kullanıcıya soruldu; kullanıcı otomatik testler + metin checklist'i
  yeterli buldu (bu ortamda ekran görüntüsü dosyaya kaydedilemediği için
  önceki oturumlarda da aynı tercih yapılmıştı). Gerçek `.app` derlendi ve
  imzalandı; ayrıca çalıştırılmadı.
