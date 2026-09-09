---
id: NEN-106
title: A multi-block run rejects its own provider's progress
milestone: M5
size: S
state: done
closed: 2026-09-09
depends_on: [NEN-090, NEN-093]
blocks: [NEN-099]
adr: [0004]
---

# NEN-106 — A multi-block run rejects its own provider's progress

## Sonuç

Birden fazla bloğa bölünen bir çeviri işi, kendi sağlayıcısının bildirdiği
ilerleme yüzünden `Permanent` hatasıyla düşmüyor.

## Bağlam

`NEN-096`'nın uçtan uca testi yazılırken ölçüldü. `translate_checkpointed`
(`nen-translate/src/checkpoint.rs:220`) bütün bloklar için **tek bir**
`TranslationCall` kullanıyor. `TranslationCall::progress`
(`nen-ports/src/translation/mod.rs:218`) ise aynı çağrı içinde `total`'ın
değişmesini `TranslationProviderError::Permanent` ile reddediyor — ADR-0004'ün
monoton ilerleme sözleşmesi gereği, doğru bir kural.

Ama bir sağlayıcı yalnız **kendi bloğunun** isteğini görür; belgenin toplam cue
sayısını bilemez, dolayısıyla `total` olarak blok cue sayısını bildirir. İki
bloğun çıktı pencereleri farklı boyda olduğu anda (varsayılan düzende neredeyse
her zaman: 50 cue'luk bir belge `37` + `13` verir) ikinci blok düşüyor.

Ölçüm: `MockTranslationProvider` ile 50 cue'luk bir belge
`Block(Provider(Permanent))` ile düşüyor; **aynı** kod 30 cue'da (tek blok)
geçiyor. `nen-translate`'in kendi testleri bunu görmedi çünkü fixture
sağlayıcısı (`EchoTranslationProvider`) hiç `progress` çağırmıyor.

Kusur sağlayıcıda değil orkestrasyonda: `NEN-092`'nin retry için zaten
kullandığı `TranslationCall::fork` deseni (bağımsız ilerleme toplamı, paylaşılan
iptal geçidi) blok sınırında da gerekiyor. Belge geneli yüzdesinin nasıl
toplanacağı `NEN-099`/`NEN-102`'nin işi.

## Kapsam

- Blok sınırında ilerleme dizisinin bağımsız başlaması
- İptal geçidinin paylaşılmaya devam etmesi (ADR-0004 Karar 2/5 bozulmadan)

## YAPILMAYACAK

- Belge geneli ilerleme toplaması ve kullanıcıya gösterimi — `NEN-099` · `NEN-102`
- `TranslationCall`'ın monotonluk kuralını gevşetmek — kural doğru

## Kanıt (DoD)

- [x] Negatif: ilerleme bildiren bir sağlayıcıyla çok bloklu bir belge baştan
      sona çeviriliyor; bugünkü kodda bu test kırmızı
- [x] Negatif: blok sınırında iptal hâlâ çalışıyor — fork paylaşılan geçidi
      koruyor (`NEN-093`'ün iptal testleri yeşil kalıyor)
- [x] Unit: her bloğun ilerleme dizisi kendi `total`'ıyla monoton

## Kanıt kaydı

**Kırmızı-önce doğrulaması.** Düzeltmeden önce üç yeni test
`nen-translate/src/checkpoint.rs`'e eklendi ve çalıştırıldı:

```
test checkpoint::tests::a_provider_reporting_its_own_block_total_completes_a_multi_block_run ... FAILED
  panicked: a provider that honestly reports its own block total must not be
  rejected: Block(Provider(Permanent))
test checkpoint::tests::each_blocks_progress_sequence_is_independently_monotonic ... FAILED
  panicked: run completes: Block(Provider(Permanent))
test checkpoint::tests::cancellation_still_stops_the_run_when_blocks_fork_their_progress ... FAILED
  left: Block(Provider(Permanent))
  right: Cancelled
test result: FAILED. 6 passed; 3 failed; 0 ignored
```

Üçü de tam olarak bu task'ın ölçtüğü sebepten düşüyor: tek `TranslationCall`
üzerinden yürüyen ikinci blok, kendi `total`'ını bir öncekinden farklı
bildirdiği için `Permanent`'a çarpıyor.

**Düzeltme.** `translate_checkpointed`
(`core/crates/nen-translate/src/checkpoint.rs`) artık her bloğun sağlayıcı
işini `&call.fork()` üzerinden sürüyor; blok sınırı `checkpoint()` çağrısı ve
`checkpoints.commit(call, ...)` dışarıdaki paylaşılan `call` üzerinde kalıyor
— iptal geçidi tek ve belge boyunca paylaşılmış, ilerleme dizisi blok başına
bağımsız. `nen-ports` (monoton `total` kuralı) dokunulmadı.

**Doğrulama — sonra (yeşil):**

```
$ cargo test -p nen-translate --lib checkpoint
running 9 tests
test checkpoint::tests::a_gate_closed_before_the_run_starts_is_checked_at_the_block_boundary ... ok
test checkpoint::tests::complete_checkpoints_become_completed_blocks_in_order ... ok
test checkpoint::tests::a_failed_block_is_never_checkpointed ... ok
test checkpoint::tests::incomplete_checkpoints_refuse_to_become_completed_blocks ... ok
test checkpoint::tests::each_blocks_progress_sequence_is_independently_monotonic ... ok
test checkpoint::tests::a_resumed_run_never_re_translates_checkpointed_blocks ... ok
test checkpoint::tests::cancellation_still_stops_the_run_when_blocks_fork_their_progress ... ok
test checkpoint::tests::interrupted_run_resumes_from_its_checkpoints_without_retranslating_them ... ok
test checkpoint::tests::a_provider_reporting_its_own_block_total_completes_a_multi_block_run ... ok
test result: ok. 9 passed; 0 failed
```

`nen-app`'in bu kusur yüzünden konmuş geçici fixture'ları (yorumlarında
`NEN-106`'ya referanslı) kaldırıldı ve gerçek `MockTranslationProvider`'a
geçirildi — artık kusurun regresyon kapısı:

- `nen-app/tests/artifact_store_roundtrip.rs`: `SilentEchoProvider` silindi;
  `a_multi_block_artifact_survives_the_store` gerçek `MockTranslationProvider`
  ile 95 cue'luk çok bloklu belgeyi çeviriyor
- `nen-app/tests/artifact_index_lookup.rs`: `layout.blocks().len() == 1`
  iddiası (yalnız bu kusur için vardı) kaldırıldı

```
$ cargo test -p nen-app --test artifact_store_roundtrip
running 3 tests
test the_same_translation_run_stores_at_the_same_address_twice ... ok
test an_assembled_artifact_survives_a_trip_through_the_store ... ok
test a_multi_block_artifact_survives_the_store ... ok
test result: ok. 3 passed; 0 failed
```

**Bütün kapılar:** `cargo test --workspace` → 809 passed / 1 ignored, 0
failed. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
-- -D warnings`, `cargo deny check` (advisories ok, bans ok, licenses ok,
sources ok) — hepsi temiz. `bash scripts/test.sh` 4/4 geçti. `nen-ffi` ve
macOS kabuğu dokunulmadı.

`docs/DECISIONS.md`, ADR eklenmedi/değiştirilmedi — `TranslationCall`'ın
monotonluk kuralı gevşetilmedi, yalnız kullanım deseni (`fork`) blok sınırına
taşındı.
