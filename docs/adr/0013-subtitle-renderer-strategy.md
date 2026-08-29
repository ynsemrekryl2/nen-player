---
adr: 0013
title: SubtitleRenderer stratejisi — engine-native çizim ve portun sınırları
status: accepted
milestone: M3
tasks: [NEN-027]
date: 2026-08-29
---

# ADR-0013 — `SubtitleRenderer` stratejisi: engine-native çizim ve portun sınırları

## Durum

`accepted`

## Bağlam

M3'ün kullanıcı hikâyesinin son cümlesi henüz doğru değil: menüden bir altyazı
seçiliyor ama ekranda hiçbir şey belirmiyor. Gömülü track seçimi motora gidiyor,
kullanıcının `.srt` dosyası ise yalnız kataloğa kaydediliyor. `NEN-028`
(milestone acceptance) buna bağlı ve M3'ün beş çıkış kriterinden ikisi
("seçim anında altyazı görünüyor", "seek sonrası doğru cue anında görünüyor")
bu karar verilmeden kanıtlanamıyor.

`docs/adr/README.md` bu numarayı baştan beri bu soru için ayırmıştı ve ADR-0033
kararı açıkça `NEN-027`'ye bıraktı.

Kısıtlar:

- **`docs/product-spec.md` §14** üç şey söylüyor: `SubtitleRenderer` **ayrı bir
  port olmalıdır** · UI renderer implementasyonunu bilmemelidir ·
  engine-native external subtitle desteği **güvenilir olduğunda**
  kullanılabilir, manuel sync veya inspector için custom overlay kullanılabilir.
- **ADR-0026** session'ı core'a verdi; gerekçesi performans değil, altyazı
  senkronu ve çeviri tetiklemesi gibi mantığın **tek bir platformdan bağımsız
  kaynaktan** yönetilmesiydi.
- **ADR-0011 Karar 3** capability'nin yalnız **gerçekten farklı olanı**
  adlandırmasını istiyor; zorunlu taban capability'ye çevrilmez.
- **K23 #4** altyazı diyaloğunu loglanamaz sayıyor. Diyalog gösterilebilir;
  ama nereye yazıldığı ve kimin elinden geçtiği bu kararın konusu.
- **Enjeksiyon iskelesi zaten duruyor:** `PlaybackEngine::inject_subtitle`,
  `Capability::ExternalSubtitleInjection`, her iki yönü de kapsayan contract
  senaryoları ve `ShellEngine::inject_subtitle(webvtt)` köprüsü NEN-021/NEN-022
  ile yazıldı. Eksik olan portun kendisi ve ucunun bağlanması.

Karar verilmezse M3 kapanmaz: `NEN-027` kod yazamaz, `NEN-028` başlayamaz.

Ölçüm kaydı: `evidence/M3/NEN-027-injection-measurement.md`
(macOS 27.0 · libmpv 2.5.0 · 2026-08-29).

## Karar

**Beş karar birlikte alınır.**

**Karar 1 — M3'te altyazıyı motor çizer.** Kullanıcının belgesi libmpv'ye
enjekte edilir ve çizimi libmpv yapar. Kendi overlay renderer'ımız M3'te
yazılmaz; manuel sync ve inspector ihtiyacı doğduğunda (M7) ikinci adapter
olarak gelir.

**Karar 2 — `SubtitleRenderer` kendi portudur.** `nen-ports` içinde kendi
trait'i, kendi capability kümesi, kendi tipli hatası, kendi fake'i ve kendi
contract kiti olur. İlk adapter'ı — engine-native olan — core'da yaşar ve
`PlaybackEngine`'in `ExternalSubtitleInjection` capability'sine delege eder;
yeni bir FFI callback yüzeyi açılmaz. Renderer capability kümesi M3'te tek
gerçek farkı adlandırır: **`ObservedText`** — renderer'ın "şu an ne çizdiğini"
söyleyebilmesi. libmpv söyleyebiliyor (`sub-text`); her renderer
söyleyemeyeceği için bu capability'dir, taban değil.

**Karar 3 — "hangi satır nasıl gösterilir" kararı core session'ındır.** Kabuk
tek çağrı yapar (`göster(kaynak)` / `gizle()`); gömülü track mi kullanıcı
dosyası mı ayrımını, kullanılamaz satırın reddini ve "aynı anda iki altyazı
çizilmez" kuralını core verir. **Altyazı diyaloğu FFI'dan kabuğa geçmez** —
belge core'da kalır, kabuk yalnız token taşır.

**Karar 4 — belge motora `memory://` üzerinden WebVTT olarak verilir.**
Kullanıcının diyaloğu geçici dosyaya **yazılmaz**. Yeni bir belge gösterilmeden
önce bir öncekinin track'i düşürülür.

**Karar 5 — renderer'ın kendi eklediği track, track enumeration'ında
görünmez.** `PlaybackEngine::tracks` medyanın kendi track'lerini bildirir;
dışarıdan eklenmiş (`external`) bir track oraya karışmaz.

## Gerekçe

**Karar 1 — çünkü §14'ün "güvenilir olduğunda" koşulu ölçüldü.** Adapter'ın
kendi ayarlarıyla kurulan bir motorda üç cue'luk bir belge enjekte edildi ve
seek sonrası çizilen metin okundu: 2.0 s → `first line`, 12.0 s (cue'suz
boşluk) → boş, 20.5 s → `third line`. Doğru cue çiziliyor ve boşlukta ekran
temiz kalıyor. Kendi overlay'imizi yazmak bugün aynı sonucu üretmek için font
seçimi, satır kırma, çakışan cue yığma, tam ekran ölçekleme ve zamanlama
döngüsü demek — M3'ün sorusu bunların hiçbiri değil, ve §14 zaten bunu M7'nin
ihtiyacına bağlıyor.

**Karar 2 — çünkü portun bedeli bugün küçük, sonra değil.** Enjeksiyonu
`PlaybackEngine`'in bir capability'si olarak bırakmak bugün bir dosya
kazandırır; ama M7'de overlay geldiğinde "altyazıyı kim çiziyor" sorusunun
sahibi olmadığı için sınır o gün icat edilir ve çağrı yerleri elden geçirilir.
Trait'i şimdi yazmak, o günü bir adapter eklemeye indiriyor. Adapter'ın core'da
durması, portun ikinci bir FFI callback yüzeyi açmadan var olmasını sağlıyor:
platformda değişen tek şey motorun `inject_subtitle`'ı implemente etmesi.

`ObservedText`'in capability olması ölçümden geliyor: mpv çizdiğini
söyleyebildiği için "gösterilen cue = `CueIndex` sonucu" iddiası **iddia değil,
test edilebilir bir eşitlik**. Söyleyemeyen bir renderer için aynı eşitlik
ölçülemez; ADR-0011 Karar 3'ün ölçütü tam olarak budur — gerçekten farklı olanı
adlandır.

**Karar 3 — çünkü ADR-0026'nın satın aldığı şey buydu.** Ayrım bugün Swift'te
duruyor (`PlayerModel.selectSubtitle`) ve doğru çalışıyor; ama aynı politika
Android ve iOS kabuklarında yeniden yazılacak ve ilk ayrışan kabuk sessizce
farklı davranacak. Diyaloğun kabuğa geçmemesi ayrıca K23 #4'ün yüzeyini
daraltıyor: kabuk metni hiç görmezse, kabuk metni loglayamaz.

**Karar 4 — çünkü ölçüldü ve alternatifi kullanıcının metnini diske yazmak.**
`sub-add memory://<webvtt> select` çağrısı `0` (success) döndü, track eklendi
ve `select` bayrağı seçimi kendisi yaptı. Geçici dosya yolunda ise diyalog
dosya sistemine düşer, izinleri ve silinmesi bizim sorumluluğumuz olur ve çöken
bir süreç dosyayı arkada bırakır. Ölçüm bu riski gereksiz kıldı.

**Karar 5 — çünkü sayı çakışıyor.** Enjekte edilen track ölçümde
`ff-index = 0` ile geldi. Port'un `TrackId`'si ff-index olduğu için bu, video
track'inin indeksiyle aynı sayı ve kataloğun gömülü kaynak id'leriyle çakışıyor.
Elenmezse kendi enjekte ettiğimiz altyazı, kataloğun hiç görmediği bir "gömülü
track" olarak menüde belirir — NEN-058'in `sub-auto`'da yakaladığı hatanın
aynısı, bu sefer kendi elimizle. NEN-058 o ölçümde aynı sayıyı görmüştü
(`tracks(.subtitle) = [3, 4, 0]`).

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| **Custom overlay renderer, M3'te.** Stil, konum ve sync offset üstünde tam kontrol; M7'nin inspector'ı için zaten gerekecek. | M3'ün sorusu çizimin kalitesi değil, çizimin **olması**. Engine-native yol ölçümde doğru cue'yu veriyor; overlay bugün font/satır kırma/yığma/ölçekleme işini M3'ün kapsamına sokar ve §14 bunu açıkça M7 ihtiyacına bağlar. Port ayrı olduğu için M7'de çağrı yeri değiştirmeden eklenir. |
| **Enjeksiyonu `PlaybackEngine` capability'si olarak bırakmak, ayrı port yazmamak.** Bugün en az kod; iskele zaten hazır ve contract kiti onu kapsıyor. | §14'ün "ayrı port olmalıdır" cümlesi ile `docs/architecture.md`'nin port tablosu düzeltilmek zorunda kalır. Asıl maliyet M7'de: sınır o gün icat edilir, custom overlay'i eklemek çağrı yerlerine dokunur. Bugünkü fark bir trait + bir fake + küçük bir kit. |
| **Ayrım kabukta kalsın** (`library.webvttOf(token)` + `session.injectSubtitle(webvtt)`). | Daha küçük değişiklik ve mevcut kodun şeklini korur; ama diyalog FFI'dan kabuğa geçer ve aynı politika her platformda yeniden yazılır. ADR-0026'nın "tek kaynak" gerekçesini doğrudan zayıflatır. |
| **Belgeyi geçici dosyaya yazıp yolunu `sub-add`'e vermek.** Her motorda çalışacağı kesin; `memory://` desteklenmeseydi tek yol buydu. | Ölçüm `memory://`'nin çalıştığını gösterdi. Kullanıcının diyaloğunu dosya sistemine bırakmak, karşılığında hiçbir şey kazanmadan K23 #4'ün yüzeyini genişletir; temizlik ve izin sorumluluğu da bize kalır. |
| **Enjekte track'i enumeration'da bırakıp katalog tarafında elemek.** Adapter daha "dürüst" olur: motorun gördüğünü aynen bildirir. | Eleme kuralı o zaman her platformda ve katalogda tekrar yazılır, ve ff-index çakışması port sınırının içine kadar taşınır. Track kimliğini bozmayan tek yer, track'i ekleyen yerdir. |

## Sonuçlar

**Olumlu:** M3'ün son iki çıkış kriteri kanıtlanabilir hâle gelir; "gösterilen
cue = `CueIndex` sonucu" ölçülebilir bir eşitliktir, iddia değil. UI hangi
motorun çizdiğini bilmez ve altyazı diyaloğu hiç görmez. M7'nin overlay'i port
sınırına hazır bir yere gelir.

**Olumsuz / kabul edilen maliyet:** M3'te altyazının görünüşü (font, boyut,
konum) motorun varsayılanıdır — kullanıcı ayarı yok. Çakışan cue'ların nasıl
yığıldığı motorun kararıdır. Renderer portu bugün tek adapter'la yaşayan bir
soyutlamadır; ikinci adapter M7'ye kadar gelmezse bu soyutlama o zamana kadar
bedelini ödemiş olmaz. Ayrım core'a taşındığı için `PlayerModel`'in bugün geçen
altyazı testleri yeni yüzeye göre yazılır.

**Geri dönüş maliyeti: ucuz–orta.** Karar 1'den dönmek (overlay'e geçmek)
port sınırı zaten orada olduğu için bir adapter eklemek; çağrı yerleri
değişmez. Karar 4'ten dönmek tek fonksiyonluk bir değişiklik. Karar 3'ten
dönmek orta: FFI yüzeyi ve kabuğun altyazı yolu geri alınır. Karar 2'den
dönmek — portu geri silmek — ucuz ama anlamsız; asıl maliyet portun hiç
yazılmamasıydı.

## İlgili task'lar

`NEN-027` (bu ADR'nin ilk ve tek kullanıcısı) · `NEN-028` (acceptance, buna
bağlı) · sonraki kullanıcı: M7 (manuel sync / inspector overlay adapter'ı)

## Notlar

<!-- Karar sonrası gözlemler -->
