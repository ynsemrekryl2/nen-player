---
adr: 0032
title: Konteynerin ISO 639-2 dil kodları tek kanonik etikete indirgenir
status: accepted
milestone: M3
tasks: [NEN-023]
date: 2026-08-26
---

# ADR-0032 — Konteynerin ISO 639-2 dil kodları tek kanonik etikete indirgenir

## Durum

`accepted`

## Bağlam

`NEN-023` gömülü track'leri kataloğa bağlarken **ölçüm** yapıldı: libmpv,
Matroska'nın yazdığı dil kodunu olduğu gibi veriyor ve Matroska ISO **639-2**
kullanıyor.

`fixtures/media/contract-clip.mkv` ve `bitmap-subs-clip.mkv` üzerinde
(libmpv 2.5.0, 2026-08-26):

```
lang=eng   lang=tur   lang=fre
```

`LanguageTag::parse` 2–3 harfli primary subtag'i geçerli sayıyor — ADR-0010
Karar 5'in bilinçli bir kararı, çünkü bazı dillerin iki harfli kodu yok
(`fil`, `haw`). Dolayısıyla `eng` **hatasız** ayrıştırılıyor ve `en`'den
**farklı** bir etiket oluyor.

Sonucu üç yerde birden kırılıyor:

1. **Menü gruplaması.** Kullanıcının `.srt` dosyası `en` etiketli (NEN-020
   iki harfli kanonik kod üretiyor), gömülü track `eng`. Menüde **iki ayrı
   İngilizce grubu** çıkıyor.
2. **Tercih eşleşmesi.** Tercihi `en` olan kullanıcı için hiçbir gömülü track
   eşleşmiyor; ADR-0010 Karar 9'un otomatik seçimi gömülü katmanı hiç göremiyor.
3. **İkinci bir yazım ekseni.** ISO 639-2'nin kendisi bazı diller için **iki**
   kod tanımlıyor: bibliyografik (/B) ve terminolojik (/T) — `fre`/`fra`,
   `ger`/`deu`, `chi`/`zho`. Aynı dil, aynı konteyner ailesinde iki yazım.

Bu, [ADR-0030](0030-language-group-granularity.md)'un çözdüğü problemin
**aynısıdır**, yalnız başka bir eksende: orada `en` ile `en-us` iki grup
oluyordu (region ekseni), burada `en` ile `eng` iki grup oluyor (kod standardı
ekseni). ADR-0030'un kendi gerekçesi — "iki yazım tek dili iki gruba bölmemeli"
— buraya değişmeden uygulanıyor.

Karar verilmezse: gömülü altyazılar M3'ün ilk gününden itibaren yanlış grupta
görünür ve tercih edilen dilde otomatik açılmaz. Şartname §8'in menü örneği
("İngilizce → English — Gömülü") üretilemez.

## Karar

**Bir dil etiketi `LanguageTag`'e girerken kanonikleştirilir: ISO 639-2/B ve
639-2/T kodları, karşılığı olan ISO 639-1 iki harfli koda indirgenir.**

- Kanonikleştirme `nen-domain`'de, `LanguageTag`'in yanında yaşar — grubu ve
  eşleşmeyi belirleyen soru orada cevaplanıyor (ADR-0030 Karar 1).
- Tablo **iki yönlü değildir**: yalnız 639-2 → 639-1. Karşılığı olmayan bir
  639-2 kodu (`fil`, `haw`, `nds`) **olduğu gibi kalır** — düşürülmez,
  reddedilmez, tahmin edilmez.
- /B ve /T aynı hedefe iner: `fre` ve `fra` ikisi de `fr` olur.
- Kanonikleştirme `LanguageTag::parse`'ın **kendisinde** uygulanır, ayrı bir
  giriş noktasında değil.

## Gerekçe

**Neden `parse`'ın içinde, konteyner sınırında değil?** Çünkü aynı üç harfli kod
birden fazla kapıdan giriyor: konteyner metadata'sı (bu task), `.nfo` sidecar'ı
ve dosya adı beyanı (ADR-0009'un katmanları), ileride OpenSubtitles yanıtı
(639-2 kullanır). Kanonikleştirmeyi kapılara dağıtmak, ADR-0030'un düzelttiği
hatanın birebir tekrarıdır: aynı soruyu birden fazla yerde sormak, ve biri
unutulduğunda sessizce iki grup üretmek. `parse` tek kapı.

**Neden tahmin yok?** `LanguageTag::parse` bugün geçersiz girdiyi `Err`
döndürüyor, "sessizce onarılmış değer" değil (ADR-0010). Bilinmeyen bir üç
harfli kodu iki harfe *zorlamak* aynı ilkeyi çiğnerdi. Tablo eksikse davranış
bugünküyle aynı kalır — yani bu karar hiçbir dili bugünkünden kötü duruma
düşürmüyor.

**Neden tam tablo?** Kısmi tablo, kapsamadığı her dil için sessizce iki grup
üretir ve bunu yalnız o dili konuşan kullanıcı fark eder. ISO 639-1'in tamamı
184 dildir; tablo veri, mantık değil ve tek yerde duruyor.

**Ölçüm.** Kararın gerekliliği tahmin değil: yukarıdaki üç `lang=` değeri bu
makinede, gerçek fixture'lardan, gerçek libmpv ile okundu.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Kanonikleştirmeyi konteyner adapter'ında yapmak | Aynı kod `.nfo`, dosya adı ve OpenSubtitles kapılarından da giriyor; her kapı kendi kopyasını taşırsa ADR-0030'un düzelttiği "aynı soru birden çok yerde" hatası geri gelir |
| Kanonikleştirmeyi **gruplama anında** yapmak (`primary_tag` içinde) | Etiketin kendisi `eng` kalır; menüde gösterilen, loglanan ve karşılaştırılan değerler ayrışır. ADR-0030 Karar 3 tam tersini şart koşuyor: kaynağın kendi etiketi korunur, ayrım yalnız gruplamada |
| Yalnız /B ↔ /T alias'larını çözmek, 639-1'e inmemek | Asıl kırılma `eng` ↔ `en`; alias'lar ikincil. Sorunun büyük kısmı çözülmeden kalır |
| Kısmi tablo (yalnız yaygın diller) | Kapsanmayan her dil sessizce iki grup üretir; hatayı yalnız o dilin kullanıcısı görür ve bildiremez |
| Bilinmeyen üç harfli kodu reddetmek (`Err`) | `fil`, `haw`, `nds` gibi diller için geçerli ve tek doğru etiket odur; reddetmek o altyazıyı kataloğun dışına atardı |
| Üç harfli kodu iki harfe kesmek (`eng` → `en`) | `tur` → `tu` (geçersiz), `fre` → `fr` (doğru ama şans eseri). Doğruluğu tesadüfe bağlı bir kural |

## Sonuçlar

**Olumlu:**

- Gömülü track'ler, kullanıcı dosyaları ve NEN-020'nin tespit ettiği diller
  **tek grup** oluşturuyor; §8'in menü örneği üretilebiliyor.
- Tercih eşleşmesi gömülü katmanı görüyor — ADR-0010 Karar 9'un otomatik seçimi
  M3'te fiilen çalışıyor.
- ADR-0009'un `.nfo` ve dosya adı katmanları aynı düzeltmeden ücretsiz
  yararlanıyor.
- `fre`/`fra` ayrımı kalkıyor, yani aynı dilin iki kod standardı tek gruba
  düşüyor.

**Olumsuz / kabul edilen maliyet:**

- `nen-domain`'e ~184 satırlık bir veri tablosu giriyor. Mantık değil veri, ama
  bakımı olan bir tablo: ISO 639-1 değişirse (nadiren olur) burası güncellenir.
- `LanguageTag::parse("eng").as_str()` artık `"en"` döndürüyor — yani
  **girdiyle çıktı birebir aynı değil**. Bu zaten `parse`'ın işi (case ve
  separator normalizasyonu var), fakat bir okuyucu için sürpriz olabilir;
  doküman yorumunda açıkça yazılır.
- Karşılığı olmayan üç harfli kodlar üç harfli kalıyor, yani etiket uzayında
  iki uzunluk birlikte yaşamaya devam ediyor.

**Geri dönüş maliyeti: ucuz.** Tablo tek fonksiyonda; kaldırmak `parse`'tan bir
çağrı silmek demek. Crate sınırı, bağımlılık grafiği ve port kontratı
değişmiyor.

## İlgili task'lar

`NEN-023` · dolaylı olarak `NEN-019` (katalog), `NEN-026` (menü), `NEN-038`
(OpenSubtitles dil kodları)

## Notlar

ADR-0030 ile aynı aileden: ikisi de "bir dilin iki yazımı tek grup olmalı"
kuralının bir eksenini çözüyor. ADR-0030 region eksenini (`en` / `en-us`),
bu ADR kod standardı eksenini (`en` / `eng` / `fre`). Üçüncü bir eksen
çıkarsa (script subtag'i — `zh-Hans` / `zh-Hant`) ayrı bir ADR ister; bu ikisi
onu **çözmüyor** ve `LanguageTag::parse` script subtag'ini bugün
`TooManySubtags` ile reddediyor.
