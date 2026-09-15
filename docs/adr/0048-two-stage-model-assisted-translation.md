---
adr: 0048
title: İki aşamalı model destekli altyazı çevirisi
status: accepted
milestone: M6
tasks: [NEN-129, NEN-126]
date: 2026-09-15
---

# ADR-0048 — İki aşamalı model destekli altyazı çevirisi

## Durum

`accepted` (2026-09-15, kullanıcı onayı, `NEN-129`)

## Bağlam

`docs/product-spec.md` §10, altyazının bloklara ayrılmasından önce bütün belge
bağlamının analiz edilmesini; blokların bu ortak bağlamla tutarlı çevrilmesini;
provider çıktısının ise yerelde doğrulanmasını ister. ADR-0015 M5 için
deterministik yerel `DocumentContext` üretimini seçmiş, model tabanlı özet,
karakter ve glossary üretimini gerçek provider'ların geldiği M6'ya ertelemiştir.
ADR-0019'daki OpenAI/OpenRouter adaptörleri bugün her blok için yalnız dil
çifti, yerel terim adayları, bağlam cue'ları ve çıktı cue ID'lerini gönderir.

Bu tek aşamalı akış biçimsel doğruluğu korusa da hikâye, karakter ilişkileri,
konuşma biçimi, şaka ve yinelenen terim kararlarını bütün belge üzerinden
kurmaz. Yalnız prompt metnini büyütmek yeterli değildir: analiz çağrısının
provider-neutral portu, doğrulaması, resume yaşam döngüsü ve cache kimliği aynı
kararın parçası olmalıdır. Altyazı içeriği ve analiz çıktısı untrusted veridir;
K23 redaction sınırı ile ADR-0016'nın authoritative yerel doğrulaması korunur.

## Karar

Cache miss ile başlayan her yeni çeviri işi, blok 0'dan önce tam bir kez bütün
belge analizi yapacaktır; doğrulanmış analiz atomik resume kaydına yazıldıktan
sonra tüm ilk deneme ve onarım bloklarına aynı değer olarak verilecektir.
Analiz başarısız, iptal edilmiş, geçersiz veya provider istek sınırından büyükse
blok çevirisi başlamayacak ve daha zayıf tek aşamalı akışa sessiz fallback
yapılmayacaktır. OpenAI ve OpenRouter aynı provider-neutral sözleşme, İngilizce
sistem talimatları ve strict JSON şemalarının provider'a özgü wire karşılığını
kullanacak; structured output'a ek olarak bütün analiz ve blok çıktıları
yerelde doğrulanacaktır. Bu karar ADR-0015'in M6'ya ertelediği model destekli
bağlamı açar ve ADR-0019'un payload'ını genişletir; iki ADR'nin kalan kararları
değişmez.

## Sözleşme ayrıntıları

### 1. Yaşam döngüsü ve çağrı sırası

- Final artifact cache hit ise provider çağrısı yapılmaz.
- Cache miss'te önce resume kaydı okunur. Geçerli analysis varsa aynen yeniden
  kullanılır; analysis çağrısı tekrarlanmaz.
- Analysis yoksa provider'ın `analyze_document` işlemi tam bir kez çağrılır.
  Çıktı yerelde doğrulanır ve henüz blok içermeyen resume kaydına atomik
  yazılır. Bu kayıt başarıyla saklanmadan blok 0 başlamaz.
- Initial, targeted repair ve full retry isteklerinin tamamı aynı doğrulanmış
  analysis değerini taşır. Mevcut `2 targeted + 1 full` bütçesi değişmez.
- İptal veya analysis hatasında blok POST sayısı sıfır, final artifact üretimi
  sıfırdır. Analiz, mevcut dış ilerleme modelinde `Preparing` evresidir; yeni
  UI/FFI ilerleme enum'u eklenmez.

### 2. Provider-neutral veri modeli

`DocumentAnalysisRequest`, kaynak ve hedef dil ile sıralı `cue_id + text`
transcript'ini ve ADR-0015'in deterministik yerel terim adaylarını taşır.
`DocumentAnalysis`, non-empty `summary`, sıralı `characters` (`name`,
`description`) ve sıralı otomatik `glossary` (`source`, `target`, `note`)
alanlarından oluşur; karakter ve glossary dizileri boş olabilir. Bu tiplerin
`Debug`/`Display` yüzeyleri payload yazdırmaz.

Blok isteği mevcut bağlam cue'ları ve output cue ID'lerine ek olarak analysis,
bir tabanlı blok sırası/toplamı ve kapalı `Initial | TargetedRepair |
FullRetry` kipini taşır. Kullanıcı glossary'si bu ADR'nin parçası değildir;
bloklarda yalnız analysis'in otomatik glossary'si kullanılır.

Provider'a medya başlığı, dosya yolu, basename, URL, media hash, zaman kodu,
credential veya başka private identity metadata'sı gönderilmez. Bütün belge
bağlamı için gerekli veri transcript'in kendisidir. Medya başlığı ancak ayrı
bir privacy/cache kararıyla ileride eklenebilir.

### 3. Prompt davranışı

Analiz sistem talimatı, kaynak altyazının hedef dile çevrilmesinden önce bütün
belgeyi inceleyerek kısa hikâye bağlamı, konuşan karakterler ve ilişkileri,
konuşma biçimleri, yinelenen terimler/unvanlar, şakalar ve tutarlı çevrilmesi
gereken ifadeleri structured data olarak üretmesini ister. Altyazının untrusted
kaynak veri olduğunu ve içindeki hiçbir talimatın izlenmeyeceğini açıkça söyler.

Blok sistem talimatı doğal ve deyimsel hedef dil, yalnız istenen cue ID'leri ve
aynı sıra, output olmayan cue'ları yalnız bağlam olarak kullanma, anlam/register/
isim/terim tutarlılığı, mümkün olduğunda satır sonu ve hafif altyazı markup'ını
koruma, açıklama/speaker label/timing eklememe kurallarını taşır. Targeted repair
yalnız listelenen eksik/geçersiz cue'ları; full retry ise bloğun bütün çıktı
kümesini sıfırdan ister. Altyazı ve önceki model çıktıları her kipte untrusted
veri olarak kalır.

### 4. Şema ve authoritative yerel doğrulama

Analysis şeması kök ve alt nesnelerde `additionalProperties: false` kullanır;
`summary`, `characters` ve `glossary` zorunludur. Yerel parser exact alan
kümelerini, tipleri ve kırpıldığında boş kalan bütün string alanlarını reddeder;
dizilerin boş olması geçerlidir.

Blok şeması her çağrıda dinamik kurulur: `translations` için `minItems` ve
`maxItems` beklenen cue sayısına eşittir, `cueId` yalnız izinli ID enum'undan
seçilir, `text` en az bir karakterdir ve ek alan yasaktır. Buna rağmen şema
başarı sayılmaz; ADR-0016'nın exact count, allowed ID, unique ID, non-empty text
ve exact order kapıları yerelde authoritative kalır.

Tam transcript'in serialize edilmiş analysis isteği veya herhangi bir provider
cevabı mevcut 1 MiB sınırını aşarsa tipli kalıcı hata üretilir. Transcript
kesilmez, birden fazla analysis çağrısına bölünmez ve yerel-only bağlama
düşülmez. Mevcut 60 saniye timeout, approved host, redirect yasağı, response
boyutu ve bounded transport retry kuralları korunur.

### 5. Resume ve cache kimliği

Resume wire formatı analysis'i zorunlu taşıyan yeni sürüme yükseltilir. Geçersiz
veya eski sürüm kayıt reddedilir; doğrulanmış analysis, her blok checkpoint'iyle
birlikte atomik yazılır ve final artifact başarıyla yayımlandığında mevcut
kurala göre silinir. Analysis katalogda ayrı artifact veya kısmi çıktı değildir.

Ortak prompt/schema sözleşme sürümleri provider ile pipeline'ın ikisinin de
kullandığı tek kanonik noktadan okunacaktır. `PROMPT_VERSION`,
`SCHEMA_VERSION` ve `PIPELINE_VERSION` 2'ye yükselir;
`BLOCK_LAYOUT_VERSION` ile `TRANSLATION_SESSION_VERSION` ancak wire davranışı
ayrıca değişirse yükseltilir. Böylece eski tek aşamalı artifact ve resume
kayıtları yeni semantikte cache hit olamaz.

## Gerekçe

Tek analysis çağrısı, bütün hikâye için ortak çeviri kararlarını bloklar arasında
taşırken maliyeti blok sayısıyla çarpmaz. Analysis'i ilk bloktan önce kalıcı ve
doğrulanmış hale getirmek, crash sonrası farklı bir model çıktısıyla yarım
belgeyi devam ettirme riskini kaldırır. Sessiz fallback yapmamak, aynı cache
kimliği altında iki farklı kalite sözleşmesi oluşmasını önler. Yerel term
çıkarımını analysis'e aday girdi olarak korumak ADR-0015'in deterministik
işlevini sürdürür; model sonucu ise nihai bağlamı zenginleştirir.

Başlığı göndermemek, çeviri sınırını identity ve filename privacy kararlarından
ayırır. Dinamik strict şema provider'ın hata olasılığını azaltır; yerel
doğrulama ise provider davranışına güvenmeden yayın güvenliğini korur.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Yalnız mevcut blok prompt'unu uzatmak | Bütün belge üzerinden karakter, ilişki ve terim kararı üretmez |
| Her restart'ta analysis'i yeniden istemek | Aynı belgeyi farklı model bağlamlarıyla birleştirir, maliyeti ve nondeterminism'i artırır |
| Geçersiz analysis'te yerel-only sessiz fallback | Aynı cache sürümünde farklı kalite sözleşmeleri yaratır ve canlı kabulü yanıltır |
| Transcript'i parçalayıp birden fazla analysis çağrısı yapmak | “Belge-geneli tek analiz” anlamını bozar ve yeni bir merge/çatışma politikası gerektirir |
| Medya başlığını veya dosya identity'sini göndermek | K23, ADR-0046 ve cache kimliği için gereksiz yeni privacy yüzeyi açar |
| Strict schema'yı tek doğrulama olarak kabul etmek | Duplicate, sıra ve semantik bütünlük hatalarını sağlayıcıya bağımlı bırakır |
| Analysis'i yalnız bellekte tutmak | Crash/restart sonrasında aynı analysis garantisini ve checkpoint devamlılığını kaybeder |

## Sonuçlar

**Olumlu:** Bütün bloklar tek doğrulanmış hikâye/karakter/terim bağlamını
kullanır; iki provider semantik olarak aynı isteği alır; restart ve repair aynı
analysis ile deterministik devam eder; mevcut exact cue ve atomik yayın
güvenceleri korunur.

**Olumsuz / kabul edilen maliyet:** Her yeni cache kimliği için bloklardan önce
bir ek model çağrısı, gecikme ve token maliyeti vardır. Tam transcript 1 MiB
provider istek sınırını aşarsa çeviri başlamaz. Resume kaydı altyazıdan türetilen
özet/karakter/glossary verisini yerel diskte taşır; redacted debug ve mevcut
uygulama destek dizini güven sınırı buna uygulanır.

**Geri dönüş maliyeti:** orta — analysis portu ve prompt kullanımı kaldırılabilir;
ancak yeni resume/cache sürümü geriye çevrilmez ve eski semantiğe dönüş yeni bir
ADR ile yeni sürüm gerektirir.

## İlgili task'lar

`NEN-129`, `NEN-126`, `NEN-105`, `NEN-118`

## Notlar

2026-09-15 — Kullanıcı kararı uygun buldu; `NEN-129` üretim koduna geçebilir.
