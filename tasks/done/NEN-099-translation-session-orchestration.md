---
id: NEN-099
title: Translation session orchestration without retarget
milestone: M5
size: M
state: done
closed: 2026-09-10
depends_on: [NEN-098, NEN-019]
blocks: [NEN-100]
adr: []
---

# NEN-099 — Translation session orchestration without retarget

## Sonuç

Kaynak seçmek çeviri başlatmıyor; açık bir komut bir çeviri işi başlatıyor ve iş
sırasında kullanıcının başka bir kaynağa geçmesi işi **retarget etmiyor**.

## Bağlam

Şartname §9 bu davranışı tek tek sayıyor: kaynak seçmek AI çeviri **başlatmaz** ·
iş başlangıç source fingerprint'ine bağlı kalır · yeni source'a retarget edilmez ·
kullanıcı iptal edebilir · sonuç hedef dil grubuna eklenir · kullanıcı başka
source izliyorsa **zorla** AI çıktısına geçilmez.

Sonuç `SubtitleSourceKind::Ai` olarak kataloğa girer — bu varyant
`nen-domain/src/source.rs` içinde **zaten var** (NEN-019), yeni bir tür açılmaz.

M5'in birinci ve altıncı çıkış kriterleri buraya bakıyor.

## Kapsam

- `nen-app` içinde çeviri oturumu use-case'i: başlatma, ilerleme, iptal, sonuç
- İşin başlangıç `SourceFingerprint`'ine bağlanması ve retarget'ın reddi
- Tamamlanan artifact'in kataloğa `Ai` kaynağı olarak eklenmesi
- Kaynak zaten hedef dildeyse işin hiç başlatılmaması
- Cache isabetinde provider'a hiç gidilmemesi

## YAPILMAYACAK

- FFI yüzeyi — `NEN-100`
- macOS komutu, ayarı ve ilerleme yüzeyi — `NEN-101` · `NEN-102`
- Aynı anda birden fazla çeviri işi — kapsam dışı, gerekirse yeni backlog task'ı

## Kanıt (DoD)

- [x] Integration: kaynak seçimi → açık komut → doğrulanmış artifact akışı uçtan uca geçiyor (mock provider ile)
- [x] Negatif: yalnız kaynak seçmek hiçbir provider çağrısı üretmiyor (çağrı sayacı 0)
- [x] Negatif: iş sırasında başka kaynak seçilince iş **başlangıç fingerprint'inde** kalıyor ve retarget edilmiyor
- [x] Negatif: kullanıcı başka bir source izlerken tamamlanan AI çıktısına **zorla geçilmiyor**
- [x] Unit: kaynak zaten hedef dildeyse iş başlatılmıyor
- [x] Unit: cache isabetinde provider hiç çağrılmıyor, artifact doğrudan dönüyor
- [x] Negatif: iptal edilen iş kataloğa hiçbir şey eklemiyor

## Kanıt kaydı

**Yeni `nen-app::translation` modülü, `SubtitleLibrary` ile `nen-translate`'in
pipeline'ını ilk kez bağlıyor.** Bugüne kadar bu iki yarıyı aynı anda gören tek
yer `artifact_store_roundtrip.rs`/`artifact_index_lookup.rs` test dosyalarıydı;
ürün kodunda "kullanıcı açık komut verdi" diyen bir use-case yoktu.
`prepare(library, token, target_language, block_layout, provider_identity, seed)`
kaynağı okuyup **değişmez** bir `TranslationJob` snapshot'ı kurar (kaynak
belgenin klonu, `TranslationPlan`, `BlockLayout`, `ArtifactMetadata`, ve
hepsinden önceden hesaplanmış `CacheKey`); `start(job, provider, store, index,
sink)` işi kendi worker thread'inde başlatıp hemen bir `TranslationJobHandle`
döner (ADR-0004 Karar 1 — "üst seviye session çağrıyı uygun worker üzerinde
yürütür", yeni bir runtime veya iptal mekanizması açılmadan; `cancel()`
doğrudan `TranslationCall::cancel`'a iner).

**Retarget, çalışma zamanında reddedilen bir durum değil — var olmayan bir
API.** `TranslationJob` kurulduktan sonra `SubtitleLibrary`'ye hiçbir yoldan
geri erişemiyor; worker yalnız kendi snapshot'ını sürüyor. Bu yüzden
`a_running_job_keeps_the_start_fingerprint_and_the_ai_output_is_never_forced_onto_screen`
testi bir "reddetme" değil, bir **imkânsızlığı** ölçüyor: gerçek
`PlaybackSession` + `MockCallGate` ile iş havadayken kullanıcı ikinci bir
kaynağa geçiyor, iş bitince `ArtifactRecord.source_fingerprint`'in başlangıç
kaynağınınkiyle birebir aynı ve ikinci kaynağınkinden farklı olduğu, **ve**
sonucu kataloğa ekledikten sonra ekranda hâlâ ikinci kaynağın gösterildiği
(`[tr]` işaretli AI metni asla zorla görünmüyor) doğrulanıyor.

**Katalog yazımı ayrı ve açık bir adım.** `SubtitleLibrary::add_translation`
`SubtitleSourceId::ai` (NEN-019'da zaten var olan varyant) ile `Ai` girişini
`add_file`/`add_embedded`'in aynı deseniyle ekliyor; oturumun bu metoda hiçbir
çağrı yolu yok. "Zorla AI çıktısına geçilmiyor" bu yüzden bir kontrol değil,
bir mimari gerçek.

**Cache isabeti, worker'ın ilk adımı.** `run_job` `index.find(job.cache_key)`
ile başlıyor; isabet `store.get` ile okuyup provider'a hiç gitmeden dönüyor,
kaçırma `translate_checkpointed` → `artifact::assemble` → `store.put`
zincirine düşüyor. `a_cache_hit_never_calls_the_provider_while_a_changed_block_layout_does`
aynı işi iki kez çalıştırıp ikincisinde `provider.calls()`'ın artmadığını,
ardından yalnız blok düzenini (ADR-0018'in bir bileşeni) değiştirip üçüncü
çalışmanın hem cache'i kaçırdığını hem provider'ı gerçekten çağırdığını
kanıtlıyor.

**İptal, `NEN-093`'ün zaten kanıtlanmış gate'inin üstünde duruyor — ikinci bir
mekanizma açılmadı.** Bir provider çağrısı `MockCallGate` ile havada tutulup
`handle.cancel()` çağrılınca, gate kapanıp gecikmiş cevap reddediliyor;
`translate_checkpointed`'in bunu bazen kendi `TranslationRunError::Cancelled`'ı,
bazen (çağrı provider'ın kendi checkpoint kontrolünde yakalanırsa)
`Block(Provider(Cancelled))` olarak döndürdüğü ölçülüp `run_job`'da ikisi de
tek bir `TranslationError::Cancelled`'a normalize edildi — orkestrasyon
katmanının kendi sözlüğünde ikisi de aynı şeyi ifade ediyor.
`cancelling_a_running_job_leaves_no_artifact_and_no_catalog_entry` iptalden
sonra `index.entries()`'in boş kaldığını ve kataloğa hiçbir `Ai` girişinin
eklenmediğini doğruluyor.

**Dört kapı elle mutasyona uğratıldı, her birinde tam olarak beklenen test(ler)
kırmızıya döndü, başkası etkilenmedi (kontrol sağır değil):**
- Cache lookup'ı atlatan mutasyon (`if false { … }`) yalnız
  `a_cache_hit_never_calls_the_provider_while_a_changed_block_layout_does`'u
  kırdı — diğer üç `translation_session.rs` testi ve tüm `translation_retarget.rs`/
  `guard_translation_debug.rs` yeşil kaldı.
- `AlreadyTargetLanguage` kapısının kaldırılması yalnız
  `a_source_already_in_the_target_language_never_starts`'ı kırdı; hata
  mesajının kendisi `TranslationJob`'ın `Debug` çıktısını da bastı ve dialog
  metni taşımadığını doğruladı.
- `SubtitleLibrary::add_translation` içindeki `catalog.insert(source)`
  satırının atlanması (yalnız o çağrı satırı, `add_file`/`add_embedded`'inkiler
  değil) yalnız `a_new_translation_completes_and_becomes_an_ai_catalog_entry`'yi
  kırdı.
- Cache identity girdisindeki `block_layout`'un gerçek config yerine sabit
  `BlockLayoutConfig::default()` kullanması yalnız aynı testin "sağırlık
  kontrolü" iddiasını (`!third.from_cache`) kırdı.

İptalde geç commit'i zorlayan beşinci bir mutasyon ayrıca denendi ama anlamlı
biçimde uygulanamadı: `store.put`'a giden her yol zaten `translate_checkpointed`'in
kendi `Cancelled` hatası **ve** `BlockCheckpoints::into_completed`'ın
(`NEN-093`) "her blok checkpoint'lenmeden tamamlanmış sayılamaz" değişmezi
tarafından iki bağımsız katmanda kapatılıyor — bu task yeni bir gate açmadı,
`NEN-093`'ün zaten mutasyon testinden geçmiş gate'ini yeniden kullandı.

**K23 guard'ları (`tests/guard_translation_debug.rs`, 3 test).**
`TranslationJob::Debug` yalnız `cue_count`/`block_size`/`overlap`/
`has_media_hash`/`has_glossary` basıyor — dialog metnini de glossary adını da
sızdırmıyor; `TranslationOutcome::Debug` zaten redakte edilmiş
`ArtifactRecord`/`ContentAddress`'e devrediyor. Kasıtlı `#[derive(Debug)]`
ikizi sentinel'ları sızdırdı (guard sağır değil).

`cargo test --workspace` **818 passed / 1 ignored** (`NEN-106` baseline 809 +
bu task'ın 9 testi: 4 `translation_session.rs` + 2 `translation_retarget.rs` +
3 `guard_translation_debug.rs`); fmt, clippy
(`--workspace --all-targets -- -D warnings`), `cargo deny check`
(yeni dış bağımlılık yok — `core/Cargo.lock` diff'i boş, `nen-app` zaten
`nen-translate`/`nen-persist`/`nen-providers`'a bağımlıydı), `bash
scripts/test.sh` **4/4** ve `bash scripts/check-docs.sh` yeşil.
`nen-ffi`, macOS kabuğu ve `nen-persist`/`nen-translate`/`nen-ports` dokunulmadı
— yalnız `nen-app` (yeni `translation.rs` + `subtitles.rs`'e tek metot).
