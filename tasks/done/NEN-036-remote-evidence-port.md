---
id: NEN-036
title: Remote media evidence port (HEAD, Content-Disposition, Range)
milestone: M3
size: M
state: done
depends_on: [NEN-018]
blocks: []
adr: [39]
---

# NEN-036 — Remote media evidence port (HEAD, Content-Disposition, Range)

## Sonuç

Uzak medyanın (http/https) kimlik kanıtları toplanır: sunucunun beyan ettiği
dosya adı, yönlendirme zincirinin sonu, boyut, ve `Range` ile çekilen ilk/son
64 KiB. NEN-018'in `MediaEvidence` alanlarını dolduran adapter budur.

## Kapsam

- Port trait'i (`nen-ports`) + macOS adapter
- `Content-Disposition` filename (RFC 6266 + RFC 5987 `filename*`), yoksa
  bounded redirect sonrası URL'in basename'i
- `Content-Length` + `Range: bytes=0-65535` / `bytes=-65536` → OpenSubtitles
  uyumlu hash; container byte metadata'sı ayrı `NEN-072` kapsamındadır
- Sunucu `Content-Disposition` vermediğinde / `Accept-Ranges: none` dediğinde
  akışın bozulmaması
- Deterministic fake HTTP client

**Politika kararı:** Provider API'leri HTTPS + approved-host, kullanıcı veya
Stremio medya URL'leri ise keyfi http/https hostlarıdır. Redirect bütçesi 5'tir;
her hedef yeniden doğrulanır ve HTTPS'ten HTTP'ye düşüş reddedilir
(ADR-0039).

## YAPILMAYACAK

- Medyanın kendisini indirmek — yalnız header ve iki 64 KiB pencere
- Query/fragment/host'u evidence'a koymak — **yasak** (§6, K23 #1/#2)
- Sunucu beyanı adını sanitize etmeden kullanmak
- Torrent/debrid acquisition — **non-goal**

## Kanıt (DoD)

- [x] Fake client'la `Content-Disposition`'dan ad çıkıyor; yoksa yönlendirme
      sonundaki basename kullanılıyor
- [x] `Range` ile hesaplanan hash, aynı içeriğin yerel hash'iyle birebir aynı
- [x] `Accept-Ranges: none` ve `Content-Disposition` yokluğunda akış bozulmuyor
- [x] Negatif: `../`, kontrol karakteri ve `filename*=UTF-8''…` içeren beyan
      adı yola dönüşmüyor, tek segmente indirgeniyor
- [x] Negatif: URL, query ve host hiçbir log/`Debug` çıktısında yok
- [x] Redirect sayısı sınırlı ve her adımda doğrulanıyor

## Kanıt kaydı

2026-09-06 doğrulaması:

- `nen-app::remote_evidence::tests::range_windows_produce_the_same_hash_as_local_contents`
  geçti; iki 64 KiB response hash'i `os_hash::of_bytes` ile aynı.
- `nen-app::remote_evidence::tests::redirect_basename_and_no_ranges_still_produce_evidence`
  geçti; final URL basename'i ve `Accept-Ranges: none` fallback'i doğrulandı.
- `nen-app::remote_evidence::tests::hostile_filename_is_flattened_before_identity_resolution`
  ve `nen-ffi::remote_evidence::tests::ffi_http_debug_never_prints_url_query_host_or_filename`
  geçti; traversal, kontrol karakteri, RFC 5987 filename ve redaction negatifleri
  gerçek testlerle korundu.
- `nen-app::remote_evidence::tests::unsafe_redirects_and_redirect_loops_are_rejected`
  geçti; HTTPS downgrade ve 5 redirect sınırı reddedildi.
- `bash scripts/test-macos.sh`: Swift paketi `159` test, `0` failure; yeni
  `URLSessionRemoteEvidenceClientTests` suite'i `2/2` geçti. Testler
  `URLProtocol` ile internetsiz ve deterministiktir.
- `cargo test --manifest-path core/Cargo.toml --workspace --no-fail-fast`:
  tüm workspace hedefleri geçti; yalnız mevcut lookup benchmark'ı ignored.
- `cargo fmt --all --check --manifest-path core/Cargo.toml`,
  `cargo clippy --workspace --all-targets --manifest-path core/Cargo.toml -- -D warnings`,
  `cd core && cargo deny check`, `bash scripts/test.sh` ve `.app` build/codesign
  kapıları geçti.
