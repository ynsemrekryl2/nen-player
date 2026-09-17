---
id: NEN-134
title: Accept final quota download and clarify hash identity events
milestone: M6
size: L
state: done
closed: 2026-09-17
depends_on: [NEN-122, NEN-131]
blocks: [NEN-126]
adr: [21, 40]
---

# NEN-134 — Accept final quota download and clarify hash identity events

## Sonuç

OpenSubtitles geçerli bir indirme linkiyle birlikte `remaining: 0` döndürdüğünde
son hak başarıyla kullanılıyor; olay günlüğündeki hash kimliği sonucu da
dosya-adı/kanonik kanıtla bulunan altyazı adaylarıyla çelişmeyecek kadar açık.

## Kapsam

- Geçerli `link` alanını kota sayacı sıfır olsa da kabul etmek; link olmayan gerçek
  kota yanıtını tipli `QuotaExhausted` olarak korumak.
- Provider fixture/unit testiyle `link + remaining: 0` regresyonunu ve link
  olmayan kota reddini ayrı ayrı kanıtlamak.
- macOS olay günlüğünde kimlik lookup sonucunu "hash kimliği" olarak
  adlandırmak; sonraki aday aramasının başka kanıtlarla başarılı olabileceğini
  yanıltıcı olmayan metinle göstermek.

## YAPILMAYACAK

- OpenSubtitles kota politikasını veya kullanıcı hesabını değiştirmek.
- Yeni kimlik/aday arama yöntemi eklemek ya da arama sırasını değiştirmek.
- Gerçek provider kredisi/kotası kullanan otomatik test yazmak.

## Kanıt (DoD)

- [x] `link + remaining: 0` yanıtı geçici linki indiriyor ve altyazı
      baytlarını döndürüyor.
- [x] Negatif: link olmayan `remaining: 0` yanıtı `QuotaExhausted` döndürüyor
      ve ikinci HTTP isteği yapmıyor.
- [x] Swift sunum testi, hash miss satırının yalnız hash kimliği sonucunu
      anlattığını ve aday arama başarısıyla çelişmediğini doğruluyor.
- [x] `cargo test -p nen-providers --test opensubtitles_download`,
      `bash scripts/test-macos.sh`, workspace kapıları ve doküman kapıları geçiyor.

## Kanıt kaydı

- `cargo test --manifest-path core/Cargo.toml -p nen-providers --test
  opensubtitles_download`: **12/12 geçti**. Pozitif
  `final_quota_download_uses_the_valid_link_before_the_remaining_count_reaches_zero`
  geçerli linki izleyip SRT baytlarını aldı; negatif
  `quota_without_a_link_is_typed_and_does_not_make_a_second_request`
  `QuotaExhausted` döndürüp istek sayısını 1'de tuttu.
- `bash scripts/test-macos.sh`: player-shell **258/258** (26 suite; hash miss +
  parsed fallback sunum beklentisi dahil), gerçek libmpv **57/57**, Keychain
  **4/4**, remote evidence **6/6** geçti.
- `cargo test --manifest-path core/Cargo.toml --workspace`, `cargo fmt
  --manifest-path core/Cargo.toml --all -- --check` ve `cargo clippy
  --manifest-path core/Cargo.toml --workspace --all-targets -- -D warnings`:
  çıkış 0.
- `cargo deny --manifest-path core/Cargo.toml check`: advisories, bans,
  licenses ve sources temiz; yalnız mevcut `hashbrown`/`syn` duplicate
  uyarıları var. `bash scripts/test.sh`: **6/6 test dosyası geçti**.
- ADR-0021 ve ADR-0040 `accepted`; `git diff --check` çıkış 0.
