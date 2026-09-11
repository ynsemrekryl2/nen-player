---
id: NEN-119
title: Decide the OpenSubtitles integration boundaries
milestone: M6
size: S
state: backlog
closed:
depends_on: [NEN-033]
blocks: [NEN-121]
adr: [21]
---

# NEN-119 — Decide the OpenSubtitles integration boundaries

## Sonuç

ADR-0021 `accepted`: OpenSubtitles adaylarının nasıl aranacağı, katalogda
hangi kimlikle duracağı (opaque public ID), seçimde indirmenin hangi güvenlik
kapılarından geçeceği ve kota/dil davranışı karara bağlanmış.

## Bağlam

ADR-0040 (`NEN-033`) yalnız **hash kimlik sorgusunu** sınırladı ve açıkça
"indirme, credential storage veya UI davranışı kararı vermez" dedi. Şartname
§7'nin OpenSubtitles paragrafı (yalnız resmi API · metadata kataloglama ·
seçilmeden indirme yok · approved host · bounded redirect · archive reddi ·
maksimum boyut · opaque public source ID · private file ID/hash/filename
sızıntısı yok) ve `security-policy.md` §3 kuralları var; **sınırın kendisi**
— hangi endpoint'ler, hangi kimlik alanı public, hangi sayılar — yok.
`SubtitleSourceKind::OpenSubtitles` ve `SubtitleSourceId::opensubtitles(...)`
`nen-domain`'de M2'den beri duruyor, üreticisi yok.

## Kapsam

ADR-0021 en az şunları kararlaştırır:

- **Arama stratejisi:** `moviehash` eşleşmesi (mevcut `NEN-033` sorgusunun
  altyazı kayıtları) → yoksa doğrulanmış kimlikten `query`/`imdb_id`/
  `season`+`episode` araması (`NEN-120`'nin `VerifiedMediaIdentity`'si);
  filename tabanlı arama hangi koşulda (`NEN-034`/`NEN-035` ile ilişki)
- **Opaque public ID:** katalogdaki kimlik OpenSubtitles'ın public
  `subtitle id`'si mi, kendi türettiğimiz bir opaque token mı; private
  `file_id`'nin core dışına (FFI, log, `Debug`) **hiç** çıkmaması; indirme
  için gereken `file_id`'nin nerede tutulduğu
- **Kataloglama:** lazy — yalnız metadata; dil (ISO 639 → `LanguageTag`,
  ADR-0032 emsali), sürüm/release adı gösterimi, `hearing_impaired`/`ai_translated`
  gibi bayrakların rozete etkisi (ADR-0035'in kapalı etiket kümesi genişler
  mi?), aday sayısı üst sınırı
- **Seçimde indirme:** `download` endpoint'i → geçici link → GET; kota
  (`remaining`/`reset_time`) davranışı — kota bitince tipli hata, sessiz
  bekleme yok
- **İndirme güvenliği (sayılarla):** approved host listesi (api/vip-api +
  link host'ları), redirect üst sınırı, boyut sınırı (`NEN-025`'in kullanıcı
  dosyası sınırıyla aynı mı), archive reddi (magic bytes + content-type),
  içerik → mevcut encoding + strict SRT kapıları (`security-policy.md` §4)
- **Dil filtresi:** tercih dilleri (`NEN-037`) sorguya girer mi, tümü gelip
  katalogda gruplanır mı
- Kullanıcı anahtarı + `User-Agent` politikası ADR-0040'la aynı; anahtar
  `NEN-111` portundan

## YAPILMAYACAK

- Kod — `NEN-120`, `NEN-121`, `NEN-122`, `NEN-123`
- Otomatik indirme sözleşmesi — `NEN-125` (ADR-0047)
- Scraping, login/oturum akışı (yalnız API key) — **yasak**
- Altyazı **yükleme** (upload) — non-goal

## Kanıt (DoD)

- [ ] `docs/adr/0021-*.md` yazıldı, kullanıcı onayıyla `accepted`
- [ ] `docs/adr/README.md`, `docs/DECISIONS.md` sayacı ve `docs/security-policy.md`
      §3'ün sayıları (redirect, boyut) ADR ile tutarlı
- [ ] ADR-0035'e (etiket kümesi) ve ADR-0010'a (katalog) gerekiyorsa Notlar
      girdisi
- [ ] `bash scripts/check-docs.sh` çıkış 0

## Kanıt kaydı

<!-- done olurken doldurulacak -->
