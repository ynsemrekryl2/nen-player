---
adr: 0040
title: OpenSubtitles hash kimlik provider sınırı
status: accepted
milestone: M6
tasks: [NEN-033]
date: 2026-09-06
---

# ADR-0040 — OpenSubtitles hash kimlik provider sınırı

## Durum

`accepted`

## Bağlam

`NEN-033`, `nen-identity` içinde üretilen OpenSubtitles hash'ini resmî
OpenSubtitles API'sine göndererek kesin medya kimliği arar. Mevcut `HttpClient`
uzak medya evidence'ı için tanımlanmış olsa da taşıma katmanı olarak provider
isteklerini de taşıyabilmesi için request header'larına ihtiyaç duyar.

`nen-providers` crate'i `nen-identity`'ye bağlanamaz (ADR-0006). Hash ve
provider kimliği bu nedenle provider'a özel port DTO'ları üzerinden geçer.
API anahtarı kullanıcıya aittir; bu task'ın Keychain veya ayarlar yüzeyi
oluşturması beklenmez. K23, API anahtarı, hash, private file ID, filename ve
ham provider cevabının loglanmasını yasaklar.

## Karar

`MediaIdentityLookup` portu `nen-ports` içinde tanımlanacaktır; request, cevap
ve hata DTO'ları hassas alanları elle redakte edecektir. Gerçek adapter
`nen-providers` içinde bulunacak ve mevcut transport-neutral `HttpClient`'ı
request header desteğiyle kullanacaktır. Adapter, yalnız HTTPS üzerindeki
`api.opensubtitles.com` ve `vip-api.opensubtitles.com` hostlarına, en fazla
beş redirect ve 1 MiB yanıt bütçesiyle istek gönderecek; `moviehash` ile
`moviehash_match=only` parametrelerini sıralı biçimde kullanacak ve `Api-Key`
ile sabit bir `User-Agent` header'ı ekleyecektir. Credential yalnız adapter'a
enjekte edilir, saklanmaz veya provider sonucuna taşınmaz. Exact sonuçlardan
tek bir kimlik tuple'ı çıkarsa `Match`, hiç exact sonuç yoksa `NoMatch`, birden
fazla farklı tuple varsa `Ambiguous` dönecektir. `nen-app`, yalnız `Match`
sonucunu mevcut evidence katmanlarının üstüne ekleyecek; diğer sonuçlar
mevcut yerel çözümlemeye düşecektir.

## Gerekçe

Provider portunu `nen-ports`'ta tutmak, gerçek ve fake adapter'ın aynı
contract'ı uygulamasını ve `nen-providers`'ın crate sınırlarını korur. HTTP
transport'ını yeniden kullanmak, redirect ve approved-host politikasını
provider adapter'ında ayrı tutarken macOS URLSession ve gelecek adapter'ların
aynı taşıma yüzeyini kullanmasını sağlar. Exact hash eşleşmesini yalnız
tekil ve doğrulanabilir bir feature kimliği olduğunda kabul etmek, aynı
medyaya ait birden fazla altyazı kaydını yanlışlıkla belirsiz saymaz; farklı
feature'ları sessizce birleştirmez.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| `nen-providers`'ın `nen-identity`'ye doğrudan bağlanması | ADR-0006'nın crate bağımlılık grafiğini ihlal eder. |
| Her provider için ayrı HTTP portu | Aynı header, boyut ve redirect taşıma davranışını çoğaltır; politika ile transport ayrımını bozar. |
| Hash'i veya credential'ı `Debug` ile taşımak | K23 #6 ve #8'e aykırı olarak sır ve medya parmak izi sızdırır. |
| İlk provider kaydını otomatik olarak `Match` saymak | Aynı hash altında farklı feature kimlikleri bulunabildiği için yanlış sessiz kimlik üretir. |
| API çağrısını playback akışına bağlamak | Provider kesintisi oynatmayı durdurmamalıdır; kimlik yalnız yardımcı evidence'dır. |

## Sonuçlar

**Olumlu:** Gerçek ve deterministic fake provider aynı portu kullanır; provider
yanıtı, credential ve hash tek bir redaction sınırında kalır; mevcut evidence
fallback'i geriye dönük korunur.

**Olumsuz / kabul edilen maliyet:** HTTP request DTO'su ve macOS FFI köprüsü
header taşıyacak şekilde genişler; API sağlayıcısının response şekli için
bounded parser ve fixture bakımı gerekir.

**Geri dönüş maliyeti:** orta — port tüketicileri ve FFI request kaydı birlikte
değişir, ancak provider adapter'ı crate içinde izole kalır.

## İlgili task'lar

`NEN-033` · gelecekteki provider genişletmeleri `NEN-034`, `NEN-035`

## Notlar

OpenSubtitles arama sözleşmesi için resmî [Search for subtitles](https://opensubtitles.stoplight.io/docs/opensubtitles-api/a172317bd5ccc-search-for-subtitles)
belgesi referans alınır. Bu ADR indirme, credential storage veya UI davranışı
kararı vermez.
