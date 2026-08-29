---
adr: 0037
title: Kabuk kromunun örttüğü bant renderer'a bildirilir
status: accepted
milestone: M3
tasks: [NEN-066]
date: 2026-08-29
---

# ADR-0037 — Kabuk kromunun örttüğü bant renderer'a bildirilir

## Durum

`accepted`

## Bağlam

`NEN-066` ölçüldü ve kusur çizimde değil: motor altyazıyı kareye doğru
bileştiriyor, uygulama kendi kromunu onun **üstüne** koyuyor
(`evidence/M3/NEN-066-measurement.md`).

Sayılar (1280×720 yüzey = 640×360 pt, ölçüldü):

| | Pencerenin altından |
|---|---|
| Motorun altyazıyı koyduğu bant | **15–32,5 pt** |
| `TransportControls` + `.padding(.bottom, 20)` | **20–134 pt** |

Altyazı bandı yüzey yüksekliğinin oranı olarak ölçekleniyor (%4,2–%8,9), bar
ise sabit 114+20 pt. Bandın üst kenarı barın üstüne ancak pencere **~1 505
pt'den uzun** olursa çıkar — tam ekran dahil hiçbir gerçek pencerede çıkmıyor.
Yani bu, belli bir pencere boyutunda görülen bir aksaklık değil, her boyutta
geçerli bir örtme.

Kısıtlar:

- **ADR-0013 Karar 3:** "hangi satır nasıl gösterilir" kararı core session'ın.
  Kabuğun motora dair tek teması yüzey kurulumu; altyazı yolunda kabuk yalnız
  token taşıyor (`NEN-027` grep kanıtı).
- **ADR-0013'ün portu** bugün açıkça "no styling, no positioning, no sync
  offset — those are M7's questions" diyor (`renderer/surface.rs`).
- **`NEN-066` YAPILMAYACAK:** "Altyazının stilini, konumunu veya boyutunu
  ayarlamak — M7."
- **ADR-0011 Karar 3:** capability yalnız gerçekten farklı olanı adlandırır.
- **ADR-0026:** politikanın tek platformdan bağımsız kaynağı çekirdektir.

Karar verilmezse `NEN-066` kapanmaz: altyazının görünür olması için konumunun
değişmesi gerekiyor ve bunun hangi katmanın işi olduğu bugün yazılı değil.

## Karar

**Altı karar birlikte alınır.**

**Karar 1 — kabuk, yüzeyin altında kendi kromunun örttüğü bandı çekirdeğe
bildirir; renderer altyazıyı o bandın dışında çizer.** Bu bir **yerleşim**
bilgisidir, stil tercihi değil: kabuk "şu dikdörtgen görünmüyor" der, "altyazı
şurada dursun" demez. M7'nin stil/konum/boyut **ayarları** kapsam dışı kalmaya
devam eder ve bu karar onları getirmez.

**Karar 2 — birim, yüzey yüksekliğinin oranıdır** (`f32`, `0.0..=0.5`).
Piksel, nokta veya dp değil. Aralık dışındaki değer tipli hatayla **reddedilir**;
sessizce kırpılmaz.

**Karar 3 — değer renderer portunun durumudur**, `show`'un parametresi değil:
`SubtitleRenderer::set_bottom_inset(f32)`. Ekranda zaten duran belgeye de
uygulanır.

**Karar 4 — taban yetenektir, capability değildir.** Her renderer kabul etmek
zorundadır; varsayılan `0.0`.

**Karar 5 — engine-native renderer bunu motora geçirir.**
`PlaybackEngine::set_subtitle_bottom_inset(f32)` portun tabanına eklenir ve
libmpv adapter'ı `sub-pos = round((1 - f) × 100)` yazar.

**Karar 6 — kabuk yalnız transport barının bandını bildirir.** Altyazı paneli
(sağa yaslı, yalnız kullanıcı seçerken açık) bildirilmez. Kontroller gizlenince
bildirilen değer `0.0`'dır.

## Gerekçe

**Karar 1 — çünkü bu bilgiyi yalnız kabuk biliyor ve kararı yalnız çekirdek
verebilir.** Kromun yüksekliği kabuğun kendi yerleşimi; hiçbir çekirdek onu
tahmin edemez. "Altyazı görünür kalsın" kuralı ise politikadır ve ADR-0026'nın
gerekçesiyle çekirdektedir — Android kendi transport barını koyduğunda aynı
kuralı yeniden yazmayacak, yalnız kendi sayısını bildirecek. Bu ayrım aynı
zamanda `NEN-066`'nın YAPILMAYACAK maddesini korur: burada seçilen bir konum
yok, örtülen bir alan var.

**Karar 2 — çünkü oran ölçüldü ve doğrusal çıktı.** Aynı ölçüm düzeneğinde
`sub-pos` üç değerde denendi (yüzey 360 pt):

| `sub-pos` | Gömülü track bandı | Enjekte belge bandı | Kayma |
|---|---|---|---|
| 100 | 15,0–32,0 pt | 18,5–32,0 pt | — |
| 80 | 84,0–101,0 pt | 87,5–101,0 pt | **69 pt = %20,0** |
| 65 | 135,5–152,5 pt | 139,0–152,5 pt | **120,5 pt = %33,5** |

Kayma tam olarak `(100 − sub-pos)` × yüzey yüksekliği, ve gömülü track ile
enjekte belge **birebir aynı** davranıyor. Piksel/pt birimi seçilseydi her
platform kendi ölçek faktörünü (HiDPI backing, Android dp) çekirdeğe taşımak
zorunda kalırdı; oran bu sorunun tamamını kabukta bırakıyor.

Aralık dışını reddetmek, kırpmak yerine: `0.9` gönderen bir kabuk altyazıyı
ekranın tepesine atardı ve sessiz kırpma bunu bir davranış gibi gösterirdi.
`0.5` tavanı ölçülen ihtiyacın (134/360 = **0,372**) üstünde ve ekranın
yarısından fazlasını örttüğünü iddia eden bir krom zaten kendi hatasıdır.

**Karar 3 — çünkü değişme sıklığı belgeninkinden bambaşka.** Kontroller her
görünüp gizlendiğinde değişiyor; belge saatte bir kez. `show`'un parametresi
olsaydı her krom animasyonunda belgeyi yeniden enjekte etmek gerekirdi —
`sub-add`'in maliyeti ve ADR-0013 Karar 4'ün "önce eskisini düşür" adımıyla
birlikte, saniyede iki kez.

**Karar 4 — çünkü örtülen alana çizmemek bir renderer'ın taban işidir.**
ADR-0011 Karar 3'ün ölçütü "gerçekten farklı olanı adlandır": burada farklı
olan bir şey yok — kendi yerleşimini yapan overlay renderer'ı için bedava,
engine-native yol için tek property yazımı. Capability yapılsaydı, karşılığı
"bu renderer altyazıyı görünmez yere çizer" olurdu; tipli bir reddin kullanıcı
için karşılığı yok.

**Karar 5 — çünkü çizen motor.** Engine-native renderer'ın kendi çizim yüzeyi
yok; portun tabanına eklemek dışında motora ulaşacak yolu yok. `sub-pos`
seçimi ölçülmüş olan: `sub-margin-y` yalnız `sub-use-margins` ile ve ASS
tarafında `sub-ass-force-margins` ile çalışıyor, `sub-pos` ise gömülü SRT ve
enjekte WebVTT'de aynı doğrusal sonucu verdi.

`round((1 − f) × 100)` eşlemesi motorun kendi alt marjını (varsayılan ~%4,2)
olduğu gibi bırakıyor; bu marj kaydırmadan sonra altyazı ile barın arasındaki
boşluğa dönüşüyor. 134/360 = 0,372 → `sub-pos = 63` → bant 149–166,5 pt,
yani barın üstünde ~15 pt boşlukla.

**Karar 6 — çünkü paneli de bildirmek altyazıyı ekranın ortasına çıkarırdı.**
Panel 260 pt; barla birlikte 394 pt, 450 pt'lik asgari pencerede yüzeyin
%87'si. Panel ayrıca sağa yaslı ve yalnız kullanıcı satır seçerken açık —
altyazının ortalandığı bandı bar kadar güvenilir biçimde örtmüyor.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| **Sabit alt marj:** altyazı her zaman 134 pt'nin üstünde dursun, krom görünsün görünmesin. En az kod, port hiç değişmez. | Kontroller zamanın büyük kısmında gizli, ve o sürede altyazı sebepsiz yere yüzeyin %37'si kadar yukarıda durur. Küçük pencerede bu ekranın ortasıdır. Bedeli her an ödenirken faydası yalnız kromun görünür olduğu anlarda. |
| **Kabuk `sub-pos`'u doğrudan yazsın.** Tek satır; ne port ne FFI değişir. | ADR-0013 Karar 3'ün kabuktan aldığı şeyi geri verir: kabuk motorla altyazı hakkında konuşmaya başlar ve aynı politika Android'de yeniden yazılır. `NEN-027`'nin grep kanıtı ("kabukta renderer implementasyon adı geçmiyor") da düşer. |
| **Kromu taşı / küçült:** bar altyazıyı hiç örtmesin. | NEN-061'in çerçevesiz, videonun üstüne binen tasarımını geri alır ve altyazı yine de tam ekranda videonun altına düşer. Örtüşme sorununu çözmez, yerini değiştirir. |
| **Birim piksel veya pt olsun.** Kabuk zaten pt biliyor, dönüşüm bir bölme. | Çekirdek yüzeyin yüksekliğini bilmiyor; bilmesi için her platformun backing scale'ini de göndermesi gerekirdi. Oran bu bilgiyi hiç istemiyor. |
| **`set_bottom_inset` capability olsun**, desteklemeyen renderer tipli reddetsin. | Reddin kullanıcı karşılığı "altyazı görünmez" olur. ADR-0011 Karar 3 capability'yi *farklı olanı* adlandırmak için ayırıyor; burada farklı olan yok. |
| **Kromun kendisi altyazıyı ölçüp etrafından dolaşsın** (kabuk `rendered_text`'i okuyup barı gizlesin). | Kabuğa diyaloğu okutur — ADR-0013 Karar 3'ün ve K23 #4'ün tam tersi. |

## Sonuçlar

**Olumlu:** `NEN-066`'nın "Sonuç" cümlesi karşılanabilir hâle gelir ve kusur
sınıf olarak kapanır: kabuk kromunu ne zaman değiştirirse değiştirsin altyazı
altında kalmaz. Kural platformdan bağımsız tek yerde durur; Android kendi
sayısını bildirmekle yetinir. Örtüşme, ekran kontrolü gerektirmeyen bir testle
ölçülebilir hâle gelir (bant oranı ile krom oranı kesişmiyor).

**Olumsuz / kabul edilen maliyet:** İki portun tabanı da genişler
(`SubtitleRenderer` ve `PlaybackEngine`), yani contract kitleri, fake'ler ve
FFI yüzeyi birlikte büyür. Altyazı, kontroller göründüğünde **yer değiştirir**
— hareket bilinçli ve M7'ye kadar ayarı yoktur. Altyazı paneli açıkken panelin
sağ alt köşesi hâlâ bir altyazı satırının sağ ucunu örtebilir (Karar 6'nın
kabul edilen maliyeti). Ve `sub-pos` yuvarlaması yüzde biriminde: 360 pt'lik
bir yüzeyde bir birim 3,6 pt, yani hizalama pt hassasiyetinde değil.

**Geri dönüş maliyeti: ucuz.** Karar 5'ten dönmek (başka bir mpv knob'una
geçmek) tek adapter satırı. Karar 2'nin birimini değiştirmek portun imzasında
tek tip. Karar 1'den bütünüyle dönmek — özelliği kaldırmak — iki port
metodunun silinmesi; çağrı yerleri tek.

## İlgili task'lar

`NEN-066` (bu ADR'nin ilk ve tek kullanıcısı) · `NEN-027` (görsel koşusu buna
bağlı) · `NEN-060` (aynı kök nedene bağlandı) · sonraki kullanıcı: M7 (overlay
renderer aynı tabanı implemente eder)

## Notlar

<!-- Karar sonrası gözlemler -->
