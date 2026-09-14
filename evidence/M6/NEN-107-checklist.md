# NEN-107 — Document-wide translation progress across the FFI boundary

Tarih: **2026-09-14** · Apple Silicon · macOS 27.0 · Xcode 26.6 · Swift
6.3.3 · libmpv 2.5.0

Kullanılan fixtures: `fixtures/media/contract-clip.mkv` ve
`fixtures/subtitles/blocks/layout-sample.srt` (95 cue, 3 blok). Gerçek
provider kredisi kullanılmadı; `.app` smoke akışı M5 mock provider ile
çalıştırıldı.

## Adımlar ve gözlem

1. **`bash scripts/build-macos-app.sh`** ile ad-hoc imzalı `.app` üretildi.
   Uygulama gerçek pencere olarak açıldı ve `contract-clip.mkv` oynatıldı.
2. **`Altyazı Dosyası Yükle…`** üzerinden `layout-sample.srt` yüklendi;
   `Kullanıcı Altyazıları` altında listelendi ve seçildi. `English` kaynağı
   seçiliyken **`AI ile Türkçe Çevir`** komutu etkin görüldü.
3. Komut gerçek `.app` içinde çalıştırıldı. 95 cue tamamlandı ve menüde
   Türkçe kaynak sayısı 1’den 2’ye çıktı; ekranın seçili altyazısı zorla AI
   çıktısına geçmedi.
4. Büyük fixture için mock provider işi ekran yakalama aralığından daha hızlı
   tamamlıyor; bu nedenle ara progress pill’i insan-zamanlı ekran görüntüsünde
   yakalanamadı (NEN-102’de kayıtlı aynı M5 sınırı). Ara değerlerin gerçek
   FFI→Swift yolunda document total, monotonic done ve yüzde kopyasıyla
   işlendiği aşağıdaki deterministik testlerle kanıtlandı.

## Kanıt kaydı

- `cargo test --workspace` — **başarılı**; `nen-ports` monotonicity kontratı,
  çok bloklu document progress ve retry regression testleri yeşil.
- `cargo clippy --workspace --all-targets -- -D warnings` — **başarılı**.
- `cargo deny check` — **başarılı**; advisories, bans, licenses ve sources
  kontrolleri yeşil.
- `bash scripts/test-macos.sh` — **268 test / 33 suite başarılı**; yeni
  `progress copy presents the document count and percentage` ve güncellenen
  `progress reaches the indicator in order...` testleri yeşil.
- `bash scripts/build-macos-app.sh` — **başarılı**; gerçek `.app` üretildi.
- `bash scripts/check-docs.sh` — **başarılı**; active=0, done kanıtları ve
  INDEX/STATUS uyumu doğrulandı.
- `bash scripts/test.sh` — **4/4 doğrulama dosyası başarılı**.
- `bash scripts/doctor.sh M3` — **başarılı**; cargo/rustc, Xcode, Swift ve
  libmpv hazır.
