---
adr: 0041
title: Sidecar keşfi medyanın dizinini bir kez listeler
status: accepted
milestone: M3
tasks: [NEN-075]
date: 2026-09-07
---

# ADR-0041 — Sidecar keşfi medyanın dizinini bir kez listeler

## Durum

`accepted`

## Bağlam

Bugün sidecar taraması **tek bir yola** bakıyor
(`nen-app::subtitle_files::sidecar_of`):

```rust
pub fn sidecar_of(media: &Path) -> Option<PathBuf> {
    media.file_name()?;
    Some(media.with_extension("srt"))
}
```

Yani `Film.mkv` için aranan tek dosya `Film.srt`. Bu, `NEN-025`'in bilinçli
kapsam kararıydı ve ADR-0034 Karar 3'te şöyle yazılı:

> "Uygulama yalnız kullanıcının gösterdiği yere bakar. Açılan medyanın dizini
> dışında hiçbir yer okunmaz; sidecar taraması aynı basename'li tek dosyaya
> bakar, dizin listelemez, özyinelemeli aramaz."

`NEN-057` uygulanırken bu kapsamın bir maliyeti ölçüldü ve `NEN-075` olarak
dosyalandı: **kullanıcı kütüphanelerinde en yaygın adlandırma olan
`Film.tr.srt` / `Film.en.srt` bu taramadan hiç geçmiyor.** Kullanıcı onları
yalnız `⇧⌘O` ile, elle gösterebiliyor — oysa dosya zaten medyanın yanında
duruyor ve M3'ün kullanıcı hikâyesi tam olarak şunu vaat ediyor
(`docs/milestones/M3-macos-slice.md`):

> "Altyazı düğmesine bastığımda gömülü track'leri **ve dosyanın yanındaki
> `.srt`**'yi gruplu bir listede görüyorum."

Dil hattında eksik bir şey yok: `NEN-057`'nin `from_file_name` ipucu zaten
ortak yükleme yolunda (`subtitle_files::load`), yani sidecar olarak bulunan bir
`Film.tr.srt`'nin dili bugünkü kodla **hâlihazırda** doğru okunur. Eksik olan
tek şey **aday kümesi**: `with_extension("srt")` böyle bir adı hiç üretemiyor.

Dil alt-uzantılı adları görmenin bilinen tek genel yolu, dizini listelemektir.
Karar verilmezse `NEN-075` kapanamaz ve M3 kapanmaz.

## Karar

Sidecar keşfi **medyanın kendi dizinini bir kez, özyinelemesiz listeleyecektir.**

1. Aday, adı `<basename>.` önekiyle başlayan ve `.srt` ile biten her girdidir
   (`Film.mkv` → `Film.srt`, `Film.tr.srt`, `Film.en.sdh.srt`,
   `Film.backup.srt`). Önek karşılaştırması birebir, uzantı karşılaştırması
   ASCII case-insensitive'dir.
2. **Yalnız o dizin okunur.** Alt dizin yok, özyineleme yok, üst dizin yok.
   Medyanın dizini `admit()`'in `root` argümanı olarak kalır, yani traversal
   kapısı aday kümesini o dizine hapsetmeye devam eder.
3. **`security-policy.md` §4 kapıları hiç değişmez.** Her aday, tek dosya
   taramasında olduğu gibi `admit()`'in symlink, regular-file, traversal ve
   boyut kapılarından **teker teker** geçer. Büyüyen tek şey aday kümesidir;
   kapıların kendisi ne gevşetilir ne atlanır.
4. Aday sayısı **16** ile sınırlıdır (`MAX_SIDECAR_CANDIDATES`). Tam basename
   eşleşmesi (`Film.srt`) sıralamada **her zaman ilk** gelir ve dolayısıyla
   sınırdan her zaman kurtulur; kalan adaylar dosya adına göre sıralanır ve
   sınırın ötesindekiler sessizce atlanır.
5. Dizin listelenemezse (`read_dir` hatası) tarama **tam basename eşleşmesine**
   düşer. Yeni yüzey hiçbir durumda bugünkünden dar olamaz.
6. Dil, ayrı bir tablo veya ayrı bir kapı ile değil, var olan
   `nen_subtitle::language::from_file_name` ipucuyla okunur. Adından dil
   çözülemeyen bir aday (`Film.backup.srt`) reddedilmez; dili içerikten gelir —
   `⇧⌘O` ile elle yüklenen dosyanın bugünkü davranışının aynısı.

ADR-0034 Karar 3'ün "dizin listelemez" cümlesi bu ADR ile düşer. ADR-0034
**supersede edilmez**: gövdesi olduğu gibi kalır, `Notlar`'a bu ADR'ye işaret
eden bir madde eklenir (ADR-0001'in açık istisnası, `NEN-056`/ADR-0035'te
kurulan precedent).

## Gerekçe

**Aday kümesi büyüdü, saldırı yüzeyi büyümedi.** Tehdit modelinde belirleyici
olan, yolu kimin türettiği ve o yolun hangi kapılardan geçtiğidir. Her iki soru
da değişmiyor: yolu yine uygulama türetiyor (kullanıcı değil) ve yine aynı dört
kapıdan geçiyor. `NEN-025`'in negatif testleri — symlink, dizin, FIFO,
traversal, boyut — yeni yüzey üzerinde **aynen tekrarlanır**; bu, `NEN-075`'in
DoD'unun üçüncü maddesidir.

**Sayının kendisi zaten sınırlıydı, artık açıkça sınırlı.** Bugün tarama 1
dosya okuyor. Karardan sonra en fazla 16 okuyor. Aradaki fark ölçülebilir ve
sabittir; patolojik bir dizin (binlerce `Film.*.srt`) medya açılışını aç
bırakamaz. Tarama zaten arka planda koşuyor (`Task.detached`, ADR-0031 Karar 4)
ve oynatmayı bloklamıyor.

**16 gerçek kütüphaneye göre seçildi.** Bir medyanın yanında 16'dan fazla
`.srt` bulunması pratikte görülmüyor; tipik kullanım bir ile üç dosya arası.
Sınır bir performans bütçesi değil, patolojik girdiye karşı bir tavan.

**Tam eşleşmenin sınırdan kurtulması bir regresyon koruması.** Bugün bulunan
`Film.srt`, yarın 16 adaylık bir dizinde alfabetik sıralama yüzünden
düşebilirdi. Sıralamada birinci olması, bu kararın var olan davranışı hiçbir
koşulda bozmamasını **yapısal** hale getirir.

**İkinci bir dil tablosu açılmadı.** Bu, projenin iki kez verdiği kararla
tutarlı (`NEN-026` menü başlıkları, `NEN-057` dosya adı ipucu): dil bilgisi tek
yerde durur. Alt-uzantının dil olup olmadığına aday **kabulünde** karar
verilmiyor; bu soruyu `from_file_name` yükleme sırasında zaten cevaplıyor.

**ADR-0034'ün ölçümü bu kararı ucuzlatıyor.** Sandbox kaldırıldığı için
`read_dir` ek bir izin istemi doğurmuyor — sandbox altında bu ölçülmüş ve
`EPERM` vermişti (`evidence/M3/NEN-025-sandbox-measurement.md`).

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Sabit dil kodu tablosu — `en`, `tr`, `pt-br`… için sabit sayıda `stat` | İkinci bir ISO tablosu açar; proje bunu `NEN-026` ve `NEN-057`'de iki kez bilinçle reddetti. `pt-BR`, `en.sdh`, `tr.forced` gibi bileşikler kombinatoryal olarak patlar ve her yeni dil elle eklenmek zorunda kalır — yani tablonun kapsamadığı dil, sessizce görünmez kalır. Dizin listelemesinin çözdüğü problemi kısmen çözüp bakım borcunu kalıcı hale getirir |
| Bugünkü hâli koru, sınırı belgele | En ucuzu, ama kullanıcı kütüphanelerindeki en yaygın adlandırmayı ürüne görünmez bırakır ve M3 hikâyesinin "dosyanın yanındaki `.srt`" maddesini yarım bırakır. Kullanıcı her seferinde zaten yanında duran dosyayı ikinci kez göstermek zorunda kalır |
| Dizini listele, ama sınır koyma | Tarama süresi dizinin içeriğine bağlı kalır. Aday başına 10 MiB'a kadar okuma + parse yapıldığı için patolojik bir dizin arka plan görevini uzun süre meşgul edebilir. Sınır ucuz, sınırsızlığın maliyeti ölçülemez |
| Yalnız dili çözülen adayları kabul et | `Film.sdh.srt` gibi, kullanıcının gerçekten medyanın yanına koyduğu ve `⇧⌘O` ile bugün sorunsuz yüklenen bir dosyayı sessizce görünmez yapardı — düzeltilen kusurun daha küçük bir kopyası. Ayrıca aday kabulüne, `from_file_name`'in zaten yükleme sırasında cevapladığı bir soruyu ikinci kez sordururdu |
| Alt dizinleri de tara (`Subs/`, `Subtitles/`) | Özyineleme yasağını gerçekten deler ve `root` kapsamını genişletir. Bu kararın çözdüğü problemin parçası değil; ihtiyaç çıkarsa kendi ADR'sini ister |

## Sonuçlar

**Olumlu:** `Film.tr.srt` medya açılır açılmaz menüde, doğru dil grubunda
görünür. Çok dilli sidecar setleri (`Film.tr.srt` + `Film.en.srt`) tek açılışta
kataloğa girer. `NEN-057`'nin dil ipucu ilk kez otomatik yolda da işe yarar.
Ürün kodunda ikinci bir yükleme yolu açılmaz — adaylar var olan `add_file`'dan
geçer.

**Olumsuz / kabul edilen maliyet:** uygulama artık medyanın dizinini
listeliyor, yani ADR-0034'ün "yalnız bilinen tek yola `stat`" sadeliği düştü.
Bu, sandbox'a dönmenin maliyetini artırır (bkz. Geri dönüş maliyeti). 16'dan
fazla eşleşen dosya bulunduran bir dizinde, ada göre sıralamada 16'dan sonra
kalan adaylar **sessizce** atlanır — kullanıcı bir bildirim görmez (ADR-0031
Karar 5 ile tutarlı). Tarama en kötü durumda 1 yerine 16 dosya okur.

**Geri dönüş maliyeti:** **ucuz.** `sidecars_of`'un gövdesi tek dosyaya
dönerse davranış bugünküne geri gelir; çağrı yerleri ve kapılar değişmez.

## İlgili task'lar

`NEN-075`, `NEN-025`, `NEN-057`

## Notlar

<!-- karar sonrası gözlemler -->
