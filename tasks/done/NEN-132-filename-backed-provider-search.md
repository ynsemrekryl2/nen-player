---
id: NEN-132
title: Filename-backed provider search
milestone: M6
size: L
state: done
closed: 2026-09-17
depends_on: [NEN-034, NEN-120, NEN-121, NEN-123, NEN-131]
blocks: [NEN-126]
adr: [0049]
---

# NEN-132 — Filename-backed provider search

## Sonuç

Dosya adı açıkça bir medya işaret ediyorsa, uygulama mevcut kimlik kanıtlarını
tek fan-in akışında toplar; en güvenilir sonucu önceleyerek IMDb/parent IMDb
kimliğini veya title/year/season/episode sorgusunu çıkarır ve ardından
OpenSubtitles'ta doğru aramayı yapar. Hash eşleşmesi ile kanonik kimlik,
dosya adından çıkarılan aday bilgisinden her zaman üstündür.

## Kapsam

- ADR-0009'daki tüm güvenli kanıt katmanlarını aynı medya açılışında toplamak:
  hash, Stremio handoff, `.nfo`, container metadata, dosya adı, üst klasör,
  kardeş dosya uzlaşması ve güvenli uzak URL yolu.
- Deterministik dosya adı ayrıştırıcısıyla film için title/year; dizi için
  title/season/episode ve release suffix temizleme; belirsiz sonucu aday olarak
  işaretleme.
- Kanıtları güven sırasıyla birleştirmek; daha zayıf bir sonuç daha güçlü
  kanıtı ezememeli. Uygun bağımsız kanıtlar paralel toplanabilir, fakat sonuç
  hiyerarşisi oylama yapmamalıdır.
- OpenSubtitles aramasını şu sıraya bağlamak: exact movie hash; kanonik IMDb /
  parent IMDb; ardından doğrulanmış title/year/season/episode. Hash veya
  kanonik kimlik yoksa dosya adı yalnız aday araması açar.
- Tek güçlü adayın katalogda gösterilmesi; birden fazla adayda seçim istemek.
  Kullanıcı seçmeden indirme yapılmamalı. OpenSubtitles `public_id` yalnız
  subtitle adayı, `file_id` yalnız seçilmiş indirme girdisidir; ikisi de medya
  kimliği sayılmaz.
- AI filename normalization yalnız medya başına açık izinle, yalnız
  temizlenmiş basename stem'iyle ve yalnız düşük güvenli aday ipucu olarak
  kullanılmalı; exact kimlik veya sessiz indirme kararı verememeli.
- Olay günlüğü denenen ve kazanan yöntemi yalnız bounded method/source
  adlarıyla gösterir; dosya adı, hash, URL ve kimlik payload'ı loglanmaz.

## YAPILMAYACAK

- Scraping, yeni hosted backend veya resmi OpenSubtitles API dışı arama
- URL query/fragment/host, tam yol, hash, anahtar veya altyazı metnini provider
  ya da AI isteğine göndermek
- AI çıktısını IMDb kimliğiymiş gibi doğrulamak veya belirsizliği gizlemek
- Kullanıcı seçmeden provider altyazısı indirmek
- `NEN-126` canlı kabulünü aynı task içinde tamamlamak

## Kanıt (DoD)

- [x] Dosya adı fixture'ları film title/year ve dizi title/season/episode
      olarak ayrıştırılır; release bilgisi title'a sızmaz
- [x] `.nfo`, container, handoff ve hash kanıtı için fan-in/öncelik testleri;
      güçlü kanıt zayıf dosya adı sonucunu ezer
- [x] Hash exact araması başarılıysa ikinci kimlik araması yapılmaz; hash miss
      sonrası IMDb/parent IMDb veya title/year/season/episode fallback'i kanıtlı
- [x] Stremio benzeri remote basename ve redirect/Content-Disposition kanıtı
      için pozitif; query/fragment/host ve private ID/hash sızıntısı için negatif
- [x] AI izni yokken istek yok; izin varken yalnız sanitize basename gider;
      AI sonucu ambiguity ve kullanıcı seçimi kapısını korur
- [x] Tek aday katalogda, çoklu aday seçim bekliyor, indirme çağrısı seçimden
      önce sıfır
- [x] Olay günlüğü hash/kanonik/parsed denemelerini ve kazanan yolu K23 verisi
      taşımadan gösterir
- [x] `cargo test --workspace`, `bash scripts/test-macos.sh`,
      `bash scripts/check-docs.sh`, `bash scripts/task-index.sh --check` ve
      `git diff --check` çıkış 0
- [x] ADR-0049 accepted ve `NEN-126` canlı kabulü yeniden READY

## Kanıt kaydı

2026-09-17 doğrulama kaydı:

- Ayrıştırma ve fan-in: `cargo test --workspace` çıkış 0; ilgili çıktılarda
  `nen-identity` 137 unit, `resolution_layers` 13, `nfo_and_container` 8,
  `nen-app` identity 65 unit ve `opensubtitles_candidates` 13 test geçti.
  `cargo test -p nen-providers --test opensubtitles_candidates` içinde 9 test
  geçti. Yeni testler film title/year, dizi season/episode, release suffix
  temizleme, `.nfo` kanonik kimliği, handoff parent IMDb önceliği ve hash
  hit/miss zincirini kapsıyor.
- Remote güvenliği: redirect/Content-Disposition kanıtı ve
  query/fragment/host sızıntısı negatifleri geçti; app aday testinde
  `remote_declared_name_is_used_without_query_or_host_data` geçti.
- AI ve seçim kapıları: `denied_ai_filename_permission_never_calls_provider`,
  `unknown_deterministic_identity_gets_an_untrusted_ai_suggestion` ve
  `OpenSubtitles selection (NEN-123)` içindeki seçimden önce indirme sayacı
  testleri geçti. Tek/çoklu aday kataloglama ve `downloadRequests == 0`
  negatifleri macOS test koşusunda doğrulandı.
- Olay günlüğü: `bash scripts/test-macos.sh` içinde 258 macOS shell testi,
  57 playback/contract testi ve 4 Keychain testi geçti; Pipeline event log
  suite K23 payload taşımadan hash/kanonik/parsed method adlarını doğruladı.
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets --all-features -- -D warnings`, `cargo deny check`,
  `cargo test --workspace` ve `bash scripts/test-macos.sh` çıkış 0 verdi.
  `cargo deny check` duplicate crate uyarılarıyla birlikte advisories, bans,
  licenses ve sources kontrollerini geçirdi.
- Doküman/ADR: ADR-0049 `status: accepted`; `NEN-126` backlog'da yeniden
  READY durumundadır. `bash scripts/check-docs.sh` tüm 10 denetimi, `bash
  scripts/task-index.sh --check` ve `git diff --check` çıkış 0 verdi.
