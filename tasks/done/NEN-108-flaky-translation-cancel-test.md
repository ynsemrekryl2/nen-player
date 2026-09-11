---
id: NEN-108
title: Stabilize the flaky nen-ffi translation cancellation test
milestone: M5
size: S
state: done
closed: 2026-09-11
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

- [x] Aynı test, art arda çok sayıda yerel koşuda (≥ 20) hep yeşil
- [x] CI'da art arda ≥ 3 koşuda yeşil (`gh run rerun` ile ölçülür)
- [x] Kök neden (adil olmayan kilit / barging worker) kod içi yorumla
      kayıt altına alınır

## Kanıt kaydı

**Kök neden ölçüldü, ikinci bir mekanizmaya taşındı — sleep büyütülmedi.**
İki testin de senkronizasyonu, worker'ı `TranslationCall::progress`'in
tuttuğu delivery-gate kilidinin **içinde** (`PausingSink::on_progress`,
callback kilit altında çağrılıyor) durduruyordu; `release()` sonrası kilidi
**kim** alacağı (worker'ın kendi bir sonraki `checkpoint()`'i mi, yoksa
bloke canceller mı) std `Mutex`'in adil-olmayan (non-FIFO) davranışına
kalıyordu — `thread::sleep(50ms)` olasılığı düşürüyordu, sıfırlamıyordu.
`nen-app`'in kendi eşdeğer testi (`tests/translation_retarget.rs` →
`cancelling_a_running_job_leaves_no_artifact_and_no_catalog_entry`) zaten
başka bir mekanizma kullanıyordu:
`nen_providers::translation_mock::MockCallGate`, provider'ı bir progress
teslim edildikten **sonra, kilidin tamamen dışında** durduruyor — `cancel()`
o kilide hiç rakipsiz giriyor.

**Bu mekanizmaya ulaşmak için iki dar test-seam eklendi, ikisi de üretim
davranışını değiştirmiyor:**
- `nen_app::translation::TranslationEnvironment::with_provider` —
  `new`'in sabit mock provider'ını parametreye çevirir; `new` bu fonksiyonu
  mock ile çağırır, gövde aynı.
- `nen_ffi::translation::FfiTranslationEngine::with_environment` —
  `#[uniffi::export]` bloğunun **dışında**, `#[doc(hidden)]`; üretilen
  Swift/Kotlin binding'de görünmüyor (`bash scripts/build-apple.sh` sonrası
  `nen_ffi.swift`'te `grep with_environment` → bulunamadı), gerçek bir
  çağıranın ulaşamayacağı tek yer. `nen-ffi/Cargo.toml`'a yalnız
  `[dev-dependencies]` altında `nen-providers` eklendi — ADR-0006 kural
  2/3'ün üretim `[dependencies]` grafiği (`nen-ffi -> nen-app`) değişmedi.

**İki test yeniden yazıldı, ikisi de artık `thread::sleep` ve ikinci bir
thread içermiyor:**
- `cancelling_a_paused_job_delivers_no_further_progress_and_leaves_no_
  artifact_or_catalog_entry`: `gate.wait_until_arrived()` sonrası
  `job.cancel()` **inline** çağrılıyor (worker rakipsiz kilide erişiyor).
- `cancelling_after_join_has_already_been_called_still_stops_the_job`:
  yeni `wait_until_finished()` yardımcı fonksiyonu `job.is_finished()`'ın
  `Taken` durumunu (join()'ün **ilk** işi) gözlemleyerek `join()`'ün
  gerçekten uçtuğu anı sabit bir `sleep` yerine **durum geçişiyle**
  yakalıyor; `cancel()` yine inline.

**Sağırlık kontrolü: iki ayrı mutasyon, tam olarak beklenen tek testi
kırdı, diğerini etkilemedi.** `job.cancel()` satırı ilk testten kaldırılınca
yalnız o test kırmızı oldu (`left: Ok(...)`, `right: Err(Cancelled)`
beklentisi); ikinci testten kaldırılınca yalnız o test kırmızı oldu — kontrol
sağır değil.

**Yerel stabilite: 30/30 ardışık koşu yeşil (varsayılan yükte), ayrıca 20/20
ardışık koşu 4 paralel `yes`-worker CPU yükü altında** (`--test-threads=4`
ile), hepsi yeşil. `cargo test --workspace` **832 passed / 1 ignored**
(`NEN-102` baseline'la birebir aynı sayı — test eklenmedi/çıkarılmadı, ikisi
yeniden yazıldı). fmt, clippy (`-D warnings`), `cargo deny check` (yalnız
workspace-içi `nen-providers` kenarı eklendi, yeni dış paket yok —
`Cargo.lock` diff'i tek satır), `bash scripts/build-apple.sh` (binding
üretimi kırılmadı, `with_environment` binding'de yok), `bash scripts/test.sh`
**4/4** ve `bash scripts/check-docs.sh` yeşil.

**`nen-ports::cancel_waits_for_an_in_flight_commit_and_blocks_every_commit_
after` kasıtlı olarak dokunulmadı** — o testte committer thread kendi
`commit_end` kaydını serbest bırakılan kilit **hâlâ elindeyken** yazıyor
(aynı thread ikinci bir kilit denemesi yapmıyor), yani worker'ın kendi
sonraki kilit talebiyle canceller arasında bir yarış hiç yok; sleep orada
yalnız kapsamı (canceller'ın gerçekten beklerken yakalanmasını) etkiliyor,
sonucu değil — flaky aile değil.

**CI:** kapanış commit'i (`f683f88` — `origin/main`'de o an duran `ab9c213`
üzerine rebase edildikten sonra push edildi) `gh run 34572292913` ile üç kez
ardışık koştu, üçü de yeşil: ilk koşu 3m16s, `gh run rerun` ile birinci tekrar
2m51s, ikinci tekrar 2m42s — hepsinde `cargo fmt --check`, `cargo clippy`,
`cargo test`, `cargo deny check`, `bash scripts/test.sh`,
`bash scripts/task-index.sh --check` ve `bash scripts/check-docs.sh` dahil
tüm adımlar geçti. DoD'un "≥ 3 CI koşusu yeşil" maddesi bununla karşılandı.
