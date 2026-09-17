---
id: NEN-133
title: Tolerate unsupported remote ranges
milestone: M6
size: S
state: done
closed: 2026-09-17
depends_on: [NEN-132]
blocks: [NEN-126]
adr: []
---

# NEN-133 — Tolerate unsupported remote ranges

## Sonuç

Stremio benzeri bir uzak medya sunucusu bounded `Range` penceresini
sağlayamadığında hash kanıtı atlanır; güvenli dosya adı/path kanıtı korunur ve
OpenSubtitles aday araması `RemoteResponseTooLarge` ile kesilmez.

## Kapsam

- Bounded range cevabı yok sayılırsa veya boyut sınırını aşarsa bunu
  desteklenmeyen hash kanıtı olarak ele alıp filename/path fallback'ine devam
  etmek.
- Remote basename ile provider aday aramasının bu koşulda çalıştığını
  deterministik testle kanıtlamak.

## YAPILMAYACAK

- Response body sınırını büyütmek veya medyanın tamamını indirmek.
- Provider cevaplarındaki `ResponseTooLarge` güvenlik reddini yumuşatmak.
- Stremio URL query/fragment/host bilgisini kimlik kanıtına katmak.

## Kanıt (DoD)

- [x] Bounded GET'i yok sayan sunucuda hash atlanır ve ikinci pencere
      istenmeden filename/path kanıtı korunur.
- [x] Bounded GET boyut sınırını aşan sunucuda hash atlanır,
      `RemoteResponseTooLarge` tüm kanıt toplamayı kesmez.
- [x] Remote filename fallback'i OpenSubtitles parsed aday sorgusuna ulaşır.
- [x] Boyut sınırı, URL/query/host redaction ve provider response cap
      regresyon testleri geçer.
- [x] `cargo test --workspace`, `bash scripts/test-macos.sh`,
      `bash scripts/check-docs.sh`, `bash scripts/task-index.sh --check` ve
      `git diff --check` çıkış 0.

## Kanıt kaydı

2026-09-17 doğrulama kaydı:

- `cargo test -p nen-app --lib remote_evidence` içinde 12/12 geçti.
  `ignored_range_keeps_declared_name_and_stops_after_the_first_window`, 200 ile
  range'i yok sayan sunucuda ikinci pencereyi istemeden adı korudu;
  `oversized_advertised_range_is_an_optional_hash_miss`, tipli
  `ResponseTooLarge` sonucunu hash miss'e indirgedi.
- `cargo test -p nen-app --test opensubtitles_candidates
  remote_declared_name_is_used_without_query_or_host_data` 1/1 geçti; başarısız
  bounded pencere sonrasında `ParsedIdentity` araması title/year ile yapıldı,
  URL host/query sentinel'ları provider isteğine girmedi.
- `cargo test --workspace` çıkış 0; `cargo fmt --all --check` ve `cargo clippy
  --workspace --all-targets --all-features -- -D warnings` çıkış 0 verdi.
  `cargo deny check`, mevcut duplicate uyarılarıyla advisories, bans, licenses
  ve sources kapılarının tümünü geçti.
- `bash scripts/test-macos.sh` çıkış 0; Swift shell, gerçek libmpv/contract ve
  Keychain testleri geçti. `URLSessionRemoteEvidenceClientTests` içindeki
  response-cap negatifi de yeşil kaldı.
- `bash scripts/test.sh` 6/6 test dosyasını geçirdi. `bash
  scripts/check-docs.sh`, `bash scripts/task-index.sh --check` ve `git diff
  --check` çıkış 0 verdi.
