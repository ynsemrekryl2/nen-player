---
id: NEN-104
title: Translation core acceptance
milestone: M5
size: S
state: backlog
closed:
depends_on: [NEN-102, NEN-044]
blocks: []
adr: []
---

# NEN-104 — Translation core acceptance

## Sonuç

M5'in altı çıkış kriteri gerçek `.app` üzerinde, hem sidecar hem gömülü track
kaynağıyla uçtan uca kanıtlandı.

## Bağlam

`NEN-028` (M3) ve `NEN-088` (M4) emsali: milestone kapanışı ayrı bir acceptance
task'ıyla kanıtlanır ve kanıt `evidence/M#/` altında bir checklist olarak durur.

M5 **mock provider** ile kapanır (`docs/milestones/M5-translation-core.md` →
Kapsam dışı); gerçek sağlayıcı koşusu M6'nın işidir. Kabul koşusunda gerçek
provider kredisi kullanılmaz (Kural 8).

## Kapsam

- `docs/milestones/M5-translation-core.md` → Çıkış kriterleri, altı madde tek tek
- Kabul senaryosu iki kaynakla: yanındaki sidecar ve gömülü metin track'i
- Restart sonrası cache reuse'un gerçek `.app`te görülmesi
- Milestone kapanış ritüeli: retro (süre · yanlış çıkan varsayımlar · ADR
  kararları · sonraki milestone'un task kırılımı)

## YAPILMAYACAK

- Gerçek provider ile koşu — M6
- Yeni özellik veya düzeltme — yol üstünde bulunan kusur yeni backlog task'ı olur (Kural 5)

## Kanıt (DoD)

- [ ] Kabul checklist'i `evidence/M5/NEN-104-checklist.md`, altı kriterin her biri için ayrı adım ve gözlem
- [ ] Sidecar kaynağıyla uçtan uca koşu: seçim → komut → doğrulanmış artifact → menüde `Ai` kaynağı
- [ ] Gömülü metin track'i kaynağıyla aynı koşu (`NEN-044`'ün çıkarımı üzerinden)
- [ ] Restart sonrası aynı çeviri **provider çağrılmadan** açılıyor
- [ ] Negatif: iptal edilen koşu ne menüde ne diskte iz bırakıyor
- [ ] Negatif: log taraması — cue metni, medya URL'si, özel yol için eşleşme **0**
- [ ] `bash scripts/test.sh`, `bash scripts/check-docs.sh`, workspace ve macOS kapıları yeşil
- [ ] M5 retro'su `docs/milestones/M5-translation-core.md`'ye yazıldı

## Kanıt kaydı

<!-- done olurken doldurulacak -->
