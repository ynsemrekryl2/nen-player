---
id: NEN-122
title: Safe download on selection
milestone: M6
size: M
state: backlog
closed:
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

- [ ] Unit: fixture link + gövde ile belge kataloğa takılıyor; ikinci seçim
      `send` sayacını artırmıyor
- [ ] Negatif (zorunlu — her biri ayrı test, her biri `send`/diske yazma
      **0** ve katalogda belge **yok**): liste dışı host · `http://` link ·
      redirect sınırı aşımı · redirect zincirinde liste dışı host · boyut
      aşımı (`Content-Length` ve gövde ayrı) · zip magic bytes · yanlış
      content-type · SRT parse hatası · kota bitmiş cevap
- [ ] Negatif (K23): link URL'si, private `file_id`, anahtar hata tiplerinin
      ve `Debug`'ların hiçbirinde yok
- [ ] Negatif: medya değiştikten sonra gelen belge takılmıyor
- [ ] Mutasyon: en az iki kapı elle kaldırılıp tam olarak beklenen test(ler)
      kırmızıya dönüyor (`NEN-096` emsali) — kayıtta
- [ ] `nen-ffi` hata eşlemesi mutasyon testi

## Kanıt kaydı

<!-- done olurken doldurulacak -->
