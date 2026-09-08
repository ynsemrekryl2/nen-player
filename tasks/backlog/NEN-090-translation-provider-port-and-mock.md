---
id: NEN-090
title: Translation provider port and deterministic mock
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-089]
blocks: [NEN-091]
adr: []
---

# NEN-090 — Translation provider port and deterministic mock

## Sonuç

Bir çeviri sağlayıcısı `nen-ports` içinde tanımlı tek bir port arkasında duruyor
ve deterministic mock bu portun contract kitini geçiyor.

## Bağlam

M5 **yalnız mock provider** ile tamamlanır; gerçek sağlayıcılar M6'da
(`docs/milestones/M5-translation-core.md` → Kapsam dışı). Kural 8 gereği testler
gerçek provider kredisi kullanamaz, dolayısıyla mock isteğe bağlı bir kolaylık
değil, M5'in tek çalışan implementasyonudur.

`docs/architecture.md` → Portlar tablosunda çeviri sağlayıcısı için bir satır
**yok**; bu task onu ekler. Emsal: `nen-ports/src/playback` ve
`nen-ports/src/renderer` — trait + paylaşılan contract kiti + fake, aynı dizinde.

## Kapsam

- `nen-ports` içinde `TranslationProvider` trait'i ve paylaşılan contract kiti
- `nen-providers` içinde deterministic mock (aynı girdi → aynı çıktı, ağ yok)
- İptal ve ilerleme sözleşmesi — ADR-0004'ün late-commit yasağıyla uyumlu
- Transient hata sınıfının porta girmesi (bounded retry'ın **politikası** core'da)
- `docs/architecture.md` Portlar tablosuna satır eklenmesi

## YAPILMAYACAK

- OpenAI / OpenRouter / gerçek HTTP — M6 (`ADR-0019`)
- Capability preflight — M6
- Cevabın doğrulanması — `NEN-091`; port cevabı **untrusted** teslim eder
- Secure credential storage — M6 (`ADR-0020`)

## Kanıt (DoD)

- [ ] Contract kiti mock adapter'da geçiyor (test adı + `cargo test -p nen-providers` çıktısı)
- [ ] Unit: aynı istek iki kez çağrıldığında mock birebir aynı cevabı veriyor
- [ ] Negatif: iptal edilen çağrı **sonuç teslim etmiyor** (late commit yok)
- [ ] Negatif: kit, bozuk/eksik cevap veren kasıtlı bir `Defect` adapter'ında kırmızıya dönüyor — kit sağır değil
- [ ] Guard: provider isteği ve cevabı hiçbir log yüzeyine düşmüyor (K23 #4, #5)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
