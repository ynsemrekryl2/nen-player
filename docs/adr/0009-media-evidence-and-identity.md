---
adr: 0009
title: Media evidence / identity çözümleme ve fallback sırası
status: accepted
milestone: M2
tasks: [NEN-018]
date: 2026-08-25
---

# ADR-0009 — Media evidence / identity çözümleme ve fallback sırası

## Durum

`accepted`

## Bağlam

Şartname §6 kullanıcıdan IMDb/Stremio veya başka **teknik ID istemeyi
yasaklıyor** (non-goal listesi de "teknik ID girişi"ni sayıyor). Buna karşılık
medya kimliği ürünün kritik bağımlılığıdır: gömülü altyazı yoksa altyazılar
OpenSubtitles'tan gelecek (§7) ve bu **ancak kimlik çözülürse** mümkündür.
Kimlik çıkmazsa geriye her açılışta kullanıcıdan başlık/yıl istemek kalır —
ürünün vaadini doğrudan zedeleyen bir kullanıcı deneyimi.

Elimizdeki kısıtlar:

- §6 kanıt listesi: optional handoff metadata · optional canonical ID · local
  filename · file size · OpenSubtitles-compatible hash · container metadata ·
  title/year · season/episode · release name · remote URL path basename.
- §6 son cümlesi: **media URL query'si identity veya log kaynağı olarak
  kullanılamaz.**
- Uzak medyada (http/https) dosyanın **gerçek adı URL'de olmayabilir**:
  debrid/CDN akışlarında URL opak (`stream.mkv`, hash adı) olup gerçek release
  adı `Content-Disposition: attachment; filename=…` header'ında (RFC 6266) veya
  yönlendirme zincirinin sonundaki URL'de durur. Ayrıca `Content-Length` +
  `Range` istekleriyle dosyanın ilk ve son 64 KiB'ı çekilebilir — yani
  OpenSubtitles-uyumlu hash ve container başlığı **uzak dosya için de**
  hesaplanabilir. `security-policy.md` §3 bounded redirect ve content-type
  doğrulamasını zaten şart koşuyor.
- §5: Stremio canonical identity gönderirse "optional güçlü kanıt", ama
  "her zaman geleceği varsayılmamalıdır".
- `docs/security-policy.md` K23 #3 (özel tam dosya yolu) ve #8 (özel hash /
  filename metadata) loglanamaz; §2 filename ve container etiketlerini
  **düşman girdi** sayıyor.
- ADR-0006: `nen-domain` sıfır bağımlılıklı ve I/O'suz; `nen-identity` =
  "Evidence, hash, release-name parse" (`docs/architecture.md` crate tablosu).

Karar verilmezse: kanıtların hangi sırayla değerlendirildiği koda gömülü ve
denetlenemez bir alışkanlık olur; her yeni kanıt kaynağı `MediaEvidence`'ın
şeklini yeniden kırar; URL'in hangi parçasının kimliğe girip hangisinin
girmediği belirsiz kalır — bu, §6'nın query yasağı karşısında bir **güvenlik**
belirsizliğidir.

## Karar

**1. Yerleşim.** Evidence modeli, OpenSubtitles-uyumlu hash ve release-name
parser'ı **tamamen `nen-identity` crate'inde** yaşayacaktır. `nen-domain`'e
medya kimliğine ait hiçbir tip eklenmeyecektir.

**2. I/O yok.** `nen-identity`'nin bütün kanıt işleme yüzeyi **saf
fonksiyonlardan** oluşacaktır. Hash `(file_size, head_bytes, tail_bytes) ->
Result<OsHash, OsHashError>` imzasını alır; klasör zinciri, kardeş dosya
listesi, `.nfo` içeriği ve container etiketleri crate'e **girdi olarak
verilir**. Dosya sistemine dokunmak `nen-ports` adapter'larının işidir (M3).

**3. Çözülemeyen ad hata değildir.** Release-name parser'ı ayrıştıramadığı
girdi için `Err` değil `Unknown` döndürecektir. Kimlik çıkmaması oynatmayı
engellemez (§6).

**4. Fallback sırası kodda ifade edilir.** §6'nın film (`title + year → title`)
ve dizi (`series + season + episode`) fallback'i `identity_candidates()` ile
**sıralı bir aday listesi** olarak üretilecektir. Adayları kullanıcıya gösterme
adımı bu ADR'nin ve NEN-018'in kapsamı **değildir** (M3+ UI, NEN-035).

**5. URL'in tüm path segmentleri kanıttır; query ve fragment değildir.**
Remote URL'den kimlik ipucu çıkarılırken **bütün path segmentleri**
(basename'den köke doğru) değerlendirilecektir. Query string, fragment ve host
`MediaEvidence`'a **hiç girmeyecektir** — ayrıştırılmaz, saklanmaz, türev
üretilmez.

**6. Katmanlı kanıt sırası.** Kimlik şu sırayla çözülecektir; ilk `Unknown`
olmayan sonuç kazanır, sonra kardeş mutabakatı uygulanır:

| Sıra | Katman | Neden bu sırada |
|---|---|---|
| 1 | Handoff metadata (Stremio canonical ID/başlık) | §5'te "güçlü kanıt"; tahmin değil, kaynağın kendi beyanı |
| 2 | `.nfo` sidecar | Kütüphane sahibi tarafından yazılmış açık ID; tahmin değil |
| 3 | Container metadata (Matroska/MP4 etiketleri, chapter, duration) | Dosyanın içinde, ad değiştirmeden sağ kalır |
| 4 | **Dosya adı beyanı** — yerel medyada dosya adı; uzak medyada `Content-Disposition` filename'i, o yoksa **son yönlendirmeden sonraki** URL'in basename'i | En yaygın ve en bilgili tek kaynak. Uzak medyada bunun karşılığı URL'in kendisi değil sunucunun beyanıdır; debrid/CDN akışlarında gerçek release adı burada durur |
| 5 | Üst klasör adları (yakından uzağa) | Kodi/Plex düzenlerinde dosya adından güvenilir |
| 6 | Kardeş dosya mutabakatı | Karar üretmez, **doğrular**: aynı klasördeki diğer bölümler dizi/sezon çıkarımını teyit eder |
| 7 | URL path segmentleri (basename'den köke) | Son çare tahmin: sunucu hiçbir şey beyan etmediğinde geriye kalan |

Her katman bağımsız olarak `Unknown` dönebilir; hiçbirinin varlığı zorunlu
değildir ve yokluğu akışı bozmaz.

Uzak medyanın bu kanıtlarını (HTTP HEAD/`Content-Disposition`, bounded redirect
zincirinin sonu, `Content-Length`, `Range` ile head/tail byte'ları) **toplamak
bir port işidir** ve Karar 2 gereği `nen-identity`'ye giremez: NEN-018
`MediaEvidence`'ta yalnız alanları tanımlar, dolduran adapter M3'te
(`NEN-036`) yazılır. Sunucudan gelen filename **düşman girdidir** —
path traversal, kontrol karakteri ve RFC 5987 `filename*=UTF-8''…`
yüzde-encode'u dahil — sanitize edilmeden kullanılmaz ve K23 #8 gereği
loglanmaz.

**7. Aday gösterimi son çaredir.** Deterministik katmanlar tükenmeden
kullanıcıya aday listesi gösterilmeyecektir; manuel başlık girişi aday
listesinden de **sonra** gelir. Hedef, medyaların büyük çoğunluğunda kullanıcıya
hiç sormadan kimliğe varmaktır. Skor/eşik modelinin kendisi bu ADR'de karara
bağlanmaz — gerçek adaylar M6'da OpenSubtitles'tan gelince anlamlıdır (NEN-035).

## Gerekçe

**Yerleşim (Karar 1):** `docs/architecture.md`'nin crate tablosu `nen-identity`'yi
zaten tam olarak bu üç iş için tanımlıyor. NEN-016'da timeline fingerprint'in
`nen-domain` yerine `nen-subtitle`'a konmasıyla aynı gerekçe geçerli:
`nen-domain`'in her kapanışta `cargo tree` ile doğrulanan sıfır-bağımlılık
özelliği korunur ve crate sınırları belgelenmiş haliyle kalır. `nen-catalog`
zaten `identity`'ye bağlı olduğu için (aynı tablo) tüketiciler kaybetmiyor.

**I/O yok (Karar 2):** ADR-0006 bu crate'e I/O vermiyor; ayrıca saf imza,
OSDb hash'inin 12.9 MB'lık kanonik test dosyaları olmadan — sentetik tamponlarla
ve bağımsız naif referans implementasyonuyla — doğrulanabilmesini sağlıyor.

**URL genişletmesi (Karar 5):** §6 yalnız "remote URL path basename" diyor.
Pratikte Stremio/debrid akışlarında basename çoğu zaman anlamsızdır
(`stream.mkv`, `video`, hash adı) ve anlamlı ad üst segmentte durur
(`/movies/Inception.2010.1080p/stream.mkv`). Basename'le sınırlı kalmak,
§6'nın *amacını* (kimliği çıkarabilmek) korumak yerine harfini korurdu.
Genişleme yalnız **path** yönündedir: query ve fragment token taşır (K23 #1/#2)
ve host'un kendisi de kimlik sinyali olduğu için (`redact_host` yalnız sabit
allowlist adları için var) evidence'a girmez. Yani genişleyen kanıt yüzeyi,
daralan kısmı olmayan bir güvenlik sınırıyla birlikte geliyor.

**Sıra (Karar 6):** üst katmanlar **beyan**dır (kaynak ne olduğunu söylüyor),
alt katmanlar **tahmin**dir (biz addan çıkarıyoruz). 4. katmanın yerel dosya
adıyla `Content-Disposition`'ı aynı rafa koyması bu ayrımın doğrudan sonucu:
ikisi de "bu dosyanın adı şudur" beyanıdır, biri dosya sisteminden biri
sunucudan gelir; URL path segmentlerinden çıkarım yapmak ise tahmindir ve bu
yüzden en altta kalır. Pratik sonucu şu: opak bir debrid URL'i, sunucu
`Content-Disposition` verdiği anda yerel bir dosya kadar iyi kimliklenir. Beyanın tahminden önce
gelmesi, §5'in "handoff güçlü kanıttır ama garanti değildir" ifadesinin doğrudan
karşılığı: varsa güvenilir, yoksa akış bozulmadan bir alt katmana düşer.
Kardeş mutabakatının karar **üretmemesi** bilinçli — tek dosyalık yanlış
`SxxEyy` yakalamalarını elemek için yeterli, ama yanlış bir çoğunluğun doğru
bir çıkarımı ezmesine izin vermemek gerekir.

**Son çare ilkesi (Karar 7):** aday listesi ucuz bir kaçış yolu gibi görünür ve
tam da bu yüzden tehlikelidir — bir kez varsayılan hale gelirse, kanıt
katmanlarını iyileştirme baskısı ortadan kalkar ve ürün her açılışta soru soran
bir şeye dönüşür. İlkeyi ADR'ye yazmak, NEN-035'in ölçülebilir bir hedefle
(gösterim oranı) açılmasını zorunlu kılıyor.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| `MediaEvidence`'ı `nen-domain`'e koymak (NEN-013'ün subtitle değer tipleri deseni) | `docs/architecture.md` crate tablosu evidence'ı açıkça `nen-identity`'ye veriyor; NEN-016'nın fingerprint kararıyla tutarsız olurdu |
| Yalnız dosya adı + hash ile yetinmek (§6'nın harfi) | Stremio/debrid ve Kodi/Plex kütüphanelerinde en bilgili kaynaklar (URL üst segmenti, klasör adı, `.nfo`) dışarıda kalırdı → aday sorma oranı artar, Karar 7'nin hedefi tutmaz |
| URL query'sini de ayrıştırmak (bazı CDN'ler başlığı query'de taşır) | §6 son cümlesi ve K23 #1/#2 doğrudan yasaklıyor; token sızıntısı riski kazancın çok üstünde |
| Host adını kanıt saymak | Host, kullanıcının hangi servisi kullandığını açığa çıkarır; `security-policy.md` yalnız allowlist'teki sabit adların loglanmasına izin veriyor. Kimlik katkısı da yok denecek kadar az |
| Uzak medyada yalnız URL'e bakıp `Content-Disposition`/yönlendirme sonunu yok saymak | Debrid/CDN akışlarında URL opaktır; gerçek release adı tam da bu iki yerdedir. Yok saymak, uzak medyayı — yani Stremio senaryosunun tamamını — kalıcı olarak kimliksiz bırakırdı |
| Uzak kanıt toplamayı `nen-identity`'ye koymak | Karar 2'yi (I/O yok) ihlal eder ve crate'i testte gerçek ağa bağımlı kılardı; port + adapter ayrımı korunuyor (NEN-036) |
| Kanıt çelişkisini şimdi skor/ağırlıkla çözmek | Ağırlıkları kalibre edecek gerçek aday kümesi M6'dan önce yok; uydurma katsayılar kalıcılaşırdı. Şimdilik deterministik "ilk kazanan" sırası, skor modeli NEN-035 |
| Kimlik çıkmayınca doğrudan kullanıcıya sormak | Ürünün çekirdek vaadini zedeler; Karar 7 bunu son çare olarak konumlandırıyor |
| Kare tabanlı görsel tanıma / video akustik parmak izi / web scraping | Sırasıyla: kullanıcının video içeriğini dışarı çıkarır · video için elverişli açık veritabanı yok · scraping **non-goal** |
| `.torrent` metadata'sını (bencode `info` → `name`/`files`) kanıt saymak | **torrent/debrid non-goal**. Ayrıca kazancı yok: yerel medyada dosyanın zaten adı var, Stremio senaryosunda ise akış uzak medyadır ve katman 1/3/4 çalışır (bkz. Notlar) |
| Stremio URL'indeki infohash'i kimlik anahtarı yapmak | Torrent indeksi sorgulamayı gerektirir → non-goal. Aynı senaryoda `Range` ile container metadata + hash zaten daha iyi ve kapsam içi bir yol veriyor |

## Sonuçlar

**Olumlu:** kimlik çözümü tek yerde, sıralı ve test edilebilir; `MediaEvidence`
şekli bir kerede tanımlanıyor, sonraki kanıt kaynakları (NEN-033 hash lookup,
NEN-034 AI normalizasyon, NEN-036 uzak kanıt portu) modeli yeniden kırmadan
bağlanabiliyor; uzak medya kimliksiz kalmıyor — `Content-Disposition` veren bir
sunucuda opak URL bile yerel dosya kadar iyi çözülüyor, `Range` desteği varsa
hash de hesaplanabildiği için NEN-033'ün en güçlü kanıtı uzak akışlarda da
elde edilebiliyor; URL'in hangi
parçasının kimliğe girdiği artık belirsiz bir alışkanlık değil, denetlenen bir
sınır.

**Olumsuz / kabul edilen maliyet:** NEN-018 büyüyor (`size: M` → `L`);
`.nfo` ve container etiketleri yeni düşman girdi yüzeyleri açıyor (boyut/derinlik
sınırı ve panik yasağı ile karşılanır); "ilk kazanan" sırası, alttaki bir
katmanın üsttekinden daha doğru olduğu nadir durumlarda yanlış sonuç verebilir —
skor modeli gelene kadar kabul edilen bir sınırlama.

**Geri dönüş maliyeti:** orta. Katman sırası ve `Unknown` politikası ucuz
değiştirilebilir (tek fonksiyon). `MediaEvidence`'ın alan kümesi ise
`nen-catalog` ve M3 UI'ı bağlandıktan sonra değiştirilirse birden çok crate'i
etkiler — bu yüzden offline kanıt katmanları sonraya bırakılmayıp burada
toplanıyor.

## İlgili task'lar

`NEN-018` · (devamı: `NEN-033`, `NEN-034`, `NEN-035`, `NEN-036`)

## Notlar

Karar 5, `docs/product-spec.md` §6'nın kanıt listesindeki "remote URL path
basename" ifadesini **genişletir**; Karar 6'nın 4. katmanı da §6'da hiç
sayılmayan bir kaynağı (`Content-Disposition` / yönlendirme sonu) kanıt listesine
ekler. ADR-0001'in izin verdiği biçimde şartname yeniden yazılmaz; §6'ya bu
ADR'ye işaret eden bir not düşülür.

**Torrent tarafı bu ADR'nin kapsamında değildir.** `.torrent` metadata'sını
okumak (bencode `info` → `name`/`files`) CLAUDE.md'nin **torrent/debrid**
non-goal'ü kapsamındadır ve önerilmez: yalnız kapsam dışı bırakılmış bir iş
akışında karşılığı olan bir özellik için core'a yeni bir parser ve düşman-girdi
yüzeyi eklerdi. Bu üründe torrent Stremio'nun işidir; bize kendi yerel streaming
sunucusundan bir http URL'i gelir, yani senaryo **uzak medyadır** ve Karar 6
olduğu gibi geçerlidir.

Bir uyarıyla: Stremio'nun yerel sunucu URL'i genelde infohash + dosya indeksi
biçimindedir, yani **katman 7 (URL path) bu senaryoda muhtemelen boş döner** ve
`Content-Disposition` gönderip göndermediği ölçülmemiştir (NEN-036/M4'te gerçek
cihazda doğrulanmalı, varsayılmamalı). O durumda kimlik iki katmandan gelir:
katman 1 (handoff extras — §5'in "güçlü kanıt"ı) ve katman 3 (`Range` ile ilk
64 KiB'dan container `title` etiketi + `Content-Length`/head/tail ile
OpenSubtitles-uyumlu hash). Bu, NEN-036'nın `Range` yeteneğini opsiyonel bir
iyileştirmeden Stremio senaryosunun **ana kimlik yoluna** çevirir — ve infohash'i
kimlik anahtarı olarak kullanmayı gerektirmez (o, torrent indeksi sorgulamak
demek olurdu: non-goal).

**Açık kalan bir politika belirsizliği (bu ADR çözmüyor):**
`docs/security-policy.md` §3'ün "approved host — liste dışına istek atılmaz"
kuralı **sağlayıcı API'leri** için yazılmış. Kullanıcının veya Stremio'nun
verdiği medya URL'i ise keyfi bir host olabilir ve player onu açmak zorundadır.
İki isteğin farklı kurallara tabi olduğu politikada açıkça yazmıyor; `NEN-036`
bu ayrımı netleştirmeden uzak kanıt toplayan ilk kod olmamalı.
