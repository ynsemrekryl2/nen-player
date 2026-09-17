---
id: NEN-139
title: Show translation token usage and cost in pipeline event log
milestone: M6
size: M
state: backlog
closed:
depends_on: [NEN-131, NEN-138]
blocks: []
adr: []
---

# NEN-139 — Show translation token usage and cost in pipeline event log

## Sonuç

Olaylar penceresindeki çeviri satırı, o çeviri işinin tükettiği input/cache/
output token sayısını ve sağlayıcı bildiriyorsa tahmini maliyetini gösterir.

## Kapsam

- `PipelineEventKind.translationFinished` satırına `NEN-138`'in
  `FfiTranslationSummary` alanlarından gelen token/maliyet bilgisini taşımak
  (bounded sayısal alanlar; K23 kapsamına giren bir şey yok).
- `PipelineEventPresentation.details(for:)` içinde genişletilmiş satıra
  "Girdi token", "Cache'lenmiş girdi", "Çıktı token" satırlarını eklemek;
  sağlayıcı maliyet bildirmiyorsa (OpenAI-direct) maliyet satırı hiç
  gösterilmez — uydurma rakam yok.
- Aynı oturumda birden fazla çeviri yapıldıysa kümülatif toplamı görmek
  isteyen kullanıcı için mevcut pencereye küçük bir toplam satırı/başlık
  eklemek.

## YAPILMAYACAK

- Core/FFI tarafında yeni veri üretmek — bu task yalnız `NEN-138`'in
  ürettiği alanları tüketir.
- Fiyat tablosu, döviz çevirisi veya bütçe/limit uyarısı gibi yeni özellikler.
- Geçmiş oturumlar arası kalıcı maliyet geçmişi — pencere zaten kalıcı değil
  (`PipelineEventLog` dokümantasyonu, task YAPILMAYACAK'ı).

## Kanıt (DoD)

- [ ] `PipelineEventLogTests`'te yeni alanların satır özetinde ve
      genişletilmiş detayda göründüğünü doğrulayan testler; maliyet yokken
      satırın gizlendiğini doğrulayan negatif test.
- [ ] Gerçek `.app` üzerinde (mock veya kayıtlı fixture sağlayıcıyla) kısa
      manuel checklist ve ekran görüntüsü: bir çeviri tamamlandığında token
      (ve varsa maliyet) satırları görünüyor.
- [ ] `bash scripts/test-macos.sh`, `bash scripts/check-docs.sh`, `bash
      scripts/task-index.sh --check`, `git diff --check` çıkış 0.

## Kanıt kaydı

<!-- Task done olurken GERÇEK çıktı ile doldurulur. Boşsa check-docs.sh hata verir. -->
