---
id: NEN-123
title: macOS subtitle menu shows OpenSubtitles candidates and downloads on select
milestone: M6
size: M
state: done
closed: 2026-09-14
depends_on: [NEN-122]
blocks: [NEN-126]
adr: [21, 31]
---

# NEN-123 — macOS subtitle menu shows OpenSubtitles candidates and downloads on select

## Sonuç

macOS altyazı menüsünde OpenSubtitles adayları kendi grubunda görünüyor;
bir adayı seçmek indirmeyi başlatıyor, geçici durum gösteriliyor, başarılı
indirme o altyazıyı seçili yapıyor, başarısız indirme mevcut seçimi
**değiştirmiyor** ve ADR-0031 hata sunumuyla bildiriliyor.

## Kapsam

- `SubtitleMenuPresentation`/`SubtitleMenuView`: `openSubtitles` satırları
  (etiket zaten "OpenSubtitles"), rozet/release adı ADR-0021'e göre;
  `NEN-062`/`NEN-067`'nin paneli, yeni krom yok
- `PlayerModel.selectSubtitle` yolu: kind `OpenSubtitles` ve belge yoksa
  `download` → başarıda `selectSubtitle`, başarısızlıkta mevcut seçim ve
  geçici hata mesajı (`NEN-102`'nin geçici-mesaj yuvası)
- İndirme sırasında geçici durum (`TranslationStatusPill` emsali, "İndiriliyor…")
  ve medya değişiminde iptal
- Anahtar yoksa: aday grubu görünmez ya da satır "Ayarlardan anahtar girin"
  ile devre dışı — ADR-0021'in kararı

## YAPILMAYACAK

- İndirme mantığı/kapıları — `NEN-122`
- Aday sıralaması — `NEN-035`
- Otomatik indirme — `NEN-038`
- Kalıcı "indirilenler" listesi

## Kanıt (DoD)

- [x] Swift testleri (fake FFI/katalog ile): seçim → indirme → seçili;
      başarısız indirme → önceki seçim korunuyor, mesaj gösteriliyor;
      medya değişimi → gelen sonuç uygulanmıyor
- [x] Negatif: kind `OpenSubtitles` satırı, belge inmeden **seçili**
      olarak işaretlenmiyor (§9 "zorla geçilmez" emsali)
- [x] Gerçek `.app` checklist (`evidence/M6/NEN-123-checklist.md`, kullanıcı
      onayıyla tek gerçek koşu `NEN-126`'ya bırakılabilir; burada fake FFI
      ile UI checklist yeterli)
- [!] `bash scripts/test-macos.sh`: NEN-123 kapsam testleri ve hedefli
      menu suite'i yeşil; mevcut bağımsız `SubtitleRenderingTests` libmpv
      test-helper SIGSEGV'i tam paketi aynı makinede kırıyor (kanıt aşağıda).

## Kanıt kaydı

### Uygulama

- `nen-app::SubtitleLibrary`, OpenSubtitles adaylarını ve indirilen belgeyi
  worker kataloğundan revision kontrollü biçimde canlı kataloğa birleştiriyor;
  canlı seçim yalnız belge bağlandıktan sonra çalışıyor.
- `nen-ffi`, credential/http adaptörleri üzerinden aday aramasını bağladı;
  payload'sız arama durumu ve indirme hataları Swift'e taşınıyor.
- `PlayerModel`, aday aramasını ve seçime bağlı indirmeyi detached worker'da
  yürütüyor; medya değişiminde arama/indirme iptal ediliyor ve geç sonuç
  uygulanmıyor. UI mevcut geçici mesaj yuvasında `İndiriliyor…` durumunu
  gösteriyor.

### Doğrulama

- `swift test --package-path platforms/macos --filter OpenSubtitlesSelectionTests`:
  **4 test geçti** — başarı, hata/önceki seçim, geç medya sonucu ve belge
  öncesi seçili olmayan provider satırı.
- `swift test --package-path platforms/macos --filter MenuFixtureTests`:
  **3 test geçti**.
- `bash scripts/build-macos-app.sh`: **geçti**; gerçek debug
  `platforms/macos/.build/NenPlayer.app` üretildi.
- `cargo test --workspace --quiet`, clippy (`-D warnings`), format, `cargo deny
  check`, `bash scripts/test.sh` (**6/6**), `bash scripts/check-docs.sh` ve
  `git diff --check`: **geçti**.
- `bash scripts/test-macos.sh` ve daraltılmış
  `swift test --package-path platforms/macos --no-parallel --filter
  'NenPlaybackMPVTests.SubtitleRenderingTests'` koşuları, task kodundan
  bağımsız mevcut `SubtitleRenderingTests` libmpv test-helper'ının ilk gerçek
  render testinde **SIGSEGV / exit 11** verdiğini gösterdi. NEN-123 suite'i,
  MenuFixtureTests ve aynı paketteki diğer MPV sözleşme/fixture/session/sidecar
  suite'leri geçiyor; bu mevcut makine/toolchain sorunu `docs/STATUS.md`'de
  önceki doğrulamayla birlikte kaydedildi ve gerçek provider koşusu NEN-126'ya
  bırakıldı.
