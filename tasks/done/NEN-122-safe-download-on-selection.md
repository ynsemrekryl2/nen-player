---
id: NEN-122
title: Safe download on selection
milestone: M6
size: M
state: done
closed: 2026-09-14
depends_on: [NEN-121]
blocks: [NEN-123, NEN-125, NEN-038]
adr: [21]
---

# NEN-122 — Safe download on selection

## Sonuç

Bir OpenSubtitles adayı **seçildiğinde** indiriliyor — yalnız HTTPS ve
approved host, her adımda host doğrulanan bounded redirect, boyut sınırı,
archive/content-type reddi, ardından encoding + strict SRT kapıları — ve
belge kataloğa takılıyor; kapılardan geçemeyen içerik diske de kataloğa da
hiçbir şey bırakmıyor.

## Kapsam

- `nen-app::SubtitleLibrary::download_opensubtitles(token)` (adı task'ın
  kararı) — `download` endpoint'i (anahtarla) → link → GET; kota bitince
  tipli `QuotaExhausted`
- Güvenlik kapıları **sırayla** (`security-policy.md` §3/§4): HTTPS ·
  approved host (link host'u dahil, ADR-0021 listesi) · redirect sayısı ve her
  adımda host · `Content-Length`/gövde boyutu · content-type ve magic bytes
  (zip/rar/gzip/7z reddi) · encoding (`NEN-015`) · strict SRT (`NEN-013`)
- Başarılı belge `attach_embedded_document` emsali idempotent takılır; ikinci
  seçimde yeniden indirilmez (oturum içi)
- Her kapı payload'sız tipli hata (`DownloadRefusal` enum'u), `nen-ffi`'ye düz
  varyantlar (`FfiEmbeddedDocumentError` emsali)
- İptal: `TranslationCall` gate'i değil, basit bir `AtomicBool`/revision —
  medya değişirse gelen belge takılmaz

## YAPILMAYACAK

- macOS UI — `NEN-123`
- İndirilen dosyayı diske kalıcı yazmak — M6'da bellek içi belge; kalıcı
  altyazı önbelleği ayrı karar (ADR-0021 erteler)
- Archive'ı **açmak** — reddedilir, açılmaz
- Otomatik indirme — `NEN-038`

## Kanıt (DoD)

- [x] Unit: fixture link + gövde ile belge kataloğa takılıyor; ikinci seçim
      `send` sayacını artırmıyor
- [x] Negatif (zorunlu — her biri ayrı test, her biri `send`/diske yazma
      **0** ve katalogda belge **yok**): liste dışı host · `http://` link ·
      redirect sınırı aşımı · redirect zincirinde liste dışı host · boyut
      aşımı (`Content-Length` ve gövde ayrı) · zip magic bytes · yanlış
      content-type · SRT parse hatası · kota bitmiş cevap
- [x] Negatif (K23): link URL'si, private `file_id`, anahtar hata tiplerinin
      ve `Debug`'ların hiçbirinde yok
- [x] Negatif: medya değiştikten sonra gelen belge takılmıyor
- [x] Mutasyon: en az iki kapı elle kaldırılıp tam olarak beklenen test(ler)
      kırmızıya dönüyor (`NEN-096` emsali) — kayıtta
- [x] `nen-ffi` hata eşlemesi mutasyon testi

## Kanıt kaydı

### Uygulama

- `nen-ports::subtitle_download`: payload'sız `DownloadRefusal` portu, 10 MiB
  gövde / 1 MiB metadata / 5 redirect bütçesi ve redacted `Debug` yüzeyi.
- `nen-providers::opensubtitles`: download endpoint'inden tek kullanımlık link
  değişimi; HTTPS + approved-host, hop başına redirect doğrulaması, response
  boyutu, content-type, arşiv magic bytes, kota ve yalnız düz metin GET.
- `nen-app::SubtitleLibrary`: credential gate, encoding ve strict SRT parse;
  başarılı belge yalnız bellekte bağlanıyor, aynı token idempotent, revision
  değişiminde late result atıl durumda kalıyor.
- `nen-ffi`: credential/HTTP adaptörleri ve tüm provider/app hataları düz,
  payload'sız `FfiDownloadError` varyantlarına eşleniyor.

### Doğrulama

- `cargo test -p nen-providers --test opensubtitles_download`: **11 passed**.
- `cargo test -p nen-app --test opensubtitles_download`: **7 passed**.
- `cargo test -p nen-ffi --test opensubtitles_download`: **2 passed**.
- `cargo test --workspace --quiet`: **çıkış 0**, tüm workspace hedefleri yeşil.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo deny check` ve `git diff --check`: **geçti**. `cargo deny` yalnız
  mevcut duplicate `hashbrown`/`syn` uyarılarını verdi; advisories, bans,
  licenses ve sources temiz.
- `bash scripts/build-macos-app.sh`: **geçti**; yeni FFI binding'i ve macOS
  uygulaması derlendi.
- `bash scripts/test.sh`: **6/6 geçti**.

### Mutasyon ölçümleri

Her değişiklik tek başına uygulanıp hedef test koşuldu ve hemen geri alındı:

| Kaldırılan kapı | Kırmızıya dönen test |
|---|---|
| Provider content-type kontrolü | `unexpected_content_type_is_rejected_before_app_attach` |
| Provider `Content-Length` kontrolü | `content_length_budget_is_enforced_before_accepting_the_body` |
| FFI `QuotaExhausted` eşlemesi yanlış varyanta çevrildi | `provider_and_application_download_errors_map_to_flat_variants` |

K23 guard'ları link, private id, credential, provider payload ve subtitle
dialogue değerlerini debug/error yüzeylerinde göstermedi; test fakes yalnız
bounded memory responses kullandı, adapter hiçbir dosya yazmadı veya archive
açmadı.
