---
id: NEN-084
title: Stremio handoff acceptance
milestone: M4
size: S
state: done
closed: 2026-09-08
depends_on: [NEN-081, NEN-082, NEN-083]
blocks: []
adr: []
---

# NEN-084 — Stremio handoff acceptance

## Sonuç

M4'ün üç çıkış kriteri gerçek Stremio'nun ölçülmüş CLI sözleşmesi ve gerçek
`.app` ile, sentetik fixture üzerinden uçtan uca kanıtlanmıştır.

## Bağlam

`NEN-028`'in emsali: kabul senaryosu milestone dokümanına yazılır, gerçek
üründe baştan sona koşulur ve koşu testlerin **yerine** değil yanına konur.
`NEN-028` bu koşuyu yaparken menüye girmeyen bir altyazı kusuru bulmuştu —
kabul koşusunun asıl değeri budur.

## Kapsam

- Kabul senaryosunun `docs/milestones/M4-stremio-macos.md`'ye yazılması
- Gerçek Stremio 5.1.26'te gönderim biçiminin yeniden doğrulanması ve aynı
  sözleşmeyle gerçek `.app` koşusu, ekran kanıtıyla
- Üç çıkış kriterinin tek tek işaretlenmesi
- Koşuda bulunan kusurların ya düzeltilmesi ya da numaralı task'a ayrılması

## YAPILMAYACAK

- Yeni özellik eklemek — kabul, var olanı kanıtlar
- M4'ü biçimsel kapatmak (retro + roadmap durumu) — ayrı ve kasıtlı bir adım,
  `NEN-028`/M3 emsalindeki gibi bu task'ın kapsamı değil
- Depoya gerçek medya koymak — yalnız `fixtures/media/*` kullanılır (ADR-0031
  Karar 2)
- Ekran kanıtına gerçek medya URL'si, port veya özel yol düşürmek — K23

## Kanıt (DoD)

- [x] `evidence/M4/NEN-084-checklist.md`: adım listesi + ekran kanıtı
- [x] Kriter 1 — Stremio'nun ölçülmüş CLI sözleşmesiyle açılan medya doğru
      pozisyondan oynuyor
- [x] Kriter 2 — argüman ve medya URL'si loglanmıyor (koşu sırasında log
      yüzeyi denetlenir; otomatik dayanağı `NEN-083`)
- [x] Kriter 3 — handoff metadata'sı yokken akış bozulmuyor
- [x] Tam regresyon: Rust workspace · `bash scripts/test-macos.sh` ·
      `.app` build + strict codesign · `bash scripts/test.sh` ·
      `bash scripts/check-docs.sh` yeşil

## Kanıt kaydı

**2026-09-08, bu makinede.** Kabul adımları ve üç kriterin sonuçları
[`evidence/M4/NEN-084-checklist.md`](../../evidence/M4/NEN-084-checklist.md)
ile kaydedildi. Gerçek Stremio 5.1.26'ın canlı harici oynatıcı menüsü yeniden
görüldü; ADR-0043 sınırı nedeniyle doğrudan Nen Player hedefi iddia edilmedi.
Ölçülmüş argv sözleşmesi gerçek `.app` üzerinde `contract-clip.mkv` ile
çalıştırıldı: **00:12 / 00:30**, oynatma açık. Metadata'sız opak loopback
fixture **00:09 / 00:30** konumunda oynadı; hata veya kesinti oluşmadı.

Unified log ve stdout/stderr negatif taramasında locator/query, özel yol ve
argv parçalarının tamamı için eşleşme sayısı **0** oldu; ham kayıtlar silindi.
Kanıt ekranları yalnız sentetik fixture içerir ve gerçek URL, token, port veya
özel tam yol taşımaz.

Tam doğrulama:

- `cargo test --manifest-path core/Cargo.toml --workspace --no-fail-fast` — yeşil
- `cargo fmt --all --check --manifest-path core/Cargo.toml` — yeşil
- `cargo clippy --manifest-path core/Cargo.toml --workspace --all-targets -- -D warnings` — yeşil
- `cd core && cargo deny check` — advisories/bans/licenses/sources yeşil
- `bash scripts/test-macos.sh` — **239 test, 0 failure**
- `bash scripts/build-macos-app.sh` — çıkış 0
- `codesign --verify --deep --strict` — valid on disk; designated requirement sağlandı
- `bash scripts/test.sh` ve `bash scripts/check-docs.sh` — kapanış doğrulamasında yeşil

## Düzeltme kaydı — 2026-09-08 (`NEN-086`)

Bu task'ın kanıtı Nen Player'ın ölçülmüş MPV argv sözleşmesini gerçek `.app`
üzerinde doğrular; gerçek Stremio'nun Nen Player bundle'ını doğrudan
hedeflediğini doğrulamaz. Stremio'nun çalışan yolu, sabit MPV executable
listesindeki mevcut `/usr/local/bin/mpv` wrapper'ıdır. Bu nedenle mevcut kabul
kanıtı tarihsel olarak korunur, ancak M4'ün gerçek Stremio → Nen Player çıkış
kriterini tek başına tamamlamaz. Geri alınabilir bridge kurulumu `NEN-087`,
gerçek Stremio kabulü `NEN-088` kapsamındadır.
