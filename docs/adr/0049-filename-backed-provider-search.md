---
adr: 0049
title: Kanıt fan-in'i ve dosya adı destekli provider aday araması
status: accepted
milestone: M6
tasks: [NEN-132]
date: 2026-09-17
---

# ADR-0049 — Kanıt fan-in'i ve dosya adı destekli provider aday araması

## Durum

**Accepted — kullanıcı onayı 2026-09-17.** Dosya adından provider araması,
zayıf aday kanıtı olarak üretim akışına bağlanabilir; exact hash ve kanonik
kimlik her zaman önceliklidir.

## Bağlam

ADR-0009 medya kanıtlarını katmanlı ve güven sıralı tanımlar. ADR-0021 ise
OpenSubtitles sınırını hash exact araması, doğrulanmış kimlik fallback'i ve
seçilmeden indirme yasağıyla belirler. Mevcut macOS akışı bu iki kararın hash
ve doğrulanmış kimlik kısmını çalıştırıyor; belirgin bir yerel/uzak dosya adı
ise aday aramasına ulaşmadan kaybolabiliyor. Bu nedenle kullanıcı dosya adına
bakınca kesin görünen bir medyada altyazı kataloğu boş kalabiliyor.

## Karar önerisi

### 1. Kanıt fan-in'i

Bir medya açılışında şu kanıtlar toplanabilir: provider-compatible hash,
Stremio handoff metadata, `.nfo`, container metadata, declared basename,
parent directories, sibling agreement ve güvenli remote URL path segments.
Toplama bağımsızsa paralel yapılabilir; karar bir oylama değildir. Güven
sırası ADR-0009'daki gibi kalır:

1. exact provider hash match
2. Stremio canonical handoff
3. `.nfo`
4. container metadata
5. declared filename / Content-Disposition basename
6. parent directories
7. sibling agreement
8. remote URL path segments

İlk kullanılabilir ve doğrulanabilir sonuç kimlik sahipliğini alır; daha zayıf
kanıt daha güçlü sonucu ezemez. Çelişki veya eksik sezon/bölüm bilgisi
"ambiguous" olarak korunur.

### 2. OpenSubtitles sorgu sınırı

Üç arama girdisi ayrıdır:

- `ExactHash`: movie hash ile exact arama
- `CanonicalIdentity`: handoff/.nfo/container kanıtından IMDb veya parent IMDb
- `ParsedIdentity`: dosya adı/klasör/izinli AI ipucundan title/year/season/episode

OpenSubtitles resmi `/api/v1/subtitles` akışı önce `ExactHash`, sonra
`CanonicalIdentity`, sonra `ParsedIdentity` ile sorgulanır. Hash sonucu
başarılıysa sonraki kimlik sorgusu gereksizdir. Hash miss sonrası kanonik
IMDb/parent IMDb aranır; bunlar yoksa title/year/season/episode aday araması
çalışır. ParsedIdentity hiçbir zaman exact verified identity olarak
etiketlenmez. Tek sonuç kullanıcıya aday olarak sunulur; birden fazla sonuç
seçim gerektirir.

OpenSubtitles `public_id` bir subtitle adayının kimliğidir; private `file_id`
yalnız indirme girdisidir. Hiçbiri medya IMDb kimliği yerine geçirilemez.

### 3. AI ve gizlilik

AI normalizasyonu deterministik katmanlar sonuçsuz kaldığında medya başına
açık izinle çalışır. İstek yalnız sanitize edilmiş basename stem'i taşır;
tam yol, URL, query/fragment/host, hash, dosya boyutu, container metadata,
private ID ve altyazı metni gönderilmez. AI sonucu yerel biçim/doğrulama
kontrollerinden geçer ve yalnız düşük güvenli `ParsedIdentity` ipucu olur.

### 4. Katalog ve indirme

Provider yanıtı önce güvenli metadata kataloğuna alınır. Kullanıcı açıkça
seçmeden indirme çağrısı yapılmaz. UI, kaynağı `hash`, `canonical` veya
`parsed` olarak gösterebilir; hash, tam yol, URL ve private ID gösterilmez.

## Sonuçlar

Bu karar dosya adı açık olduğunda kullanıcıya beklenen adayları ulaştırır;
aynı zamanda zayıf bir parse'ın yanlış medyayı kesinmiş gibi seçmesini önler.
Arama sayısı artabilir ve belirsiz dosya adları kullanıcı seçimi gerektirebilir.
AI kullanımı daha yüksek gizlilik ve izin maliyeti taşır; deterministik
ayrıştırma varsayılan yol olarak kalır.

## İlişkili kararlar

- [ADR-0009](0009-media-evidence-and-identity.md) — kanıt katmanları ve sıra
- [ADR-0021](0021-opensubtitles-integration-boundaries.md) — provider sınırı
- [ADR-0040](0040-opensubtitles-hash-identity-boundary.md) — hash kimliği
- [ADR-0046](0046-filename-privacy-for-ai-normalization.md) — AI dosya adı gizliliği
