# NEN-025 — kapanış kanıtı

**Tarih:** 2026-08-27 · **Makine:** Apple M5 / macOS 27 · **libmpv** 2.5.0
**Yapı:** `bash scripts/build-macos-app.sh` — ad-hoc imza, **entitlement yok**
(ADR-0034). `codesign -d --entitlements -` boş `[Dict]` veriyor.

## Otomatik testler

| Paket | Sonuç |
|---|---|
| `cargo test` (workspace) | **489 passed, 0 failed** (`NEN-051` kapanışında 453'tü) |
| `swift test --package-path platforms/macos --no-parallel` | **56 test / 7 suite**, art arda **2/2** |
| aynı paket, paralel | art arda **4/4** |

Paralel modda **bir** kırmızı gözlendi ve teşhis edildi: derlemenin de aynı
anda koştuğu ilk turda `transientMessageExpires`. Yalnız o test 1/1, seri 3/3,
paralel (derleme sıcakken) 4/4 yeşil. Yani yük ilişkili, `NEN-025` ile
ilişkisiz; `NEN-049`'a gözlem olarak işlendi.

## Güvenlik kapıları — negatif testler (§4, zorunlu)

Hepsi gerçek dosya sistemi üzerinde, geçici dizinlerde gerçek nesnelerle.
`core/crates/nen-app/tests/subtitle_file_gates.rs` (21 test):

| §4 | Test | Ne kanıtlıyor |
|---|---|---|
| #2 symlink | `a_symlinked_subtitle_is_refused_and_never_followed` | Hedef **geçerli** SRT; izlenseydi belge üretilirdi. Katalog boş kalıyor |
| #3 traversal | `a_path_containing_a_parent_component_is_refused` | `..` içeren yol, kök **içinde** bir dosyaya çözülse bile reddediliyor |
| #3 traversal | `a_file_outside_the_root_is_refused_even_when_spelled_plainly` | Kök dışı dosya |
| #3 traversal | `a_directory_symlink_cannot_smuggle_a_file_in_from_outside_the_root` | Son bileşen dürüst bir düz dosya; kaçış bir üst seviyede. Parent canonicalize bunu yakalıyor |
| #1 regular file | `a_directory_is_refused` · `a_fifo_is_refused_rather_than_read` | FIFO'yu açmak sonsuza kadar bloke olurdu; testin **dönmesi** kanıtın yarısı |
| #1 regular file | `a_path_that_is_not_there_is_refused_rather_than_catalogued_as_broken` | Olmayan dosya "hatalı kaynak" olmuyor |
| #4 boyut | `a_file_over_the_size_limit_is_refused` | Seyrek dosya, sınır + 1 bayt |
| #4 boyut | **`an_oversized_file_is_refused_without_being_opened`** | Dosya hem büyük hem `chmod 000`. Kapı açmayı deneseydi `Unreadable` dönerdi; `TooLarge` dönmesi ancak hiçbir şey açılmadıysa mümkün |
| #4 boyut | `a_file_exactly_at_the_size_limit_is_still_admitted` | Kapı "aşan"; off-by-one hiçbir negatif testin yakalamayacağı bir yanlış red üretirdi |
| ADR-0031 K5 | `every_gate_refusal_leaves_the_catalog_completely_empty` | Dört kapının **her biri** için ayrı ayrı: katalogda iz yok |

**Boyut sınırı yeni sabit değil.** `MAX_SUBTITLE_BYTES = encoding::MAX_INPUT_BYTES`
(10 MiB, NEN-015/ADR-0008). `the_file_gate_and_the_decoder_share_one_limit`
ikisinin ayrışmasını engelliyor: ayrışsalardı aradaki bant tamamen okunup sonra
reddedilirdi, yani §4 #4'ün yasakladığı davranış.

## Kimlik ve dedup (ADR-0010 Karar 2)

- `loading_the_same_file_twice_leaves_one_catalog_entry`
- `two_spellings_of_one_file_are_one_entry` — `dir/./Film.srt` ile `dir/Film.srt`
- `a_sidecar_and_the_same_file_loaded_by_hand_are_one_entry`
- `two_different_files_are_two_entries` — her şeyi tek girişe indiren bir
  digest yukarıdakilerin üçünü de geçer ve işe yaramaz; bu onun kontrolü
- `reloading_a_repaired_file_clears_the_mark_it_used_to_carry`

## Bozuk kaynak playback'i durdurmuyor

- `a_malformed_subtitle_is_catalogued_and_marked_rather_than_dropped`
- `a_malformed_subtitle_leaves_every_other_source_alone` — hata kendi girişine
  hapsedilmiş, **dönülüyor**, fırlatılmıyor
- Kabuk: `brokenSubtitleDoesNotInterrupt` — bildirim yok, `fatalMessage` nil,
  `pauseCount == 0`, `shutdownCount == 0`

## K23 redaction

`core/crates/nen-app/tests/guard_subtitle_file_debug.rs` (4 test). Üçü yolun,
dosya adının ve replik metninin `Debug` çıktısında olmadığını gösteriyor;
dördüncüsü **kontrolün kontrolü**: derive edilmiş bir ikiz aynı değerleri
sızdırıyor. Guard, bir impl `#[derive(Debug)]` ile değiştirildiği gün kırmızıya
döner.

## Gerçek `.app` üzerinde acceptance

ADR-0034'ün değiştirdiği şey yalnız imzalı `.app`'te gözlenebilir — unit
testlerde zaten sandbox yok. Bu yüzden dört senaryo gerçek uygulamada koşuldu.
Ölçüm sırasında geçici bir log satırı kullanıldı (**commit edilmedi**, ölçümden
sonra kaldırıldı); hiçbir yol loglanmadı.

| # | Senaryo | Beklenen | Gözlenen |
|---|---|---|---|
| 1 | Yanında geçerli `.srt` olan medya açılır | sessizce bulunur | `SIDECAR-RESULT added count=1` · bildirim **yok** · medya oynadı |
| 2 | Yanında **symlink** `.srt` olan medya açılır | sessizce elenir | `SIDECAR-RESULT rejected(symlink) count=0` · bildirim **yok** |
| 3 | `⇧⌘O` → 11 MiB `.srt` elle yüklenir | geçici bildirim | **"Bu altyazı dosyası çok büyük."** transport üzerinde; yol yok; oynatma sürdü |
| 4 | `⇧⌘O` → symlink `.srt` elle yüklenir | — | `panelGaveSymlink=false`, `added` — aşağıya bak |

**Senaryo 4 bir hipotezi çürüttü.** `NSOpenPanel` symlink'i **çözüyor**: panel
gerçek hedefi döndürdüğü için kullanıcı panel üzerinden symlink teslim
edemiyor. Yani symlink kapısı elle yükleme yolunda değil, **tarama yolunda** ve
gelecekteki panel-dışı girişlerde (sürükle-bırak, son medya, M4 Stremio
handoff) iş görüyor. Senaryo 2 kapının gerçekten çalıştığını gösteriyor;
`a_symlinked_subtitle_is_refused_and_never_followed` ise kapıyı doğrudan
kanıtlıyor. Bu yüzden DoD'un symlink maddesi karşılanmış sayılıyor, fakat
**panel üzerinden değil**.

DoD'un "elle yüklenen dosya reddedilince bildirim üretiliyor" maddesi
senaryo 3 ile karşılandı; "tarama sırasında reddedilen dosya bildirim
üretmiyor" maddesi senaryo 2 ile — ikisi aynı kapı, farklı keşfeden,
yalnız bildirim farkı. Kabukta da ikizler var:
`explicitRefusalIsAnnounced` / `scannedRefusalIsSilent`.

**Ekran kaydı yerine ölçüm.** ADR-0031 Karar 2 kanıt görüntülerini `fixtures/`
medyasıyla sınırlıyor; buradaki medya `fixtures/media/contract-clip.mkv`'nin
kopyası ve hiçbir yüzeyde yol görünmedi. Görsel yerine log satırları
kaydedildi, çünkü `NEN-025`'in gözle görülür tek çıktısı senaryo 3'ün
bildirimi — katalog NEN-026'ya kadar görünür değil.

## Kapsam dışında bırakılanlar

- `NEN-056` — ADR-0031 Karar 5'in `çok büyük` etiketi üretilemez durumda
- `NEN-057` — dosya adından dil ipucu (`Film.tr.srt`)
- `NEN-058` — yanında symlink `.srt` olan medya açılmıyor (gözlendi, teşhis
  edilmedi; senaryo 2'nin yan bulgusu)

## Ölçüm kaydı

Sandbox ölçümünün tamamı: `evidence/M3/NEN-025-sandbox-measurement.md`
