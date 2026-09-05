# NEN-028 — macOS dikey dilim kabulü, kanıt kaydı

Koşum: **2026-09-05** · macOS 27.0 · Xcode 26.6 · Swift 6.3.3 · libmpv 2.5.0
(Homebrew, dinamik) · `platforms/macos/.build/NenPlayer.app`, ad-hoc imzalı,
tam ekran.

Medya **yalnız depodaki sentetik fixture**: `fixtures/media/menu-clip.mkv`'nin
geçici bir kopyası (`Nen Demo.mkv`) ve ikinci bir kopyası (`Nen Kisayol.mkv`).
Yanlarına konan `.srt`'ler elle yazıldı ya da
`fixtures/subtitles/malformed/end-before-start.srt`'ten kopyalandı; hiçbiri
depoya girmedi. Senaryonun kendisi:
`docs/milestones/M3-macos-slice.md` → "Kabul senaryosu (NEN-028)".

Kayıtlarda tam dosya yolu, query, motor adı veya özel medya metadata'sı yok
(K23). Karelerdeki replikler uydurmadır; dosya seçici yalnız klasör adını
(`run`) ve dosya adlarını gösteriyor.

## DoD karşılıkları

| DoD maddesi | Durum | Kanıt |
|---|---|---|
| Medya, katalog taraması bitmeden oynuyor | ✅ | `NEN-028-playing.jpg` · `theMenuDoesNotWaitForTheScan` |
| Bozuk `.srt` playback'i durdurmuyor, kaynağı işaretliyor | ✅ | `NEN-028-defective-source.jpg` · `aBrokenLoadedFileIsMarkedInTheMenu` |
| Menüde tekrar yok · `Dil Belirsiz` · `Kapalı` | ✅ | `NEN-028-menu.jpg` · `NEN-028-closed.jpg` · dedup adımı 6 |
| Symlink ve path-traversal reddediliyor | ✅ | `NEN-028-symlink-refused.jpg` · aşağıdaki kapı tablosu |
| Seek sonrası doğru cue anında | ✅ | `NEN-028-selection.jpg` · `NEN-028-after-seek.jpg` · `NEN-028-gap.jpg` · benchmark |
| Ekran kaydı `evidence/M3/` altında | ✅ | `NEN-028-slice.mp4` (aşağıda; `.gitignore` gereği commit edilmez) |

## Kriter kriter — üründe ne görüldü

**K1 — medya taramayı beklemiyor.** `Aç` düğmesine basıldıktan 0,7 s sonra
alınan karede video çiziliyor ve sayaç `00:00 / 00:20`'de
(`NEN-028-playing.jpg`). Altyazı menüsü oynatma sürerken açıldı; ne medya ne
menü tarama için bekledi. Kabuk tarafındaki kural
`SubtitleMenuTests.theMenuDoesNotWaitForTheScan` ile, taramanın ayrı bir
`Task` olduğu `PlayerModel.startSidecarScan` ile korunuyor (ADR-0031 Karar 4).

**K2 — bozuk kaynak playback'i durdurmuyor.** Oynatma sürerken `Bozuk.srt`
elle yüklendi: bildirim çıkmadı, oynatma kesilmedi (kare `00:04`'te ⏸ ikonuyla
oynuyor), ve satır menüde **soluk**, ikinci satırı `biçim hatalı`
(`NEN-028-defective-source.jpg`). Satıra tıklamak seçimi değiştirmedi.

**K3 — menü.** `Kapalı` ayrı, sonra `Kullanıcı Altyazıları` · `Türkçe` ·
`English` · `Français` · `Dil Belirsiz`; grup sayaçlarının hepsi `1`
(`NEN-028-menu.jpg`). Dili olmayan gömülü track `Dil Belirsiz` içinde
`Adsız parça`. **Dedup:** kataloğa sidecar olarak girmiş `Nen Demo.srt` bir de
`⇧⌘O` ile elle yüklendi — sayaç `1` kaldı. `Kapalı`'ya basıldığında ekran
o anda boşaldı ve kolon 2 `Altyazılar kapalı.` dedi (`NEN-028-closed.jpg`).

**K4 — symlink ve traversal.** Kapı üç yüzeyden çağrılıyor ve üçü ayrı ayrı
kanıtlandı:

| Yüzey | Yolu kim veriyor | Bu koşuda |
|---|---|---|
| Sidecar taraması | uygulamanın kendisi | Geçerli bir SRT'ye işaret eden kısayol sidecar **sessizce reddedildi**: `Nen Kisayol.mkv` açıldığında menüde `Kullanıcı Altyazıları` grubu hiç yok, bildirim de yok (`NEN-028-symlink-refused.jpg`) |
| `⇧⌘O` paneli | macOS paneli | Panel kısayolu **kendisi çözüyor**; uygulamaya hedefin yolu geliyor ve menüde hedefin adı beliriyor. Kapı bu yüzeyden çağrılmıyor — davranış kararı `NEN-071`'e ayrıldı |
| Port seviyesi | test | `a_symlinked_subtitle_is_refused_and_never_followed` · `a_path_containing_a_parent_component_is_refused` · `a_file_outside_the_root_is_refused_even_when_spelled_plainly` · `a_directory_symlink_cannot_smuggle_a_file_in_from_outside_the_root` · `every_gate_refusal_leaves_the_catalog_completely_empty` |

`..` içeren bir yol panelden üretilemiyor (panel yolu standardize ediyor), bu
yüzden traversal ayağının ürün karşılığı yok; kapının kendisi yukarıdaki
negatif testlerle korunuyor. Kabuk tarafı:
`explicitRefusalIsAnnounced` (kullanıcı reddi duyurulur) ·
`scannedRefusalIsSilent` (tarama susar) — ikisi aynı dosya, aynı verdikt,
farklı bulan.

**K5 — seek sonrası doğru cue.** `00:15`'te (3. cue'nun içi) kullanıcı dosyası
seçildi ve replik **oynatma gerekmeden** çizildi (`NEN-028-selection.jpg`).
`00:10`'a sarıldığında 2. replik anında geldi (`NEN-028-after-seek.jpg`),
`00:00`'da (cue'suz an) ekran boş kaldı (`NEN-028-gap.jpg`). Oynatarak
geçilen 1. cue de çizildi.

Ölçüm dayanağı — `bash scripts/bench-cue-lookup.sh`, release build:

| Belge | Index p50 | p95 | p99 | Lineer p50 | Hız |
|---|---|---|---|---|---|
| 50k cue, çakışmasız | 100 ns | 105 ns | 107 ns | 32,30 µs | 323× |
| 50k cue, derinlik 3 | 106 ns | 112 ns | 142 ns | 32,07 µs | 303× |

## Otomatik kapılar (hepsi bu ağaçta, aynı gün)

| Kapı | Sonuç |
|---|---|
| `cargo test --manifest-path core/Cargo.toml --workspace` | **560 passed / 0 failed / 1 ignored**, 72 hedef |
| `swift test --package-path platforms/macos` (paralel) | **151/151** yeşil (düzeltme öncesi ağaç) |
| `swift test … --no-parallel` (düzeltme sonrası) | **154/154** yeşil — 3 yeni test |
| `bash scripts/test.sh` | çıkış 0 (doctor + check-docs shell testleri) |
| `bash scripts/build-macos-app.sh` | çıkış 0 |
| `codesign --verify --deep --strict` | `valid on disk` · `satisfies its Designated Requirement` |
| `ContractTests.theRealAdapterPassesTheSharedContractKit` | yeşil — gerçek adapter fake ile aynı kiti geçiyor |

## Koşuda bulunan ve düzeltilen kusur

**Elle yüklenen altyazı dosyası menüye hiç ulaşmıyordu.**
`PlayerModel.loadSubtitleFile(at:)` kataloğa ekliyor ve sayacı güncelliyordu
ama `refreshSubtitleMenu()` çağırmıyordu — oysa kaynak kodun kendi yorumuna
göre menüyü yenileyen **tek** yol odur. Sonuç: `⇧⌘O` ile yüklenen dosya (bozuk
olsun, sağlam olsun) menüde görünmüyordu; sidecar yolu ise taramanın
yenilemesi sayesinde çalıştığı için kusur bugüne kadar görünmemişti.

Bu doğrudan K2'nin ürün yüzeyini kırıyordu: ADR-0031 Karar 5 "bozuk kaynağın
yüzeyi menüdür" diyor, menüye giremeyen satırın işareti de yok.

Düzeltme tek satır: `addFile` sonrası `refreshSubtitleMenu()`. Testler önce
yazıldı ve **kırmızı görüldü**:

```
✘ "a file the user loads is in the menu as soon as it is loaded"
    (user?.entries.map(\.label) → nil) == ["Chosen.srt"]
✘ "a broken file the user loads is in the menu, marked and out of reach"
    (row?.defect → nil) == .malformed
✔ "a file the user loads and has refused leaves no row behind"   ← negatif kontrol
```

Negatif kontrol düzeltmeden önce de sonra da yeşil: menü "her yüklemede"
yenilenmiyor, yalnız kataloğun gerçekten değiştiği yerde. Düzeltmeden sonra
üçü de yeşil, paket **154/154**.

## Kayıt

`evidence/M3/NEN-028-slice.mp4` — 46,1 s · 1280×832 · 12 fps · 1 157 464 bayt ·
sha256 `aa304edb27dcb914…`. Kaynak kareler koşu boyunca ≈2,5 fps ile alındı ve
12 fps ile kodlandı: kayıt **≈5× hızlandırılmıştır**, gerçek zamanlı değildir.
Kayıt tam ekran uygulamayla başlar; uygulama dışı hiçbir kare yoktur.

`.gitignore` `evidence/**/*.mp4` yolunu dışarıda tutuyor (depo politikası,
`NEN-024-playback.mp4` de aynı durumda), bu yüzden kayıt commit edilmez —
yerelde bu yolda durur ve yukarıdaki metadata ile doğrulanır. Karelerin
kendisi commit edilir.

## Bu koşudan çıkan, buraya yazılmayan iş

- `NEN-071` — `⇧⌘O` panelinin kısayolu çözmesi: davranış kararı ve o kararın
  ürün yüzeyindeki testi.
- `NEN-049` — üçüncü kez gözlenen paralel-koşu kırmızısı (bu kez gerçek libmpv
  testinde, `successiveMediaReportTheirOwnDisplaySize`); gözlem o task'a not
  edildi, iş eklenmedi (Kural 5).
