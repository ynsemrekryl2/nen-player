# NEN-026 — altyazı menüsü, elle acceptance

Koşum: **2026-08-27** · macOS · `platforms/macos/.build/NenPlayer.app` (ad-hoc
imzalı geliştirme paketi) · libmpv 2.5.0 (Homebrew, dinamik).
Sistem dili koşu sırasında **`tr-TR`** — bu yüzden tercih edilen dil grubu
`Türkçe` ve otomatik seçim onu açıyor.

Medya **yalnız fixture** (ADR-0031 Karar 2): `fixtures/media/menu-clip.mkv` —
4 subtitle track (`eng`/"English" · `fre`/"Français" · `tur`/"Türkçe" ·
dilsiz ve başlıksız). Üreteci: `fixtures/media/menu-clip.ffmpeg.txt`.

Bozuk sidecar, koşudan önce yanına kondu ve sonra kaldırıldı:

```bash
cp fixtures/subtitles/malformed/end-before-start.srt fixtures/media/menu-clip.srt
```

Ekran görüntüleri: `NEN-026-menu.jpg` (menü, §8 karşılaştırması) ·
`NEN-026-defective-source.jpg` (kusurlu kaynak satırı). İkisi de tam ekran
oynatıcıdan; ekranda tam yol, başka uygulama veya özel dosya adı yok.

---

## Adımlar

| # | Adım | Beklenen | Sonuç |
|---|---|---|---|
| 1 | Medyayı aç | Video hemen oynuyor | ✅ |
| 2 | Transport'taki altyazı düğmesi | Panel açılıyor, iki kolon, sabit yükseklik | ✅ |
| 3 | Kolon 1'in sırası | `Kapalı` (ayırıcıyla ayrı) → `Kullanıcı Altyazıları` → `Türkçe` (tercih) → `English` → `Français` → `Dil Belirsiz` | ✅ tam bu sırada |
| 4 | Grup başlıklarının dili | **Endonim**: `English` · `Français` · `Türkçe` | ✅ UI Türkçeyken de endonim |
| 5 | Chrome'un dili | `Kapalı` · `Kullanıcı Altyazıları` · `Dil Belirsiz` Türkçe | ✅ |
| 6 | `Kullanıcı Altyazıları` grubu | Var (bozuk sidecar kataloğa girdi), sayaç `1` | ✅ |
| 7 | Kolon 2 → `Kullanıcı Altyazıları` | `menu-clip.srt` soluk, tıklanamaz, ikinci satırında `biçim hatalı` | ✅ `NEN-026-defective-source.jpg` |
| 8 | Bozuk satıra tıkla | Hiçbir şey olmuyor | ✅ seçim değişmedi |
| 9 | `Türkçe` grubuna tıkla | Kolon 2 `Türkçe` / `Gömülü` gösteriyor | ✅ |
| 10 | `Dil Belirsiz` grubuna tıkla | Tek satır, etiketi `Adsız parça`, rozeti `Gömülü` | ✅ |
| 11 | `Français` seç | Aktif işareti `Türkçe`'den `Français`'ye geçiyor, iki kolonda da | ✅ |
| 12 | `Kapalı`'ya bas | Aktif işareti `Kapalı`'ya geçiyor · kolon 2 `Altyazılar kapalı.` diyor | ✅ |
| 13 | Otomatik seçim | Oynatma başlarken tercih edilen dildeki gömülü track seçiliyor | ✅ `Türkçe` işaretli açıldı |
| 14 | Kaynak seçmek indirme/çeviri başlatmıyor | Ağ yok, disk yazımı yok | ✅ kod yolunda böyle bir çağrı yok; `nen-app` tek dosya okuyor |
| 15 | Aynı kaynak iki kez görünmüyor | Her grup sayacı `1` | ✅ |
| 16 | Ekran görüntüsü | Fixture medya, dosya yolu görünmüyor | ✅ |

## Koşuda bulunan iki kusur — ikisi de düzeltildi

**1. Kolon 2 yalan söylüyordu.** Otomatik seçim `Türkçe`'yi açtığında menü
`Kapalı`'ya bakıyor kalıyordu ve kolon 2 `Altyazılar kapalı.` yazıyordu —
ekranda altyazı varken. Kural "kolon 1'deki vurgu her zaman kolon 2'nin
gösterdiğidir" olduğu için doğru davranış seçimin bakılan grubu da taşıması.
`selectSubtitle` artık satırın grubuna geçiyor.

**2. Panel video üzerinde okunmuyordu.** Popover'ın kendi materyali video
yüzeyinin üstünde yeterince örtmüyordu; parlak bir karede grup adları
okunamıyordu. Panele açık bir opak zemin kondu.

**3. Sebep etiketi panelin en soluk yazısıydı.** Kusurlu satır hem %40 opaklık
hem ikincil renk alıyordu, yani çift kararma. `biçim hatalı` satırın var olma
sebebi olduğu için artık birincil renkte yazılıyor ve satır opaklığı 0.55.

Ayrıca panel yüksekliği 300 → **260** pt: M3'ün altı satırı 300'de panelin
yarısını boş bırakıyordu. Sabit kalması ADR-0031 Karar 4.2'nin gereği,
yalnız ölçüsü verilere yaklaştırıldı.

## Kapsanmayan / ayrılan

**Seçilen gömülü track tam ekranda ekrana çizilmedi → `NEN-060`.** Seçimin
kendisi çalışıyor: `MenuFixtureTests.everySubtitleTrackCanBeSelectedAndReadsBack`
dört track'in her birini seçip `ff-index`'i geri okuyor, ve menüde aktif işareti
yerine geçiyor. Çizim de en az bir kez çalıştı — **pencere modunda** otomatik
seçilen Türkçe track ekranda göründü. Fakat tam ekranda hiçbir denemede altyazı
belirmedi. Tek değişken denendi, kök neden **ölçülmedi**; bu yüzden burada
kapatılmıyor, `NEN-060` olarak ayrıldı (Kural 5).

**Güvenlik kapıları otomatik testlerle kanıtlandı, elle değil.** Symlink
sidecar'ın menüde hiç görünmediği `SubtitleMenuTests.aRefusedFileIsNotInTheMenu`
ile gerçek bir symlink üzerinde koşuyor; traversal · dizin/FIFO · boyut sınırı
`nen-app`'in `subtitle_file_gates.rs` paketinde gerçek dosya sistemiyle. `.app`
üzerinde elle tekrarı `NEN-025`'in checklist'inde zaten var.

## Temizlik

`fixtures/media/menu-clip.srt` kaldırıldı; depoya girmedi.
