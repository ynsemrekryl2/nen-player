---
adr: 0019
title: Gerçek çeviri sağlayıcı sınırı
status: accepted
milestone: M6
tasks: [NEN-114]
date: 2026-09-11
---

# ADR-0019 — Gerçek çeviri sağlayıcı sınırı

## Durum

`accepted` (2026-09-11, kullanıcı onayı, `NEN-114`)

## Bağlam

M5, senkron ve object-safe `TranslationProvider` portunu, belge bağlamlı blok
isteğini, strict yerel doğrulamayı, bounded repair'i ve provider/model içeren
cache identity'yi yalnız deterministik mock ile kanıtladı. M6'nın gerçek
çeviri kolu için dört sınır hâlâ açık: `HttpClient` yalnız `HEAD`/`GET`
taşıyor; gerçek sağlayıcı payload'ı ve structured-output sözleşmesi yok;
transient retry ile permanent hata ayrımı kesin değil; roadmap'in S3 dilsel
kalite ve S9 çoklu artifact sunumu kararlaşmadı.

Kullanıcının ChatGPT Plus aboneliği OpenAI Platform API anahtarı veya API
kredisi sağlamaz. Bu nedenle M6 canlı kabulü ayrı bir OpenRouter hesabı, API
key'i ve küçük bir bakiye kullanacaktır. Doğrudan OpenAI adapter'ı da
uygulanacaktır; ancak onun canlı çalışması kullanıcının sonradan ayrı OpenAI
Platform API key'i ve kredisi eklemesine bağlıdır ve M6 kabulü için canlı
OpenAI bakiyesi aranmayacaktır.

Şartname ve önceki kararlar şu kısıtları zaten sabitler: altyazı diyaloğu,
raw provider cevabı ve API key loglanmaz (`security-policy.md`); provider
cevabı düşmancadır ve yerel doğrulama authoritative kalır (ADR-0016);
provider/model, prompt ve schema sürümü cache identity'ye girer (ADR-0018);
provider'a `TimeSpan` verilmez, yalnız `CueId + text` taşınır (`NEN-090`).

## Karar

### 1. Varsayılan sağlayıcı ve sabit upstream

Gerçek çevirinin varsayılan sağlayıcısı **OpenRouter**, varsayılan profili
**Ekonomik**, teknik model kimliği **`openai/gpt-5.6-luna`** olacaktır.
OpenRouter routing isteği yalnız OpenAI upstream'ini kabul edecektir:
`provider.order: ["OpenAI"]`, `allow_fallbacks: false` ve
`require_parameters: true`. Böylece OpenRouter aynı model adını başka bir
veri işleyiciye gönderemez ve gerekli structured-output parametrelerini
desteklemeyen bir route'a sessizce düşemez.

Doğrudan OpenAI ikinci, desteklenen bir sağlayıcı olacaktır; M6 kabulünün
zorunlu canlı yolu değildir. Hosted Nen Player backend'i olmayacak, iki yol
da kullanıcının kendi credential ve sağlayıcı bakiyesini kullanacaktır.

### 2. HTTP portunun gelecek sözleşmesi

`NEN-115`, mevcut çağrıları bozmadan `nen-ports::http` sözleşmesini şu şekilde
genişletecektir:

- `HttpMethod::Post` eklenecek; mevcut `Head` ve `Get` korunacaktır.
- `HttpRequest`, redakte edilen `body: Option<Vec<u8>>` ve
  `timeout_ms: Option<u64>` taşıyacaktır. `Debug`/`Display` body içeriğini
  göstermeyecektir.
- JSON POST kurulumunu tekleştiren `post_json(...)` yardımcı kurucusu
  eklenecektir; serialization provider adapter'ının sorumluluğunda kalacaktır.
- Provider request ve response body'lerinin her biri en fazla **1 MiB**
  olacaktır; aşım typed, payload'sız permanent transport hatasıdır.
- Her denemenin timeout'u **60.000 ms** olacaktır. Redirect bütçesi sıfırdır;
  adapter otomatik redirect izlemeyecektir.

Provider isteği yalnız HTTPS ve tam host eşleşmesiyle
**`api.openai.com`** veya **`openrouter.ai`** hedefine gidebilir. Medya HTTP
kuralları ADR-0039'da ayrı kalır; kullanıcı medya host'u bu allowlist'e
alınmaz. Header, body, URL path/query, subtitle metni ve raw cevap loglanmaz;
güvenli telemetri yalnız approved sabit host, sağlayıcı/model profili, byte
boyut sınıfı, süre ve payload'sız hata varyantıdır.

### 3. Ortak provider payload'ı ve çıktı şeması

Her iki adapter aynı semantik isteği kuracaktır:

- `source_language` ve `target_language`
- `context_terms`
- tüm blok bağlamını taşıyan `context_cues: [{ cue_id, text }]`
- yalnız çevrilip döndürülmesi gereken `output_cue_ids`

Sistem talimatı modeli yalnız `output_cue_ids` içindeki cue'ları çevirmeye;
bağlam cue'larını ve terimleri tutarlılık için kullanıp çıktılamamaya;
anlamı, register'ı, özel isim/terim tutarlılığını ve anlamlı satır sonlarını
korumaya; açıklama veya ek alan üretmemeye zorlayacaktır. Provider çıktısının
tek şekli aşağıdaki semantik JSON'dur:

```json
{
  "cues": [
    { "cue_id": "cue-id", "text": "translated text" }
  ]
}
```

JSON Schema kökünde ve cue nesnesinde `additionalProperties: false` olacak;
`cues`, `cue_id` ve `text` zorunlu olacaktır. Şema provider'a yapı garantisi
verdirir ama güven sınırı değildir: izinli/tam/tekil cue ID kümesi, sıra,
boş metin, uzunluk ve diğer bütün kurallar ADR-0016'nın yerel doğrulamasından
geçmeden cevap yayınlanmayacak veya checkpoint edilmeyecektir. Provider'a
zaman damgası ya da `TimeSpan` gönderilmeyecektir.

İlk gerçek sözleşmede `PROMPT_VERSION = 1` ve `SCHEMA_VERSION = 1` kalacaktır.
Prompt'un semantik talimatı değişirse `PROMPT_VERSION`, request/response veri
şekli veya JSON Schema'nın kabul kümesi değişirse `SCHEMA_VERSION` elle
artırılacaktır. Yazım düzeltmesi semantiği değiştirmiyorsa bump gerekmez;
kararsızlıkta sürüm artırılır. ADR-0018'in diğer sürüm kuralları değişmez.

### 4. OpenRouter adapter'ı ve capability preflight

OpenRouter çeviri çağrısı `POST /api/v1/chat/completions` üzerinden yapılacak;
ortak çıktı şeması `response_format.type = "json_schema"`, adlandırılmış
`json_schema`, `strict: true` biçiminde gönderilecektir. İstek madde 1'deki
provider routing alanlarını da taşıyacaktır.

Bir modelle ilk iş başlamadan önce adapter
`GET /api/v1/model/{author}/{slug}` çağrısı yapacak ve yanıttaki
`supported_parameters` içinde **`structured_outputs`** arayacaktır. Olumlu
ve olumsuz capability sonucu tam model kimliği başına **15 dakika**
cache'lenecektir. Transport/408/429/5xx, parse hatası veya diğer preflight
hataları cache'lenmeyecektir. Capability eksikse hiçbir çeviri POST'u
atılmadan yeni, payload'sız
`StartRefusal::ProviderCapabilityMissing` dönecektir.

Varsayılan `openai/gpt-5.6-luna` kimliği ve structured-output desteği,
OpenRouter'ın güncel model kaydında doğrulanmıştır. Runtime preflight yine de
zorunludur; doküman doğrulaması çalışan servis capability'sinin yerine
geçmez.

### 5. Doğrudan OpenAI adapter'ı

Doğrudan adapter `POST https://api.openai.com/v1/responses` kullanacaktır.
İstek `store: false` taşıyacak; ortak çıktı şeması Responses API biçiminde
`text.format.type = "json_schema"`, adlandırılmış schema ve `strict: true`
olarak gönderilecektir. Çıktı ortak provider cevabına indirgenecek ve aynı
yerel doğrulamadan geçecektir.

Bu adapter fixture ve contract testleriyle kanıtlanacaktır. Canlı kullanım,
ChatGPT aboneliğinden ayrı bir OpenAI Platform API key'i ve faturalama kredisi
gerektirir; M6 kabul checklist'i bunları zorunlu kılmayacaktır.

### 6. Retry ve hata sınıfları

Bir çeviri veya preflight denemesi transport hatası, HTTP 408, 429 veya 5xx
ile biterse ilk çağrıdan sonra en fazla iki retry yapılacaktır. Sunucu geçerli
`Retry-After` verirse bekleme **10 saniyeye clamp** edilir; yoksa veya
geçersizse sırasıyla **500 ms** ve **2 saniye** beklenir. Toplam en fazla üç
deneme vardır. Her beklemeden ve yeni denemeden önce cancellation kontrolü
yapılır; iptal edilmiş çağrı beklemez ve yeniden denenmez.

HTTP 401/403, diğer 4xx yanıtları, model refusal'ı, eksik/bozuk JSON ve schema
uyuşmazlığı permanent'tır; transport retry'ı yapılmaz. Capability eksikliği
iş başlamadan typed refusal'dır. Provider'dan parse edilmiş fakat yerel
doğrulamadan geçemeyen bir cevap transport retry'ına değil, yalnız
`NEN-092`/ADR-0016'nın mevcut bounded repair ve tam-blok bütçesine gider. Bir
başarısızlık iki bütçeyi birden tüketmez.

### 7. Credential yaşam döngüsü

Seçili adapter anahtarı `SecureCredentialStore`dan iş kurulurken alacaktır;
adapter anahtarı kalıcı durumda saklamayacak ve iş bitince düşürecektir
(ADR-0020). OpenRouter seçiliyken yalnız OpenRouter key'i, doğrudan OpenAI
seçiliyken yalnız OpenAI key'i gerekecektir. Eksik credential payload'sız
typed start refusal üretecek, ağ isteği yapılmayacaktır.

### 8. Kullanıcıya açık model profilleri

Kullanıcı serbest teknik model ID'si girmeyecektir. Seçim yüzeyi üç kararlı
profil gösterecektir:

| Profil | OpenRouter model kimliği |
|---|---|
| **Ekonomik** (varsayılan) | `openai/gpt-5.6-luna` |
| **Dengeli** | `openai/gpt-5.6-terra` |
| **Yüksek kalite** | `openai/gpt-5.6-sol` |

Gerçek teknik kimlik yine `TranslationProviderIdentity` ve cache identity'ye
girecektir; insan etiketi kimliğin yerine geçmez. Doğrudan OpenAI adapter'ı
aynı üç profil anlamını sağlayıcının karşılık gelen model kimliklerine
eşleyecektir. Profil eşlemesini değiştirmek cache kimliğini sessizce yeniden
kullanamaz; teknik model kimliği değiştiği için yeni artifact üretir.

### 9. S9 — aynı dilde çoklu artifact sunumu

Aynı hedef dildeki bütün doğrulanmış AI artifact'leri ayrı seçenek olarak
gösterilecektir. İnsan etiketi **`AI · sağlayıcı · model profili`** biçiminde
olacaktır (ör. `AI · OpenRouter · Ekonomik`). Grup açıldığında en yeni artifact
başlangıçta seçili olacaktır; kullanıcı eski veya başka sağlayıcı/profil
artifact'ine dönebilecektir. Diskte yan yana saklama ve tam cache ayrımı
ADR-0018'deki provider/model kimliğiyle değişmeden korunur. `NEN-118` bu
projeksiyonu uygulayacaktır.

### 10. S3 — EN→TR yayın kalitesi kabulü

“Yayın kalitesi” iddiası yalnız **İngilizce → Türkçe** dil çifti için ve
`NEN-126`'daki ölçüm geçerse kullanılacaktır. Kabul korpusu telif-temiz, tek
bir **24 cue** belgesi olacak; günlük diyalog, deyim, register ve tutarlı özel
isim/terminoloji vakaları içerecektir. Belge varsayılan
OpenRouter/`openai/gpt-5.6-luna` ile gerçek ağ koşusunda çevrilecektir.

Geçiş şartı birlikte şudur: **24/24 cue** zorunlu düzeltme gerektirmeyecek;
anlam tersine dönmesi, atlama/ekleme ve isim/terim tutarsızlığı sayılarının
her biri **0** olacaktır. Yapısal yerel doğrulamanın geçmesi tek başına
dilsel geçiş sayılmaz. Diğer dil çiftleri için “yayın kalitesi” iddiası
yapılmayacaktır.

Luna bu eşiği geçemezse `NEN-126` ve M6 kırmızı kalacak; ayrı bir düzeltme
task'ı varsayılan profili `openai/gpt-5.6-terra`ya yükseltecek ve aynı
OpenRouter hesabı, yalnız OpenAI upstream'i, aynı korpus ve aynı eşikle kabul
yeniden çalıştırılacaktır. Bu, tek task içinde model karşılaştırması değildir.
Terra da geçemezse M6 kapanmayacaktır.

`NEN-126` credential önkoşulu **OpenSubtitles + seçili gerçek çeviri
sağlayıcısının anahtarı** olacaktır. Varsayılan canlı yol için OpenRouter
key'i ve küçük bir OpenRouter bakiyesi yeterlidir; canlı OpenAI key'i/bakiyesi
aranmaz. Doğrudan OpenAI adapter'ının kanıtı fixture/contract testleridir.

## Gerekçe

OpenRouter'ı varsayılan yapmak, kullanıcıyı ChatGPT Plus ile ayrı OpenAI API
faturalamasını karıştıran bir kabul kapısına bağlamadan Luna/Terra/Sol
profillerini tek hesapla sunar. Upstream'i yalnız OpenAI'ye sabitlemek model
adının arkasındaki veri işleyiciyi deterministik kılar; fallback'i kapatmak
capability veya sağlayıcı değişimini sessiz davranış olmaktan çıkarır.

Strict structured output, parse yüzeyini küçültür; fakat provider çıktısını
güvenilir saymadığı için ADR-0016'nın authoritative yerel doğrulaması korunur.
60 saniyelik deneme, 1 MiB sınır, sıfır redirect ve toplam üç deneme, gerçek
ağ gecikmesini desteklerken işi sınırsız bekleme/bellek/kota tüketiminden
korur. 15 dakikalık iki yönlü capability cache'i her blok öncesi metadata
çağrısını önler; transient hatayı cache'lememek kısa servis kesintisini kalıcı
capability reddine dönüştürmez.

24 cue'luk tek belge, bağlam tutarlılığını cue başına bağımsız örneklerden
daha iyi ölçerken insan incelemesini küçük ve tekrarlanabilir tutar. Sıfır
kritik hata ve 24/24 düzeltmesiz koşulu “yayın kalitesi” sözünü ölçülebilir
hale getirir. Luna başarısızlığında Terra'ya otomatik/sessiz fallback yerine
ayrı task açmak maliyet ve varsayılan davranış değişimini görünür tutar.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| ChatGPT Plus'ı API credential/kredisi saymak | ChatGPT aboneliği ile OpenAI Platform API key ve billing ayrı ürünlerdir; canlı API çağrısını finanse etmez |
| Doğrudan OpenAI'yi M6 canlı kabulinde zorunlu kılmak | İkinci bir ücretli hesap/bakiye kapısı açar; adapter fixture/contract ile kanıtlanabilir |
| OpenRouter'ın herhangi bir upstream'e fallback yapmasına izin vermek | Veri işleyiciyi ve gerçek model davranışını sessizce değiştirir |
| OpenRouter capability preflight yapmamak | Seçili modelin strict structured output taşımadığı ancak ücretli POST'tan sonra anlaşılır |
| Teknik model ID'sini serbest metinle kullanıcıya açmak | Desteklenmeyen capability, typo ve cache kimliği karmaşası üretir; üç profil ürün ihtiyacını karşılar |
| Provider JSON'unu yerel doğrulama olmadan kabul etmek | Structured output yalnız şekli sınırlar; eksik/yanlış cue veya anlamsal bozulmayı güvenilir yapmaz |
| Her hata için retry | 401/403, diğer 4xx, refusal ve bozuk çıktıda boşuna kota/zaman tüketir; cancellation sonrasında late work riski yaratır |
| S9'da yalnız en yeni artifact'i göstermek | Diskte var olan provider/model alternatiflerini kullanıcıdan saklar ve kalite/maliyet seçimini geri döndürülemez kılar |
| Bütün dil çiftleri için yayın kalitesi iddiası | Yalnız EN→TR korpusu ölçülür; ölçülmeyen dile genelleme yapılamaz |
| Luna başarısızken aynı task'ta sessiz Terra fallback'i | Varsayılan maliyet/kalite kararını ve kabul ölçümünü görünmez biçimde değiştirir |

## Sonuçlar

**Olumlu:** Gerçek provider sınırı iki adapter için ortak ve test edilebilir;
varsayılan canlı kabul tek OpenRouter hesabıyla yapılabilir; upstream,
structured-output capability, timeout/boyut/retry ve cache davranışı
deterministiktir; S3 ve S9 ölçülebilir ürün sözleşmelerine dönüşür.

**Olumsuz / kabul edilen maliyet:** OpenRouter aracı bir faturalama ve
erişilebilirlik bağımlılığı ekler; yalnız OpenAI upstream şartı bazı bölgelerde
route bulunamamasına yol açabilir. Capability preflight bir metadata çağrısı
ekler. Strict schema model/prompt değişikliklerinde elle sürüm disiplini ister.
EN→TR dışındaki diller yayın kalitesi etiketi alamaz.

**Geri dönüş maliyeti:** orta — `TranslationProvider` ve ortak payload sabit
kalır; varsayılan sağlayıcı/profil veya adapter değişebilir. Fakat provider
kimliği, prompt/schema sürümü ve kabul iddiası değişeceğinden yeni artifact
kimliği, yeni ADR/task ve kalite kabulünün tekrarı gerekir.

## İlgili task'lar

`NEN-114` · `NEN-115` · `NEN-116` · `NEN-117` · `NEN-118` · `NEN-126`

## Notlar

Karar hazırlanırken doğrulanan güncel kayıtlar:

- OpenRouter Luna kaydı:
  <https://openrouter.ai/openai/gpt-5.6-luna-20260709>
- OpenRouter structured outputs:
  <https://openrouter.ai/docs/guides/features/structured-outputs>
- OpenRouter provider routing:
  <https://openrouter.ai/docs/guides/routing/provider-selection>
- OpenAI API quickstart (ayrı API key ve billing kredisi):
  <https://developers.openai.com/api/docs/quickstart>
- OpenAI structured outputs:
  <https://developers.openai.com/api/docs/guides/structured-outputs>
- OpenAI GPT-5.6 Luna model kaydı:
  <https://developers.openai.com/api/docs/models/gpt-5.6-luna>

User glossary bu ADR'nin kapsamı dışındadır; `GlossaryIdentity::none()` M6'da
kalır. Streaming/SSE, otomatik provider fallback'i, serbest model kataloğu ve
hosted backend de kapsam dışıdır.
