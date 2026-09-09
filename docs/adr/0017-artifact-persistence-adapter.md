---
adr: 0017
title: Doğrulanmış artifact'lerin persistence adapter'ı
status: accepted
milestone: M5
tasks: [NEN-095, NEN-096, NEN-098]
date: 2026-09-09
---

# ADR-0017 — Doğrulanmış artifact'lerin persistence adapter'ı

## Durum

`accepted` — 2026-09-09

## Bağlam

`docs/product-spec.md` §11: "Final çeviri yalnız bellekte tutulmamalıdır…
Başlangıç için SQLite metadata index + content-addressed artifact files
**değerlendirilebilir**. Seçim ADR ile gerekçelendirilmelidir."

Bugün bu bir **aday**dır, karar değil: `docs/architecture.md` → Portlar
tablosunda `Persistence` satırı "aday: SQLite + CAS" diyor ve
`docs/DECISIONS.md` → "Ertelenmiş kararlar" bu satırı ADR-0017'ye bağlıyor.
`core/crates/nen-persist/src/lib.rs` üç satırlık boş iskelet.

Karar gerektiren noktalar:

1. **Index ve içerik nerede durur?** Tek bir veritabanı mı, yoksa metadata
   index + ayrı içerik dosyaları mı? İkincisi §11'in önerisi ama şart değil.
2. **Atomik commit nasıl sağlanır?** Yarım yazılmış bir WebVTT dosyası M5'in
   "yarım/progressive çıktı yayınlanmaz" kriterini doğrudan ihlal eder.
3. **Depo kökü nereden gelir?** `docs/architecture.md` "yol platformdan enjekte
   edilir" diyor — macOS'ta sandbox ve `ADR-0034`'ün dosya erişim kuralları
   burada geçerli.
4. **Offline garantisi.** Kullanıcı kararı (2026-09-08 — S4): kaydedilmiş bir
   artifact ağ olmadan açılıp oynatılır; ayrı bir "offline modu" anahtarı
   yoktur. Bu, okuma yolunun hiçbir ağ portuna dokunmamasını şart koşar.
5. **Ephemeral session ile persistent artifact ayrımı** (§11) hangi tarafta
   durur?
6. **Yeni bağımlılık maliyeti.** SQLite seçilirse `cargo deny` yüzeyi ve
   `NEN-043`'ün bundling yükü büyür mü?

## Karar

1. **Depo modeli: dosya sistemi, veritabanı yok.** Kalıcı depo iki parçadan
   oluşur — içerik adresli artifact dosyaları ve bunların üstünde, **dizin
   taranarak türetilen** bir metadata index. Ayrı, elle bakımı gereken tek bir
   index dosyası (`index.json` gibi) yok: index bir sorgu katmanıdır, kaynak
   değil. Bunun sonucu, index bir şey diyor ama diskte içerik başka şey
   durumu — index'i elle senkron tutmaya çalışan bir sistemin sürekli riski —
   yapısal olarak daralır, çünkü index'in kendisi her açılışta veya sorguda
   dizinin bugünkü halinden yeniden üretilir.

2. **Bir artifact tek kanonik dosyadır.** `ValidatedSubtitleArtifact`'in
   metadata alanları (§11'in saydığı liste), normalize edilmiş cue'lar ve
   WebVTT çıktısı **tek bir serileştirilmiş dosyada** birlikte tutulur.
   Dosyanın adresi kendi tam içeriğinin `blake3` hash'idir (`blake3` ADR-0007
   ile zaten kabul edilmiş bir workspace bağımlılığı; serileştirme
   `serde_json` ile — o da zaten workspace'te). **Yeni dış bağımlılık
   getirilmez.** Ayrı bir `.vtt` dosyası diske yazılmaz; kullanıcının WebVTT'yi
   dışarı aktarması M5 kapsamında yok, gerektiğinde tek dosyadan türetilebilir.
   Tek dosya olmasının kazancı: tek atomik yazım, tek hash, iki parçanın
   birbirinden ayrı düşmesi diye bir durum olmaz.

3. **Atomik commit: geçici dosyaya yaz → `rename`.** Yazım aynı hedef dizine
   benzersiz bir geçici adla yapılır, dosya `fsync` edilir, sonra hedef adrese
   `rename` ile taşınır (dizin de `fsync` edilir). Hedef adında hiçbir anda
   kısmi bir dosya görünmez — `rename` POSIX'te atomiktir. Commit sırası
   **içerik önce, index kaydı sonra**: index'te kaydı olup içeriği olmayan bir
   durum `NEN-098`'in tipli hatasıyla karşılanır (zaten planlanmış bir DoD
   maddesi); içeriği olup index'te kaydı olmayan bir dosya ise zararsız bir
   yetim (bir sonraki index taramasında görünür), hiçbir okuyucuyu yanıltmaz.

4. **Depo kökü platformdan enjekte edilir.** Core kökün ne olduğuna karar
   vermez, yalnız kökün **dışına çıkan** hiçbir adresi kabul etmez (kanonik
   yol karşılaştırması; `NEN-096`'nın negatif testi). macOS ADR-0034 gereği
   sandbox'sız dağıtıldığı için kabuk kökü `~/Library/Application
   Support/<bundle-id>/…` altında verir; bu bir ADR-0034 sonucu, bu ADR'nin
   kendisi bir yol seçmez.

5. **Ephemeral ↔ persistent sınırı: bugün yalnız tamamlanmış artifact
   kalıcı.** `NEN-094`'ün ürettiği `ValidatedSubtitleArtifact` bu depoya
   yazılan tek şeydir. `NEN-093`'ün `BlockCheckpoints`'i **ileride** (M6)
   kalıcı olacak — çeviri ortasında kapanan uygulamanın kaldığı yerden devam
   etmesi gerçek sağlayıcı maliyeti ortaya çıktığında değerli hale geliyor —
   ama bu ADR o kararı şimdiden iki koşulla bağlar: (a) checkpoint resume
   verisi **ayrı bir alanda** durur, bu ADR'nin tanımladığı artifact
   deposunun bir parçası olmaz; (b) o alan hiçbir okuma yolundan bir artifact
   gibi görünemez — `BlockCheckpoints::into_completed`'ın "her blok
   checkpoint'li değilse `CompletedBlocks` üretilmez" değişmezi (`NEN-093`)
   bunu tip düzeyinde zaten garanti ediyor ve bu garanti korunacak. **M5'te
   checkpoint yalnız bellekte kalır**; kalıcı hale getirilmesi `NEN-105`
   (M6, bu ADR'ye referansla) task'ına bırakılır.

6. **Offline garantisi (S4) depo modelinin doğal sonucu.** Dosya sistemi
   okuma/yazma hiçbir ağ portu çağırmaz; bu, tasarımdan gelen bir kısıt değil
   seçimin kendisinden gelen bir sonuç. Yine de `NEN-098` bunu iddiaya değil
   ölçüme dayandırmak için ayrı bir negatif test taşır: ağsız bir ortamda
   arama ve okuma çalışır, `HttpClient` çağrı sayacı sıfır kalır.

## Gerekçe

Kararın kalbi ölçek: bu tek kullanıcılı bir masaüstü uygulaması, ve bir
kullanıcının diskinde birikecek çeviri artifact sayısı ömür boyu birkaç yüzü
geçmez — her biri bir kaynak+hedef dil+provider kombinasyonu için bir dosya.
Bu ölçekte bir dizin taraması milisaniyeler sürer; SQLite'ın verdiği
transaction/indeks/sorgu gücü hiç kullanılmayan bir kapasitedir, ama
bedeli her zaman ödenir: yeni bir C bağımlılığı (`cargo deny` yüzeyi büyür),
`NEN-043`'ün ölçülü 48 dylib'lik Homebrew kapanışına yeni bir bağımlılık
zinciri eklenmesi riski, ve bir şema/migration disiplini ki bugün hiçbir alan
henüz kararlı değilken (cache identity `NEN-097`'de kararlaşacak) erken bir
yük olur.

Tek-dosya artifact modeli, `NEN-094`'ün zaten ürettiği tek bir tip
(`ValidatedSubtitleArtifact`) ile birebir örtüşüyor — ayrı metadata/`.vtt`
dosyaları iki taraflı tutarlılık sorunu (`NEN-098`'in "kayıt var, içerik yok"
maddesinin ikiye katlanmış hali: "metadata var, vtt yok" veya tersi) açardı,
kapsam dışı olan bir "dışa aktar" özelliği için baştan ödenen bir bedel.

Checkpoint'in M6'ya ertelenmesi, `NEN-093`'ün zaten kanıtladığı
"yarım iş asla `CompletedBlocks` olamaz" garantisini bozmadan, gerçek maliyeti
(gerçek sağlayıcı kredisi) ortaya çıkmadan karmaşıklık eklememe kararı — mock
provider'la çalışan M5'te baştan başlamanın maliyeti ölçülebilir derecede
düşük.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| SQLite (rusqlite) metadata index | §11'in adı geçen adayı, ama bu ölçekte kullanılmayan bir güç; C bağımlılığı `cargo deny` yüzeyini ve `NEN-043`'ün bundle kapanışını büyütür, şema versiyonlama/migration yükü getirir, index içerikten ayrı bir doğruluk kaynağı olur |
| redb (saf Rust gömülü DB) | C bağımlılığı yok ama yine de yeni bir dış bağımlılık ve yeni bir dosya formatı; bu ölçekte dizin taramasının çözmediği bir sorunu çözmüyor |
| Merkezi tek `index.json` dosyası | Her yazımda tüm index'in yeniden serileştirilip atomik yazılmasını gerektirir — artifact sayısı büyüdükçe yazım maliyeti O(n)'e döner; dizin taraması bunun yerine her artifact'i bağımsız bırakır |
| Metadata + ayrı `.vtt` dosyası ikilisi | İki dosyanın atomik olarak birlikte commit edilmesini ve tutarlılıklarının ayrıca doğrulanmasını gerektirir; "dışa aktar" özelliği zaten M5 kapsamında değil, kazanç bedelini karşılamıyor |
| M5'te kalıcı checkpoint implementasyonu | Gerçek maliyeti (kredi/kota kaybı) ortaya çıkmadan önce ikinci bir depo biçimi ve ikinci bir temizleme politikası M5'e girer; mock provider'la yeniden başlamanın maliyeti bugün ölçülebilir derecede düşük |

## Sonuçlar

**Olumlu:** yeni dış bağımlılık yok; artifact deposu `NEN-094`'ün tipiyle
birebir örtüşüyor; index içerikten türediği için index/içerik ayrışması
yapısal olarak daralıyor; offline garantisi (S4) tasarımın doğal sonucu,
ayrı bir kısıt olarak zorlanmıyor.

**Olumsuz / kabul edilen maliyet:** dizin taraması SQLite'ın indeksli
sorgusundan yavaş — ama bu ölçekte (birkaç yüz dosya) ölçülemeyecek kadar
küçük bir fark. Karmaşık sorgular (ör. "belirli bir provider'a ait tüm
artifact'leri filtrele") ölçekle birlikte SQLite'a geçişi gerektirebilir;
`Persistence` portu bu geçişi karar değişmeden absorbe edecek şekilde
korunuyor (`docs/architecture.md`'nin zaten yazdığı gibi).

**Geri dönüş maliyeti:** port sınırı SQLite'a geçişi izole ediyor —
`nen-persist` adapter'ı değişir, `nen-app` ve üstü dokunulmaz. Kalıcı
checkpoint (`NEN-105`) implemente edilmeden önce bu karar değişirse (ör.
resume alanı da içerik adresli depoya girsin diye) maliyet düşük, çünkü
henüz hiçbir kod bu ADR'ye dayanarak yazılmadı.

## İlgili task'lar

`NEN-095` (karar) · tüketiciler: `NEN-096`, `NEN-098`, `NEN-105` (M6)

## Notlar

Kullanıcı kararları (2026-09-09): index teknolojisi yalnız dosya sistemi
(SQLite/redb reddedildi); artifact dosya biçimi tek kanonik dosya (metadata +
cue'lar + WebVTT bir arada, ayrı `.vtt` yok); blok checkpoint'lerinin kalıcı
olması karara bağlandı ama implementasyonu M6'ya (`NEN-105`) bırakıldı.
