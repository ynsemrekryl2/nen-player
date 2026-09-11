---
id: NEN-111
title: SecureCredentialStore port with contract kit and in-memory fake
milestone: M6
size: M
state: done
closed: 2026-09-11
depends_on: [NEN-110]
blocks: [NEN-112, NEN-116, NEN-120]
adr: [20]
---

# NEN-111 — SecureCredentialStore port with contract kit and in-memory fake

## Sonuç

`nen-ports::credentials` portu, contract test kiti ve deterministic in-memory
fake'i var; `nen-ffi` `ForeignSecureCredentialStore` ile bir platform
adapter'ını core'a bağlayabiliyor; anahtar taşıyan hiçbir tip `Debug`/`Display`
ile değerini sızdırmıyor.

## Kapsam

- ADR-0020'nin port tanımı: kapalı `CredentialKind` enum'u (`OpenSubtitles` ·
  `OpenAi` · `OpenRouter`), `get` · `set` · `delete`, payload'sız tipli hata
  (`Unavailable` · `Denied` · `Corrupt` gibi — ADR'nin listesi)
- Secret newtype: `OpenSubtitlesApiKey`'in emsali (trim, uzunluk, kontrol
  karakteri reddi; `Debug`/`Display` → `<redacted>`), üç sağlayıcı için tek
  tip ya da kind'a bağlı — ADR-0020'nin kararı
- Contract kiti (`nen-ports`'un `PlaybackEngine`/`ArtifactStore` kitleri
  emsali): set→get birebir, delete→get yok, kind'lar birbirine karışmıyor
- `InMemoryCredentialStore` fake (`nen-providers` veya `nen-ports`, ADR'ye
  göre) — testlerin tek deposu
- `nen-ffi`: `#[uniffi::export(with_foreign)] trait ForeignSecureCredentialStore`
  + Rust `SecureCredentialStore`'a köprü (`ForeignHttpClient` deseni);
  `FfiCredentialError` düz, payload'sız
- K23 guard testi: fake'e sentinel anahtar konup port tiplerinin ve FFI
  hatalarının `Debug`/`Display` çıktısı taranıyor (`guard_ffi_translation_debug.rs`
  emsali)

## YAPILMAYACAK

- Gerçek Keychain — `NEN-112`
- Ayarlar UI'ı — `NEN-113`
- Anahtarın çeviri/kimlik akışına bağlanması — `NEN-116`, `NEN-118`, `NEN-120`
- Anahtarı dosyaya/`UserDefaults`'a yazan herhangi bir fallback — **yasak**

## Kanıt (DoD)

- [x] Contract kiti fake ile geçiyor (`cargo test -p nen-ports -p nen-providers`)
- [x] `nen-ffi` köprüsü: Rust'ta yazılan foreign fake üzerinden set/get/delete
      roundtrip testi
- [x] Negatif (zorunlu, K23): sentinel anahtar hiçbir port/FFI tipinin
      `Debug`/`Display` çıktısında yok; kasıtlı `#[derive(Debug)]` ikizi
      sızdırıyor (guard sağır değil)
- [x] Negatif: `CredentialKind`'lar birbirinin değerini döndürmüyor
- [x] `bash scripts/build-apple.sh` binding üretimi kırılmadı; üretilen
      `nen_ffi.swift`'te `ForeignSecureCredentialStore` var
- [x] `cargo deny check` — yeni dış bağımlılık yok (ya da gerekçesi task'ta)

## Kanıt kaydı

**Doğrulanmış kapanış, 2026-09-11.**

- `cargo test -p nen-ports -p nen-providers -p nen-ffi` geçti; credentials
  unit/contract, foreign fake roundtrip ve K23 guard testleri yeşil.
- `cargo test --workspace --quiet` geçti; tüm workspace suite'leri yeşil,
  yalnız önceden var olan `lookup_bench` baseline testi ignored kaldı.
- `cargo fmt --all -- --check` ve `cargo clippy --workspace --all-targets -- -D warnings`
  geçti.
- `cargo deny check` geçti (`advisories/bans/licenses/sources ok`); yeni dış
  bağımlılık eklenmedi. Mevcut duplicate `hashbrown`/`syn` uyarıları yalnız
  bilgi seviyesinde kaldı.
- `bash scripts/build-apple.sh` geçti. Üretilen Swift binding'de
  `ForeignSecureCredentialStore`, `FfiSecureCredentialStore` ve yalnız
  `contains`/`set`/`delete` UI yüzeyi doğrulandı; foreign `get` yalnız Rust
  adapter'ında kaldı.
- `bash scripts/test.sh` geçti (4/4 script suite). K23 sentinel guard'ı
  kasıtlı derived-debug ikiziyle sağır olmadığını da doğruladı.
