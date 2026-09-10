---
adr: 0031
title: macOS kabuk etkileşim modeli — hata sunumu, ekran gizliliği ve ayar yüzeyi
status: accepted
milestone: M3
tasks: [NEN-024, NEN-025, NEN-026, NEN-037]
date: 2026-08-25
---

# ADR-0031 — macOS kabuk etkileşim modeli: hata sunumu, ekran gizliliği ve ayar yüzeyi

## Durum

`accepted`

## Bağlam

M3'ün üç task'ı doğrudan UI üretiyor (`NEN-024` kabuk, `NEN-026` altyazı menüsü,
`NEN-037` dil tercihi ayarı) fakat UI'ın davranışı hiçbir belgede
kararlaştırılmamış. Şartname §8 menünün **içeriğini** sabitliyor — gruplama,
sıra, dedup, `Kapalı`, endonim başlıklar — **yüzeyini ve durum davranışını**
değil.

Karar bekleyen beş boşluk, üçü başka belgelerin açıkça buraya havale ettiği
konular:

1. **`Failed` sunumu tanımsız.** `PlaybackEngine` typed hata döndürüyor
   ([ADR-0011](0011-playback-port-contract.md)) ama bunun kullanıcı karşılığı
   yok. Aynı anda `NEN-025` "bozuk `.srt` playback'i **durdurmaz**" diyor —
   yani hatalar tek bir yüzeye toplanamaz.
2. **`EventsLost` sonrası resync.** ADR-0011 bunu kontratla zorunlu kılıp
   uygulamasını platform shell'e bıraktı ve kendi *Sonuçlar* bölümünde
   uyardı: "unutulursa UI bayat kalır. NEN-024/NEN-026'da bu açıkça ele
   alınmalı."
3. **Ekrana çıkan medya kimliği.** [`security-policy.md`](../security-policy.md)
   §1 #1 medya URL'sini, #3 özel tam dosya yolunu yasaklıyor — ama **log** için;
   ekran kapsam dışı. Buna karşılık `NEN-024` ve `NEN-028`'in DoD'si ekran
   kaydı istiyor ve kayıt `evidence/M3/` altına, yani **depoya** giriyor. Kural
   yazılmazsa M3'ün ilk kanıtı `/Users/<ad>/…` yolunu ve uzak medyada token'lı
   query'yi depoya yazar — K23'ün lafzını değil, koruduğu şeyi ihlal eder.
4. **Katalog dolarken menü.** M3 çıkış kriteri "medya, katalog taraması
   bitmeden oynuyor" diyor; menünün tarama sırasındaki hali ve liste büyürken
   seçimin ne olacağı tanımsız.
5. **Ayar yüzeyi çelişkisi.** `NEN-024`'ün *YAPILMAYACAK*'ı "Tercihler/ayarlar
   ekranı → sonraki milestone'lar" diyor; `NEN-037` ise M3'te ve
   `depends_on: [NEN-024]`. İkisi aynı anda doğru olamaz.

Karar verilmezse her biri implementasyon anında ad hoc çözülür ve ekran kaydı
kanıtı üretildikten **sonra** geri dönmek gerekir.

## Karar

### Karar 1 — Hata üç sınıfa ayrılır, her sınıfın kendi yüzeyi vardır

| Sınıf | Ne zaman | Yüzey |
|---|---|---|
| **Fatal** | Medya açılamıyor/çözülemiyor — ortada playback yok | Video yüzeyinin kendisi hata durumuna döner (boş durumla **aynı** yüzey), tek çıkış "Başka dosya aç" |
| **Kaynak düzeyi** | Bir altyazı kaynağı kullanılamıyor, video oynuyor | Yalnız altyazı menüsünde (Karar 5). Playback yüzeyine **çıkmaz** |
| **Geçici** | Kullanıcının doğrudan eylemi reddedildi (seek başarısız, yüklenen dosya güvenlik kapısından döndü, track değişimi reddedildi) | Transport üzerinde kendiliğinden kaybolan kısa bildirim |

Kullanıcı hata **varyantının** Türkçe karşılığını görür. Payload, dosya yolu,
URL, provider yanıtı ve **motor adı** hiçbir sınıfta gösterilmez — motor adı
yasağı `NEN-024`'te zaten var, bu karar onu hata metinlerine de genişletir.

### Karar 2 — Ekranda tam yol ve query yoktur; kanıt kaydı fixture ile üretilir

Pencere başlığı ve her hata metni medyayı yalnız **dosya adıyla** anar. Tam yol
hiçbir yüzeyde gösterilmez (macOS'ta proxy-icon zaten yolu verir). Uzak medyada
gösterilen şey **redakte host + dosya adı**; query string **hiçbir koşulda**
ekrana çıkmaz.

Ek olarak: `evidence/` altına giren ekran görüntüsü ve kaydı yalnız
`fixtures/` altındaki medyayla üretilir. Bu karar `security-policy.md` §1'in
kapsamını **genişletiyor** — yasak listesi bundan sonra yalnız log'u değil,
**depoya giren kanıtı** da kapsar.

### Karar 3 — `EventsLost` sessizce resync edilir

UI `EventsLost { dropped }` gördüğünde tam durumu (pozisyon, state, track'ler)
yeniden sorgular ve kendini günceller. Kullanıcıya **bildirim gösterilmez** —
"olay kaybettim" kullanıcı için bilgi taşımaz, önemli olan ekrandaki değerin
doğru olmasıdır. `dropped` **sayısı** loglanır; sayı, §1'in "Loglanabilecekler"
kümesindedir.

Kanıt `NEN-024`'e düşer: uygulamayı arka plana alıp öne getirdikten sonra
pozisyon ve durum doğru olmalıdır (ADR-0011'in ölçtüğü backgrounding senaryosu).

### Karar 4 — Menü taramayı beklemez; liste büyürken seçim ve odak kaymaz

Altyazı menüsü açıldığı anda `Kapalı` ve gömülü track'leri gösterir — bunlar
playback ile birlikte gelir (`NEN-023`), tarama gerektirmez. Sidecar bulundukça
ilgili dil grubu büyür ve tarama sürerken bunu belirten bir işaret bulunur.

Bağlayıcı kısım, sonradan gelen kaynağın **var olan durumu bozmaması**:

1. Yeni kaynak eklenmesi seçili kaynağı **değiştirmez**.
2. Menü açıkken eklenen kaynak odağı ve scroll pozisyonunu **sıfırlamaz**.
3. Otomatik seçim yalnız **bir kez**, oynatma başlangıcında çalışır; sonradan
   keşfedilen bir kaynak — tercih edilen dilde olsa bile — otomatik seçimi
   **yeniden tetiklemez**. Bu, §8'in "oynatma başlarken otomatik seçilir"
   ifadesinin tek tutarlı okumasıdır.

### Karar 5 — Hatalı kaynak menüde kalır; güvenlikten dönen dosya kataloğa hiç girmez

İkisi farklı şeydir ve farklı davranır:

- **Kataloğa giren ama kullanılamayan kaynak** (encoding/parse hatası): menüde
  **kalır**, soluk gösterilir, **seçilemez**, yanında kapalı kümeden kısa bir
  sebep etiketi taşır — `okunamadı` · `biçim hatalı` · `çok büyük`. Kullanıcı
  sidecar'ının neden çalışmadığını görür; teknik detay ve yol gösterilmez.
- **Güvenlik kapısından dönen dosya** (symlink, path traversal, dizin/FIFO,
  boyut sınırı — `NEN-025`, `security-policy.md` §4): kataloğa **hiç girmez**,
  menüde izi olmaz. Kaynak olmadı ki listelensin. Kullanıcı bu dosyayı
  **açıkça kendisi yüklediyse** Karar 1'in *geçici* sınıfından bildirim alır;
  sidecar taramasında sessizce elenir.

### Karar 6 — M3'te tek ayar yüzeyi vardır: iki dil seçici

`NEN-037` M3'te kalır ve macOS'un standart Settings penceresini açar, fakat bu
pencere M3'te **yalnız birinci/ikinci tercih edilen dil seçicilerini** içerir.
`NEN-024`'ün "Tercihler/ayarlar ekranı → sonraki milestone'lar" yasağı buna
daraltılır: yasak olan **genel ayarlar ekranı**dır, `NEN-037`'nin iki seçicisi
değil.

## Gerekçe

**Karar 1.** Ürünün omurgası "altyazı sorunu playback'i durdurmaz"
(`NEN-025`, M3 çıkış kriteri). Tek bildirim yüzeyi bu omurgayla çelişir: dosya
açılamadığında ortada transport bile yoktur, bozuk sidecar ise transport'a
çıkarsa her oynatmada gürültü üretir. Sınıfı belirleyen soru "hata ne kadar
ciddi" değil, **playback'in devam edip etmediği**.

**Karar 2.** Log yasağının gerekçesi (§1 #3: "kullanıcı adı ve kütüphane
yapısını açığa çıkarır") ekran kaydı depoya girdiği anda birebir geçerli olur.
Yasağı ekrana değil **kanıta** genişletmek, kullanıcının kendi makinesinde
kendi yolunu görmesini engellemeden sızıntıyı kapatıyor. Dosya adının kendisi
de kimlik taşır ("Movie.2019.WEB-DL.mkv" → medya parmak izi, §1 #8) — bu yüzden
kanıt kaydı fixture'la üretilir, basename kuralı tek başına yetmez.

**Karar 3.** ADR-0011 `EventsLost`'u "sessiz kayıp" ile "bloke olan üretici"
arasındaki üçüncü yol olarak tanımladı; görünürlük **tüketici için** gerekli,
kullanıcı için değil. Kullanıcının gördüğü tek şey doğru pozisyondur.

**Karar 4.** Menü taramayı beklerse, "medya beklemeden oynar" ilkesinin menü
tarafındaki karşılığı bozulur — kullanıcı oynayan videoda hazır bir gömülü
track'e erişemez. Üç alt kural olmadan kısmi liste kabul edilemez: liste
altından kayan bir menü, kullanıcının yanlış kaynağı seçmesine yol açar.
3. maddenin alternatifi ("yeni gelen tercih edilen kaynağı otomatik seç")
kullanıcının izlediği altyazıyı habersizce değiştirebilir; §9'un "zorla AI
çıktısına geçilmez" ilkesiyle aynı aileden bir yasak.

**Karar 5.** Reddedilen dosyayı listelemek iki şeyi birden yapardı: kullanıcıya
hiçbir zaman kullanılamayacak bir satır göstermek ve reddedilen yolun varlığını
menüde teyit etmek. Kataloğa giren bozuk kaynağı **gizlemek** ise farklı bir
hata: kullanıcı `.srt`'sini yanına koymuştur, listede yoksa uygulamanın onu
görmediğini sanır.

**Karar 6.** `NEN-037` `S` boyutunda ve ADR-0010 Karar 4'ün UI karşılığı;
menünün tercih sırasının M3'te kanıtlanması ona bağlı. macOS Settings penceresi
SwiftUI'de `Settings` scene'i ile birkaç satır — çelişkiyi task'ı ertelemek
yerine yasağı daraltarak çözmek daha ucuz.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Tek bildirim yüzeyi (hepsi transport üstünde) | Dosya açılamadığında transport yok; kaynak düzeyi hatalar her oynatmada gürültü üretir |
| Fatal hata için modal alert, gerisi yalnız log'a | Bozuk sidecar'ın neden listede olmadığı kullanıcıya hiç görünmez — `NEN-025`'in "hatalı işaretle" davranışının UI karşılığı kaybolur |
| Ekran serbest, yalnız log kısıtlı (K23 lafzı) | `evidence/M3/` depoya giriyor; kanıtın elle sansürlenmesine güvenmek insan hatasına açık |
| Yalnız basename kuralı, fixture zorunluluğu yok | Basename de kimlik taşır (§1 #8); gerçek bir film adı depoya girer |
| `EventsLost`'u kullanıcıya göstermek | Kullanıcının yapabileceği bir şey yok; teknik iç durumu UI'a sızdırır |
| Menü taramayı beklesin | "Medya beklemeden oynar" ilkesini menü tarafında bozar; hazır gömülü track erişilemez olur |
| Sonradan gelen tercih edilen kaynağı otomatik seç | İzlenen altyazıyı habersizce değiştirir — §9'un zorla geçiş yasağıyla aynı aile |
| Hatalı kaynağı menüden düşürmek | Kullanıcı dosyasının neden yok olduğunu anlayamaz |
| Hatalı kaynağı seçilebilir bırakmak | Seçildiğinde hiçbir şey olmayan satır üretir |
| Tercih ayarını altyazı popover'ının içine koymak | macOS kullanıcısının ayarı arayacağı yer değil; popover'ı büyütür |
| `NEN-037`'yi M4+'a ertelemek | ADR-0010 Karar 4/10'un menü sırası M3'te kanıtsız kalır |

## Sonuçlar

**Olumlu:**

- ADR-0011'in açık bıraktığı `EventsLost` sorumluluğu kapanıyor ve **kanıtı**
  `NEN-024` DoD'sine bağlanıyor.
- M3'ün ilk ekran kaydı gizli veri taşımadan üretilebiliyor; kanıt kuralı
  `NEN-028`'e kadar tüm task'larda aynı.
- `NEN-024` ↔ `NEN-037` çelişkisi çözülüyor.
- Karar 4'ün üç alt kuralı `NEN-026`'ya doğrudan test edilebilir madde veriyor.

**Olumsuz / kabul edilen maliyet:**

- Üç hata yüzeyi tek yüzeyden fazla kod; boş durum ile fatal hata yüzeyinin
  paylaşılması bu maliyeti kısmen geri veriyor.
- Fixture zorunluluğu, gerçek bir filmle "şuna bak" demeyi kanıt kaydı için
  yasaklıyor — geliştirme sırasında serbest, yalnız `evidence/` altına giren
  kayıt bağlı.
- Karar 4.3 nedeniyle, oynatma başladıktan sonra bulunan tercih edilen dildeki
  sidecar elle seçilmek zorunda.

**Geri dönüş maliyeti: ucuz.** Altı kararın hiçbiri port kontratına veya
domain modeline dokunmuyor; hepsi platform shell içinde yaşıyor. En pahalısı
Karar 4.3 — geri dönmek otomatik seçimi olay-tetiklemeli hale getirmek demek,
bu da `nen-catalog`'un `auto_select` çağrı noktasını değiştirir, mantığını
değil.

## İlgili task'lar

`NEN-024` · `NEN-025` · `NEN-026` · `NEN-037` · kanıt tarafı `NEN-028`

## Notlar

Bu ADR yalnız **bağlayıcı** kararları taşır. Aynı beyin fırtınasında alınan
kabuk şekli kararları — tek pencere modeli, kontrollerin otomatik gizlenmesi,
boş durum yerleşimi, klavye kısayol kümesi, süre göstergesi biçimi, menünün
popover olması, rozet metinleri — mimari değildir ve ilgili task dosyalarının
*Kapsam* bölümüne yazılmıştır.

Beyin fırtınasının 2. turu `NEN-024` çalışır hale geldikten sonra, `NEN-026`
başlamadan yapılacaktır: ekrandaki gerçek şeye bakarak görsel ince ayar ve §8'in
görünen metninin doğrulanması.

**Karar 5'in etiket kümesi [ADR-0035](0035-subtitle-source-reason-labels.md)
ile daraltıldı (2026-08-27, `NEN-056`).** Karar 5'in birinci maddesi
`çok büyük` etiketini sayıyor, ikinci maddesi ise boyut sınırını güvenlik
kapısına koyuyor — kapıdan dönen dosya kataloğa hiç girmediği için o etiket
hiçbir zaman üretilemezdi. ADR-0035 kümeyi `okunamadı` · `biçim hatalı` olarak
sabitliyor; Karar 5'in geri kalanı ve bu ADR'nin diğer beş kararı yürürlükte.
Gövde ADR-0001 gereği olduğu gibi bırakıldı.

**Açık kullanıcı seçimi kısayolu [NEN-071] ile ayrıştırıldı (2026-09-07).**
`Altyazı Dosyası Yükle…` paneli `resolvesAliases = true` ile yapılandırılır;
`NSOpenPanel` symlink/alias hedefini uygulamaya vermeden çözer ve hedef normal
bir kullanıcı altyazısı olarak yüklenir. Bu, ADR-0031 Karar 5'teki uygulamanın
keşfettiği veya doğrudan güvenlik kapısına ulaşan symlink'in reddedilmesiyle
çelişmez: iki yüzeyin kullanıcı niyeti ve tehdit modeli farklıdır.

**Karar 6'nın kapsamı `NEN-101` ile genişledi (2026-09-10).** Karar 6'nın
"M3'te tek ayar yüzeyi" ifadesi kapsamı adıyla **M3'e** koyuyordu; M5'in
`NEN-101`'i aynı Settings penceresine üçüncü bir satır — AI çeviri hedef
dili — ekliyor. Bu, Karar 6'nın kendisini ihlal etmiyor: yasaklanan şey genel
bir ayarlar ekranı, pencere hâlâ yalnız dil seçicilerinden ibaret ve
`NEN-024`'ün yasağı aynı biçimde yürürlükte. Gövde ADR-0001 gereği olduğu gibi
bırakıldı.
