# M3 — macOS Vertical Slice

## Amaç

İlk gerçek ürün. Kullanıcı hikâyesi:

> "macOS'ta bir video dosyası açıyorum, hemen oynuyor. Altyazı düğmesine
> bastığımda gömülü track'leri ve dosyanın yanındaki `.srt`'yi gruplu bir
> listede görüyorum. Birini seçiyorum, altyazı ekranda beliriyor. İleri
> sarıyorum, doğru replik anında görünüyor."

## Kapsam

- `PlaybackEngine` port kontratı + capability modeli + contract test kiti
- libmpv adapter (playback + track enumeration)
- macOS SwiftUI kabuk: dosya aç, video yüzeyi, transport
- Çerçevesiz pencere kromu, okunabilir cam transport ve üst medya şeridi
- Kullanıcı altyazısı yükleme + sidecar keşfi (güvenlik kapılarıyla)
- Altyazı menüsü UI ve transport üstüne hizalı panel yüzeyi
- `SubtitleRenderer` port + libmpv injection adapter

## Kapsam dışı

- OpenSubtitles → M6
- AI çeviri → M5
- Manuel/otomatik sync → M7/M8
- Stremio handoff → M4
- Persistence/cache → M5
- Diğer platformlar → M9–M11

## Ön koşul

**Tam Xcode ve libmpv kurulu olmalıdır** (`scripts/doctor.sh M3` ile
doğrulanır). Bu makinede kapı 2026-08-25'te açıldı (Xcode 26.6 · libmpv 2.5.0);
kapı açılırken doctor'da bulunan bir yanlış pozitif `NEN-041`'e ayrıldı. M3 o
tarihten beri sürüyor.

## Çıkış kriterleri (NEN-028 acceptance)

Bu beş madde kanıtlanmadan M3 kapanmaz. Hepsi **2026-09-05**'te `NEN-028`
kabul koşusunda gerçek `.app`te gösterildi — koşum kaydı, kareler ve otomatik
dayanaklar: `evidence/M3/NEN-028-checklist.md`.

- [x] Medya, katalog taraması bitmeden oynuyor
- [x] Bozuk bir `.srt` playback'i **durdurmuyor**, yalnız o kaynağı hatalı işaretliyor
- [x] Menüde aynı kaynak iki kez görünmüyor; dili bilinmeyen kaynak
      `Dil Belirsiz` grubunda; `Kapalı` her zaman var
- [x] Symlink ve path-traversal ile verilen altyazı dosyası reddediliyor —
      symlink hem üründe hem testlerde, traversal kapının kendi seviyesinde
      (aşağıda "Symlink ve path-traversal'ın yeri")
- [x] Seek sonrası doğru cue anında görünüyor (NEN-017 benchmark'ı ile birlikte)

Ek olarak: gerçek libmpv adapter'ı, fake adapter ile **aynı** contract kitini
geçmelidir — `ContractTests.theRealAdapterPassesTheSharedContractKit` yeşil.

Kriterler kanıtlandı; M3'ün **biçimsel kapanışı** (retro + roadmap durumu) ayrı
bir karara bağlıdır ve `NEN-028` onu kapsamaz.

## Kabul senaryosu (NEN-028)

Bu senaryo beş çıkış kriterini **üründe** gösterir; her adımın karşılığı olan
kriter sağ kolonda. Otomatik dayanakları `evidence/M3/NEN-028-checklist.md`
listeler — koşu, testlerin yerine geçmez, onların yanına konur.

Medya yalnız depodaki sentetik fixture'dır (ADR-0031 Karar 2). Koşu sırasında
üretilen hiçbir dosya depoya girmez.

### Hazırlık

```bash
bash scripts/build-macos-app.sh          # .app + ad-hoc imza
WORK="$(mktemp -d)"
cp fixtures/media/menu-clip.mkv "$WORK/Nen Demo.mkv"
cp fixtures/subtitles/malformed/end-before-start.srt "$WORK/Bozuk.srt"
# elle yazılmış, üç cue'luk geçerli sidecar: 2–6 s · 8,5–11,5 s · 15–18 s
$EDITOR "$WORK/Nen Demo.srt"
# symlink kapısının ürün yüzeyi: ikinci bir medya, yanında kısayol sidecar
mkdir "$WORK/symlink"
cp fixtures/media/menu-clip.mkv "$WORK/symlink/Nen Kisayol.mkv"
cp "$WORK/Nen Demo.srt" "$WORK/symlink/gercek.srt"
ln -s "$WORK/symlink/gercek.srt" "$WORK/symlink/Nen Kisayol.srt"
open platforms/macos/.build/NenPlayer.app
```

`menu-clip.mkv` dört gömülü subtitle track taşır — `eng` / `fre` / `tur` ve
dili de başlığı da olmayan bir tanesi. Sidecar'ın cue'ları gömülü track'inkiyle
(1–4 s ve 10–13 s) bilerek çakışmaz: ekrandaki metin hangi kaynaktan geldiğini
kendisi söyler.

### Adımlar

| # | Adım | Beklenen | Kriter |
|---|---|---|---|
| 1 | Uygulamayı aç | Tek pencere, boş durum: sürükle-bırak alanı ve `Aç…` | — |
| 2 | `⌘O` → `Nen Demo.mkv` | Video **hemen** oynuyor; sayaç ilerliyor | K1 |
| 3 | Oynarken altyazı düğmesine bas | Panel açılıyor; ne medya ne menü taramayı bekledi | K1 |
| 4 | Kolon 1'i oku | `Kapalı` → `Kullanıcı Altyazıları` (1) → `Türkçe` → `English` → `Français` → `Dil Belirsiz` | K3 |
| 5 | `Dil Belirsiz` grubuna bak | Tek satır: `Adsız parça` · rozet `Gömülü` | K3 |
| 6 | `⇧⌘O` → **aynı** `Nen Demo.srt` | `Kullanıcı Altyazıları` sayacı hâlâ **1** — aynı kaynak iki kez yok | K3 |
| 7 | Oynarken `⇧⌘O` → `Bozuk.srt` | Oynatma kesilmiyor, bildirim yok; satır soluk, ikinci satırı `biçim hatalı`, tıklanamıyor | K2 |
| 8 | `Nen Demo.srt` satırını seç | Replik **oynatma gerekmeden** ekranda | K5 |
| 9 | Cue içindeki bir ana sar | Hedef anın repliği **anında** görünüyor | K5 |
| 10 | Cue'suz bir ana sar | Ekran boş | K5 |
| 11 | `Kapalı`'ya bas | Ekran o anda boşalıyor; `Kapalı` her zaman listede | K3 |
| 12 | `⌘O` → `symlink/Nen Kisayol.mkv` | Video oynuyor, gömülü track çiziliyor; menüde `Kullanıcı Altyazıları` grubu **yok** — kısayol sidecar sessizce reddedildi | K4 |

### Symlink ve path-traversal'ın yeri

Kapı üç yüzeyden çağrılabilir ve senaryo hepsini aynı yerden kanıtlamaz:

- **Sidecar taraması** — yolu uygulamanın kendisi türetir. Tehdit modelinin
  asıl hâli budur ve adım 12 onu üründe gösterir: geçerli bir SRT'ye işaret
  eden kısayol bile kataloğa girmez, ve kullanıcıya hiçbir şey söylenmez
  (ADR-0031 Karar 5).
- **`⇧⌘O` paneli** — `NSOpenPanel` kısayolu uygulamaya vermeden **kendisi
  çözer**; panele bir kısayol seçildiğinde uygulamaya hedefin yolu gelir. Yani
  symlink reddi bu yüzeyden tetiklenemez. `NEN-071` kararıyla bu davranış
  `resolvesAliases = true` ile açıkça sabitlenmiştir: hedef normal kullanıcı
  altyazısı olarak yüklenir. Uygulamanın keşfettiği veya doğrudan kapıya gelen
  symlink yolları yine reddedilir.
- **`..` içeren yol** — panel yolu standardize ettiği için kullanıcı
  arayüzünden üretilemez. Traversal ayağı kapının kendi seviyesinde, negatif
  testlerle kanıtlanır (`core/crates/nen-app/tests/subtitle_file_gates.rs`).

Ayrıntı ve koşum kaydı: `evidence/M3/NEN-028-checklist.md`.

## Task'lar

`NEN-021` · `NEN-022` · `NEN-023` · `NEN-024` · `NEN-025` · `NEN-026` ·
`NEN-027` · `NEN-028` · `NEN-061` · `NEN-062`

Yerleşim regresyonları: `NEN-070` — ekran genişliğindeki pencerede transport
kontrollerinin yatay sınırları; `NEN-073` — üst gradient, fare çıkışı ve güvenli
minimum pencere boyutu; `NEN-074` — akıcı canlı video resize.

İptal edilen kapsam: `NEN-063` — teknik video kalitesi rozeti
(ADR-0036 `rejected`).

## Bağımlılıklar

M2. ADR-0012 (macOS motoru, linkleme ve proje lisansı) **2026-08-26'da kabul
edildi**: motor libmpv, adapter Swift'te (`platforms/macos/`), geliştirmede
dinamik link, proje lisansı GPL-3.0-or-later. Depo köküne `LICENSE` eklendi ve
roadmap **S12** kapandı. `.app` içine gömme + notarization bu milestone'un çıkış
kriterlerinden **değil** — `NEN-043`, dağıtımdan (S11) önce.

## Retro

<!-- M3 kapanışında doldurulacak -->
