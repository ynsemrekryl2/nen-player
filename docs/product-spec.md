# Nen Player — Ürün Şartnamesi (Kanonik)

> Bu dosya ürünün kanonik tanımıdır. Buradaki bir madde yalnız kullanıcı onayıyla
> değişir. Kod, task ve ADR'ler bu dosyaya uymak zorundadır; çelişki varsa bu
> dosya kazanır. Değişiklik geçmişi git'tedir.

## 1. Ürün vizyonu

Nen Player; yerel ve uzak medyaları oynatan, erişilebilir bütün altyazı
kaynaklarını kataloglayan, kullanıcının seçtiği altyazıyı isteğe bağlı olarak AI
ile hedef dile çeviren ve altyazıyı video ile senkron gösteren multiplatform bir
medya player olacaktır.

Temel akış:

media handoff veya dosya seçimi → medya mümkün olan en kısa sürede oynatılır →
medya kanıtları toplanır → altyazı kaynakları arka planda taranır → tek altyazı
kataloğu oluşturulur → kullanıcı bir altyazı seçer → seçili kaynak gösterilir →
kullanıcı isterse AI çeviri komutu verir → çeviri sıkı biçimde doğrulanır →
kalıcı subtitle artifact oluşturulur → hedef dil grubuna eklenir → video
üzerinde senkron gösterilir.

## 2. Hedef platformlar

macOS · Android · Android TV · iOS · tvOS/Apple TV · Windows · Linux

Geliştirme sırası: `docs/roadmap.md` (şartnamedeki sıradan tek gerekçeli sapma
orada belgelidir).

## 3. Mimari kararlar

Klasik web frontend + ayrı localhost backend **istenmiyor**.

Doğru model: platform-native UI → application API → cihaz içinde çalışan shared
portable core → platform portları ve adapter'ları.

Final uygulamada **bulunmayacaklar:** Node.js companion, Express server,
localhost HTTP orchestration, browser player, web dashboard.

Shared core için varsayılan aday **Rust**. Önce Swift ve Kotlin binding, async
progress, cancellation, typed error ve büyük cue listeleri için teknik spike
yapılmalıdır.

**Platforma özgü kalacaklar:** UI · lifecycle · playback motoru · embedded track
extraction · dosya seçici ve sandbox izinleri · secure credential storage ·
Stremio handoff · platform subtitle renderer.

**Shared core içinde bulunacaklar:** media evidence ve identity modeli ·
subtitle source catalog · SRT/WebVTT · translation orchestration · strict
validation · checkpoint/cancellation · cache/artifact identity · persistence
kontratı · manual sync · audio alignment · typed safe error modeli.

## 4. Playback

`PlaybackEngine` **capability tabanlı bir port** olmalıdır. En az şu
operasyonları kapsamalıdır:

load · play · pause · stop · absolute seek · relative seek · current position ·
duration · buffering/ready/ended/failure state · playback rate · volume ·
embedded audio track enumeration · embedded subtitle track enumeration · audio
track selection · subtitle track selection · embedded text extraction
capability · external subtitle injection capability · event stream ·
shutdown/lifecycle.

İlk motor politikası:

| Platform | Motor |
|---|---|
| macOS | libmpv |
| Windows / Linux | libmpv |
| iOS / tvOS | AVPlayer / AVKit |
| Android / Android TV | Media3 veya spike sonucuna göre uygun native motor |

Kullanıcı playback motorunun adını **görmemeli** ve motor **seçmemelidir**.
Application katmanı engine adına değil **capability**'ye göre davranmalıdır.

Medya; subtitle discovery veya AI translation beklenmeden oynatılmalıdır.
Subtitle hatası playback'i durdurmamalıdır.

## 5. Stremio entegrasyonu

Nen Player bir Stremio subtitle add-on **değildir**.

Stremio'dan external player olarak otomatik medya açmak **ana özelliktir**.
Özellikle Android TV'de temel kullanım senaryosudur.

**Android / Android TV:** standart `ACTION_VIEW` external-player Intent · video
MIME türleri · http/https/content URI · başlangıç pozisyonu · geçici URI
izinleri · playback sonucu ve son pozisyonu Stremio'ya döndürme · gerçek cihaz
testi · media URI veya token içeren extras'ı **loglamama**.

**macOS:** external-player launcher veya open-with handoff · positional
file/http/https · optional start position · argüman ve medya URL'sini
**loglamama**.

Stremio canonical media identity gönderirse optional güçlü kanıt olarak
kullanılabilir. Her zaman geleceği varsayılmamalıdır.

## 6. Medya kimliği

Kullanıcıdan IMDb, Stremio veya başka teknik ID **istenmeyecektir**.

Kanıtlar: optional handoff metadata · optional canonical ID · local filename ·
file size · OpenSubtitles-compatible hash · container metadata · title/year ·
season/episode · release name · remote URL path basename.

> Bu kanıt listesi [ADR-0009](adr/0009-media-evidence-and-identity.md) ile
> **genişletildi**: uzak medyada URL'in yalnız basename'i değil **tüm path
> segmentleri** ipucu sayılır, ve sunucunun beyan ettiği ad
> (`Content-Disposition`, yönlendirme zincirinin sonu) ayrı bir kanıt katmanıdır.
> Query, fragment ve host **hiçbir koşulda** kimliğe girmez — aşağıdaki yasak
> aynen geçerlidir. Katmanların değerlendirilme sırası da o ADR'dedir.

**Film fallback:** title + year → title → insan tarafından anlaşılır aday seçimi.

**Dizi fallback:** series title + season + episode → insan tarafından anlaşılır
aday seçimi.

Kimlik çıkarılamazsa medya yine oynar. Kullanıcıya teknik ID değil; başlık, yıl,
sezon, bölüm düzeltme alanları gösterilebilir.

Media URL query'si identity veya log kaynağı olarak kullanılmamalıdır.

## 7. Altyazı kaynakları

Bütün kaynaklar tek `SubtitleSourceCatalog` içinde bulunmalıdır.

Kaynak türleri: `embedded` · `user` · `opensubtitles` · `ai`.

Player bütün erişilebilir altyazıları tarar. AI çeviri için otomatik kaynak
önceliği uygulanmaz.

Tüm kaynakları kataloglamak, bütün dosyaları hemen indirmek veya extract etmek
anlamına **gelmez**. Pahalı işler lazy yapılmalıdır.

**Embedded:** track metadata hemen listelenir · text extraction yalnız
seçildiğinde veya AI talebinde yapılabilir · bitmap track gösterilebilir fakat
çevrilemez olarak işaretlenebilir.

**Kullanıcı kaynakları:** "Altyazı Dosyası Yükle" · yerel medya yanında aynı
basename'e sahip SRT · regular-file kontrolü · symlink/path traversal reddi ·
boyut ve encoding kontrolü · strict SRT parse · dil tanıma · kullanıcı grubunda
gösterme.

**OpenSubtitles:** yalnız resmi API · scraping yok · metadata adaylarını
kataloglama · seçilmeden download yapmama · HTTPS/approved-host kontrolü ·
bounded redirect · archive reddi · maksimum boyut · encoding ve strict SRT
doğrulaması · belirsiz adaylarda güvenli kullanıcı seçimi · opaque public source
ID · private file ID/hash/filename sızıntısı yok.

## 8. Altyazı menüsü

**Tek bir altyazı düğmesi** olacaktır.

```
Altyazılar

Kapalı

Kullanıcı Altyazıları
  Movie.en.srt                  İngilizce
  subtitle.srt                  Fransızca

İngilizce
  English                       Gömülü
  English — WEB-DL              OpenSubtitles

Fransızca
  Français                      Gömülü

Türkçe
  Türkçe                        Gömülü
  AI Türkçe                     AI
```

Kurallar: kullanıcı kaynakları yalnız "Kullanıcı Altyazıları" grubunda · diğer
kaynaklar dillere göre · aynı kaynak tekrar gösterilmez · origin küçük bir rozet
olabilir · dili bilinmeyen kaynak "Dil Belirsiz" grubunda · AI sonucu hedef dil
grubunda AI rozetiyle · ayrı özel "AI subtitle mode" **yok** · "Kapalı" her
zaman bulunur.

> Bu bölüm [ADR-0010](adr/0010-subtitle-source-catalog.md) ile **genişletildi**.
> Yukarıdaki sekiz kural aynen geçerlidir; eklenenler: (1) kullanıcının
> **birinci ve ikinci tercih edilen altyazı dili** vardır ve o diller diğer dil
> gruplarının üstünde listelenir, geri kalanlar `LanguageTag` sırasında; (2) bir
> dilin görünen adı UI dili ne olursa olsun **o dilin kendi adıdır** ("English",
> "Français", "Türkçe") — yukarıdaki örnekteki grup başlıkları bu nedenle
> endonime döner; (3) tercih edilen dilde **hazır** bir kaynak varsa (yalnız
> `embedded` veya `user`) oynatma başlarken otomatik seçilir — otomatik seçim
> hiçbir indirme veya çeviri tetiklemez. Kanonik menü örnekleri ve otomatik
> seçim tür önceliği o ADR'dedir.

## 9. Kaynak seçimi ve AI çeviri

**Kaynak seçmek AI çeviri başlatmaz.**

Kullanıcı bir subtitle source seçer. Bu kaynak: o anda gösterilen subtitle
source olur **ve** AI çeviri komutunun kaynak subtitle'ı olur.

Kullanıcı ayrıca açıkça "AI ile <hedef dile> çevir" komutunu verir.

Hedef dil kullanıcı ayarıdır; varsayılan Türkçe olabilir. Kaynak zaten hedef
dildeyse translation başlatılmaz.

Çeviri sırasında kullanıcı başka source seçerse: mevcut iş başlangıç source
fingerprint'ine bağlı kalır · yeni source'a retarget edilmez · kullanıcı iptal
edebilir · sonuç hedef dil grubuna eklenir · kullanıcı başka source izliyorsa
zorla AI çıktısına geçilmez.

## 10. Translation pipeline

Korunması gereken temel davranışlar:

- source ve target language dinamik
- bütün subtitle dokümanı için bağlam analizi
- overlapping block translation
- varsayılan block size **40**, izin verilen aralık **30–60**
- varsayılan overlap **6**
- cue ID / sıra / zamanların **aynen** korunması
- provider cevabının **untrusted** kabul edilmesi
- structured output olsa bile **local validation'ın authoritative** olması
- exact cue count · yalnız izin verilen cue ID · unique cue ID · non-empty text
- beklenen cue sırasına normalization
- en fazla **iki targeted repair**, en fazla **bir full-block retry**
- yalnız tamamen doğrulanmış block checkpoint
- cancellation kontrolleri
- final subtitle yalnız bütün belge doğrulanınca
- **progressive veya yarım subtitle publication yok**
- UTF-8 WebVTT

**Provider portları:** deterministic mock · OpenAI Responses API · OpenRouter ·
gelecekte başka provider'lar.

**OpenRouter:** structured-output capability preflight · yalnız bounded
transient network/5xx retry · raw response loglamama.

**User glossary:** optional · source/target dil çiftine bağlı · kullanıcı
glossary'si otomatik glossary'ye göre öncelikli · cache identity'ye dahil.

## 11. Artifact ve cache

Final çeviri yalnız bellekte tutulmamalıdır.

`ValidatedSubtitleArtifact` en az şunları içermelidir: artifact ID · source
fingerprint · subtitle timeline fingerprint · source language · target language ·
normalized cues · WebVTT · provider/model · pipeline versions · glossary
identity · media identity fingerprint · createdAt.

**Cache identity:** source fingerprint · source language · target language ·
provider/model · media context · glossary · block size/overlap · pipeline
version · prompt version · schema version · block-layout version ·
translation-session version.

Prompt/schema/pipeline semantiği değişince **uyumsuz cache kullanılmamalıdır**.

Final artifact: yalnız validation sonrası · atomik commit · restart sonrası
reuse · kullanıcı temizleyene kadar saklanabilir.

Ephemeral session ile persistent artifact ayrılmalıdır.

Başlangıç için SQLite metadata index + content-addressed artifact files
değerlendirilebilir. Seçim ADR ile gerekçelendirilmelidir.

## 12. Manuel senkronizasyon

Üç seviye: (1) basit offset, (2) replik temelli manuel senkronizasyon,
(3) iki replikle drift düzeltmesi.

Kullanıcı videonun mevcut anında istediği subtitle cue'yu seçip "Bu replik şimdi
başlamalı" komutunu verebilmelidir.

```
offset = currentPlaybackTime - selectedCue.startTime
```

Bu offset seçili subtitle timeline'ının tamamına uygulanır. İki anchor ile:

```
renderTime = cueTime × rate + offset
```

doğrusal drift düzeltmesi üretilebilir.

**Özgün cue zamanları değiştirilmemelidir.** Düzeltme ayrı `SyncProfile` olarak
saklanır. SyncProfile anahtarı: media fingerprint · audio track identity ·
subtitle timeline fingerprint.

AI çeviri cue zamanlarını koruduğu için aynı source timeline'dan türetilen AI
artifact aynı SyncProfile'ı kullanabilmelidir.

Undo, reset, preview ve restart sonrası persistence gereklidir.

## 13. AI audio-assisted auto sync

Net ürün hedefidir. Kullanıcı açıkça "Sesle Otomatik Senkronize Et" komutunu
vermeden başlamamalıdır.

Hedef pipeline: seçili audio track → gerekli audio segmentlerini alma → VAD →
timestamp'li speech recognition → subtitle metni normalization →
semantic/cross-lingual alignment → güvenilir anchor üretimi → outlier eleme →
constant offset modeli → linear drift modeli → gerektiğinde piecewise alignment →
confidence evaluation → kullanıcı önizlemesi → açık kullanıcı onayı.

**Confidence:** high · medium · low · rejected.

Düşük güvenli sonuç otomatik uygulanmaz. **High sonuç bile kullanıcı onayı
olmadan kalıcı uygulanmaz.**

**Privacy:** kullanıcı komutu olmadan audio analizi yok · raw audio loglama yok ·
geçici audio işlem sonrası temizlenir · remote AI'a audio gönderilecekse açık
kullanıcı izni · varsayılan `localOnly` · mümkünse yalnız seçili zaman
pencereleri · tam medya dosyasını varsayılan olarak cloud'a yükleme yok.

**Sync modelleri:** constant offset · linear drift · piecewise mapping.

Farklı intro, recap, reklam, eksik veya ek sahne gibi release farkları piecewise
model için değerlendirilebilir.

## 14. Subtitle renderer

`SubtitleRenderer` ayrı bir port olmalıdır. Adapter'lar: libmpv renderer · Apple
timed-text renderer · Media3 renderer · custom overlay.

UI renderer implementasyonunu bilmemelidir.

Engine-native external subtitle desteği güvenilir olduğunda kullanılabilir.
Manuel sync veya inspector için custom overlay kullanılabilir.

**Cue lookup lineer bütün-liste taraması olmamalıdır.** Binary search, indexed
timeline veya boundary event kullanılmalıdır. Seek sonrası doğru cue hemen
gösterilmelidir.

## 15. Secure credential storage

API anahtarları plaintext config içinde tutulmamalıdır.

| Platform | Depo |
|---|---|
| Apple | Keychain |
| Android | Keystore destekli encrypted storage |
| Windows | Credential Manager |
| Linux | Secret Service |

Kullanıcı kendi OpenSubtitles / OpenAI / OpenRouter anahtarını girer.

Secret: loglanmaz · UI'da tekrar tam gösterilmez · artifact içine girmez ·
crash/telemetry içine girmez.

İlk ürün için ayrı cloud backend gerekli değildir.

## 16. Security ve privacy

**Asla loglanmayacaklar:** medya URL'si · token-bearing query · özel tam dosya
yolu · subtitle diyaloğu · raw provider response · API key · OpenSubtitles
private file ID · özel hash/filename metadata.

**Bütün dış girdiler untrusted:** subtitle metni · provider cevabı ·
OpenSubtitles cevabı · filename · media metadata · handoff extras · glossary.

**Kurallar:** scraping yok · DRM bypass yok · torrent/debrid acquisition yok ·
path traversal reddi · symlink reddi · regular-file doğrulaması · approved HTTPS
host · bounded redirect · archive/oversize reddi · atomik artifact commit ·
cancellation sonrası late commit yok · tests gerçek provider kredisi/kotası
kullanmaz.

## 17. Non-goals

İlk kapsam dışında: Stremio subtitle add-on · browser player · web dashboard ·
hosted multi-user backend · merkezi kullanıcı hesabı · scraping · torrent/debrid
acquisition · DRM bypass · progressive incomplete translation · tam subtitle
editörü · cloud sync · sosyal özellikler · kullanıcıya playback engine
seçtirmek · teknik ID girişi · düşük güvenli auto sync'i kullanıcı onayı olmadan
uygulamak · protected/DRM audio extraction · Mac App Store dağıtımı (ADR-0034:
sandbox içinde sidecar keşfi ölçülüp elendi; kanal Developer ID + notarization).

## 18. Geliştirme ve test yaklaşımı

küçük vertical slice'lar · aynı anda tek ana implementation task · architecture
değişiklikleri ADR · deterministic fake provider/client · unit, golden,
contract, integration, security ve platform testleri · gerçek cihaz acceptance
testleri · test kanıtı olmadan task DONE değil · unrelated refactor yok ·
commit yalnız doğrulanmış kapanışta, push kullanıcı kararıyla.
