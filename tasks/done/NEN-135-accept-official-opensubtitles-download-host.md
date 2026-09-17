---
id: NEN-135
title: Accept official OpenSubtitles download host
milestone: M6
size: S
state: done
closed: 2026-09-17
depends_on: [NEN-134]
blocks: [NEN-126]
adr: [50]
---

# NEN-135 — Accept official OpenSubtitles download host

## Sonuç

OpenSubtitles'ın resmî `www.opensubtitles.com/download/...` geçici linki,
mevcut bounded indirme güvenliği korunarak altyazıyı indiriyor.

## Kapsam

- `www.opensubtitles.com` hostunu yalnız HTTPS ve `/download/` path'i için
  OpenSubtitles geçici link allowlist'ine eklemek.
- İlk geçici link ve redirect hop'unda `www` hostunu ayrı pozitif provider
  contract testleriyle kanıtlamak.
- Benzer görünen alt alan adı, farklı path, HTTP ve liste dışı redirect'in
  istekten önce reddedildiğini negatif testlerle korumak.
- Credential header'ın metadata isteğinden geçici link GET'ine taşınmadığını
  mevcut testle birlikte doğrulamak.

## YAPILMAYACAK

- Wildcard `*.opensubtitles.com` veya registrable-domain allowlist.
- Redirect'leri platform HTTP istemcisine otomatik izletmek.
- Kota, aday arama, private `file_id` ya da UI hata taksonomisini değiştirmek.
- Gerçek provider kotası kullanan otomatik test.

## Kanıt (DoD)

- [x] Provider contract: ilk `www.opensubtitles.com/download/...` linki SRT
      baytlarını döndürüyor.
- [x] Provider contract: approved bir hosttan `www` download path'ine redirect
      en fazla beş hop kuralıyla izleniyor.
- [x] Negatif: `www` üzerinde `/download/` dışı path, benzer alt alan adı,
      HTTP ve liste dışı redirect reddediliyor; reddedilen hedefe istek yok.
- [x] Negatif: geçici link GET'inde `Api-Key` header'ı yok.
- [x] `cargo test -p nen-providers --test opensubtitles_download`, workspace
      test/fmt/clippy/deny ve doküman kapıları geçiyor.

## Kanıt kaydı

- `nen-providers::opensubtitles`, `www.opensubtitles.com` hostunu yalnız tam
  host eşleşmesi, HTTPS ve boş olmayan `/download/` path'iyle kabul ediyor;
  mevcut üç host, beş redirect, 10 MiB, content-type ve archive kapıları
  değişmedi.
- `cargo test --manifest-path core/Cargo.toml -p nen-providers --test
  opensubtitles_download`: **17/17 geçti**. İlk `www` linki ve approved
  hosttan `www` redirect'i SRT baytlarını döndürdü; yanlış path, lookalike
  host, HTTP downgrade ve liste dışı redirect hedefe istek atmadan reddedildi.
  Geçici link GET'inde `Api-Key` bulunmadığı ayrı assertion ile doğrulandı.
- `cargo test --manifest-path core/Cargo.toml --workspace`: çıkış 0.
- `cargo fmt --manifest-path core/Cargo.toml --all -- --check` ve `cargo
  clippy --manifest-path core/Cargo.toml --workspace --all-targets -- -D
  warnings`: çıkış 0.
- `cargo deny --manifest-path core/Cargo.toml check`: advisories, bans,
  licenses ve sources temiz; yalnız mevcut `hashbrown`/`syn` duplicate
  uyarıları var.
- `bash scripts/test.sh`: **6/6 test dosyası geçti**.
