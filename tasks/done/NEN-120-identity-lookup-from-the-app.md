---
id: NEN-120
title: Verified identity lookup runs from the app
milestone: M6
size: M
state: done
closed: 2026-09-14
depends_on: [NEN-033, NEN-111]
blocks: [NEN-064, NEN-121]
adr: [40]
---

# NEN-120 — Verified identity lookup runs from the app

## Sonuç

Bir medya açıldığında `nen-app`, `NEN-033`'ün OpenSubtitles hash sorgusunu
playback'i bloklamadan, anahtarı credential store'dan alarak çalıştırıyor ve
sonucu `nen-ffi` üzerinden `VerifiedMediaIdentity` olarak kabuğa veriyor;
anahtar yoksa veya hash yoksa hiçbir istek çıkmıyor.

## Bağlam

`nen-app::identity::apply_provider_identity` ve
`nen-providers::opensubtitles::OpenSubtitlesIdentityLookup` `NEN-033`'te
kapandı ama hiçbir çağıranı yok: `nen-ffi`'de kimlik çıkışı, macOS'ta anahtar
yok. `NEN-064` (kimlik şeridi) bu yüzden bugün başlatılamaz. ADR-0040 "API
çağrısını playback akışına bağlamak" alternatifini reddetti — sorgu yardımcı
evidence'dır, oynatmayı bekletmez.

## Kapsam

- `nen-app`: medya evidence toplandıktan sonra (`NEN-018` `os_hash`,
  `NEN-036` remote evidence) ayrı bir worker'da `apply_provider_identity`;
  `mediaPresentationRevision` emsali bir revision guard — medya değiştiyse
  geç sonuç yok sayılır
- `HttpClient` ve `SecureCredentialStore` `nen-ffi` üzerinden kabuktan
  enjekte (mevcut `collect_remote_evidence` deseni)
- `nen-ffi`: `FfiVerifiedMediaIdentity { title, year, season, episode }`
  çıkışı (ADR-0033 push/pull kararına uygun yol); `Debug` elle, hash/anahtar
  yok
- Anahtar yoksa: sorgu **yapılmaz**, tipli `NoCredential` sonucu (hata değil
  — kimlik yalnız evidence)
- Sonucun `MediaEvidence::resolve` üzerinden kataloğa/menüye etkisi
  `NEN-033`'te zaten var; burada yalnız çağrı yolu açılır

## YAPILMAYACAK

- Kabuk şeridi/UI — `NEN-064`
- Altyazı aday araması — `NEN-121`
- Sorgu sonucunu playback'i bekletmek için kullanmak — ADR-0040 reddetti
- Anahtarı Rust tarafında saklamak — `NEN-111` portu her seferinde sorulur
  (bellekte kısa)

## Kanıt (DoD)

- [x] Unit (`nen-app`): fake credential store + `FakeMediaIdentityLookup` ile
      `Match` sonucu evidence'a giriyor; medya revision değiştiğinde geç sonuç
      **uygulanmıyor**
- [x] Negatif (zorunlu): anahtar yokken lookup `calls() == 0`; hash yokken
      `calls() == 0` (`NEN-033`'ün mevcut testi bu yolda da geçerli)
- [x] `nen-ffi` testi: sorgu sonucu FFI kaydına eşleniyor; K23 guard —
      `FfiVerifiedMediaIdentity` ve hata tiplerinin `Debug`'ında hash/anahtar
      sentinel'ı yok
- [x] Negatif: sorgunun `Transport` hatası playback oturumunu etkilemiyor
      (session testi — mevcut `PlaybackSession` üzerinde)
- [x] `bash scripts/build-apple.sh` binding'de yeni kayıt var

## Kanıt kaydı

2026-09-14:
- `nen-app` unit testleri fake credential/provider ile exact match, missing
  credential/hash ve evidence fallback davranışlarını doğruladı; 58/58 geçti.
- `nen-ffi/tests/identity_lookup.rs` FFI eşlemesini, typed `NoCredential`,
  erken hash kapısını ve K23 Debug redaction guard'ını doğruladı; 4/4 geçti.
- macOS shell testleri lookup'ın playback'i bekletmeden başlattığını, eski
  revision cevabını atladığını ve transport hatasında playback'i bozmadığını
  doğruladı; `bash scripts/test-macos.sh` 287 test / 35 suite ile geçti.
- `bash scripts/build-apple.sh` yeni `FfiVerifiedMediaIdentity` kaydını ve
  `lookupVerifiedIdentityByHash` binding'ini üretti.
- Kapılar: `cargo test --workspace`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`,
  `bash scripts/test.sh` (6/6), `bash scripts/check-docs.sh` (10/10) ve
  `git diff --check` başarılı. `cargo deny` yalnız mevcut duplicate crate
  uyarılarını raporladı; advisories/bans/licenses/sources geçti.
