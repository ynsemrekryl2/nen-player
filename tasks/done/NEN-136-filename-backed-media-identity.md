---
id: NEN-136
title: Filename-backed media identity fallback
milestone: M6
size: L
state: done
closed: 2026-09-17
depends_on: [NEN-132, NEN-135]
blocks: [NEN-126]
adr: [9, 49]
---

# NEN-136 — Filename-backed media identity fallback

## Sonuç

Kesin hash eşleşmesi bulunamadığında, açık ve tekil dosya adı kanıtı medya
kimliği aramasına düşer; doğru provider sonucu oyuncu şeridine ulaşır.

## Kapsam

- Exact hash sorgusunu ilk ve üstün yol olarak korumak; miss sonrasında yerel
  dosya adı, uzak beyan edilen ad ve güvenli ayrıştırılmış title/year/
  season/episode ile provider medya kimliği araması yapmak.
- Provider arama cevabındaki tekil `feature_details` kaydını medya kimliği
  olarak eşlemek; subtitle `public_id` veya `file_id` değerlerini medya kimliği
  olarak kullanmamak.
- Core → FFI → macOS açılış zincirini bounded evidence ile bağlamak; mevcut
  playback, credential ve K23 redaction kapılarını korumak.
- Screenshot'taki release biçimini ve hash miss → filename identity hit
  zincirini deterministic fixture/test ile kanıtlamak.

## YAPILMAYACAK

- Hash eşleşmesinin önceliğini değiştirmek veya belirsiz provider sonuçlarını
  doğrulanmış kimlik diye göstermek.
- AI filename normalization, scraping, görsel/ses tanıma veya teknik ID girişi.
- Subtitle adayı seçimini, indirmeyi veya otomatik indirme politikasını
  yeniden tasarlamak.

## Kanıt (DoD)

- [x] Release-style film adı title/year olarak provider kimliğine düşüyor;
      release suffix ve özel dosya yolu query'ye sızmıyor.
- [x] Negatif: hash hit filename fallback'i çalıştırmıyor; ambiguous/no-match
      basename fallback'ini koruyor; subtitle ID'leri medya kimliği olmuyor.
- [x] FFI ve macOS regression testi, media identity'nin subtitle adaylarından
      bağımsız olarak şeride ulaştığını gösteriyor.
- [x] `cargo test --workspace`, `bash scripts/test-macos.sh`,
      `bash scripts/check-docs.sh`, `bash scripts/task-index.sh --check` ve
      `git diff --check` çıkış 0.

## Kanıt kaydı

- `cargo test --workspace`: exit 0; workspace testleri ve doc-test'ler geçti.
- `cargo test -p nen-app -p nen-ffi -p nen-providers -p nen-identity`: exit 0;
  uygulama, FFI, provider ve release-name regression'ları geçti.
- `cargo test -p nen-identity --test release_name_golden`: 4/4 geçti;
  screenshot release adı fixture'ı dahil.
- `cargo test -p nen-providers --test opensubtitles_lookup parsed_filename_query`:
  2/2 geçti; tekil feature eşleşmesi ve farklı feature belirsizliği kanıtlandı.
- `bash scripts/test-macos.sh`: exit 0; 259 shell/player, 57 playback/contract
  ve 4 Keychain testi geçti. `parsedFilenameIdentityIsDisplayOnly` geçti.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  exit 0.
- `cargo deny check`: exit 0; `advisories ok, bans ok, licenses ok, sources ok`.
- `bash scripts/test.sh`: exit 0; 6 test dosyasının hepsi geçti.
- `bash scripts/check-docs.sh`, `bash scripts/task-index.sh --check` ve
  `git diff --check`: exit 0.
- ADR-0009 ve ADR-0049: `status: accepted` doğrulandı.
