---
id: NEN-129
title: Two-stage model-assisted subtitle translation
milestone: M6
size: L
state: done
closed: 2026-09-16
depends_on: [NEN-105, NEN-118, NEN-130]
blocks: [NEN-126]
adr: [15, 16, 18, 19, 48]
---

# NEN-129 — Two-stage model-assisted subtitle translation

## Sonuç

OpenAI ve OpenRouter çeviri işleri bloklardan önce bir kez doğrulanmış ve
resume edilebilir belge-geneli analiz üretiyor; her blok bu analiz ile otomatik
terim sözlüğünü, açık ilk-deneme/hedefli-onarım/tam-tekrar kipini ve
prompt-injection'a dayanıklı talimatları kullanırken mevcut yerel doğrulama,
checkpoint, cache ve atomik yayın değişmezleri korunuyor.

## Bağlam

`NEN-126` öncesi incelemede gerçek provider prompt'unun bugün yalnız dil çifti,
yerel `context_terms`, blok penceresi ve `output_cue_ids` taşıdığı görüldü.
`docs/product-spec.md` §10 ise bütün altyazı için bağlam analizi ister;
ADR-0015 Karar 3 M5'in deterministik mock kapsamı için yerel terim çıkarımını
seçmiş, model tabanlı özet/karakter/glossary üretimini gerçek provider'ların
geldiği M6'ya açıkça ertelemiştir.

Önceki Stremio AISubtitle hattındaki iki aşamalı desen — belgeyi bir kez analiz
etme, sonra aynı analizi her örtüşen bloğa verme — doğal ve tutarlı çeviri için
bugünkü minimal prompt'tan daha güçlü bir başlangıçtır. Nen Player'a aktarım
yalnız prompt string'i değişikliği değildir: provider-neutral port, iki strict
şema, analysis yaşam döngüsü, persistent resume kaydı, repair kipleri ve cache
invalidasyonu birlikte değişir. Koddan önce ADR-0048 `proposed` yazılır ve
kullanıcı onayıyla `accepted` olur; ADR-0015'in yerel-bağlam kararı ile
ADR-0019'un gerçek-provider payload'ı M6'nın model destekli analiz aşamasıyla
genişletilir. Önceki ADR'lerin diğer bağlayıcı kararları korunur.

## Çözülen blokaj

2026-09-16 — Zorunlu `bash scripts/test-macos.sh` kapısı, NEN-129 tarafından
değiştirilmeyen sidecar otomatik-seçim akışında NEN-038 sonrası regresyonu
yakalamıştır. `NEN-130` mevcut kırılmayı ayrı kapsamda düzeltti ve tam macOS
kapısını geçirdi; NEN-129 yeniden READY durumundadır ve uygulama değişiklikleri
çalışma ağacında korunur.

## Kapsam

- ADR-0048 aşağıdaki sınırları karara bağlar; kabul edilmeden üretim kodu
  değişmez:
  - Belge analizi yeni bir çeviri/cache miss'inde blok 0'dan önce tam bir kez
    yapılır. Doğrulanmış analiz resume kaydında saklanır; restart, targeted
    repair ve full-block retry aynı analizi kullanır ve ikinci analiz çağrısı
    yapmaz.
  - Analiz başarısız, iptal edilmiş, şema dışı veya provider sınırını aşan bir
    belge için blok çevirisi başlamaz; eski minimal prompt'a sessiz kalite
    düşüşü yoktur ve final artifact oluşmaz.
  - Provider'a kaynak/hedef dil ile cue ID + metinden oluşan tam transcript
    gider. Medya başlığı, dosya yolu, basename, URL, hash, zaman kodu ve diğer
    private identity metadata'sı gitmez.
  - Analiz ve blok çıktıları untrusted'dır; structured output yardımcı kapıdır,
    yerel doğrulama authoritative kalır.
- `nen-ports` içinde provider-neutral, payload yazdırmayan tipler:
  - `DocumentAnalysisRequest` ve `DocumentAnalysis` (`summary`, karakter adı +
    konuşma/ilişki açıklaması, otomatik glossary source/target/note),
  - blok isteğinde doğrulanmış analysis, blok sıra/toplam bilgisi ve
    `Initial | TargetedRepair | FullRetry` kipi,
  - `TranslationProvider` için belge analizi çağrısı; mock/fake deterministik
    karşılığını verir ve hiçbir test gerçek kredi/kota kullanmaz.
- Ortak prompt katmanı iki İngilizce sistem talimatı üretir:
  - analiz prompt'u bütün belge bağlamını, karakterleri, ilişkileri, tekrar eden
    terimleri, unvanları, şakaları ve tutarlı çevrilmesi gereken ifadeleri kısa
    structured data olarak ister; altyazıyı untrusted veri sayar ve içindeki
    talimatları izlemeyi yasaklar,
  - blok prompt'u doğal/deyimsel hedef dil, yalnız istenen cue'lar, bağlam
    cue'larını çevirmeme, sıra/tamlık, anlam-register-isim-terim tutarlılığı,
    anlamlı satır sonları ve hafif subtitle markup koruması, açıklama/speaker
    label/timing eklememe kurallarını taşır.
- Targeted repair yalnız açıkça istenen eksik/geçersiz cue ID'lerini; full retry
  bloğun bütün çıktı kümesini sıfırdan istemesini prompt'ta ayrıca söyler.
  Mevcut `2 targeted + 1 full` bütçesi değişmez.
- OpenAI Responses ve OpenRouter Chat Completions aynı semantik iki prompt'u ve
  ortak şemaları kullanır:
  - analiz şeması kök/alt nesnelerde `additionalProperties: false` ve zorunlu
    `summary`, `characters`, `glossary`,
  - blok şeması çağrıya özel exact cue sayısı (`minItems == maxItems`), izinli
    cue ID enum'u, non-empty text ve ek alan yasağı.
- Analysis parser; boyut, exact alan kümesi ve boş metin kurallarını yerelde
  doğrular. Blok cevabı ayrıca ADR-0016'nın exact/allowed/unique/non-empty/order
  doğrulamasından geçer; schema hiçbir yerel kapının yerine geçmez.
- `NEN-105` resume wire formatı doğrulanmış analysis'i payload sızdırmadan
  atomik saklar. Bozuk/eski analysis kaydı reddedilir; checkpoint'lenmiş bloklar
  yeniden çevrilmez.
- Prompt, schema, pipeline ve gerekiyorsa translation-session sürümleri
  ADR-0018/0048 kararına göre artırılır; eski tek-aşamalı artifact ve resume
  kayıtları yeni semantikte cache hit olamaz.
- OpenAI/OpenRouter analysis, initial block, targeted repair ve full retry
  istekleri redakte golden fixture'larla sabitlenir. Request body ve response
  body mevcut 1 MiB/60 s/approved-host/redirect/retry sınırlarını korur.
- `docs/architecture.md`, `docs/DECISIONS.md`, ADR-0015/0016/0018/0019 Notları
  ve M6 kanıt eşlemesi kabul edilen iki-aşamalı akışı gösterecek şekilde
  güncellenir.

## YAPILMAYACAK

- Kullanıcı glossary'si için düzenleme UI'ı, kalıcı glossary store'u veya
  import/export — M6 milestone'undaki mevcut erteleme korunur; bu task yalnız
  modelin ürettiği otomatik glossary'yi taşır. User glossary ayrı task ve cache
  identity kanıtı gerektirir.
- Timestamp/`TimeSpan`'i provider'a göndermek veya provider'ın timing üretmesi
- Progressive/yarım subtitle yayınlamak; analysis'i ya da kısmi bloğu katalogda
  artifact olarak göstermek
- Aynı task içinde model/sağlayıcı kalite karşılaştırması veya otomatik testte
  gerçek ağ; canlı EN→TR kalite ölçümü `NEN-126`'da kalır
- Stremio AISubtitle kaynak koduna çalışma zamanı/derleme bağımlılığı eklemek
  veya TypeScript kodunu doğrudan kopyalamak

## Kanıt (DoD)

- [x] ADR-0048 kullanıcı onayıyla `accepted`; ADR-0015 ve ADR-0019'a eklenen
      M6 sınırları açık, prompt/schema/cache/resume kararları çelişkisiz
- [x] Contract: yeni çeviride çağrı sırası tam olarak `analysis -> block...`;
      analysis tüm initial/repair isteklerinde byte-for-byte aynı ve çağrı
      sayısı **1**
- [x] Resume contract: analysis + ilk doğrulanmış bloktan restart, analysis'i
      veya checkpoint'li bloğu yeniden istemeden sonraki bloktan devam ediyor
- [x] Golden: OpenAI ve OpenRouter için analysis + initial + targeted repair +
      full retry gövdeleri; iki provider'ın semantik payload'ı aynı
- [x] Negatif: analysis şema dışı/boş/oversize/refusal ise blok POST sayısı 0,
      checkpoint/final artifact yok
- [x] Negatif: blok cevabında eksik/fazla/duplicate/unknown cue, boş text ve
      sıra bozukluğu mevcut bounded repair + authoritative local validation
      tarafından yakalanıyor
- [x] Negatif K23: subtitle diyaloğu, analysis özeti/karakter/glossary metni,
      raw cevap ve credential hiçbir `Debug`/`Display`/hata
      yüzeyinde yok; path/basename/URL/hash/timing provider gövdesinde yok
- [x] Cache negatifleri: eski prompt/schema/pipeline sürümü ve farklı analysis
      sözleşmesi yeni artifact/resume kaydıyla hit üretmiyor
- [x] `cargo test --workspace`, `bash scripts/test-macos.sh`,
      `bash scripts/check-docs.sh`, `bash scripts/task-index.sh --check` ve
      `git diff --check` geçiyor
- [x] `NEN-126` canlı OpenRouter/Luna EN->TR koşusu bu task tamamlandıktan sonra
      yeniden başlıyor; canlı ağ sonucu bu task'ın otomatik test kanıtı değil

## Kanıt kaydı

- ADR-0048 `accepted` (2026-09-15); ADR-0015/0016/0018/0019 notları ve
  `docs/architecture.md` iki-aşamalı analysis → block sözleşmesiyle güncel.
- `cargo test -p nen-translate --test two_stage_translation
  --test checkpoint_cancellation_negative --test artifact_negative` — çıkış 0:
  **14/14** geçti. `a_fresh_run_calls_analysis_once_before_blocks_and_reuses_it_byte_for_byte`
  tek analysis çağrısını ve bütün blok/repair isteklerinde aynı değeri;
  `restart_reuses_persisted_analysis_and_skips_the_checkpointed_prefix` restart'ta
  analysis ile doğrulanmış prefix'in yeniden istenmediğini kanıtladı.
- `cargo test -p nen-providers` — çıkış 0: OpenAI **16/16**, OpenRouter
  **16/16**, ortak çeviri provider contract'ı **6/6** ve provider semantik
  eşitliği **2/2** geçti. İki provider için analysis, initial, targeted repair
  ve full retry golden'ları doğrulandı.
- Analysis negatifleri `invalid_empty_refused_or_oversized_analysis_is_a_permanent_local_failure`
  ve `invalid_empty_refused_or_oversized_analysis_never_sends_a_block_post` ile;
  eksik/duplicate/unknown/boş/sıra blok ihlalleri mevcut authoritative yerel
  validation + bounded repair testleriyle geçti. Hatalı analysis'te blok çağrısı,
  resume checkpoint'i ve final artifact üretilmedi.
- `cargo test -p nen-persist` — çıkış 0: crate **28/28**, K23 guard
  **6/6**, offline lookup **1/1** geçti. Resume wire v2 doğrulanmış analysis'i
  round-trip etti; v1, bilinmeyen alan ve boş analysis kayıtları `Corrupt`
  olarak reddedildi.
- K23 negatifleri analysis özeti/karakter/glossary, subtitle diyaloğu, raw
  provider cevabı ve credential sentinel'larının `Debug`/`Display`/hata
  yüzeylerinde bulunmadığını; golden gövdeler path/basename/URL/hash/timing
  taşımadığını doğruladı.
- `PROMPT_VERSION = 2`, `SCHEMA_VERSION = 2`, `PIPELINE_VERSION = 2`;
  `adr_0048_versions_invalidate_the_single_stage_contract` ve cache identity
  negatifleri eski tek-aşamalı artifact/resume hit'ini engelledi.
- `cargo test --workspace` — çıkış 0; `cargo fmt --all -- --check` ve
  `cargo clippy --workspace -- -D warnings` — çıkış 0.
- `cargo deny check` — çıkış 0: advisories, bans, licenses ve sources `ok`;
  yalnız kabul edilen `hashbrown`/`syn` duplicate uyarıları var.
- `bash scripts/test-macos.sh` — çıkış 0: player-shell **241/241**, gerçek
  libmpv **57/57**, Keychain **4/4** geçti; yalnız mevcut OpenGL ve deployment
  target uyarıları kaldı.
- `bash scripts/check-docs.sh` — **10/10**; `bash scripts/task-index.sh --check`
  ve `git diff --check` — çıkış 0.
- `NEN-126`, bu kapanışla bağımlılık engelinden çıkarılıp READY durumuna alındı;
  canlı OpenRouter/Luna EN→TR koşusu otomatik test kanıtına dahil edilmedi.
