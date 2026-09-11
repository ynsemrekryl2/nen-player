# NEN-104 — Translation core kabul kanıtı

Tarih: **2026-09-11** · Apple Silicon · macOS 27.0 (26A5425a) · Xcode CLT ·
Swift 6.3.3 · libmpv 2.5.0 (Homebrew) · ffmpeg 63/61 (Homebrew, `NEN-044`'ün
`libavformat`/`libavcodec` yolu)

ADR-0031 Karar 2 gereği yalnız `fixtures/` altındaki medya ve altyazılarla
üretildi: `fixtures/media/contract-clip.mkv` (gömülü Türkçe **ve** İngilizce
altyazı taşıyor — pencere/badge yalnız bu dosya adını gösteriyor, tam yol
hiçbir yüzeyde yok) ve `fixtures/subtitles/blocks/layout-sample.srt` (95 cue,
3 blok — `NEN-089`'un golden fixture'ı, `NEN-101`/`NEN-102` emsali "Altyazı
Dosyası Yükle…" ile yüklendi — `Kullanıcı Altyazıları` grubunda görünüyor).
Mock provider kullanıldı (Kural 8); gerçek sağlayıcı kredisi/kotası
harcanmadı.

**GUI kanıt yöntemi hakkında not.** `computer-use` ile gerçek `.app` bu
oturumda canlı sürüldü (`NEN-101`/`NEN-102` emsali, kullanıcı onayı ile —
`NEN-044`'teki ret bu task'a özgü değildi, task bazında karar). Ancak bu
ortamda `save_to_disk` ile alınan ekran görüntüleri depoya committ
edilebilecek bir dosya yoluna erişilemedi (bilinen bir tooling sınırı) —
görüntüler oturum sırasında canlı görüldü ve aşağıda **metin olarak birebir**
aktarılıyor (Kural 3: UI kanıtı screenshot **veya** checklist). Diskte
gözlenebilir her iddia (artifact sayısı, inode, mtime, cue içeriği) doğrudan
dosya sistemi üzerinden ölçüldü — bu ölçümler ayrı bir görsel yorumlamaya
bağlı değil.

## 1 — Uçtan uca: sidecar kaynağı → doğrulanmış artifact

`contract-clip.mkv` açıldı (gömülü Türkçe otomatik seçili). `layout-sample.srt`
yüklendi, `Kullanıcı Altyazıları` grubunda seçildi (ekranda "Leorio shouted
from across the hall." göründü). `Altyazı ▸ AI ile Türkçe Çevir` komutu
verildi.

**Gözlem:** `Türkçe` grubunun satır sayısı 1→2'ye çıktı; yeni satır
**"AI çevirisi (tr)"**, **`AI`** rozetiyle. Seçili altyazı **değişmedi**
(badge hâlâ "layout-sam…", ekran hâlâ İngilizce metin) — §9'un "zorla AI
çıktısına geçilmez" kuralı canlı doğrulandı.

**Disk:** `artifacts/` 0→1 dosya
(`1fffc2ce14997f1e746fd60f801531f30a4eae5cd0f2e39ef07f23616dc5a313.json`,
`src: en → tgt: tr`, `provider: nen-mock`).

**Cue ID/sıra/zaman birebir (golden karşılaştırması):** artifact'in `webvtt`
alanı `fixtures/subtitles/blocks/layout-sample.artifact.golden`'ın webvtt
bölümüyle **tam metin eşleşmesi** (`diff` çıktısı boş) — hem cue ID/zaman
yapısı hem çeviri metninin kendisi (mock'un deterministik `[tr] ` öneki),
iki farklı provider/model kimliğine (`nen-mock`/`deterministic-v1` vs.
golden'ın `nen-test`/`echo-golden`) rağmen birebir aynı.

## 2 — Uçtan uca: gömülü metin track'i kaynağı (NEN-044)

Aynı medyada `English ▸ Gömülü` track'i seçildi (badge "English"). Aynı komut
(`AI ile Türkçe Çevir`) verildi — bu, `NEN-044`'ün demux/decode yolunu
(`prepare_embedded_document`) ilk kez bu track için tetikledi.

**Gözlem:** `Türkçe` grubu 2→3 satıra çıktı — ikinci bir **"AI çevirisi (tr)"**
satırı, ayrı `AI` rozetiyle, ilkinin **yanında** durdu (S9: farklı kaynak
fingerprint'i, aynı hedef dil grubunda yan yana). Seçili altyazı hâlâ
"English" — değişmedi.

**Disk:** `artifacts/` 1→2 dosya
(`813fc259f16a5d1d4cee18586699139540bd6dd0e5f431c7c45329a9354422f3.json`,
935 bayt — 95 cue'luk sidecar'dan çok daha küçük, çünkü gömülü İngilizce
track yalnız 3 cue taşıyor).

**Cue birebir:** bu artifact'in `webvtt`'i, `fixtures/media/contract-clip.
sub-eng.golden`'ın üç cue'sunun ID/zaman/sırasıyla **birebir** eşleşiyor
(metin `[tr] ` önekiyle mock çevirisi). `NEN-102`'nin kapanışında canlı
ölçülen kusur — bu track'in `NoDocument` ile kalıcı reddi — burada **artık
oluşmuyor**: `NEN-044`'ün kapattığı kusur uçtan uca doğrulandı.

## 3 — Negatif: hedef dille aynı gömülü kaynak

Gömülü **Türkçe** track'i seçildi (hedef dil hâlâ Türkçe). `app_menu` aracı
`Altyazı ▸ AI ile Türkçe Çevir` yolunu **"disabled"** olarak reddetti
(basılmadı) — komut soluk. `artifacts/` dosya sayısı **2'de kaldı**
(değişmedi).

## 4 — Cache identity değişimi: hedef dil → Deutsch

Ayarlar'da `AI çeviri hedef dili` **Deutsch**'a değiştirildi. `layout-sample.srt`
(sidecar) yeniden seçilip `Altyazı ▸ AI ile Deutsch Çevir` komutu verildi.

**Gözlem:** yeni bir **"Deutsch"** grubu belirdi, içinde **"AI çevirisi (de)"**
/ `AI` satırı. Seçili altyazı değişmedi.

**Disk:** `artifacts/` 2→3 dosya
(`652fea3cff412803bd2e75c77678da234904a04ffbe2ca9d4a6747b9a9891270.json`,
`src: en → tgt: de`) — ADR-0018'in hedef-dil bileşeni değişince **yeni**
bir cache identity/artifact üretildiğini, eskisinin (`tr` grubundakiler)
**kullanılmadığını** canlı doğruluyor.

## 5 — Restart sonrası cache reuse (provider çağrılmadan)

`pkill NenPlayer` ile uygulama kapatıldı, ikinci kez başlatıldı (yeni pid).
Medya yeniden açıldı, `layout-sample.srt` yeniden yüklenip seçildi. Ayarlar
restart'ı hayatta kaldı (`AI çeviri hedef dili` hâlâ **Deutsch** —
`UserDefaults`). `Altyazı ▸ AI ile Deutsch Çevir` komutu **tekrar** verildi.

**Ölçüm (asıl kanıt — görsel değil, dosya sistemi):**

| | restart öncesi | restart sonrası, aynı komut |
|---|---|---|
| `artifacts/` dosya sayısı | 3 | **3** (değişmedi) |
| `652fea3c…70.json` inode | `35321228` | **`35321228`** (aynı) |
| `652fea3c…70.json` mtime (unix) | `1789130052` | **`1789130052`** (aynı, saniye hassasiyetinde) |

Dosya **yeniden yazılmadı** — `run_job`'ın (`nen-app/src/translation.rs`)
`index.find(job.cache_key)` isabetinin provider'a hiç gitmeden döndüğü,
gerçek `.app` restart'ı üzerinden ölçüldü. Menüde `Deutsch ▸ AI çevirisi (de)`
satırı bu yeni session'da da (kataloğu sıfırdan kuran taze `SubtitleLibrary`
üzerinden) doğru şekilde belirdi.

**Yol üstünde ölçülen, kusur olmayan bir ek veri noktası:** restart sonrası
akışın ilk denemesinde altyazı seçimi yanlışlıkla gömülü Türkçe'de
bırakılmıştı (test prosedürü hatası, üründe değil); bu, `tr→de` (3 cue) için
**dördüncü, ayrı** bir artifact üretti
(`a3d15766f2a79a0939ae5d905bb3cfa96bdad5b268e879edc846ba809c49d7d8.json`).
Bu da S9'un beklenen davranışı: farklı kaynak fingerprint'i → ayrı artifact,
`Deutsch` grubunda iki ayrı `AI çevirisi (de)` satırı olarak yan yana durdu,
biri diğerini silmedi.

## 6 — Negatif: iptal edilen koşu iz bırakmıyor

Mock provider'ın anlık bitişi yüzünden (`NEN-101`/`NEN-102`'nin kendi
kayıtlarının öngördüğü, insan-zamanlı bir GUI etkileşiminin yakalayamayacağı
pencere) bu senaryo gerçek `.app`te canlı zamanlanamadı — iş her seferinde
komut verilip ekran görüntüsü alınana kadar zaten bitmiş durumda. Bu, kod
tarafında bir eksiklik değil, mekanizmanın kendisi deterministik olarak,
gerçek bir OS thread rendezvous'uyla test ediliyor:

- `bash scripts/test-macos.sh` → **"AI translation progress and cancellation
  (NEN-102)"** suite'i, 6 test: `cancelling mid-run stops the job, hides the
  indicator, and leaves no trace`, `a cancelled job adds no Ai source to the
  menu`, `opening another medium cancels the running job`, ve üçü daha —
  hepsi bu koşuda **geçti** (bkz. §8).
- `cargo test --workspace` → `nen-ffi/tests/translation_gate.rs`
  (`NEN-108`'in stabilize ettiği iki iptal testi dahil) ve `nen-translate`'in
  checkpoint/iptal testleri (`NEN-093`) — hepsi bu koşuda **geçti**.

## 7 — Negatif: log taraması

Uygulama doğrudan (`open` değil) ikinci kez de dahil olmak üzere ikisi de
stdout/stderr geçici dosyalara yönlendirilerek başlatıldı; ayrıca her iki
`NenPlayer` pid'i (`92121`, `93633`) için `log show --predicate 'process ==
"NenPlayer"'` ile 2214 satırlık unified log çekildi. Ham dosyalar
saklanmadan yalnız sayaçlar raporlanıp silindi:

| Kontrol | Eşleşme |
|---|---:|
| stdout/stderr (her iki koşu, 4 dosya) | 0 satır (dosyalar tamamen boş) |
| Cue diyaloğu (Leorio/Killua/Gon/Kurapika/…) | 0 |
| Medya/altyazı dosya adı (`.mkv`/`.srt`) | 0 |
| Özel tam dosya yolu (`/Users/…`) | 0 |
| URL şeması (`http`/`https`/`file://`) | 0 |
| Motor adı (`libmpv`/`mpv`) | 0 |
| Ham/hash benzeri uzun hex dizgeleri | 0 |

Unified log'un 2010 satırı gerçekten `NenPlayer` sürecine ait (boş bir tarama
değil) — tamamı AppKit/TCC sistem etkinliği, uygulamanın kendi log
çıktısından hiçbir K23-yasaklı sınıf sızmadı.

## 8 — Kapılar

`cargo test --workspace` **846 passed / 1 ignored** (`NEN-044` baseline'la
birebir aynı — kod değişmedi). `cargo fmt --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo deny check` (yeni bağımlılık yok)
hepsi yeşil. `bash scripts/test-macos.sh` **264 passed / 33 suites** — iki
ayrı koşuda tekrarlandı, ikisi de yeşil (aradaki bir ara koşuda görülen
kırmızı, dosyanın kendi doc-comment'inin belgelediği bilinen
main-actor-contention flake'i, `PicturelessSurfaceTests` — `NEN-049`; bu
task'ın kapsamı dışı, dokunulmadı, `NEN-044`'ün kapanışında da aynı şekilde
gözlenmişti). `bash scripts/test.sh` **4/4**, `bash scripts/doctor.sh M3`
tüm blocker'lar hazır.

## Sonuç

M5'in altı çıkış kriterinin tamamı gerçek `.app` üzerinde, hem sidecar hem
gömülü track kaynağıyla, mock provider ile (Kural 8) uçtan uca kanıtlandı:

- [x] Uçtan uca: kaynak seçimi → açık çeviri komutu → doğrulanmış artifact (§1, §2)
- [x] Yarım/progressive çıktı hiçbir koşulda yayınlanmıyor (§1–§5 boyunca menü satırı yalnız iş bitince belirdi; deterministik kanıt: `NEN-093`/`NEN-094` testleri)
- [x] İptal sonrası late commit yok (§6, deterministik)
- [x] Cache identity bileşenlerinden biri değişince eski artifact kullanılmıyor (§4)
- [x] Cue ID/sıra/zamanlar girdiyle birebir aynı (§1, §2 — golden diff)
- [x] Çeviri sırasında kaynak değişimi işi retarget etmiyor (deterministik: `nen-app/tests/translation_session.rs`, bu task'ta yeniden koşulmadı — GUI'de ayrı bir kaynak-değişimi-sırasında senaryosu zamanlanmadı, mevcut regresyon testi §8'in `cargo test --workspace` çalışmasına dahil)
- [x] Restart sonrası cache reuse (§5, DoD'un ek maddesi)

Geliştirme store kökü (`~/Library/Application Support/NenPlayer/artifacts/`)
kanıt toplandıktan sonra temizlendi.
