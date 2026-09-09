---
id: NEN-100
title: Translation FFI surface with progress, cancel and log guard
milestone: M5
size: M
state: done
closed: 2026-09-10
depends_on: [NEN-099]
blocks: [NEN-101]
adr: []
---

# NEN-100 — Translation FFI surface with progress, cancel and log guard

## Sonuç

Bir çeviri işi FFI sınırından başlatılıp ilerlemesi izlenebiliyor ve iptal
edilebiliyor; çevrilen metin bu sınırdan hiçbir log yüzeyine sızmıyor.

## Bağlam

`nen-ffi` **tek dış kapıdır** (`docs/architecture.md` → crate tablosu; ADR-0006
kural 2). `ADR-0033` Karar 3, playback olay akışının teslimat yönünü **pull**
olarak sabitliyor — core kabuğa itmiyor, kabuk `drain_events()` çağırıyor;
push (`EventSink`) o kararda açıkça reddedilen alternatif. Bu, playback'in
~30 Hz'lik sürekli olay akışına özgü bir tercih (ADR-0033 gerekçesi). Bir
çeviri işinin ilerlemesi aynı profile girmiyor — blok başına birkaç kesikli
olay — ve `ADR-0004` Karar 1/2/5 zaten kendi `JobHandle` + tek delivery gate
desenini tanımlıyor (`tasks:` alanında bu task adıyla anılıyor). Planlama
sırasında kullanıcı kararıyla netleşti: çeviri ilerlemesi bir foreign
`ForeignTranslationProgressSink` ile **push** taşınır; ADR-0033'e bu ayrımı
kaydeden bir Notlar girdisi eklenir, yeni bir ADR açılmaz (ADR-0043/`NEN-081`
emsali).

K23 #4 (subtitle diyaloğu) burada en yüksek riskli sınır: cue metni FFI'dan
geçtiği için `Debug`/`Display` türevleri elle yazılmalı. Emsal: `NEN-083`'ün
handoff guard'ları ve `NEN-080`'in elle yazılmış locator `Debug`'ı.

## Kapsam

- Çeviri işini başlatan, ilerlemesini ileten ve iptal eden FFI yüzeyi
- Yardımcıların (provider/store/index) `nen-app` tarafında bir kompozisyon
  kökünde kurulması — `nen-ffi` yalnız ilkel değer alır, yeni crate'e
  bağlanmaz (ADR-0006 kural 3)
- Biten işin sonucunun `SubtitleLibrary::add_translation` ile kataloğa
  eklenmesi için FFI çağrısı (`NEN-099`'un ayrı ve açık adımı)
- Tipli hataların FFI taksonomisine bağlanması — bugün `nen-ffi`'de zaten var
  olan kod deseni (ADR-0006 kural 2 + K23); `docs/adr/0005-*` henüz yazılmadı
  (`docs/adr/README.md` → "Planlanan"), bu task onu kararlaştırmıyor
- Cue metni taşıyan her tipin `Debug`'ının shape-only olması

## YAPILMAYACAK

- macOS kabuğu — `NEN-101` · `NEN-102`
- Kotlin/Android bağlaması — M10
- Çeviri ilerlemesi için birden fazla teslimat yolu (yalnız push, drain
  yok) — mevcut `TranslationCall` gate'i tek mekanizma kalır
- Playback olay akışının pull yönü — `ADR-0033`'ün kendi kapsamı değişmez,
  yalnız notla genişler

## Kanıt (DoD)

- [x] FFI testi: iş başlatılıyor, ilerleme olayları sırayla geliyor, sonuç teslim ediliyor
- [x] Negatif: iptal sonrası hiçbir sonuç olayı gelmiyor (late commit yok)
- [x] Guard: FFI yüzeyinden çıkan hiçbir `String`/`Debug`/hata gösterimi cue metni,
      medya URL'si veya özel yol taşımıyor — negatif kontrolle (K23 #1, #3, #4)
- [x] Negatif: guard geçici olarak kaldırıldığında tarama testi kırmızıya dönüyor
- [x] `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` yeşil

## Kanıt kaydı

**`nen-ffi::translation` çeviri işini açan tek dış kapı oldu — bir iş artık
FFI sınırından başlatılabiliyor, ilerlemesi izlenebiliyor, iptal edilebiliyor
ve sonucu kataloğa eklenebiliyor; `nen_app::translation`'ın (`NEN-099`)
hiçbir iç tipi bu sınırı geçmiyor.** `nen-app`'e yeni bir kompozisyon kökü —
`TranslationEnvironment` — eklendi: `nen-ffi` hâlâ yalnız `nen-app`'e bağımlı
(ADR-0006 kural 3 mekanik olarak korunuyor, `Cargo.toml` değişmedi), ama
`nen-persist::FilesystemArtifactStore`'u M5'in deterministic mock
provider'ıyla (`nen-providers`) burada eşliyor. Gate yalnız ilkel değer alıp
veriyor: bir depo kök yolu (`String`), bir token (`u32`), bir hedef dil
(`String`) — `BlockLayoutConfig`, `ArtifactId`, `GlossaryIdentity` gibi hiçbir
`nen-translate` tipi FFI'ya hiç adlandırılmadı.

**İlerleme push ile taşınıyor — ADR-0033'ün playback için sabitlediği pull
yönünün bir istisnası değil, kapsamı dışı bir karar.** Planlama sırasında
kullanıcı kararıyla netleşti ve ADR-0033'e bunu kaydeden bir Notlar girdisi
eklendi (ayrı commit, `5b26cc3`): Karar 3'ün pull tercihi playback'in ~30
Hz'lik sürekli olay akışına özgü (bir dispatch kuyruğunda birikme riski); bir
çeviri işinin ilerlemesi blok başına birkaç kesikli olay ve zaten
`nen_ports::translation::TranslationCall`'ın (ADR-0004 Karar 2/5) tek
delivery gate'i bir push'un ihtiyaç duyduğu şey. Yeni
`ForeignTranslationProgressSink` (`#[uniffi::export(with_foreign)]`) bunu
taşıyor; ikinci bir kuyruk veya `drain_*` yolu açılmadı.

**İptal, ikinci bir mekanizma açılmadan `TranslationCall`'ın kendi geçidinin
dış kabuğu.** `FfiTranslationJob::cancel()` doğrudan
`TranslationJobHandle::cancel()`'a iniyor. Gerçek bir çok-thread'li testte
(`translation_gate.rs::cancelling_a_paused_job_...`) sink kendi callback'i
içinde — delivery gate'in kilidini tutarken — duraklatılıyor, başka bir
thread `cancel()` çağırıyor (bu nedenle aynı kilide takılı kalıyor), sonra
callback serbest bırakılıyor. Ölçülen değişmez: `cancel()` döndüğü an
kaydedilen olay sayısı, işçi thread'i tamamen bitene kadar **artmıyor** — bu,
"kaç olayın iptalden önce yarıştığı"ndan bağımsız, ADR-0004 Karar 2'nin
kendisinden doğan bir garanti (`TranslationCall::progress`'in `cancelled`
kontrolü sink'e ulaşmadan önce). İptal edilen iş `Err(Cancelled)` döndürüyor,
`artifacts/` boş kalıyor ve kataloğa hiçbir `Ai` satırı eklenmiyor.

**Test, `nen-ports`'un kendi ölçülmüş yarış deseninin aynısını izliyor.**
İlk yazımda bu test 31 koşudan birinde `Ok` ile yanlış geçti — paused
callback'in kilidi bıraktığı an, aynı işçi thread'i yeni bir OS thread'in
zamanlanmasından önce kilide "barge" edip kalan tüm cue'ları bitirebiliyordu.
Düzeltme `nen-ports/src/translation/mod.rs`'nin kendi
`cancel_waits_for_an_in_flight_commit_and_blocks_every_commit_after`
testinin **aynı** çaresini uyguladı: canceller thread'e kilide ulaşması için
kısa bir `thread::sleep(50ms)` payı. Düzeltmeden sonra 100/100 koşu yeşil
(ayrıca 30/30'luk bağımsız bir ön-ölçüm).

**Beş kapı elle mutasyona uğratıldı, her birinde tam olarak beklenen
test(ler) kırmızıya döndü, sonra geri alındı** (`diff` ile orijinale dönüş
doğrulandı):
1. `catalog_into`'nun `Done`-only şartı kaldırılınca (her durumda `Some(0)`
   dönünce) — yalnız `catalog_into_answers_none_while_running_and_some_once_joined`
   ve `cancelling_a_paused_job_...` kırmızı oldu.
2. `join()`'ün ikinci çağrıda `AlreadyJoined` yerine eski özeti tekrar
   döndürmesi — yalnız `joining_a_job_twice_answers_already_joined_the_second_time`
   kırmızı oldu.
3. `StartRefusal::AlreadyTargetLanguage → FfiTranslationStartError` eşlemesi
   yanlış varyanta (`Unusable`) çevrilince — yalnız
   `starting_is_refused_when_the_source_is_already_the_target_language`
   kırmızı oldu.
4. `FfiTranslationSummary`'ye `record.webvtt`'i taşıyan bir alan eklenince
   (K23 #4 ihlali) — yalnız guard dosyasındaki
   `a_real_run_never_lets_the_dialogue_or_the_store_root_reach_progress_or_summary_output`
   kırmızı oldu, diğer üç guard testi (sağır olmadığının kanıtı) yeşil kaldı.
5. `TranslationProgress → FfiTranslationProgress`'te `done`/`total` alanları
   takas edilince — yalnız `progress_events_arrive_in_order_and_the_summary_matches_the_source`
   kırmızı oldu.

**K23 guard'ı (`guard_ffi_translation_debug.rs`, 4 test) gerçek bir koşuyla
ölçüyor, tip düzeyinde varsayım yapmıyor.** `nen-app`'in kendi
`guard_translation_debug.rs`'inden farkı: bu sınırdaki hiçbir tipin elle
yazılmış `Debug`'ı yok — hepsi kapalı enum/sayı/dil etiketinden ibaret
(`FfiTranslationProgress`, `FfiTranslationSummary`, iki hata tipi). Guard
bunun yerine sentinel diyalog + sentinel'lı depo kök yoluyla gerçek bir iş
koşturup topladığı her `FfiTranslationProgress`'in ve nihai
`FfiTranslationSummary`'nin `{:?}` çıktısını tarıyor; iki hata tipinin
**her varyantının** hem `{:?}` hem `{}` çıktısı ayrıca taranıyor. Sağır
olmadığı hem pozitif alan kontrolleriyle (yukarıdaki mutasyon 4) hem elle
yazılmış `#[derive(Debug)]` ikiz testiyle (`nen-app` emsali) gösteriliyor.

**Uçtan uca kabuk kanıtı: `bash scripts/build-apple.sh` binding üretimini
kırmadan tamamladı** ve üretilen `nen_ffi.swift`, `FfiTranslationEngine`,
`FfiTranslationJob`, `ForeignTranslationProgressSink`,
`FfiTranslationProgress`, `FfiTranslationSummary` ve her iki hata tipini
gerçek Swift tipleri olarak içeriyor — `NEN-101` bugünden çağrılabilir bir
yüzey buluyor. Üretilen dosyalar `.gitignore`'lu, commit edilmedi.

`cargo test --workspace` **831 passed / 1 ignored** (`NEN-099` baseline 818 +
bu task'ın 13 testi: `translation_gate.rs` 9, `guard_ffi_translation_debug.rs`
4); fmt, clippy (`-D warnings`), `cargo deny check` (yeni dış bağımlılık yok —
`Cargo.lock` diff'i boş) ve `bash scripts/test.sh` **4/4** yeşil. Doküman
düzeltmesi (ADR-0033 Notlar + task dosyasının yanlış ADR-0033/ADR-0005
referansları) ayrı commit'te (`5b26cc3`) zaten yeşil push edildi. `nen-persist`,
`nen-translate`, `nen-providers`, macOS kabuğu dokunulmadı — yalnız `nen-app`
(yeni `TranslationEnvironment`) ve `nen-ffi` (yeni `translation` modülü).
