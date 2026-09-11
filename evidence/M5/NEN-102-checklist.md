# NEN-102 — macOS translation progress and cancellation surface

Tarih: **2026-09-11** · Apple Silicon · macOS 27.0 (26A5425a) · Xcode CLT ·
Swift 6.3.3 · libmpv 2.5.0 (mpv 0.41.0_8, Homebrew)

ADR-0031 Karar 2 gereği yalnız `fixtures/` altındaki medya ve altyazılarla
üretildi: `fixtures/media/contract-clip.mkv` (gömülü Türkçe **ve** İngilizce
altyazı taşıyor) ve `fixtures/subtitles/blocks/layout-sample.srt` (95 cue'luk
İngilizce diyalog, `NEN-089`'un golden fixture'ı — varsayılan 40-cue blok
boyutuyla 3 bloğa ayrılıyor). Ekran görüntülerinde özel yol, URL veya gerçek
bir film adı yok.

## Adımlar ve gözlem

1. **`bash scripts/build-macos-app.sh` ile ad-hoc imzalı `.app` üretildi**,
   doğrudan `open` ile başlatıldı.
2. **`contract-clip.mkv` açıldı**, gömülü Türkçe altyazı otomatik seçildi
   (ADR-0031 Karar 4.3).
3. **`layout-sample.srt` `Altyazı Dosyası Yükle…` ile yüklendi** ve
   `Kullanıcı Altyazıları` grubunda **kendi başlığıyla** listelendi —
   ayrı bir `English` başlığı **değil**: medyanın kendi gömülü İngilizce
   track'i (`NEN-044` — embedded text extraction, henüz backlog) ayrı bir
   satır, kendi dil grubunda (`English`, `Gömülü`) duruyor. İkisini
   ayırt etmek bu task'ın kendi ölçümü oldu (bkz. "Yol üstünde ölçülen
   davranış" altında).
4. **`layout-sample.srt` seçildi** — ekranda "Leorio shouted from across the
   hall." göründü, badge "layout-sam…" oldu.
5. **`Altyazı` menüsünde `AI ile Türkçe Çevir` etkin** (kaynak İngilizce,
   hedef Türkçe — ikisi farklı).
6. **Komut verildi.** İş arka planda başlayıp bitti (mock provider anlık —
   `NEN-101`'in kendi gözlemiyle aynı); menüde **Türkçe** başlığının satır
   sayısı 1→2'ye çıktı, altında **"AI çevirisi (tr)"** satırı **`AI`
   rozetiyle** belirdi. **Ekrandaki altyazı ve seçili token değişmedi**
   (badge hâlâ "layout-sam…", metin hâlâ İngilizce) — §9'un "zorla AI
   çıktısına geçilmez" kuralı canlı doğrulandı.
7. **Negatif: kaynak zaten hedef dilde.** Gömülü Türkçe altyazı seçildi
   (hedef dil hâlâ Türkçe). `Altyazı` menüsü `AI ile Türkçe Çevir`i
   **soluk (devre dışı)** gösterdi.

## İlerleme göstergesi ve İptal — gerçek `.app`'te canlı yakalanamadı

`NEN-101`'in kendi kaydının öngördüğü gibi, M5'in mock provider'ı 95 cue'luk
bir işi de (3 blok) ekran görüntüsü döngüsünün (~saniyenin altında) çok
altında bir sürede bitiriyor; birden çok deneme, komutu verip hemen ekran
görüntüsü alma sırasında ilerleme hapını canlı yakalayamadı — iş her seferinde
zaten bitmiş durumdaydı. Bu, task'ın kendi planında öngörülen bilinen risk:
insan-zamanlı iptalin ancak M6'nın gerçek (ağ gecikmeli) sağlayıcısıyla
gözlemlenebileceği kaydı.

Bu, kod tarafında bir eksiklik değil — ilerleme/iptal mekanizmasının kendisi
**deterministik olarak**, gerçek bir gecikme kullanmadan (Rust worker
callback'inin ortasında gerçek OS thread'i parklayan bir rendezvous ile),
`TranslationProgressTests.swift`'in 7 testiyle kanıtlanıyor — bu testler
tam olarak bu adımın canlı sürümünü sağlıyor: işin duraklatılmış hâlde
iptal edilmesi, göstergenin kaybolması, menüye hiçbir `Ai` satırı eklenmemesi,
ekranın hiç değişmemesi. Bkz. "Kanıt kaydı" → Swift testleri.

## Yol üstünde ölçülen davranış (kusur değil, NEN-044'ün kapsamının canlı teyidi)

Medyanın gömülü **İngilizce** track'ini (badge "English", "Gömülü") doğrudan
seçip çeviriyi tetiklemek `FfiTranslationStartError::NoDocument` ile
reddediliyor — ekranda "Bu kaynağın içeriği henüz okunamadı." beliriyor ve
**kalıcı olarak** öyle kalıyor (birkaç saniye beklense de). Bu bir gecikme
değil: gömülü, birincil-olmayan-dil track'lerin metin çıkarımı `NEN-044`
(backlog, `NEN-103`'ün ADR-0045 kararına bağlı) — bu task'ın kapsamı dışında.
`NEN-102`'nin kendi DoD'u yalnız **kullanıcı altyazısı** (`Kullanıcı
Altyazıları` grubu) ve zaten-metni-olan gömülü birincil dil (Türkçe) üzerinden
kanıtlanıyor; gömülü İngilizce satırın `canTranslateSelectedSubtitle`'ı
**doğru şekilde etkin gösterdiği** (dil farklı olduğu için) ama işin kendisi
`NoDocument` ile reddedildiği ölçüldü — bu, `translateSelectedSubtitle()`'ın
kendi kapısının yalnız dil eşitliğine baktığını, belge varlığını FFI'ya
bıraktığını doğruluyor (tasarım gereği, `FfiTranslationStartError.NoDocument`
zaten bu senaryo için var).

## Kanıt kaydı

`bash scripts/test-macos.sh` **256 passed / 31 suites** (uçtan uca 2 ayrı
koşuda tekrarlandı, ikisi de yeşil) — `NEN-101`'in 249 testine bu task'ın 7
testi (`TranslationProgressTests`) eklendi. `cargo test --workspace` **832
passed / 1 ignored** (`NEN-100` baseline 831 + bu task'ın bulduğu ve düzelttiği
kusurun regresyon testi). fmt, clippy, `cargo deny check`, `bash
scripts/test.sh` **4/4** yeşil. Ayrıntı: `tasks/done/NEN-102-*.md`.
