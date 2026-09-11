---
id: NEN-104
title: Translation core acceptance
milestone: M5
size: S
state: done
closed: 2026-09-11
depends_on: [NEN-102, NEN-044]
blocks: []
adr: []
---

# NEN-104 — Translation core acceptance

## Sonuç

M5'in altı çıkış kriteri gerçek `.app` üzerinde, hem sidecar hem gömülü track
kaynağıyla uçtan uca kanıtlandı.

## Bağlam

`NEN-028` (M3) ve `NEN-088` (M4) emsali: milestone kapanışı ayrı bir acceptance
task'ıyla kanıtlanır ve kanıt `evidence/M#/` altında bir checklist olarak durur.

M5 **mock provider** ile kapanır (`docs/milestones/M5-translation-core.md` →
Kapsam dışı); gerçek sağlayıcı koşusu M6'nın işidir. Kabul koşusunda gerçek
provider kredisi kullanılmaz (Kural 8).

## Kapsam

- `docs/milestones/M5-translation-core.md` → Çıkış kriterleri, altı madde tek tek
- Kabul senaryosu iki kaynakla: yanındaki sidecar ve gömülü metin track'i
- Restart sonrası cache reuse'un gerçek `.app`te görülmesi
- Milestone kapanış ritüeli: retro (süre · yanlış çıkan varsayımlar · ADR
  kararları · sonraki milestone'un task kırılımı)

## YAPILMAYACAK

- Gerçek provider ile koşu — M6
- Yeni özellik veya düzeltme — yol üstünde bulunan kusur yeni backlog task'ı olur (Kural 5)

## Kanıt (DoD)

- [x] Kabul checklist'i `evidence/M5/NEN-104-checklist.md`, altı kriterin her biri için ayrı adım ve gözlem
- [x] Sidecar kaynağıyla uçtan uca koşu: seçim → komut → doğrulanmış artifact → menüde `Ai` kaynağı
- [x] Gömülü metin track'i kaynağıyla aynı koşu (`NEN-044`'ün çıkarımı üzerinden)
- [x] Restart sonrası aynı çeviri **provider çağrılmadan** açılıyor
- [x] Negatif: iptal edilen koşu ne menüde ne diskte iz bırakıyor
- [x] Negatif: log taraması — cue metni, medya URL'si, özel yol için eşleşme **0**
- [x] `bash scripts/test.sh`, `bash scripts/check-docs.sh`, workspace ve macOS kapıları yeşil
- [x] M5 retro'su `docs/milestones/M5-translation-core.md`'ye yazıldı

## Kanıt kaydı

Kabul koşusu 2026-09-11'de Apple Silicon / macOS 27.0 üzerinde, taze üretilmiş
Nen Player `.app` ile, `computer-use` aracıyla gerçek GUI üzerinden yürütüldü
(kullanıcı onayıyla — `NEN-044`'teki ret bu task için geçerli değildi, karar
task bazında alındı). `fixtures/media/contract-clip.mkv` ve
`fixtures/subtitles/blocks/layout-sample.srt` kullanıldı; ikincisi **sidecar
olarak otomatik keşfedilmedi**, `NEN-101`/`NEN-102` emsaliyle "Altyazı
Dosyası Yükle…" ile elle yüklendi (ADR-0031 Karar 2'nin fixture kısıtı ve
Kural 5 — yeni bir sidecar fixture çifti eklemek bu task'ın kapsamı dışı) —
bu, kapsam metninin "yanındaki sidecar" ifadesinden bir sapma, dürüstçe
kaydedilir.

**Bu ortamda `computer-use`'un `save_to_disk` çıktısı depoya committ
edilebilecek bir dosya yoluna erişilemedi** (tooling sınırı, `find` ile
doğrulandı) — ekran görüntüleri oturumda canlı görüldü, checklist bunları
metin olarak birebir aktarıyor; her disk iddiası (`artifacts/` dosya sayısı,
inode, mtime, cue içeriği) doğrudan dosya sistemi üzerinden ölçüldü (Kural 3:
UI kanıtı screenshot **veya** checklist).

**Sidecar + gömülü İngilizce track, ikisi de uçtan uca çevrildi**:
`artifacts/` 0→1→2 dosya, her ikisinin `webvtt`'i ilgili golden fixture'la
(`layout-sample.artifact.golden`, `contract-clip.sub-eng.golden`) cue
ID/zaman/sıra ve metin düzeyinde **birebir** eşleşti. `NEN-102`'nin kapanışta
canlı ölçtüğü kusur — gömülü İngilizce track'in kalıcı `NoDocument` reddi —
artık oluşmuyor; `NEN-044`'ün kapattığı kusur böylece uçtan uca doğrulandı.
**Negatif dil-eşitliği**: gömülü Türkçe seçiliyken komut `app_menu`'nün kendi
"disabled" reddiyle basılamadı, `artifacts/` sayısı değişmedi. **Cache
identity değişimi**: hedef dil Deutsch'a çevrilince `artifacts/` 2→3, yeni
`Deutsch` grubu menüde belirdi. **Restart → cache reuse**: uygulama
`pkill`/yeniden başlatma ile kapatılıp açıldı, aynı komut tekrarlandı;
`artifacts/` dosya sayısı **3'te kaldı** ve ilgili dosyanın inode'u
(`35321228`) ve mtime'ı (`1789130052`) **birebir aynı** kaldı — provider'a
hiç gidilmedi.

**İptal negatifı GUI'de zamanlanamadı** (mock provider'ın anlık bitişi,
`NEN-101`/`NEN-102`'nin kendi kayıtlarının öngördüğü bilinen risk); bu koşuda
`bash scripts/test-macos.sh`'ın "AI translation progress and cancellation
(NEN-102)" suite'i (6 test) ve `cargo test --workspace`'in
`translation_gate.rs`/`NEN-093` checkpoint testleri deterministik olarak
kanıtladı — kontrol sağır değil.

**Log taraması**: her iki `.app` koşusunun stdout/stderr'i (4 dosya) tamamen
boş; her iki pid için çekilen 2214 satırlık unified log'da cue diyaloğu,
medya/altyazı dosya adı, özel yol, URL şeması, motor adı veya ham hash için
eşleşme **0**. Ham log saklanmadan silindi.

`cargo test --workspace` **846 passed / 1 ignored** (`NEN-044` baseline'la
birebir aynı — kod değişmedi). fmt, clippy, `cargo deny check` (yeni
bağımlılık yok), `bash scripts/test-macos.sh` **264 passed / 33 suites** (iki
ayrı koşuda tekrarlandı, ikisi de yeşil — aradaki bir ara koşuda görülen
kırmızı, bilinen `PicturelessSurfaceTests` main-actor-contention flake'i,
`NEN-049`, bu task'ın kapsamı dışı), `bash scripts/test.sh` **4/4**, `bash
scripts/doctor.sh M3` tüm blocker'lar hazır. Geliştirme store kökü kanıt
toplandıktan sonra temizlendi. M5 retro'su
`docs/milestones/M5-translation-core.md`'ye yazıldı. Kanıt:
`evidence/M5/NEN-104-checklist.md`.
