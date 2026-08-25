---
adr: 0010
title: SubtitleSourceCatalog gruplama, dedup ve menü projeksiyon kuralları
status: accepted
milestone: M2
tasks: [NEN-019]
date: 2026-08-25
---

# ADR-0010 — SubtitleSourceCatalog gruplama, dedup ve menü projeksiyon kuralları

## Durum

`accepted`

## Bağlam

Şartname §7 **tek** bir `SubtitleSourceCatalog` şart koşuyor: dört kaynak türü
(`embedded` · `user` · `opensubtitles` · `ai`) aynı katalogta toplanacak. §8 ise
o katalogtan üretilecek menünün **tam metnini** örnekle veriyor ve sekiz kural
sayıyor (kullanıcı kaynakları kendi grubunda · diğerleri dile göre · aynı kaynak
tekrar gösterilmez · dili bilinmeyen `Dil Belirsiz` grubunda · AI sonucu hedef
dil grubunda AI rozetiyle, ayrı "AI subtitle mode" **yok** · `Kapalı` her zaman
var). M2'nin çıkış kriterlerinden biri bu menünün katalog projeksiyonundan
**birebir** üretilmesi.

Şartnamenin **söylemediği** ve bu ADR'nin eklediği şey: kullanıcının **birinci ve
ikinci tercih edilen altyazı dili**. §9 yalnız çeviri *hedef* dilini bir kullanıcı
ayarı sayıyor; izleme tercihini hiç ele almıyor. Bu, ürünün asıl kullanım
biçiminde her açılışta tekrarlanan bir iştir: kullanıcı aynı iki dili arıyor ve
her seferinde listede onları bulmak zorunda kalıyor.

Elimizdeki kısıtlar:

- §7: "Tüm kaynakları kataloglamak, bütün dosyaları hemen indirmek veya extract
  etmek anlamına **gelmez**. Pahalı işler lazy yapılmalıdır." Yani katalog
  kurulurken hiçbir kaynağın **içeriği elde değildir** — ne gömülü track'in
  metni, ne OpenSubtitles adayının dosyası.
- §7: "Player bütün erişilebilir altyazıları tarar. **AI çeviri için** otomatik
  kaynak önceliği uygulanmaz." Yasak, cümlenin kendi ifadesiyle AI çeviri
  kaynağının otomatik seçilmesine dair.
- §9: kullanıcının seçtiği kaynak hem **gösterilen** kaynaktır hem **AI çeviri
  komutunun kaynağıdır**; ve "kaynak seçmek AI çeviri başlatmaz" — çeviri yalnız
  açık komutla başlar.
- §7 OpenSubtitles için "seçilmeden download yapmama" diyor.
- §8 örneğindeki dil grubu sırası: İngilizce → Fransızca → Türkçe. Bu, görünen
  **Türkçe** adların alfabetik sırası değildir (Fransızca < İngilizce < Türkçe
  olurdu); BCP-47 etiketlerinin sırasıdır (`en` < `fr` < `tr`) ve aynı zamanda
  dillerin **kendi adlarının** sırasıdır (English < Français < Türkçe).
- §8 kullanıcı grubunda ham dosya adı gösteriyor (`Movie.en.srt`).
  `docs/security-policy.md` K23 #8 (özel hash / filename metadata) ve #3 (özel
  tam dosya yolu) bunların **loglanmasını** yasaklıyor. Gösterilebilir olmakla
  loglanabilir olmak ayrı şeylerdir.
- §7 + glossary: OpenSubtitles kaynağının dışarı gösterilen kimliği **opaque
  public source ID**'dir; private file ID / hash / filename sızdırılmaz.
- ADR-0006 crate tablosu: `nen-domain` I/O'suz ve **sıfır bağımlılıklı**,
  `nen-catalog` = "Source catalog, gruplama, dedup" (domain, subtitle, identity).
- `docs/glossary.md` `SubtitleSource`'u `nen-domain`'e, `SubtitleSourceCatalog`'u
  `nen-catalog`'a veriyor.
- Çekirdek üç platformu (macOS · Android TV · iOS/tvOS) besliyor; UI dili
  platform tarafının sorunudur.

Karar verilmezse: menü kuralları UI koduna dağılır ve platform başına yeniden
yorumlanır (§8'in "tek menü" vaadi platform başına farklı çıkar); "aynı kaynak"
tanımı yazılmadığı için dedup her çağrı yerinde yeniden icat edilir; hangi
sıranın **gösterim** hangisinin **öncelik** olduğu belirsiz kalır — bu §7'nin
yasağı karşısında doğrudan bir ürün riskidir; ve kullanıcı dosya adlarının log'a
düşüp düşmeyeceği tesadüfe kalır (K23).

## Karar

**1. Yerleşim.** `SubtitleSource` · `SubtitleSourceKind` · `SubtitleSourceId` ·
`LanguageTag` · `SubtitlePreferences` **`nen-domain`**'de yaşayacaktır; katalog,
dedup, gruplama, menü projeksiyonu ve otomatik seçim politikası
**`nen-catalog`**'ta. Hiçbiri dış bağımlılık getirmez; `nen-domain`'in
sıfır-bağımlılık özelliği `cargo tree` ile doğrulanmaya devam eder.

**2. Dedup metadata kimliğine dayanır, içeriğe değil.** Katalogda "aynı kaynak"
`SubtitleSourceId` eşitliğidir; kimlik tür başına şöyle kurulur:

| Tür | Kimlik | Neden |
|---|---|---|
| `embedded` | oynatılan medyanın track index'i | Track'ler tek medyaya ait ve numaralı; metin çıkarımı gerektirmez |
| `user` | dosya yolunun **opak digest'i** | Aynı dosyayı iki kez yüklemek tek giriş üretir; ham yol saklanmaz (K23 #3) |
| `opensubtitles` | **opaque public source ID** | §7'nin zaten şart koştuğu dış kimlik; private file ID kullanılmaz |
| `ai` | (kaynak source id + hedef dil) | Aynı kaynaktan aynı dile ikinci çeviri ayrı bir giriş değildir |

Aynı `SubtitleSourceId` ikinci kez eklenirse **upsert** uygulanır: giriş yerinde
güncellenir, **ekleme sırası korunur**. Katalog `SourceFingerprint` (NEN-016) ile
içerik düzeyinde dedup **yapmayacaktır**.

**3. `Kapalı` bir kaynak değildir.** Katalogta girişi yoktur; menü
projeksiyonunun sabit ilk girdisidir ve katalog boşken de bulunur. Dedup, dil
gruplama ve sıralama kurallarının hiçbirine girmez.

**4. Kullanıcının iki tercih edilen dili vardır ve menüde diğer dillerin
üstündedir.** `SubtitlePreferences` iki opsiyonel `LanguageTag` taşır (birinci ve
ikinci tercih). Grup sırası:

```
Kapalı
Kullanıcı Altyazıları        (kaynak yoksa gösterilmez)
<birinci tercih>             (o dilde kaynak yoksa gösterilmez)
<ikinci tercih>              (aynı kural; birinciyle aynıysa yok sayılır)
<geri kalan diller>          LanguageTag artan sırada
Dil Belirsiz                 (her zaman en son)
```

`Kullanıcı Altyazıları` §8'de olduğu gibi `Kapalı`'nın hemen altında kalır;
tercih edilen diller **dil grupları arasında** en üsttedir. Tercihler yalnız
sırayı belirler — bir grubu göstermez, gizlemez veya içeriğini değiştirmez.

**5. Grup içi sıra ekleme sırasıdır; gösterim sırası seçim önceliği değildir.**
Grup içinde tür bazlı sıralama, skor veya ağırlık **yoktur**. §8 örneğindeki sıra
(gömülü sonra OpenSubtitles, gömülü sonra AI) `docs/architecture.md`'nin veri
akışında kaynakların zaten toplandığı sıradan çıkar; ayrıca bir kural yazılmaz.

**6. Dil dışarıdan gelir; katalog dil tespit etmez.** Kaynağın dili
`Option<LanguageTag>`'dir; `None` → `Dil Belirsiz`. Tespit ve güven eşiği
NEN-020'nin işidir ve katalog onun sonucunu **olduğu gibi** gruplar.
`LanguageTag` normalize edilmiş BCP-47'dir (küçük harfli primary subtag +
opsiyonel region: `en`, `pt-br`); geçersiz girdi `Err` döner, sessizce
düzeltilmez.

**7. Menü projeksiyonu yapısaldır; dil adları endonimdir.** Projeksiyon `Closed`
· `UserSubtitles` · `Language(LanguageTag)` · `UnknownLanguage` grup türlerini ve
tür rozetini (`SubtitleSourceKind`) döndürür; görünen metin **platform UI'ının**
sorumluluğudur (NEN-026). Bir dilin görünen adı, UI dili ne olursa olsun **o
dilin kendi adıdır**: `en` → "English", `fr` → "Français", `tr` → "Türkçe" —
"İngilizce"/"Fransızca" değil. Menünün geri kalan metni (`Kapalı`, `Kullanıcı
Altyazıları`, `Dil Belirsiz`, `Gömülü`) UI diline çevrilir. Projeksiyon
**türetilmiştir**: katalogta saklanmaz, her çağrıda üretilir.

**8. Ayrı "AI subtitle mode" yoktur.** `Ai` yalnız bir `SubtitleSourceKind`
rozetidir; ayrı bir grup türü, ayrı bir menü veya ayrı bir mod
**eklenmeyecektir**. AI sonucu hedef dilinin normal grubunda görünür (§8, §9).

**9. Otomatik seçim yalnız tercih edilen dillerde ve şimdilik yalnız hazır
kaynaklarda uygulanır.** Oynatma başlarken:

1. Sırayla birinci, sonra ikinci tercih diline bakılır. **Başka hiçbir dil
   otomatik seçilmez.**
2. O dilin grubunda **tür önceliğine** göre ilk uygun kaynak seçilir. Öncelik
   sırası **`embedded` → `user` → `opensubtitles`**; aynı türden birden fazla
   kaynak varsa aralarında menü sırası (ekleme sırası) geçerlidir.
3. **Bu sıranın son basamağı şimdilik kapalıdır.** Otomatik seçim bugün yalnız
   `embedded` ve `user` kaynakları arasından yapılır; `opensubtitles` otomatik
   seçilmez, çünkü seçilmesi indirme demektir (§7 "seçilmeden download
   yapmama") ve yanlış kimlik çıkarımı yanlış altyazıyı sessizce indirir.
   `ai` otomatik seçime **hiç** girmez — §9 çevirinin yalnız açık komutla
   başlamasını şart koşuyor.
4. Hiçbiri bulunamazsa altyazı `Kapalı` başlar.
5. Otomatik seçim, kullanıcının elle seçimiyle **tamamen aynı** anlamı taşır:
   §9 gereği gösterilen kaynak olur ve "AI ile çevir" komutu verilirse o komutun
   kaynağı olur. Otomatik seçim **hiçbir koşulda çeviri, indirme veya extraction
   başlatmaz**; yalnız zaten oynatılabilir bir kaynağı gösterir.

**`opensubtitles` basamağı reddedilmiş değil ertelenmiştir.** Açılması iki şeye
bağlıdır: (a) medya kimliği çıkarımının **ölçülmüş** olarak yeterli doğrulukta
olması — `NEN-033` (hash → IMDb ID), `NEN-035` (güven skoru ve aday sıralaması,
ölçülebilir kabul kriteriyle), `NEN-036` (uzak kanıt portu); (b) açılışta
otomatik ağ isteği ve indirme, kullanıcı-görünür bir davranış sözleşmesi olduğu
için **kendi ADR'si**. Karar 9'un tür önceliği bugünden üç basamaklı yazılıyor
ki o gün gelince sıra yeniden tartışılmasın, yalnız son basamak açılsın.

Bu politika saf bir fonksiyondur (`nen-catalog`) — uygulanması (yükleme,
render) M3'ün işidir.

**10. Şartname §8 bu ADR ile genişletilmiştir.** §8'e ADR'ye işaret eden bir not
düşülür; §8 yeniden yazılmaz (ADR-0001'in izin verdiği biçim, ADR-0009'un §6
precedent'i). Genişletilmiş menünün kanonik biçimi budur — tercih **ayarlanmamış**
hali, yani §8'in örneğiyle aynı kaynak kümesi:

```
Altyazılar

Kapalı

Kullanıcı Altyazıları
  Movie.en.srt                  English
  subtitle.srt                  Français

English
  English                       Gömülü
  English — WEB-DL              OpenSubtitles

Français
  Français                      Gömülü

Türkçe
  Türkçe                        Gömülü
  AI Türkçe                     AI
```

Aynı katalog, **birinci tercih `tr`, ikinci tercih `en`** iken:

```
Altyazılar

Kapalı

Kullanıcı Altyazıları
  Movie.en.srt                  English
  subtitle.srt                  Français

Türkçe
  Türkçe                        Gömülü
  AI Türkçe                     AI

English
  English                       Gömülü
  English — WEB-DL              OpenSubtitles

Français
  Français                      Gömülü
```

§8'e göre tek fark grup başlıklarının endonime dönmesi ve tercih sırasıdır;
sekiz kuralın hiçbiri değişmemiştir.

**11. Kaynak etiketleri gösterilir, loglanmaz.** `SubtitleSource` ve
`SubtitleSourceId` `Debug`'ı **elle** yazılacaktır; `#[derive(Debug)]` yasaktır.
`Debug` çıktısı tür, dil ve sayılarla sınırlıdır — dosya adı, yol parçası,
OpenSubtitles private file ID veya bunlardan türetilmiş digest'in kendisi çıktıya
girmez. NEN-019 bu nedenle **negatif kontrollü guard testi** taşır (CLAUDE.md
Kural 3).

## Gerekçe

**Yerleşim (Karar 1):** glossary `SubtitleSource`'u zaten `nen-domain`'e veriyor
ve tipler gerçekten bağımlılıksız değerler — NEN-016'da `TimelineFingerprint`'i
`nen-subtitle`'a taşımayı gerektiren sebep (blake3 bağımlılığı) burada yok.
`SubtitleSource` yalnız `nen-catalog` tarafından değil, ileride `nen-translate`
(§9: seçilen kaynak çevirinin kaynağıdır), `nen-sync` ve `nen-ffi` tarafından da
tüketilecek; `nen-catalog`'a koymak bu tüketicileri katalog crate'ine bağlardı.

**Dedup (Karar 2):** §7'nin lazy kuralı, içerik tabanlı dedup'ı fiilen
yasaklıyor — `SourceFingerprint` hesaplamak için önce indirmek/extract etmek
gerekir, yani katalog kurulumu tam da §7'nin engellediği şeyi yapardı. Metadata
kimliği ise dört türde de zaten elde: track index numaralı, OpenSubtitles zaten
opak bir public ID veriyor, kullanıcı dosyası bir yolla geliyor, AI çıktısı
kaynağını biliyor. Kullanıcı yolunun **digest**'lenmesi iki işi birden yapıyor:
aynı dosyanın iki kez yüklenmesini tek girişe indiriyor ve K23 #3'ün yasakladığı
ham yolu modele hiç sokmuyor. Upsert'ün sırayı koruması bilinçli: bir kaynağın
dili sonradan (NEN-020) dolduğunda menü sırasının kayması, kullanıcının bakarken
listesinin altından kayması demek olurdu.

**Tercih edilen diller (Karar 4):** izleme dili, kullanıcı başına neredeyse sabit
ve en çok tekrarlanan tercihtir; onu her açılışta listede aratmak ürünün en sık
yaşanan sürtünmesidir. İki tercih, bir tercihten anlamlı biçimde daha iyi ve
üçten anlamlı biçimde daha basit: tipik durum "ana dilim, yoksa İngilizce"dir ve
bu tam olarak iki slottur. Tercihler yalnız **sırayı** değiştiriyor — grup
gizlemek veya filtrelemek, §8'in "bütün kaynaklar tek menüde" ilkesini bozardı ve
kullanıcının bulamadığı bir altyazı üretirdi.

**Sıra ve endonim (Karar 5, 7):** dil gruplarını `LanguageTag`'e göre sıralamak,
§8 örneğinin gözlemlenen sırasıyla (`en` < `fr` < `tr`) uyuşan **ve** UI
dilinden bağımsız kalan tek kural. Görünen adın **endonim** olması bu kuralı
kullanıcı için de doğru yapıyor: "English < Français < Türkçe" hem etiket
sırasıyla aynı hem gerçekten alfabetik — yani sıralama çekirdekte tamamlanıyor,
golden üç platformda sabit kalıyor **ve** kullanıcı gördüğü listeyi alfabetik
görüyor. Türkçe exonim kullanılsaydı ("İngilizce", "Fransızca") bu üçü aynı anda
sağlanamazdı: ya sıra UI diline bağlanır (golden platformlar arası kayar) ya da
kullanıcı alfabetik olmayan bir liste görür. Endonim ayrıca çok dilli izleyici
için doğrudan daha okunur — "Português" arayan biri "Portekizce"yi taramak
zorunda kalmaz.

Grup **içi** sıranın ekleme sırası olması §7'nin harfini koruyor: tür bazlı bir
sıralama tablosu yazmak (embedded > opensubtitles > ai) fiilen bir öncelik
tablosu olurdu. §8 örneğindeki sıra `architecture.md`'nin veri akışından
kendiliğinden çıkıyor. `Dil Belirsiz`'in en sonda olması ürün kararı:
çözülememiş bir şeyin çözülmüş dillerin arasına karışmaması gerekir.

**Otomatik seçim (Karar 9):** §7'nin yasağı, kendi ifadesiyle **AI çeviri için**
otomatik kaynak önceliğine dairdir; tercih edilen dilde zaten oynatılabilir bir
altyazıyı göstermek o yasağın konusu değildir. Tür önceliğinin (`embedded` →
`user` → `opensubtitles`) menü sırasından **ayrı ve açıkça** yazılması bilinçli:
gösterim sırası hiçbir öncelik taşımıyor (Karar 5), tek öncelik burada, tek
yerde ve okunabilir halde duruyor. Sıra maliyetten çıkıyor — gömülü track
dosyanın içinde ve hiçbir ek iş gerektirmiyor, kullanıcı dosyası diskte ve
okunması gerekiyor, OpenSubtitles ağdan geliyor ve indirilmesi gerekiyor.

Bugün son basamağın kapalı olması, kuralın kendisiyle değil **kanıtın
olgunluğuyla** ilgili: otomatik indirme, kimlik çıkarımı yanlış olduğunda
kullanıcının istemediği bir altyazıyı sessizce indirir ve bunun ölçülmüş bir
doğruluk zemini M6'dan önce yok (ADR-0009 Karar 7 ve NEN-035 aynı boşluğu
işaret ediyor). `ai`'nin hiç girmemesi ise ölçüyle ilgili değil ilkeyle: §9
çeviriyi açık komuta bağlıyor.

Politikanın bugünkü haliyle `embedded`/`user` ile sınırlı olması iki kuralı
birden koruyor: §7'nin lazy kuralı (otomatik seçim hiçbir indirme tetiklemiyor)
ve §9'un açık komut şartı (hiçbir çeviri tetiklemiyor). Seçim ayrıca menüde
görünür ve tek tıkla değiştirilebilir.

Kabul edilen gerilim şudur: §9 gereği seçili kaynak aynı zamanda AI komutunun
kaynağıdır, dolayısıyla otomatik seçim dolaylı olarak varsayılan bir çeviri
kaynağı belirler. Bu, kullanıcının bilinçli kararıdır ve şu üç koruma ile
sınırlandırılmıştır: seçim menüde görünür · yalnız kullanıcının kendi tercih
ettiği dilde yapılır · çeviri yine yalnız açık komutla başlar.

**K23 (Karar 11):** §8'in kullanıcı grubunda dosya adı **göstermesi**, o adın
loglanabilir olduğu anlamına gelmiyor — K23 #8 tam olarak bunu yasaklıyor.
NEN-006 (`Redacted<T>`), NEN-010 (sanitizing constructor) ve NEN-018
(`DerivedEvidence`) bu ayrımı üç kez kurdu; katalog aynı deseni sürdürür.
Negatif kontrol olmadan guard testi yanlış sebeple geçebilir (yanlış yazılmış
needle, hiç doldurulmamış alan) — bu yüzden kasıtlı bozuk ikiz zorunlu.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| `SubtitleSource`'u `nen-catalog`'a koymak (ADR-0009'un `MediaEvidence` deseni) | Glossary onu `nen-domain`'e veriyor ve ADR-0009'un gerekçesi burada tersine işliyor: tip bağımlılıksız, tüketicileri katalog dışında da var (`nen-translate` §9, `nen-ffi`) |
| İçerik fingerprint'iyle (`SourceFingerprint`) dedup | §7'nin lazy kuralını ihlal eder — dedup için indirme/extract gerekir. Ayrıca aynı içeriğin iki kaynaktan gelmesi §8'de gizlenmesi gereken bir durum değil, kaynağı rozetle ayrılır |
| Tek tercih edilen dil | "Ana dilim, yoksa İngilizce" en yaygın durum ve tek slot bunu ifade edemez; ikinci slot kullanıcıyı ilk tercihin bulunmadığı her medyada listeye geri göndermekten kurtarıyor |
| Üç veya daha fazla tercih / sıralanabilir tam dil listesi | Kazanç hızla azalıyor, ayar ekranı ve persistence karmaşıklaşıyor; üçüncü tercihin gerçekten kullanıldığına dair bir gösterge yok. Gerekirse `SubtitlePreferences` genişletilebilir (geri dönüşü ucuz) |
| Tercih edilmeyen dilleri gizlemek / katlamak | §8 "bütün kaynaklar tek menüde" diyor; gizleme, kullanıcının bulamadığı altyazı üretir |
| Görünen (exonim) ada göre sıralamak — "Fransızca, İngilizce, Türkçe" | Sıra UI diline bağlanır; aynı çekirdek Türkçe ve İngilizce UI'da farklı menü üretir, golden platformlar arası sabit kalmaz. §8'in kendi örneği de Türkçe alfabetik değil |
| Endonim yerine Türkçe exonim ("İngilizce", "Fransızca") | Sıralama, golden sabitliği ve kullanıcının gördüğü alfabetik düzen aynı anda sağlanamaz (Gerekçe → Karar 5, 7); ayrıca çok dilli izleyici için daha az okunur |
| Endonim tablosunu `nen-catalog`'a koymak | Tablo UI dilinden bağımsız olduğu için çekirdeğe konabilirdi, ama projeksiyon zaten etiketi döndürüyor ve tablo saf sunum. M10'da ikinci platform gelince çekirdeğe terfi ettirmek kırıcı olmayan bir değişiklik — şimdi eklemek erken |
| Grup içi tür bazlı sıralama (embedded > opensubtitles > ai) | Fiilen bir öncelik tablosudur; §7'nin yasağına yaklaşır. §8'in sırası zaten ekleme sırasından çıkıyor |
| Otomatik seçimi tüm dillere açmak | Kullanıcının hiç istemediği bir dilde altyazı açardı; tercih kavramının kendisini anlamsızlaştırır |
| Otomatik seçime `opensubtitles`'ı **şimdi** açmak | Reddedilmedi, **ertelendi** (Karar 9): her oynatma başında otomatik indirme demek ve kimlik çıkarımı yanlışsa yanlış altyazı sessizce iner. Doğruluk ölçülünce (NEN-033/035/036) kendi ADR'siyle açılır |
| Otomatik seçim önceliğini menü sırasına (ekleme sırası) bırakmak | Fonksiyonel bir kararı örtük bir toplama sırasına bağlardı; ayrıca Karar 5'in "gösterim sırası öncelik değildir" iddiasını çürütürdü. Öncelik tek yerde ve açık yazılıyor |
| Otomatik seçimin `ai` kaynağını seçebilmesi | §9 çevirinin yalnız açık komutla başlamasını şart koşuyor; mevcut bir AI artifact'ini otomatik açmak da kullanıcıyı sormadan AI çıktısına bağlar |
| Otomatik seçim yok, yalnız sıralama | Kullanıcı açıkça otomatik seçim istedi; §7'nin yasağı AI çeviri kaynağına dair ve Karar 9'un `embedded`/`user` sınırı o yasağın koruduğu şeyi bozmuyor |
| `Kapalı`'yı katalogda gerçek bir source olarak tutmak | Dil, tür, dedup ve rozet kurallarının **hepsine** istisna gerektirir; `SubtitleSourceKind`'a beşinci varyant eklemek modeli yanlış yerinden bozar |
| `Dil Belirsiz`'i dil gruplarının arasına yerleştirmek | Çözülememiş bir grubun çözülmüş dillerin arasına karışması; sırası da etiketi olmadığı için tanımsız |
| Menüyü katalogta önbelleğe almak | Projeksiyon ucuz ve saf; önbellek geçersizleştirme sorunu yaratır ve katalogu ikinci bir doğruluk kaynağı yapar |
| Dil tespitini katalogda yapmak | NEN-020'nin kapsamı; katalog metne erişmek zorunda kalır ve §7'nin lazy kuralına çarpar |
| Güven skoru / aday sıralamasını burada çözmek | ADR-0009 Karar 7'nin aynı gerekçesi: kalibre edecek gerçek veri M6'dan önce yok (NEN-035) |

## Sonuçlar

**Olumlu:** §8'in sekiz kuralı tek yerde, test edilebilir ve platform bağımsız;
menü üç platformda da aynı yapıdan üretiliyor. Sıralama tamamen çekirdekte
tamamlanıyor — endonim kararı sayesinde golden UI dilinden bağımsız **ve**
kullanıcının gördüğü liste gerçekten alfabetik. Tercih edilen diller, ürünün en
sık tekrarlanan sürtünmesini kaldırıyor. Dedup kuralı yazılı olduğu için "aynı
kaynak" her çağrı yerinde yeniden icat edilmiyor. Gösterim sırasının öncelik
**olmadığı** kayıtlı — §7 kod incelemesinde denetlenebilir bir iddia oluyor.
Otomatik seçim hiçbir ağ isteği veya çeviri tetiklemediği için §7'nin lazy kuralı
ve §9'un açık komut şartı bozulmuyor. K23 sınırı katalog tipleri doğarken
çekiliyor.

**Olumsuz / kabul edilen maliyet:** §8'in örnek menüsü artık şartnamede yazdığı
gibi değil (grup başlıkları endonim, sıra tercihe bağlı) — §8'e not düşülüyor
ama iki metin arasındaki fark okuyucunun ADR'yi açmasını gerektiriyor.
Metadata tabanlı dedup, aynı altyazının iki farklı kaynaktan gelmesini
birleştirmez; kullanıcı aynı içeriği iki satırda görebilir. Grup içi ekleme
sırası, kaynakların toplanma sırasına örtük bir bağımlılık yaratıyor —
golden testi kaynakları `architecture.md`'deki akış sırasında ekliyor ve bu bir
kural olarak değil **gözlem** olarak kayıtta. Otomatik seçim, tercih edilen
dilde hem gömülü track hem sidecar varsa **gömülü track'i** açar (Karar 9 tür
önceliği); dosyasının yanına altyazı koyan kullanıcı yine de menüden tek tıkla
ona geçer. Ve §9 gereği otomatik seçim dolaylı olarak varsayılan bir AI çeviri
kaynağı belirliyor — bilinçli kabul edilen gerilim (Gerekçe → Karar 9).

**Geri dönüş maliyeti:** ucuz–orta. Sıralama, tercih ve otomatik seçim kuralları
saf fonksiyonlarda (`project`, `auto_selection`), değiştirmesi ucuz; tercih
sayısını üçe çıkarmak veya endonim tablosunu çekirdeğe terfi ettirmek kırıcı
değil. `SubtitleSourceId`'nin şekli ise `nen-catalog` + M3 UI + FFI bağlandıktan
sonra değişirse birden çok crate'i etkiler — bu yüzden dört türün kimliği baştan
burada karara bağlanıyor.

## İlgili task'lar

`NEN-019` · (tüketici: `NEN-026`; besleyen: `NEN-020`; tercih ayarının UI ve
kalıcılığı: yeni task, M3; otomatik indirme basamağı: yeni task, M6 — kapısı
`NEN-033` · `NEN-035` · `NEN-036`)

## Notlar

**Karar 9'un üçüncü basamağı için hedef durum.** Medya kimliği çıkarımı ölçülmüş
biçimde yeterli olduğunda, tercih edilen dilde hiçbir yerel kaynak yoksa
OpenSubtitles'tan otomatik indirme açılacaktır: birinci tercih dilinde aday
varsa o, yoksa ikinci tercih dilinde. Bu ADR o davranışı **karara bağlamıyor** —
yalnız tür önceliğini (`embedded` → `user` → `opensubtitles`) bugünden
sabitliyor ki açılış anında sıra yeniden tartışılmasın. Açılışın ön koşulları
Karar 9'da yazılı; kendi ADR'sinde en az şu üçü ele alınmalı: kimlik güven
eşiğinin sayısal değeri, kullanıcının bunu kapatabilmesi, ve ölçülü/sayaçlı
bir başarısızlık davranışı (yanlış altyazı inerse ne olur).
