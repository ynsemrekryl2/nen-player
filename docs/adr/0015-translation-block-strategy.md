---
adr: 0015
title: Çeviri blok stratejisi — boyut, overlap ve belge bağlamı
status: accepted
milestone: M5
tasks: [NEN-089]
date: 2026-09-08
---

# ADR-0015 — Çeviri blok stratejisi — boyut, overlap ve belge bağlamı

## Durum

`accepted`

## Bağlam

`docs/product-spec.md` §10 üç sayıyı sabitliyor: varsayılan block size **40**,
izin verilen aralık **30–60**, varsayılan overlap **6**. Ayrıca "bütün subtitle
dokümanı için bağlam analizi" istiyor.

Sabitlenmeyen ve kod yazılmadan önce kararlaştırılması gereken şeyler:

1. **Blok sınırı nereye düşer?** Sabit cue sayısı mı, yoksa sahne/sessizlik
   boşluğu gibi bir sinyal mi sınırı kaydırabilir? Kaydırırsa blok düzeni
   belgeye bağlı olarak değişir ve `block-layout version`'ın anlamı değişir.
2. **Overlap'in sözü kimindir?** Overlap penceresindeki cue iki blokta da
   çevriliyor; sonuçta hangi bloğun çevirisi geçerli sayılır?
3. **Bağlam analizi ne üretir?** Belgenin tamamından çıkan bu şey ne kadar
   büyür, her bloğa aynen mi girer, ve provider cevabını doğrulayan tarafın
   (`NEN-091`) onunla bir işi var mıdır?
4. **Cue kimliği provider'a nasıl gösterilir?** `CueId(u32)` doğrudan mı geçer,
   yoksa blok içi bir indeks mi? Doğrulamanın "yalnız izin verilen cue ID"
   kuralı buna dayanacak.
5. **`block-layout version` nasıl türetilir?** Cache identity (§11) bunu bir
   bileşen olarak tüketiyor; blok düzeninin semantiği değiştiğinde eski cache
   kullanılamaz olmalı.

M5'in kalite ölçütü **yapısal doğruluktur** (kullanıcı kararı, 2026-09-08 — S3):
dilsel kalite çıtası gerçek model geldiğinde M6'da kapanır. Dolayısıyla bu ADR
bağlam analizinin *dilsel* faydasını iddia etmez, yalnız **deterministik ve
yeniden üretilebilir** olmasını şart koşar.

## Karar

1. **Blok sınırı** sabit cue sayısıyla belirlenir; sahne veya sessizlik gibi
   içerik sinyaline göre kaydırılmaz. Belgenin son penceresi
   `cueCount - blockSize` konumuna sabitlenir, böylece son blok da her zaman
   tam boy olur ve ayrı bir kısa-son-blok durumu oluşmaz. **Overlap** (`V`)
   blok boyutu (`B`) gibi doğrulanan bir parametredir: kapı
   `1 ≤ V < B / 2`'dir. Aralık dışı bir değer tipli hata ile reddedilir;
   varsayılan **6** kalır. Alt sınır (`V ≥ 1`) komşu bloklar arasında hiç
   bağlam kalmamasını, üst sınır (`V < B / 2`) ise aşağıdaki 2. maddenin
   sınır kuralı altında bir bloğun çıktı kümesinin boş kalmasını engeller —
   düzenli bir blokta çıktı uzunluğu tam olarak `B − V ≥ 1`'dir.
2. **Overlap'in sözü**, çağrıdan **önce** deterministik olarak bölünür: iki
   komşu bloğun overlap penceresindeki cue'lar arasındaki sınır, pencerenin
   orta noktasına (`floor` ile) sabitlenir. Her cue tam olarak bir bloğun
   çıktı kümesine (`outputCueIds`) girer; overlap'teki diğer cue'lar yalnız
   komşu bloğa **bağlam** olarak görünür ve hiçbir zaman iki kez çevrilmez.
3. **Bağlam analizi**, belge genelinde tekrar eden özel isim ve terimleri
   çıkaran, tamamen yerel ve deterministik bir çıkarımdır — provider'a gitmez.
   Kural somut ve şöyledir:
   - Her cue satırı Unicode harf/rakam dizilerine (token) bölünür; ayraç
     olmayan her karakter (kesme işareti, tire, noktalama, boşluk) token
     sınırıdır.
   - İlk karakteri `is_uppercase()` olan ve en az 2 karakter uzunluğundaki
     token'lar **aday**dır. Bir satırın **ilk** token'ı, yalnız cümle başı
     büyük harfi olabileceği için, o satırdaki geçiş "satır başı" sayılır;
     aynı satırdaki sonraki token'lar satır başı sayılmaz.
   - Bir aday **terim** olur ancak toplam geçiş sayısı **≥ 3** ve bu
     geçişlerden **en az biri satır başı dışında** ise (yalnız cümle başında
     geçen bir kelime özel isim olduğunu kanıtlamaz).
   - Terimler harf duyarlı karşılaştırılır (`Tom` ve `TOM` ayrı terimdir).
   - Kalan terimler **azalan geçiş sayısına**, eşitlikte **artan alfabetik
     sıraya** göre sıralanır ve ilk **32** terimde kesilir
     (`CONTEXT_TERM_LIMIT`).
   - Büyük/küçük harf ayrımı taşımayan yazı sistemlerinde (ör. bazı CJK
     metinleri) bu kural aday üretmez; çıktı boş kümedir. Bu, M5'in kapsamı
     dışındaki bir dilsel eksiklik değil, kuralın kendi tanımının doğal
     sonucudur ve burada açıkça kabul edilir.
   - Çıktı küçük, yapılandırılmış bir küme (terim + geçiş sayısı) olarak her
     bloğa **aynen** girer. `NEN-091`'in doğrulaması bu çıktının içeriğiyle
     ilgilenmez — çıktı model üretimi olmadığı için "provider ihlali"
     kapsamına girmez.
   - **Güvenlik:** çıkarılan terim altyazı diyaloğunun bir parçasıdır, yani
     `docs/security-policy.md` §1 (K23 #4) kapsamındadır. Terim taşıyan
     tipler (`ContextTerm`, `DocumentContext`) `nen_domain::subtitle::Cue`
     emsaliyle **elle** `Debug` uygular ve yalnız terim/blok **sayılarını**
     basar — terim metnini asla.
4. **Cue kimliği** provider'a doğrudan gerçek `CueId` değeri olarak gösterilir
   (blok içi ayrı bir indeks türetilmez). Doğrulama (`NEN-091`) bunu doğrudan
   domain tipiyle karşılaştırır, ek bir eşleme katmanı gerekmez.
5. **`block-layout version`**, `nen-translate` içinde tek bir yerde elle
   artırılan bir sabittir. Bu ADR'nin 1 ve 2. maddelerindeki kurallardan biri
   değiştiğinde sabit artırılır; cache identity (`NEN-097`/ADR-0018) bunu
   olduğu gibi tüketir.

## Gerekçe

Madde 2 (overlap sahipliğinin çağrıdan önce bölünmesi), ADR'nin kendi 2.
sorusunu ("overlap'teki cue iki blokta da çevriliyor, hangisi geçerli?")
kökünden ortadan kaldırır: cue hiçbir zaman iki kez çevrilmediği için "hangisi
kazanır" sorusu doğmaz. Aynı yaklaşım, çevrilmemiş halde bir referans
implementasyonda (Stremio AISubtitle projesi, `src/translation/blocks.ts`)
üretimde ölçülmüş biçimde çalıştı; ek fayda: overlap penceresi çift
çevrilmediği için provider maliyeti de artmaz.

Madde 1'in overlap kapısı (`1 ≤ V < B / 2`) sonradan eklenen bir doğrulamadır,
çünkü şartname overlap için yalnız bir varsayılan (6) veriyordu, blok boyutu
gibi bir izin aralığı vermiyordu. Kapı olmadan `V ≥ B` gibi bir değer, madde
2'nin orta nokta kuralıyla birleştiğinde bir bloğun çıktı kümesini tamamen
yutabilir veya negatif/sıfır uzunluklu bir pencere üretebilirdi — bu, "her
cue tam olarak bir bloğun çıktı kümesine girer" değişmezini (madde 2) kırardı.
Kapı bu sınıfı kaynağında kapatır.

Madde 3, M5'in S3 kararına (2026-09-08, yapısal doğruluk ölçütü) sıkı sıkıya
bağlıdır: bağlam analizinin dilsel faydası M5'te iddia edilmiyor, dolayısıyla
onu bir provider çağrısı yapmadan, tamamen yerel ve test edilebilir tutmak
hem determinizmi hem de Kural 8'i (gerçek provider kredisi kullanılmaz)
karşılar. Model tabanlı özet/karakter/glossary üretimi M6'ya (gerçek
provider'larla) ertelenir; bu ADR o kapıyı kapatmaz, yalnız M5'in kapsamını
netleştirir. Kuralın kendisi ("büyük harfle başlayan, ≥3 kez geçen, en az bir
kez satır başı dışında görünen token") ölçülebilir ve altyazı metninde özel
isimlerin (karakter adları, yer adları) tipik imzasını hedefler; büyük/küçük
harf ayrımı olmayan yazı sistemlerinde boş küme üretmesi kabul edilen bir
sınırdır, sessiz bir hata değildir.

Madde 4, doğrulamanın (`NEN-091`) provider cevabını doğrudan `CueId(u32)` ile
karşılaştırmasına izin verir — ayrı bir blok-içi indeks, hem gereksiz bir
çeviri katmanı hem de "hangi indeks hangi cue'ya karşılık geliyor" tipinde
yeni bir ihlal sınıfı yaratırdı.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Blok sınırının sahne/sessizlik sinyaline göre kayması | Belgeye bağlı, deterministik değil; `block-layout version`'ın anlamını belirsizleştirir (bu ADR'nin kendi bağlam bölümü) |
| Overlap'teki cue'ların iki blokta da çevrilip sonradan birleştirilmesi | "Hangi bloğun çevirisi geçerli" tartışmasını doğurur; gereksiz çift provider maliyeti |
| Overlap'in blok boyutu gibi doğrulanmadan, sınırsız bir sayı olarak bırakılması | `V ≥ B` gibi bir değer madde 2'nin orta nokta kuralıyla birleşince boş veya negatif uzunluklu çıktı kümesi üretebilir |
| Cue kimliğinin blok-içi bir indeks olarak gösterilmesi | Doğrulama gerçek `CueId` yerine ek bir eşleme katmanına muhtaç kalır |
| Bağlam analizinin bir provider çağrısı olması | M5 ölçütü yapısal doğruluk (S3); ikinci, belirsiz bir LLM çağrısı determinizm şartını karşılamaz ve Kural 8'i zorlar |
| Bağlam terimlerinin harf büyüklüğü ayrımı olmayan yazı sistemleri için ayrı bir yedek kural taşıması | M5'in kapsamı yapısal doğruluk; ikinci bir dilsel sezgi kuralı determinizmi karmaşıklaştırır ve gerçek fayda M6'da provider'larla ölçülmeden doğrulanamaz |

## Sonuçlar

**Olumlu:** blok düzeni tamamen deterministik ve golden testle doğrulanabilir;
overlap cue'ları hiç çift çevrilmez; doğrulama katmanı ek eşleme gerektirmez;
overlap kapısı boş çıktı kümesi sınıfını derleme zamanı değil ama girdi
doğrulama zamanı kapatır.

**Olumsuz / kabul edilen maliyet:** M5'teki bağlam analizinin dilsel değeri
yoktur — yalnız determinizm garantisi taşır; gerçek fayda M6'da gerçek
provider'larla ölçülecek. Büyük/küçük harf ayrımı olmayan yazı sistemlerinde
bağlam kümesi her zaman boştur.

**Geri dönüş maliyeti:** blok sınırı kuralı, overlap bölünmesi veya bağlam
çıkarım kuralı değiştirilirse `block-layout version` artırılır ve bu bileşene
bağlı tüm cache girişleri doğal olarak geçersiz kalır (ADR-0018).

## İlgili task'lar

`NEN-089` · tüketiciler: `NEN-090`, `NEN-097`, `NEN-103`

## Notlar

Karar taslağı, önceki bir projedeki (Stremio AISubtitle, TypeScript) çalışan
bir çeviri hattının karşılaştırmalı incelemesinden türetildi — bkz. o hattın
`blocks.ts` ve `structured-output.ts` dosyalarındaki overlap bölme ve
strict-schema desenleri. Kod taşınmadı, yalnız buradaki üç karara giren
fikirler değerlendirildi.

Bağlam çıkarım kuralı (madde 3) ve overlap doğrulama kapısı (madde 1), task
`NEN-089`'un kendi ön koşulunun ("ne tutulur, ne kadar büyür") ilk taslakta
eksik bıraktığı iki nokta olarak 2026-09-08'de kullanıcı kararıyla eklendi;
ADR'nin geri kalanı ilk taslaktan değişmedi.
