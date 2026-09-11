---
id: NEN-111
title: SecureCredentialStore port with contract kit and in-memory fake
milestone: M6
size: M
state: backlog
closed:
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

- [ ] Contract kiti fake ile geçiyor (`cargo test -p nen-ports -p nen-providers`)
- [ ] `nen-ffi` köprüsü: Rust'ta yazılan foreign fake üzerinden set/get/delete
      roundtrip testi
- [ ] Negatif (zorunlu, K23): sentinel anahtar hiçbir port/FFI tipinin
      `Debug`/`Display` çıktısında yok; kasıtlı `#[derive(Debug)]` ikizi
      sızdırıyor (guard sağır değil)
- [ ] Negatif: `CredentialKind`'lar birbirinin değerini döndürmüyor
- [ ] `bash scripts/build-apple.sh` binding üretimi kırılmadı; üretilen
      `nen_ffi.swift`'te `ForeignSecureCredentialStore` var
- [ ] `cargo deny check` — yeni dış bağımlılık yok (ya da gerekçesi task'ta)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
