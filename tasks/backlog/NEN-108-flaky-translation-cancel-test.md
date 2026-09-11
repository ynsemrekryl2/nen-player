---
id: NEN-108
title: Stabilize the flaky nen-ffi translation cancellation test
milestone: M5
size: S
state: backlog
closed:
depends_on: []
blocks: []
adr: []
---

# NEN-108 — Stabilize the flaky nen-ffi translation cancellation test

## Sonuç

`nen-ffi::tests::cancelling_a_paused_job_delivers_no_further_progress_and_leaves_no_artifact_or_catalog_entry`
artık CI'da tutarlı geçiyor; testin kendi "cancel() gerçekten mid-run yakalar"
iddiası, o bloğun tamamlanmasıyla yarışan bir OS zamanlama şansına bağlı
kalmıyor.

## Bağlam

`NEN-102`'nin commit'i (`afbcb1e`) GitHub Actions'ta (`run 34569242736`) ilk
koşuda `cargo test` adımında kırmızı verdi:

```
thread 'cancelling_a_paused_job_delivers_no_further_progress_and_leaves_no_artifact_or_catalog_entry' panicked
assertion `left == right` failed
  left: Ok(FfiTranslationSummary { from_cache: false, cue_count: 5, target_language: "tr" })
 right: Err(Cancelled)
```

Bu test dosyası (`core/crates/nen-ffi/tests/translation_gate.rs`) NEN-102'nin
kendi commit'inde **bu testin gövdesine dokunmadan** yalnız SONUNA yeni bir
test eklendi (`cancelling_after_join_has_already_been_called_still_stops_the_job`
— o test aynı koşuda **geçti**). `gh run rerun --failed` ile **aynı commit**
üzerinde tekrar çalıştırıldığında tüm `cargo test` adımı dahil **tamamen
yeşil** döndü (3m11s). Yani kusur NEN-102'nin eklediği koddan bağımsız,
testin kendi ölçülmüş sınırlaması — `NEN-085`'in `spike-async-cancel` için
tespit ettiğiyle aynı aile.

Testin kendi yorum satırı bunu zaten öngörüyordu: "a freshly unblocked worker
thread can otherwise barge back onto the lock before a woken waiter is
scheduled… What the assertions below prove does not depend on this sleep for
correctness — only for reliably exercising the case where cancellation
actually lands before the run finishes." `NEN-102`'nin kendi Swift tarafında
**ölçülerek** bulunan aynı mekanizma: kilidi serbest bırakan thread (worker),
OS'un uyuyan bir waiter'ı (canceller) uyandırıp zamanlamasından **önce**
kilidi tekrar alabiliyor — adil olmayan mutex/futex davranışı. Bunun
oluşma sıklığı platforma/yüke göre değişiyor; GitHub Actions'ın runner'ı bu
sefer macOS'takinden daha müsait bir zamanlama verdi.

## Kapsam

- Bu testin (ve varsa aynı desendeki diğer testlerin —
  `cancel_waits_for_an_in_flight_commit_and_blocks_every_commit_after`
  dahil `nen-ports`'ta) "worker mid-run duruyor, cancel() gerçekten yakalar"
  iddiasını, thread zamanlama şansına bağlı olmayan bir mekanizmaya taşımak
- Ölçülmüş kök nedenin (adil olmayan kilit) belgelenmesi

## YAPILMAYACAK

- `TranslationCall`/`ArtifactStore`'un davranışını değiştirmek — yalnız
  TEST'in kendi senkronizasyonu ölçülüyor
- ADR-0004'ün gate semantiğine dokunmak

## Kanıt (DoD)

- [ ] Aynı test, art arda çok sayıda yerel koşuda (≥ 20) hep yeşil
- [ ] CI'da art arda ≥ 3 koşuda yeşil (`gh run rerun` ile ölçülür)
- [ ] Kök neden (adil olmayan kilit / barging worker) kod içi yorumla
      kayıt altına alınır

## Kanıt kaydı

<!-- done olurken doldurulacak -->
