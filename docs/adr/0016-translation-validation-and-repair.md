---
adr: 0016
title: Çeviri doğrulama ve onarım politikası — yerel doğrulama authoritative
status: proposed
milestone: M5
tasks: [NEN-091]
date: —
---

# ADR-0016 — Çeviri doğrulama ve onarım politikası — yerel doğrulama authoritative

## Durum

`proposed`

## Bağlam

`docs/product-spec.md` §10 iki şeyi tartışmaya kapatıyor: provider cevabı
**untrusted**'dır, ve structured output kullanılsa bile **yerel doğrulama
authoritative**'dir. Doğrulama kümesi de sayılı: exact cue count · yalnız izin
verilen cue ID · unique cue ID · non-empty text · beklenen cue sırasına
normalization. Onarım bütçesi de sayılı: en fazla **2 targeted repair** + **1
full-block retry**.

Kararlaştırılması gereken, sayıların arasında kalan boşluk:

1. **İhlal taksonomisi.** Hangi ihlaller ayrı sınıflar? "Eksik cue" ile "bozuk
   zaman alanı" aynı muameleyi mi görür?
2. **Hangi ihlal onarılabilir?** Targeted repair yalnız ihlal eden cue'ları
   yeniden ister; ama örneğin provider bir `TimeSpan`'i değiştirdiyse bu
   onarılacak bir eksiklik mi, yoksa doğrudan blok başarısızlığı mı?
3. **Normalizasyonun sınırı nerede?** Karışık sırada gelen geçerli bir cevap
   normalize edilir. Peki tekrar eden ID? Fazladan whitespace? Nerede
   "düzeltilir", nerede "reddedilir"?
4. **Zaman ve ID dokunulmazlığı.** Provider'ın cue ID veya `TimeSpan`
   döndürmesi gerekiyor mu, yoksa bunlar hiç gönderilmeyip yerelde mi
   birleştirilir? İkinci seçenek bütün bir ihlal sınıfını imkânsız kılar.
5. **Hata mesajı ne taşıyabilir?** K23 #4 gereği cue diyaloğu hiçbir log
   yüzeyine düşemez — ihlal raporu bu yüzden metin taşıyamaz, yalnız şekil
   (hangi cue ID, hangi ihlal sınıfı).

## Karar

1. **Zaman ve ID dokunulmazlığı önce, taksonomiden bağımsız olarak
   çözülür:** provider'a `TimeSpan` hiç gönderilmez; cue ID yalnız hangi
   metnin hangi cue'ya ait olduğunu belirtmek için gönderilir. Nihai birleşme
   yerelde, orijinal `SubtitleDocument`'in kendi `TimeSpan`'leriyle yapılır.
   Bunun sonucu: "provider zamanı/ID'yi değiştirdi" diye bir ihlal sınıfı
   **yapısal olarak var olamaz** — port tipi bunu ifade etmez.
2. **İhlal taksonomisi**, madde 1'in sonucuyla daralır:
   `count-mismatch` · `missing-cues` · `duplicate-cues` · `unknown-cues` ·
   `empty-text` · `invalid-response` (ayrıştırılamayan/şekli bozuk cevap).
   Altı sınıf da yalnız **şekil** taşır, hiçbiri cue metni içermez.
3. **Onarılabilir ihlaller:** bir output cue ID'sinin sonuçta güvenilir,
   tekil bir metne sahip olmaması — nedeni ister `missing-cues`, ister
   `duplicate-cues` (o ID'nin bütün kopyaları elenir), ister `unknown-cues`
   (bloğa ait olmayan ID'ler yok sayılır), ister `empty-text` olsun — o ID'yi
   **targeted repair**'in isteyeceği kalan kümeye ekler. `count-mismatch` ve
   `invalid-response`, hangi belirli cue'nun bozuk olduğu belirlenemediği
   için targeted repair'e girmez, doğrudan full-block retry'a düşer.
4. **Normalizasyonun sınırı:** kabul edilen tek normalizasyon, geçerli
   cevabın `outputCueIds` sırasına yeniden dizilmesi ve metin kenarlarındaki
   boşluğun kırpılmasıdır. Tekrar eden ID ve boş/yalnızca-boşluk metin
   **düzeltilmez** — o ID doğrudan repair listesine düşer (madde 3).
5. **Hata mesajı** yalnız şekil taşır: ihlal sınıfı (enum varyantı) ·
   `blockNumber` · `blockCount` · beklenen sayı · (varsa) alınan sayı. Sayısal
   `CueId` değeri taşınabilir (K23 kapsamında değildir), cue metni hiçbir
   biçimde taşınmaz.

## Gerekçe

Madde 1, önceki bir projedeki (Stremio AISubtitle) çalışan koddan doğrudan
alınan bir tasarım: provider'ın `{cueId, text}` dışında bir şey döndürmemesi,
zaman alanlarını "provider bozdu mu" diye doğrulamayı gereksiz kılıyor —
doğrulama testiyle değil, tip sistemiyle garanti ediliyor. Bu, K23'ün "cue
diyaloğu hiçbir log yüzeyine düşemez" kısıtıyla da uyumlu: zaman zaten yerelde
kaldığı için ihlal raporunun taşıyabileceği en hassas alan zaten sayısal
`CueId`'dir, metin değil.

Madde 3'ün ayrımı (targeted-repair-edilebilir vs. doğrudan full-retry),
onarım bütçesinin (`NEN-092`, en fazla 2 targeted + 1 full retry) amacını
netleştirir: targeted repair yalnız "hangi cue eksik" belli olduğunda anlamlı
bir istek üretebilir; `count-mismatch` ve `invalid-response` bu bilgiyi
vermez, dolayısıyla targeted repair'i bu ikisinde denemek gereksiz bir
provider çağrısı ekler.

Madde 4, aynı referans hattın `inspectBlock` mantığından: tekrar eden ID'yi
"ilk geçerli eşleşmeyi tut" diye sessizce çözmek yerine, o ID'yi tamamen
elemek ve tekrar istemek — hangi kopyanın doğru olduğuna dair keyfi bir
seçimden kaçınır.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Ayrı bir "bozuk `TimeSpan`" ihlal sınıfı | Zaman hiç gönderilmediği için (madde 1) bu sınıf yapısal olarak boş kalır |
| Tekrar eden/bilinmeyen ID'lerin sessizce ilk geçerli eşleşmeye indirgenmesi | Hangi kopyanın doğru olduğuna dair keyfi, denetlenemez bir seçim; yerine tekrar isteme |
| Hata mesajının cue metninin bir kısmını (debug amaçlı) taşıması | K23 #4 ihlali — negatif testle kapatılan bir kural |
| `count-mismatch`/`invalid-response` için de targeted repair denemek | Hangi cue'nun bozuk olduğu bilinmiyor; anlamsız bir istek üretir, bütçeyi boşa harcar |

## Sonuçlar

**Olumlu:** taksonomi küçük ve tüketilebilir; zaman/ID ihlal sınıfı yapısal
olarak yok edildiği için o sınıfın davranışsal testi gerekmez, yalnız port
tipinin `TimeSpan` taşımadığı derleme zamanında sabittir; hata mesajı doğuştan
redaction'a uyumlu.

**Olumsuz / kabul edilen maliyet:** `NEN-091`'in bugünkü DoD ifadesi
("provider'ın değiştirdiği bir `TimeSpan` veya cue ID reddediliyor") artık
davranışsal bir negatif test değil, tipin bunu ifade edemediğini gösteren bir
derleme-zamanı/şekil kontrolüne karşılık gelir — bu ADR kabul edildiğinde
task'ın DoD ifadesi buna göre güncellenmelidir.

**Geri dönüş maliyeti:** ileride provider'ın zaman öneren bir özelliği
(ör. otomatik zaman kayması) gerekirse, yeni bir ihlal sınıfı ve `schema
version` artışı gerekir — bugünkü karar bunu yapısal olarak dışlamaz, yalnız
M5 kapsamının dışında tutar.

## İlgili task'lar

`NEN-091` · tüketiciler: `NEN-092`, `NEN-093`, `NEN-094`

## Notlar

Karar taslağı, önceki bir projedeki (Stremio AISubtitle, TypeScript) çalışan
bir çeviri hattının karşılaştırmalı incelemesinden türetildi — bkz. o hattın
`translation/pipeline.ts` (`inspectBlock`, `translateBlockWithRepair`)
dosyasındaki taksonomi ve zaman/ID hiç göndermeme deseni. Kod taşınmadı,
yalnız buradaki beş karara giren fikirler değerlendirildi.
