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
tarihte başladı ve **2026-09-07'de kapandı** — 14 gün.

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

Kriterler 2026-09-05'te kanıtlandı, ama **biçimsel kapanış** (retro + roadmap
durumu) kullanıcı kararıyla ayrı tutuldu: `NEN-028`'in kapsamına alınmadı ve
`milestone: M3` etiketli **her** task'ın bitmesi beklendi (2026-09-06 kararı).
Son task `NEN-043` 2026-09-07'de kapandı; M3 aynı gün kapatıldı. Aşağıdaki
**Retro** o kapanışın kaydıdır.

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

`milestone: M3` etiketli **46** task: 44 `done`, 2 `canceled`. Kanonik liste
milestone açılırken 10 task sayıyordu; kalan 36'sı M3 sürerken açıldı
(gerekçesi Retro → "Yanlış veya eksik varsayımlar").

**Kanonik dilim** — milestone açılırken planlanan hat:
`NEN-021` · `NEN-022` · `NEN-023` · `NEN-024` · `NEN-025` · `NEN-026` ·
`NEN-027` · `NEN-028`

**Playback kontratı ve motor davranışı:**
`NEN-045` · `NEN-051` · `NEN-052` · `NEN-053` · `NEN-054` · `NEN-055` ·
`NEN-058`

**Kabuk, pencere ve krom:**
`NEN-042` · `NEN-046` · `NEN-047` · `NEN-048` · `NEN-050` · `NEN-061` ·
`NEN-062` · `NEN-067` · `NEN-068` · `NEN-070` · `NEN-073` · `NEN-074` ·
`NEN-077`

**Video yüzeyi ve altyazı çizimi:**
`NEN-066` · `NEN-069`

**Altyazı kataloğu, dil ve dosya kapıları:**
`NEN-036` · `NEN-037` · `NEN-039` · `NEN-056` · `NEN-057` · `NEN-071` ·
`NEN-075`

**Test altyapısı ve araçlar:**
`NEN-040` · `NEN-041` · `NEN-049` · `NEN-059` · `NEN-065` · `NEN-076`

**Paketleme:** `NEN-043`

**İptal edilen kapsam:** `NEN-060` — tam ekranda çizilmeyen gömülü track
(`NEN-066` ile aynı kök nedene katlandı) · `NEN-063` — teknik video kalitesi
rozeti (ADR-0036 `rejected`).

## Bağımlılıklar

M2. ADR-0012 (macOS motoru, linkleme ve proje lisansı) **2026-08-26'da kabul
edildi**: motor libmpv, adapter Swift'te (`platforms/macos/`), geliştirmede
dinamik link, proje lisansı GPL-3.0-or-later. Depo köküne `LICENSE` eklendi ve
roadmap **S12** kapandı. `.app` içine gömme + notarization bu milestone'un çıkış
kriterlerinden **değil** — `NEN-043`, dağıtımdan (S11) önce.

## Retro

**Süre ve çıktı.** M3, 2026-08-25'te (toolchain kapısı açıldığı gün) başladı ve
2026-09-07'de **46 task** ile kapandı — 44 `done`, 2 `canceled`, 14 gün.
Milestone'un vaat ettiği kullanıcı hikâyesi üründe geçerli: gerçek `.app`
dosyayı açıyor, oynatıyor, gömülü track'leri ve sidecar'ları gruplu bir menüde
gösteriyor, seçileni ekrana çiziyor ve seek sonrası doğru repliği anında
veriyor. Kapanış regresyonu: Rust workspace **607** test, macOS Swift paketi
**202/202** (22 suite), `cargo fmt`/`clippy`/`deny check`, `.app` build'i,
strict codesign, `bash scripts/test.sh` (3/3) ve `bash scripts/check-docs.sh`
(çıkış 0) yeşil.

**Yanlış veya eksik varsayımlar.** Bu milestone'un en pahalı dersi kapsamda
değil **teşhiste**: M3'ün task sayısı 10'dan 46'ya çıktı, çünkü kapanan her
task yol üstünde ölçülebilir bir kusur bırakıyordu (kural 5 gereği her biri
yeni bir backlog task'ı oldu). Ölçüm dört kez, önce yazılmış teşhisin kendisini
çürüttü:

- `NEN-066` — "altyazı çizilmiyor" yanlıştı. mpv dört durumun dördünde de
  (gömülü/enjekte × oynarken/duraklatılmış) çiziyordu; 134 pt'lik cam transport
  o piksel bandını örtüyordu. Çare render yolunda değil, kabuğun kromunu
  çekirdeğe bildirmesindeydi → ADR-0037.
- `NEN-049` — "paralel testler birbirini bozuyor" yanlıştı. Kusur
  `waitForGeometry`'nin **ilk** non-nil değeri kabul edip giden medyanın
  boyutunu okumasıydı; task'ın açılışta önerdiği izolasyon kırmızıyı
  düzeltmez, **gizlerdi**. Ürün kaynak kodu değişmedi.
- `NEN-069` — "videosuz medyada kare çizilmiyor" yanlıştı. Çizim doğruydu,
  eksik olan **isteyen**di: update callback yalnız yeni kare üretildiğinde
  tetikleniyor, videosuz medya hiç kare üretmiyordu.
- `NEN-076` — doğrulamanın kendisi yanlıştı. `check-docs.sh`'ın STATUS
  güncellik denetimi kapanış tarihini git geçmişinden okuyordu; CI'ın
  `fetch-depth: 1` shallow clone'unda bu her done task'ı "bugün" gösteriyor ve
  yalnız doküman içeren commit'leri kırmızıya döndürüyordu. Tarih artık task'ın
  kendi `closed` frontmatter alanından okunuyor.

Platform varsayımı da bir kez ürünü değiştirdi: `NEN-025`'in sidecar keşfi kod
yazılmadan **önce** ölçüldü ve planlanan çözüm elendi — App Sandbox içinde
kardeş `.srt` üç ayrı kurulumda `EPERM` verdi, çünkü Powerbox uzantısı seçilen
**dosyaya** çıkıyor, dizinine değil. ADR-0034 sandbox'ı düşürdü, `security-policy.md`
§4 kapılarını tek savunma hattı ilan etti ve Mac App Store'u non-goal yaptı.

**Kararlar.** M3 döneminde **15 ADR** yazıldı: 0011, 0012, 0013, 0030, 0031,
0032, 0033, 0034, 0035, 0037, 0038, 0039, 0041, 0042 `accepted`; **0036
`rejected`** (teknik video kalitesi rozeti — `NEN-063` onunla birlikte iptal
edildi). Ayrıca M1/M2'den gelen 0008, 0009, 0010, 0026 ve 0029 bu milestone'da
ilk kez üründe sınandı.

**Hiçbir ADR `superseded` olmadı** — ama iki kez bu ihtimal açıkça tartılıp
reddedildi ve depo bunun için bir precedent kurdu: ADR-0035, ADR-0031'i bütün
olarak supersede etmenin `adr: [.., 31]` taşıyan 7 done task'ı
`check-docs.sh` adım 6'da kırmızıya düşüreceğini ölçtü; ADR-0041 aynı yolu
izleyip ADR-0034'ün Karar 3'ünü **değiştirdi**, gövdesini yerinde bıraktı ve
Notlar'a işaret ekledi. Yani "karar değişti" ile "karar geçersiz" bu
milestone'da ayrı iki şey olarak sabitlendi.

**Sonraki milestone.** Sıra **M4 — Stremio Handoff (macOS)**'ta (kullanıcı
kararı, 2026-09-07); task kırılımı bu kapanışla birlikte üretiliyor. Ön koşul
yok: toolchain kapısı açık ve `NEN-043` `.app`'i Homebrew'dan bağımsız hale
getirdi. M4'ün "optional start position" maddesi hazır bir zemine oturuyor —
ADR-0042 yüklenirken verilen seek'in ne yapacağını `NEN-052`'de zaten karara
bağladı.

**Kapanışta açık kalan, M3'ün dışına yazılan iş.** Developer ID imzası,
hardened runtime, notarization ve `spctl` `NEN-043`'ün YAPILMAYACAK'ına ve
roadmap **S11**'e taşındı: Apple Developer Program üyeliği gerektiriyor, bu
makinede yok. `.app` bugün ad-hoc imzalı ve side-loading ile çalışıyor.
