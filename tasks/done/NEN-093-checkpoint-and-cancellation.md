---
id: NEN-093
title: Checkpoint only validated blocks and cancel without late commit
milestone: M5
size: M
state: done
closed: 2026-09-08
depends_on: [NEN-092]
blocks: [NEN-094]
adr: []
---

# NEN-093 — Checkpoint only validated blocks and cancel without late commit

## Sonuç

Yalnız tamamen doğrulanmış bir blok checkpoint'leniyor ve iptal edilen bir
çeviri işi **hiçbir koşulda** sonradan bir şey commit etmiyor.

## Bağlam

Şartname §10: "yalnız tamamen doğrulanmış block checkpoint" · "cancellation
kontrolleri" · "progressive veya yarım subtitle publication yok". M5'in iki
çıkış kriteri doğrudan buraya bakıyor.

Late-commit yasağının kontratı `ADR-0004`'ün konusu ve `NEN-009`
(`spike-async-cancel`) bunu M1'de ölçmüştü — o spike'ın kanıt deseni
(iptal sonrası commit sayacının **sıfır** kalması) burada ürün koduna taşınır.
Spike kodunun kendisi terfi etmez (Kural 7).

## Kapsam

- Blok başına checkpoint kaydı — yalnız doğrulama geçtikten sonra
- Blok sınırlarında iptal kontrolü ve işin sonlanma yolu
- Yarıda kalmış işin checkpoint'inden devam edebilmesi
- İptal sonrası hiçbir yazma/teslim yolunun açık kalmaması

## YAPILMAYACAK

- Checkpoint'in **diske** yazılması — `NEN-096`; burada checkpoint iş içi durumdur
- Artifact üretimi — `NEN-094`
- Kullanıcıya ilerleme gösterimi — `NEN-100` (FFI) · `NEN-102` (macOS)

## Kanıt (DoD)

- [x] Unit: doğrulama başarısız olan blok checkpoint'lenmiyor (sayaç ile ayırt ediliyor)
- [x] Unit: yarıda bırakılan iş, checkpoint'lenmiş bloklardan devam ediyor, onları yeniden çevirmiyor
- [x] Negatif: iptalden sonra commit sayacı **0** — geç gelen provider cevabı hiçbir şey yazmıyor
- [x] Negatif: iptal kontrolü kaldırıldığında bu test kırmızıya dönüyor (kontrol sağır değil)
- [x] Negatif: yarısı doğrulanmış bir iş **hiçbir** kısmi belge yayımlamıyor

## Kanıt kaydı

**Karar:** `nen-ports::translation::TranslationCall`'a yeni bir `commit<T>(&self,
f: impl FnOnce() -> T) -> Result<T, TranslationProviderError>` metodu eklendi.
Gate kilidi tutulurken `f` çalışır; kapalıysa `f` hiç çağrılmaz. Böylece
checkpoint yazımı, `progress`/`finish` ile **aynı** kilit sınırını paylaşır —
check-then-write yarışı yok (ADR-0004 Karar 2/5, kullanıcı onaylı).

**`nen-translate::checkpoint` (yeni modül).** `BlockCheckpoints` yalnız
`TranslationCall::commit` üzerinden dolduruluyor; `into_completed()` bütün
bloklar checkpoint'lenmeden `CompletedBlocks` üretmiyor (`IncompleteRun`
hatası — tip düzeyinde yarım yayın yasağı). `translate_checkpointed` bloğu
sırayla sürüyor: zaten checkpoint'li blok provider'a hiç gitmeden atlanıyor,
her blok sınırında `call.checkpoint()` kontrol ediliyor, geçen blok
`repair::translate_block_with_repair` (NEN-092) ile çevriliyor ve yalnız
başarılıysa `commit` ediliyor.

**Test dosyaları:**
- `core/crates/nen-ports/src/translation/mod.rs` — 2 yeni birim testi:
  `commit_after_cancel_never_runs_the_closure`,
  `cancel_waits_for_an_in_flight_commit_and_blocks_every_commit_after`
  (iki thread, gerçek karşılıklı dışlama — `cancel()` süren `commit`'in
  bitmesini bekliyor; olay sırası `commit_start, commit_end, cancel_done`
  ölçüldü).
- `core/crates/nen-translate/src/checkpoint.rs` — 6 birim testi: başarısız
  blok checkpoint'lenmiyor · tam çalışma sonrası resume no-op (0 provider
  çağrısı) · gerçek yarıda kesilmiş çalışma (1 blok checkpoint'li, geri
  kalanı resume'da yeniden çevriliyor, checkpoint'li olan **değil**) ·
  çalışma başlamadan kapalı gate blok sınırında yakalanıyor, provider hiç
  çağrılmıyor · eksik checkpoint kümesi `into_completed()`'i reddediyor ·
  tamamlanan bloklar sırayla korunuyor.
- `core/crates/nen-translate/tests/checkpoint_cancellation_negative.rs`
  (yeni dosya) — 4 negatif/guard testi: **gerçek çok-thread'li iptal** —
  blok 1'in provider cevabı barrier ile havada tutulurken başka bir thread
  `cancel()` çağırıyor, cevap `finish()` içinde reddediliyor, blok 1 hiç
  checkpoint'lenmiyor (checkpoint sayısı 1/N, `into_completed` reddediyor) ·
  çalışma başlamadan kapalı gate · çalışma hatasının `Debug`/`Display`'i
  sentinel metni sızdırmıyor (K23 #4) · `#[derive(Debug)]` ikizinin sentinel'ı
  gerçekten sızdırdığını gösteren kontrol testi.

**Mutasyon kontrolü (elle ölçüldü, DoD "kontrol sağır değil" maddesi):**
- Blok sınırı `call.checkpoint()` çağrısı kaldırılınca: tam olarak 2 test
  kırmızıya döndü — `a_gate_closed_before_the_run_starts_is_checked_at_the_block_boundary`
  (in-crate) ve `cancel_before_the_run_starts_prevents_every_commit`
  (negatif dosya); ikisi de `Cancelled` yerine provider'ın `finish()`
  içinde geç yakaladığı `Block(Provider(Cancelled))` aldı.
- `TranslationCall::commit`'in gate kontrolü kaldırılıp her zaman `Ok(f())`
  dönecek şekilde sabitlenince: tam olarak 3 test kırmızıya döndü —
  `commit_after_cancel_never_runs_the_closure`,
  `cancel_waits_for_an_in_flight_commit_and_blocks_every_commit_after`
  (`nen-ports`) ve `interrupted_run_resumes_from_its_checkpoints_without_retranslating_them`
  (`nen-translate`, checkpoint sayısı beklenenden 1 fazla çıktı).
- Her iki mutasyon da ölçümden hemen sonra geri alındı; diff `/dev/null`
  (dosyalar özgün haline döndü, kanıt yalnız test çıktısı).

**Kanıt (gerçek çıktı):**
- `cargo test -p nen-ports translation::` — **10 passed** (8 + 2 yeni)
- `cargo test -p nen-translate` — lib **36 passed**, `checkpoint_cancellation_negative` **4 passed**, `block_layout_golden` **1 passed**, `guard_context_debug` **3 passed**, `validation_negative` **4 passed**
- `cargo test --workspace` — **724 passed / 1 ignored**, 0 failed
- `cargo fmt --check` — temiz
- `cargo clippy --workspace --all-targets -- -D warnings` — temiz
- `cargo deny check` — advisories/bans/licenses/sources ok (yeni bağımlılık yok)
- `bash scripts/test.sh` — 4/4 dosya geçti (doctor + stremio-bridge)
- `bash scripts/check-docs.sh` — **SONUÇ: tüm denetimler geçti** (0 hata)

**Kapsam dışı bırakılanlar (plana göre):** diske yazma (`NEN-096`), artifact/
WebVTT (`NEN-094`), FFI/macOS ilerleme yüzeyi (`NEN-100`/`NEN-102`), `nen-app`
oturum orkestrasyonu ve retarget reddi (`NEN-099`) dokunulmadı.
