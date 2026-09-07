---
id: NEN-028
title: macOS vertical slice acceptance
milestone: M3
size: S
state: done
closed: 2026-09-05
depends_on: [NEN-027]
blocks: []
adr: []
---

# NEN-028 — macOS vertical slice acceptance

## Sonuç

"Dosya aç → oynat → katalog → seç → ekranda göster" akışı uçtan uca çalışır ve
beş kabul maddesi kanıtlanmıştır.

## Kapsam

- `docs/milestones/M3-macos-slice.md` içine adım adım manuel test senaryosu
- Uçtan uca çalıştırma ve ekran kaydı
- M3 retro'sunun yazılması

## YAPILMAYACAK

- Yeni özellik eklemek — bu bir **doğrulama** task'ı
- M4+ kapsamındaki hiçbir şey

## Kanıt (DoD)

- [x] Medya, katalog taraması bitmeden oynuyor
- [x] Bozuk bir `.srt` playback'i **durdurmuyor**, yalnız o kaynağı hatalı işaretliyor
- [x] Menüde aynı kaynak iki kez yok; dili bilinmeyen `Dil Belirsiz` grubunda;
      `Kapalı` her zaman var
- [x] Symlink ve path-traversal ile verilen altyazı **reddediliyor**
- [x] Seek sonrası doğru cue anında görünüyor (NEN-017 benchmark'ıyla birlikte)
- [x] Ekran kaydı `evidence/M3/` altında kayıtlı

## Kanıt kaydı

**2026-09-05, bu makinede.** Senaryo `docs/milestones/M3-macos-slice.md` →
"Kabul senaryosu (NEN-028)" içinde yazıldı ve gerçek `.app`te baştan sona
koşuldu. Tam kayıt: `evidence/M3/NEN-028-checklist.md`.

Beş kriterin de ürün karşılığı görüldü:

- **K1** — `Aç`'tan 0,7 s sonra video çiziliyor, sayaç `00:00 / 00:20`
  (`NEN-028-playing.jpg`); menü tarama beklemeden açılıyor.
- **K2** — oynatma sürerken yüklenen bozuk `.srt` oynatmayı kesmedi, bildirim
  üretmedi; satır menüde soluk ve `biçim hatalı`
  (`NEN-028-defective-source.jpg`).
- **K3** — `Kapalı` · `Kullanıcı Altyazıları` · `Türkçe` · `English` ·
  `Français` · `Dil Belirsiz`, sayaçlar `1`; aynı dosya elle ikinci kez
  yüklendiğinde sayaç `1` kaldı; `Kapalı` ekranı o anda boşalttı
  (`NEN-028-menu.jpg`, `NEN-028-closed.jpg`).
- **K4** — geçerli bir SRT'ye işaret eden kısayol sidecar sessizce reddedildi:
  ikinci medyanın menüsünde `Kullanıcı Altyazıları` grubu hiç yok
  (`NEN-028-symlink-refused.jpg`). `⇧⌘O` panelinin kısayolu kendisi çözdüğü
  ölçüldü ve `NEN-071`'e ayrıldı; traversal ayağı port seviyesindeki negatif
  testlerle duruyor.
- **K5** — seçim anında 3. replik (`NEN-028-selection.jpg`), `00:10`'a seek'te
  2. replik anında (`NEN-028-after-seek.jpg`), cue'suz anda ekran boş
  (`NEN-028-gap.jpg`). `bash scripts/bench-cue-lookup.sh`: 50k cue'da index
  p99 = 107 ns, lineer p50 = 32,3 µs (323×), release build.

Kayıt: `evidence/M3/NEN-028-slice.mp4` — 46,1 s · 1280×832 · 12 fps ·
1 157 464 bayt (kareler ≈2,5 fps alındı, ≈5× hızlandırılmış). `.gitignore`
`evidence/**/*.mp4`'ü dışarıda tuttuğu için commit edilmez; kareler edilir.

Otomatik kapılar: Rust workspace **560 passed / 1 ignored** (72 hedef) ·
macOS paketi **154/154** (paralel ve seri) · `scripts/test.sh` çıkış 0 ·
`.app` build ve strict codesign yeşil ·
`ContractTests.theRealAdapterPassesTheSharedContractKit` yeşil.

**Koşuda bir kusur bulundu ve düzeltildi:** `loadSubtitleFile(at:)`
`refreshSubtitleMenu()` çağırmadığı için `⇧⌘O` ile yüklenen dosya menüye hiç
girmiyordu — K2'nin ürün yüzeyini kıran bir boşluk. Üç test önce yazılıp
kırmızı görüldü (biri düzeltmeden bağımsız yeşil kalan negatif kontrol),
sonra tek satırlık düzeltme uygulandı.

**Kapsamdan çıkarılan:** M3 retro'su ve roadmap'in milestone durumu bu task'ta
yazılmadı — kullanıcı kararı (2026-09-05): "sadece task'ı kapat". Çıkış
kriterleri milestone dosyasında işaretlendi, retro boş bırakıldı.
