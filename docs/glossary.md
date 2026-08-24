# Sözlük

Terimler bu anlamlarıyla kullanılır. Kodda İngilizce tip adı verilir; burada
Türkçe açıklaması ve yaşadığı crate belirtilir.

## Altyazı

| Terim | Tip | Crate | Anlam |
|---|---|---|---|
| **Cue** | `Cue` | `nen-domain` | Tek bir altyazı repliği: ID, başlangıç, bitiş, metin. ID ve zamanlar çeviri boyunca **aynen korunur** |
| **SubtitleDocument** | `SubtitleDocument` | `nen-domain` | Sıralı cue listesi + dil + kaynak bilgisi. Parse edilmiş, doğrulanmış hali |
| **Cue index** | `CueIndex` | `nen-subtitle` | Zamandan cue'ya logaritmik erişim yapısı. **Lineer tarama yasak** (şartname §14) |
| **Timeline fingerprint** | `TimelineFingerprint` | `nen-domain` | Yalnız cue **zamanlarından** türetilen özet. Metin değişince değişmez, tek cue 1 ms kayınca değişir. SyncProfile ve AI artifact eşleşmesinin anahtarı |
| **Source fingerprint** | `SourceFingerprint` | `nen-domain` | Kaynak altyazının **içeriğinden** (metin dahil) türetilen özet. Cache identity'nin parçası |

> Bu ikisi karıştırılmamalıdır: *timeline* fingerprint zamanlamayı, *source*
> fingerprint içeriği kimliklendirir. AI çeviri source fingerprint'i değiştirir,
> timeline fingerprint'i **değiştirmez** — SyncProfile bu yüzden paylaşılabilir.

## Katalog ve kaynak

| Terim | Tip | Crate | Anlam |
|---|---|---|---|
| **SubtitleSource** | `SubtitleSource` | `nen-domain` | Katalogdaki tek giriş. Türü: `embedded` · `user` · `opensubtitles` · `ai` |
| **SubtitleSourceCatalog** | `SubtitleSourceCatalog` | `nen-catalog` | **Tek** katalog. Tüm kaynak türleri burada; menü bunun projeksiyonudur |
| **Origin badge** | — | UI | Kaynak türünün küçük rozeti (Gömülü / OpenSubtitles / AI) |
| **Dil Belirsiz** | — | `nen-catalog` | Dil tespiti güven eşiğini geçemeyen kaynakların grubu |
| **Lazy** | — | `nen-catalog` | Kataloglama ≠ indirme/extract. Pahalı iş yalnız seçim veya AI talebinde yapılır |

## Medya kimliği

| Terim | Tip | Crate | Anlam |
|---|---|---|---|
| **MediaEvidence** | `MediaEvidence` | `nen-identity` | Kimlik için toplanan ham kanıtlar: filename, boyut, OS-uyumlu hash, container metadata, release name, handoff metadata |
| **MediaIdentity** | `MediaIdentity` | `nen-identity` | Kanıtlardan çözümlenmiş kimlik: başlık, yıl, sezon, bölüm. Çözümlenemezse medya **yine oynar** |
| **Release name** | — | `nen-identity` | Dosya adındaki sürüm etiketi (WEB-DL, BluRay, grup adı). Aday eşleştirmede kullanılır |
| **Opaque public source ID** | — | `nen-catalog` | OpenSubtitles kaynağının dışarı gösterilen kimliği. Private file ID / hash / filename **sızdırılmaz** |

## Çeviri

| Terim | Tip | Crate | Anlam |
|---|---|---|---|
| **Block** | `TranslationBlock` | `nen-translate` | Birlikte çevrilen ardışık cue grubu. Varsayılan 40, izin 30–60 |
| **Overlap** | — | `nen-translate` | Ardışık blokların paylaştığı cue sayısı. Varsayılan 6. Bağlam sürekliliğini sağlar |
| **Checkpoint** | — | `nen-translate` | **Yalnız tamamen doğrulanmış** bir bloğun kalıcılaştırılması. Yarım blok checkpoint'lenmez |
| **Strict validation** | — | `nen-translate` | Provider cevabının yerel doğrulaması: exact cue count · yalnız izin verilen ID · unique ID · non-empty text · beklenen sıra. **Authoritative** — structured output olsa bile |
| **Targeted repair** | — | `nen-translate` | Doğrulamayı geçemeyen belirli cue'lar için hedefli yeniden istek. En fazla **2** |
| **Full-block retry** | — | `nen-translate` | Tüm bloğun yeniden istenmesi. En fazla **1** |
| **Glossary** | `Glossary` | `nen-domain` | Dil çiftine bağlı terim sözlüğü. Kullanıcı glossary'si otomatik olana önceliklidir. Cache identity'ye dahildir |

## Artifact ve cache

| Terim | Tip | Crate | Anlam |
|---|---|---|---|
| **ValidatedSubtitleArtifact** | `ValidatedSubtitleArtifact` | `nen-domain` | Kalıcı çeviri çıktısı. Yalnız **tüm belge** doğrulandıktan sonra, **atomik** commit ile oluşur |
| **Cache identity** | `CacheKey` | `nen-domain` | Artifact'ın yeniden kullanılabilirlik anahtarı. Prompt/schema/pipeline/block-layout/session versiyonlarını içerir |
| **Ephemeral session** | `TranslationSession` | `nen-translate` | Çalışan çeviri işinin geçici durumu. Artifact **değildir**; iptal edilirse iz bırakmaz |
| **Atomik commit** | — | `nen-persist` | Artifact ya tamamen görünür ya hiç yoktur. İptal sonrası **late commit yasak** |

## Senkronizasyon

| Terim | Tip | Crate | Anlam |
|---|---|---|---|
| **SyncProfile** | `SyncProfile` | `nen-sync` | Zamanlama düzeltmesi. Anahtarı: media fingerprint + audio track identity + timeline fingerprint. **Orijinal cue zamanları değişmez** |
| **Anchor** | `SyncAnchor` | `nen-sync` | "Bu replik şimdi başlamalı" komutuyla oluşan hizalama noktası |
| **Constant offset** | — | `nen-sync` | `renderTime = cueTime + offset` |
| **Linear drift** | — | `nen-sync` | `renderTime = cueTime × rate + offset` (iki anchor ile) |
| **Piecewise mapping** | — | `nen-sync` | Bölüm bölüm farklı düzeltme. Farklı intro/recap/reklam/eksik sahne durumları için |
| **Confidence** | `Confidence` | `nen-sync` | `high` · `medium` · `low` · `rejected`. **High bile onaysız uygulanmaz** |
| **localOnly** | — | `nen-sync` | Audio analizinin varsayılan gizlilik modu: ses cihazdan çıkmaz |

## Mimari

| Terim | Anlam |
|---|---|
| **Port** | Core'un ihtiyacını tanımlayan trait. Cihaz/OS API'sine dokunan her şey porttur |
| **Adapter** | Bir portun somut implementasyonu (libmpv, Keychain, Media3…) |
| **Capability** | Bir adapter'ın desteklediği yetenek. Application katmanı motor **adına** değil capability'ye bakar |
| **Contract test** | Bir portun tüm adapter'larının geçmek zorunda olduğu ortak test kiti |
| **Vertical slice** | Uçtan uca çalışan, kullanıcı-görünür en küçük dilim |
| **Spike** | Karar vermek için yazılan, ürüne terfi etmeyen ölçüm kodu (`core/spikes/`) |
