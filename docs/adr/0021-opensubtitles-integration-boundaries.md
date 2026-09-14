---
adr: 0021
title: OpenSubtitles entegrasyon sınırları
status: accepted
milestone: M6
tasks: [NEN-119]
date: 2026-09-14
---

# ADR-0021 — OpenSubtitles entegrasyon sınırları

## Durum

`accepted` — 2026-09-14, kullanıcı onayı.

## Bağlam

ADR-0040 yalnız OpenSubtitles hash kimlik sorgusunun port, approved host,
bounded response ve exact-match sınırlarını karara bağladı; aday araması,
katalog kimliği ve seçimde indirme bu kararın dışında kaldı. Şartname §7
resmî API'yi, seçilmeden indirmemeyi, güvenli indirmeyi ve opaque public source
ID'yi zorunlu kılar. Bu sınır yazılmazsa private `file_id`, medya hash'i veya
filename metadata'sı FFI/UI/log yüzeylerine taşınabilir; metadata kataloglama
ile indirme birbirine karışabilir.

## Karar

### 1. Arama ve kataloglama

Yalnız resmî OpenSubtitles `GET /api/v1/subtitles` endpoint'i kullanılacaktır;
scraping, login/oturum ve kullanıcı filename'ını provider araması olarak
kullanma yapılmayacaktır. Arama sırası şöyledir:

1. NEN-018 hash'i varsa `moviehash` + `moviehash_match=only` ile exact arama.
2. Exact sonuç yoksa NEN-120'nin doğrulanmış kimliğinden film için `imdb_id`,
   dizi için `parent_imdb_id` veya `query` + `season` + `episode` ile arama.
3. Hash ve doğrulanmış kimlik yoksa provider'a arama isteği gönderilmeyecek;
   filename tabanlı arama ancak ayrı ve kabul edilmiş bir gizlilik kararıyla
   açılabilecektir.

Tercih edilen birinci/ikinci diller varsa bu diller API sorgusuna geçirilir;
tercih yoksa dil filtresi gönderilmez. Provider'ın döndürdüğü sonuçlar yine
aynı katalogta gruplanır. Sonuç kümesi
deterministik biçimde ilk **16** adaya, yanıt gövdesi ise ADR-0040'taki **1 MiB**
sınırına tabi olacaktır. Her adayın dili normalize `LanguageTag` olur; release
adı yalnız bounded, sanitize edilmiş görüntü metadata'sıdır. `hearing_impaired`
ve `ai_translated` yalnız kapalı rozet alanlarıdır; ADR-0035'teki defect
etiketleri genişletilmez.

Arama yalnız metadata üretir: hiçbir aday dosyası arama sırasında indirilmez,
çıkarılmaz veya kalıcı altyazı kaynağına çevrilmez. Kullanıcı seçmediği sürece
OpenSubtitles kaynağı otomatik seçilemez.

### 2. Kimlik ve private alanların yaşamı

Katalogdaki `SubtitleSourceId::opensubtitles(...)` değeri OpenSubtitles'ın
**public `subtitle id`** alanıdır. Bu değer loglanabilir opaque public ID
olarak kabul edilir. Provider'ın private `file_id` alanı yalnız Rust provider
adapter'ının kısa ömürlü belleğinde, seçilen adayın indirme çağrısını kurmak
için tutulur; `SubtitleSource`, FFI, persistence, log ve `Debug` yüzeylerine
hiç girmez. Public ID ile private file ID eşlemesi kullanıcıya teknik ID
girdisi istemeden aynı provider oturumu içinde çözülür.

### 3. Açık seçimde güvenli indirme

Yalnız kullanıcının bir OpenSubtitles adayını açıkça seçmesi indirmeyi başlatır.
Provider'ın `POST /api/v1/download` çağrısı private `file_id` ile adapter
içinde yapılır. Kota `remaining == 0` veya kota yanıtı verdiğinde çağrı
bekletilmez ve otomatik tekrar edilmez; payload'suz typed quota refusal döner.
401/403 ve parse/validation hataları kalıcıdır; yalnız ADR-0040'ın bounded
transport/5xx retry politikası uygulanır.

Download yanıtındaki geçici link yalnız HTTPS üzerinde `api.opensubtitles.com`,
`vip-api.opensubtitles.com` veya `dl.opensubtitles.com` hostlarından birine
aitse kabul edilir. Redirect otomatik izlenmez; her hop yeniden doğrulanır ve
zincir en fazla **5** hop sürer. İçerik **10 MiB**'yi aşarsa veya
`Content-Type` beklenmeyen/archive ise akış kesilir ve reddedilir; zip, rar,
gzip ve benzeri arşivler magic bytes ile de reddedilir. Kalan baytlar mevcut
encoding sanitization ve strict SRT parser kapılarından geçmeden kataloglanmaz.

Başarılı indirme seçilen adayı kullanıcı kaynağı gibi gösterir; indirme
metadata arama sırasında değil yalnız açık seçimden sonra üretilir. Kalıcı
indirilen-altyazı önbelleği bu ADR'nin kararı değildir.

## Gerekçe

Hash ve doğrulanmış kimlik, kullanıcının izlediği medyayı filename göndermeden
aramaya yeterli kanıtı verir. Public ID ile private file ID'yi ayırmak,
OpenSubtitles'ın indirme gereksinimini karşılayıp K23 #7–#8'i korur. On altı
aday, 10 MiB subtitle ve 1 MiB metadata yanıt bütçeleri mevcut dosya/provider
kapılarıyla ölçülebilir kalır. İndirmeyi açık seçime bağlamak §7'nin lazy
politikasını ve yanlış kimlikten otomatik indirme riskini korur.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Filename veya release adına göre otomatik provider araması | Özel filename metadata'sını dışarı çıkarır; ayrı gizlilik/izin kararı olmadan K23'e aykırıdır |
| Arama sonuçlarını `file_id` ile kataloglamak | Private provider kimliğini FFI, persistence veya Debug'a sızdırabilir; katalog public opaque ID taşımalıdır |
| Arama yanıtındaki ilk adayı otomatik indirmek | Seçilmeden download yasağını ve yanlış kimlikte sessiz indirmeyi ihlal eder |
| Download link'ini URLSession'a sınırsız bırakmak | Approved host, redirect, boyut, archive ve content-type kapılarını ortadan kaldırır |
| Kota bitince reset zamanına kadar beklemek | UI'ı belirsiz biçimde bloke eder; kota tükenmesi typed refusal olmalıdır |

## Sonuçlar

**Olumlu:** Arama lazy metadata ile sınırlı kalır; katalog public opaque ID
taşır; private file ID, hash ve filename güvenlik sınırında kalır; indirme
yalnız açık seçim ve ölçülebilir dosya/redirect kapılarıyla yapılır.

**Olumsuz / kabul edilen maliyet:** Kimliksiz medya OpenSubtitles adaylarını
otomatik bulamaz; private file ID process/adapter yaşamını aşamaz; 16 aday
kapasitesi dışındaki sonuçlar sonraki bir kullanıcı arama sözleşmesini
gerektirir.

**Geri dönüş maliyeti:** orta — provider candidate DTO'su, indirme adapter'ı,
FFI projection ve negatif güvenlik testleri birlikte değişir; public/private
kimlik ayrımı korunarak transport adapter değiştirilebilir.

## İlgili task'lar

`NEN-033` · `NEN-035` · `NEN-036` · `NEN-120` · `NEN-121` · `NEN-122` · `NEN-123`

## Notlar

Resmî referanslar: [Search for subtitles](https://opensubtitles.stoplight.io/docs/opensubtitles-api/a172317bd5ccc-search-for-subtitles)
ve [Download](https://opensubtitles.stoplight.io/docs/opensubtitles-api/6be7f6ae2d918-download).

## Onay kaydı

2026-09-14: Kullanıcı `tamam onaylıyorum devamke` mesajıyla ADR-0021 kararlarını
onayladı.
