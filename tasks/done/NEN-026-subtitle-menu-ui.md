---
id: NEN-026
title: Subtitle menu UI
milestone: M3
size: M
state: done
closed: 2026-08-27
depends_on: [NEN-019, NEN-025, NEN-056]
blocks: [NEN-027]
adr: [10, 31, 35]
---

# NEN-026 — Subtitle menu UI

## Sonuç

Tek bir altyazı düğmesi, katalog projeksiyonunu şartname §8'deki gruplu yapıda
gösterir.

## Kapsam

### Yüzey (2. tur beyin fırtınası, 2026-08-27 — ADR-0031 Notlar)

Bu bölüm mimari değildir; ADR-0031 Notlar bu tür kabuk şekli kararlarını
açıkça task dosyasına havale ediyor. Referans tasarım Claude Design
`Glass Video Player` mockup'ıdır; **kabuğu alınmış, semantiği §8'de
bırakılmıştır**.

- Tek altyazı düğmesi; yüzeyi transport üzerinde **popover**
- **Sabit yükseklikli panel, kolon içi scroll.** Panel kaynak geldikçe
  büyümez; büyüme scroll alanının içinde kalır (ADR-0031 Karar 4.2)
- **İki kolon.** Kolon 1 = projeksiyonun bölümleri, kolon 2 = seçili bölümün
  girdileri. Mockup'ın üçüncü kolonu (`AI İLE ÇEVİR` · gecikme · manuel ve
  otomatik senkronizasyon) M5 · M7 · M8'dir ve **M3'te yoktur**; grid ileride
  üçüncü kolona açılacak biçimde kurulur
- **Kolon 1'deki vurgu her zaman kolon 2'nin ne gösterdiğidir — istisnasız.**
  "Hangi bölüme bakıyorum" (vurgu) ile "hangisi aktif" (işaret) iki ayrı
  gösterge
- Kolon 1 satırları, bölümün girdi sayısını taşır

### Kolon 1 — bölümler

Sıra ve varlık kuralı `nen_catalog::menu::project`'in verdiğidir; Swift yalnız
çizer, gruplama/sıra/dedup **yeniden yazılmaz**:

- `Kapalı` her zaman ve ilk; ince bir ayırıcı ile diğerlerinden ayrı
- `Kullanıcı Altyazıları` — **yalnız boş değilse**
- Tercih edilen diller (birinci, ikinci), sonra kalan diller `LanguageTag`
  sırasında — hepsi **yalnız kaynağı olan diller**
- `Dil Belirsiz` — **yalnız boş değilse**, ve hep son
- **Dil grubu başlıkları endonim** ("English", "Français", "Türkçe") —
  `nen-catalog`'un projeksiyonu yalnız `LanguageTag` döndürür (ADR-0010
  Karar 7), metni yazan bu task'tır; NEN-019'un `menu_projection_golden`
  testi §8'in **yapısını** kanıtlıyor, §8'in **görünen metnini** ilk kez
  üreten yer burasıdır
- **Bölüm seçimi grup kimliğine bağlanır, satır indeksine değil.** Tarama
  bitince `Kullanıcı Altyazıları` yoktan var olup 2. sıraya girer ve altındaki
  her dil grubu bir satır kayar; Karar 4.2'nin en somut kırılma noktası budur

### `Kapalı`

- Tıklanınca altyazı **anında kapanır** ve kolon 2'yi devralır: kolon 2 sessiz
  bir `Altyazılar kapalı.` gösterir
- Kullanıcı kapalıyken bir dile tıklarsa kolon 2 o dilin girdilerini gösterir,
  `Kapalı` vurguyu kaybeder ama **aktif işaretini korur** — ekranda altyazı
  belirmez, yalnız bakınılır

### Kolon 2 — girdiler

- Satır: birincil metin = girdinin etiketi; ikincil satır = origin rozeti
  `Gömülü` · `OpenSubtitles` · `AI` — soluk
- **Hatalı kaynak (ADR-0031 Karar 5):** kataloğa girmiş ama kullanılamayan
  kaynak menüde **kalır**, soluk ve **seçilemez** gösterilir, ikincil satırında
  kapalı kümeden kısa sebep etiketi taşır — `okunamadı` · `biçim hatalı`. Küme
  tam olarak bu iki elemandır (ADR-0035 Karar 2); `çok büyük` dosya kataloğa
  hiç girmediği için etiketi de yoktur (ADR-0035 Karar 1)
- **"Soluk" iki ayrı şeydir, iki ayrı mekanizmayla yazılır.** Proje
  `soluk + seçilemez`i kusurlu kaynak için harcamıştır; kullanıcının bilerek
  verdiği karar aynı dille gösterilirse öğrenilen işaret bozulur.
  Bilgi metni → **ikincil renk**; kusurlu satır → **opaklık** düşük ve
  tıklanamaz
- `Bu dil için altyazı yok.` satırı **yazılmaz** — kolon 1 yalnız dolu
  bölümleri listelediği için üretilemez (ADR-0035 Karar 3'ün mantığı)
- Kolon 2'nin boş halleri üç ayrı, üçü de üretilebilir:

  | Durum | Kolon 2 |
  |---|---|
  | `Kapalı` seçili, başka kaynak **var** | `Altyazılar kapalı.` |
  | Hiç kaynak yok (kolon 1'de yalnız `Kapalı`) | `Bu medya için altyazı bulunamadı.` |
  | Hiç kaynak yok **ve tarama sürüyor** | tarama işareti |

  İkinci hal birinciyi yener.

### Davranış

- **Tarama beklenmez (ADR-0031 Karar 4):** menü açılır açılmaz `Kapalı` ve
  gömülü track'leri gösterir, sidecar bulundukça bölüm büyür, tarama sürerken
  bunu belirten bir işaret bulunur. Liste büyürken: seçili kaynak değişmez ·
  odak ve scroll sıfırlanmaz · otomatik seçim **yeniden tetiklenmez**
- **Sidecar keşfi asenkron bir faza alınır** — `openMedia` taramayı beklemez.
  "Tarama sürüyor" işareti ancak böyle gerçek bir durum olur
- **Tercih M3'te sistem dilinden tohumlanır** (`Locale.preferredLanguages`);
  `NEN-037` geldiğinde ayar bu tohumun yerine geçer. İkincil tercih M3'te yok
- **Seçim:** gömülü track seçimi motora iner (`select_track`); `Kapalı` →
  `select_track(nil)`. Harici belge (kullanıcı dosyası / sidecar) seçimi yalnız
  durum olarak kaydedilir — ekranda görünmesi `NEN-027`
- Seçim → o anki gösterilen kaynak **ve** AI çeviri komutunun kaynağı olur

## YAPILMAYACAK

- Render → NEN-027
- "AI ile çevir" komutu → M5
- Gecikme / manuel / otomatik senkronizasyon → M7, M8
- Ayrı "AI subtitle mode" — **yasak**
- Kaynak seçiminin çeviri başlatması — **yasak** (§9)
- Aynı listenin menü bar'a ikinci kez yansıtılması — M3'te tek yüzey
- Güvenlik kapısından dönen dosyanın menüde görünmesi — **yasak**; o dosya
  kaynak olmadı (ADR-0031 Karar 5, ADR-0035 Karar 1, NEN-025). Boyut sınırını
  aşan dosya da bu gruptadır
- Sebep etiketinde teknik detay veya dosya yolu — **yasak**
- `AIOSTREAMS` gibi bir kaynak rozeti — CLAUDE.md non-goal listesinde
  "Stremio subtitle add-on" var; o yön istenirse kendi ADR'siyle
- Gruplama, sıra, dedup veya "yalnız doluysa göster" kuralının Swift'te ikinci
  kez yazılması — hepsi `nen-catalog`'un ve golden'larının

## Kanıt (DoD)

- [x] Şartname §8 örneğiyle **görünen metin dahil** (endonim başlıklar,
      Türkçe chrome) eşleşen ekran görüntüsü — M2'nin projeksiyon golden'ı bu
      metni üretmiyordu, ilk gerçek kanıt burada
- [x] Aynı kaynak menüde iki kez görünmüyor
- [x] `Kapalı` her koşulda mevcut
- [x] Boş bölüm hiç çizilmiyor — `Kullanıcı Altyazıları` ve `Dil Belirsiz`
      yalnız doluyken görünüyor
- [x] `Kapalı` altyazıyı kapatıyor ve kolon 2 `Altyazılar kapalı.` diyor;
      kapalıyken bir dile bakmak altyazıyı **açmıyor**
- [x] Kolon 2'nin üç boş hali doğru mesajı seçiyor (kaynaksız hal `Kapalı`
      halini yeniyor)
- [x] Kaynak seçmek hiçbir çeviri/indirme işi **başlatmıyor** (log/çağrı kanıtı)
- [x] Menü, sidecar taraması sürerken açılıyor ve gömülü track'ler seçilebiliyor
- [x] Menü açıkken yeni kaynak eklendiğinde seçili kaynak, odak ve scroll
      değişmiyor (checklist)
- [x] `Kullanıcı Altyazıları` sonradan doğup araya girdiğinde bakılan bölüm ve
      seçim değişmiyor
- [x] Oynatma başladıktan sonra bulunan tercih edilen dildeki sidecar otomatik
      **seçilmiyor**
- [x] Bozuk `.srt` menüde soluk ve seçilemez, sebep etiketi görünüyor
- [x] Negatif: güvenlik kapısından dönen dosya menüde **hiç görünmüyor** —
      symlink · traversal · dizin/FIFO · **boyut sınırını aşan dosya**
- [x] Ekran görüntüsü `fixtures/` medyasıyla üretilmiş, dosya yolu görünmüyor

## Kanıt kaydı

**Kapandı 2026-08-27.** Menü çekirdeğin projeksiyonunu çiziyor; gruplama, sıra,
dedup ve "boş grup görünmez" kuralı `nen-catalog`'da kaldı, Swift yalnız
**görünen metni** üretiyor — §8'in metninin ilk kez var olduğu yer burası.

### Ne yazıldı

- **Çekirdek** (`nen_app::subtitles`): kaynağa ömür boyu sabit, opak bir `u32`
  token veren kayıt; `nen_catalog::project`'in girdilerini bu katmandaki kusur
  haritasıyla birleştiren `menu()`; `auto_selection` ve `embedded::track_of`
  sarmalayıcıları. Yeni gruplama mantığı **yazılmadı**.
- **FFI** (`nen_ffi::subtitles`): `FfiMenuGroup` · `FfiMenuEntry` ·
  `FfiMenuSection` · `add_embedded` · `menu` · `auto_selection` ·
  `embedded_track_of` · `is_usable`. Geçit saf kaldı.
- **Kabuk**: asenkron sidecar taraması, `.ready`'de gömülü track kataloglama,
  tek-atış otomatik seçim, `SubtitleMenuView` (iki kolon, sabit yükseklik),
  `SubtitleMenuPresentation` (endonim + Türkçe chrome + rozet + sebep etiketi).
- **Fixture**: `fixtures/media/menu-clip.mkv` + reçetesi — `eng`/`fre`/`tur`
  başlıklı üç track ve dilsiz bir dördüncü; §8'in örneğini üretebilen ilk medya.

### Token neden kimlik değil

`SubtitleSourceId`'nin anahtarı bir yol digest'i (K23 #8) ve Swift'in üretilmiş
struct'ı her alanını `String(reflecting:)` ile basar — `NEN-023`'te ölçülmüş
sınır. Sayaç basacak bir şey taşımıyor, ve liste büyürken değişmediği için
kullanıcının işaret ettiği satır yerinden oynamıyor.

### Kaç test

Rust **491 → 513**, Swift **62 → 86**. Yeni dış bağımlılık yok, `deny.toml`
değişmedi. `nen-app` `nen-catalog`'u re-export etti (`nen-ffi` ADR-0006 gereği
yalnız `nen-app`'e bakabiliyor; kenar eklenmedi).

### Negatif kontroller

| # | Kusur | Kırmızı |
|---|---|---|
| K1a | Otomatik seçim `refreshSubtitleMenu`'ye taşındı (guard yerinde) | 1 test — tek atış boş katalogda harcanıyor, tercih edilen track hiç açılmıyor |
| K1b | Tarama sonrası ikinci deneme + guard kaldırıldı | 2 test — geç gelen sidecar açılıyor, ve track iki kez seçiliyor |
| K1c | Aynı ikinci çağrı yeri, guard **yerinde** | **yeşil** — guard'ın işi yaptığının kanıtı |
| K2 | Kusurlu satırın seçilemezliği kaldırıldı | 1 test |
| K3 | `Kapalı` kolon 2'yi devralmıyor | 1 test |
| K4 | Token upsert'te yeniden numaralanıyor | 1 test |
| K4b | Token menüdeki **sıradan** türetiliyor (alternatif tasarım) | 1 test |
| K5 | Kusur join'i düşürüldü | 1 test |
| K6 | Menü her tazelendiğinde bakılan bölüm sıfırlanıyor | 1 test |
| K7 | Kolon 2'nin boş hali önceliği ters | 1 test |

**Kontrollerin kendisi iki kez düzeltildi.** İlk turda K3 **0 kırmızı** verdi:
test `Kapalı`'ya basmadan önce zaten `Kapalı`'ya bakıyordu, yani ölçtüğünü
sandığı şeyi ölçmüyordu. Aynı şekilde geç-sidecar testinin sidecar'ı İngilizce
metindi ve Türkçe tercihle **hiç eşleşmiyordu**; fixture Türkçe metne çevrildi
ve tercih eşleşir hale geldi. Üçüncüsü: tek-atış guard'ını tek başına kaldırmak
hiçbir testi kırmıyor, çünkü özelliği bugün çağrı yerinin kendisi tutuyor —
guard ancak ikinci bir çağrı yeri eklendiğinde taşıyıcı oluyor (K1b/K1c).

### Elle koşuda bulunan üç kusur

`.app` üzerinde ölçüldü ve düzeltildi: (1) otomatik seçim sonrası kolon 2
`Altyazılar kapalı.` diyordu — ekranda altyazı varken; seçim artık bakılan
grubu da taşıyor. (2) Panel video üzerinde okunmuyordu; opak zemin kondu.
(3) Sebep etiketi çift kararma yüzünden panelin en soluk yazısıydı; artık
birincil renkte. Ayrıca panel 300 → 260 pt.

### Kapsanmayan, açıkça yazılıyor

**Seçilen gömülü track tam ekranda ekrana çizilmedi → `NEN-060`.** Seçimin
kendisi gerçek motorla ölçüldü ve doğru
(`MenuFixtureTests.everySubtitleTrackCanBeSelectedAndReadsBack`: dört track'in
her biri seçiliyor ve `ff-index` geri okunuyor). Çizim **pencere modunda** bir
kez çalıştı, tam ekranda hiçbir denemede çalışmadı. Tek değişken denendi, kök
neden ölçülmedi; `NEN-058`'in açıldığı andaki gibi gözlem olarak ayrıldı.

**Güvenlik kapıları otomatik, elle değil.** Symlink sidecar'ın menüde hiç
görünmediği gerçek bir symlink üzerinde koşuyor
(`SubtitleMenuTests.aRefusedFileIsNotInTheMenu`); traversal, dizin/FIFO ve
boyut sınırı `nen-app`'in gerçek dosya sistemli paketinde. `.app` üzerindeki
elle tekrarı `NEN-025`'in kaydında.

**Yol üstünde bulunan test iskeleti kusuru düzeltildi.** `TempFixture`'ın dizin
adı yalnız etiket + pid'di; swift-testing testleri paralel koşturduğu için aynı
etiketi kullanan iki test aynı dizini paylaşıyor ve `init`'teki `removeItem`
kardeşinin fixture'larını yarı yolda siliyordu. Sayaç eklendi. Ayrıca
`PlayerModel`'in tercihi `Locale.preferredLanguages`'tan **statik** okuması
otomatik seçim testlerini makinenin sistem diline bağlıyordu; enjekte edildi.

Tam kanıt: `evidence/M3/NEN-026-checklist.md` · `evidence/M3/NEN-026-menu.jpg` ·
`evidence/M3/NEN-026-defective-source.jpg`.
