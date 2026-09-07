---
id: NEN-084
title: Stremio handoff acceptance
milestone: M4
size: S
state: backlog
closed:
depends_on: [NEN-081, NEN-082, NEN-083]
blocks: []
adr: []
---

# NEN-084 — Stremio handoff acceptance

## Sonuç

M4'ün üç çıkış kriteri gerçek Stremio ve gerçek `.app` ile, uçtan uca
kanıtlanmıştır.

## Bağlam

`NEN-028`'in emsali: kabul senaryosu milestone dokümanına yazılır, gerçek
üründe baştan sona koşulur ve koşu testlerin **yerine** değil yanına konur.
`NEN-028` bu koşuyu yaparken menüye girmeyen bir altyazı kusuru bulmuştu —
kabul koşusunun asıl değeri budur.

## Kapsam

- Kabul senaryosunun `docs/milestones/M4-stremio-macos.md`'ye yazılması
- Gerçek Stremio 5.1.26 → gerçek `.app` koşusu, ekran kanıtıyla
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

- [ ] `evidence/M4/NEN-084-checklist.md`: adım listesi + ekran kanıtı
- [ ] Kriter 1 — Stremio'dan açılan medya doğru pozisyondan oynuyor
- [ ] Kriter 2 — argüman ve medya URL'si loglanmıyor (koşu sırasında log
      yüzeyi denetlenir; otomatik dayanağı `NEN-083`)
- [ ] Kriter 3 — handoff metadata'sı yokken akış bozulmuyor
- [ ] Tam regresyon: Rust workspace · `bash scripts/test-macos.sh` ·
      `.app` build + strict codesign · `bash scripts/test.sh` ·
      `bash scripts/check-docs.sh` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
