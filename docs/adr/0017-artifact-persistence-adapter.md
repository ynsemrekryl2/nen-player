---
adr: 0017
title: Doğrulanmış artifact'lerin persistence adapter'ı
status: proposed
milestone: M5
tasks: [NEN-095, NEN-096, NEN-098]
date: —
---

# ADR-0017 — Doğrulanmış artifact'lerin persistence adapter'ı

## Durum

`proposed`

## Bağlam

`docs/product-spec.md` §11: "Final çeviri yalnız bellekte tutulmamalıdır…
Başlangıç için SQLite metadata index + content-addressed artifact files
**değerlendirilebilir**. Seçim ADR ile gerekçelendirilmelidir."

Bugün bu bir **aday**dır, karar değil: `docs/architecture.md` → Portlar
tablosunda `Persistence` satırı "aday: SQLite + CAS" diyor ve
`docs/DECISIONS.md` → "Ertelenmiş kararlar" bu satırı ADR-0017'ye bağlıyor.
`core/crates/nen-persist/src/lib.rs` üç satırlık boş iskelet.

Karar gerektiren noktalar:

1. **Index ve içerik nerede durur?** Tek bir veritabanı mı, yoksa metadata
   index + ayrı içerik dosyaları mı? İkincisi §11'in önerisi ama şart değil.
2. **Atomik commit nasıl sağlanır?** Yarım yazılmış bir WebVTT dosyası M5'in
   "yarım/progressive çıktı yayınlanmaz" kriterini doğrudan ihlal eder.
3. **Depo kökü nereden gelir?** `docs/architecture.md` "yol platformdan enjekte
   edilir" diyor — macOS'ta sandbox ve `ADR-0034`'ün dosya erişim kuralları
   burada geçerli.
4. **Offline garantisi.** Kullanıcı kararı (2026-09-08 — S4): kaydedilmiş bir
   artifact ağ olmadan açılıp oynatılır; ayrı bir "offline modu" anahtarı
   yoktur. Bu, okuma yolunun hiçbir ağ portuna dokunmamasını şart koşar.
5. **Ephemeral session ile persistent artifact ayrımı** (§11) hangi tarafta
   durur?
6. **Yeni bağımlılık maliyeti.** SQLite seçilirse `cargo deny` yüzeyi ve
   `NEN-043`'ün bundling yükü büyür mü?

## Karar

<!-- Kullanıcı onayıyla doldurulacak. -->

## Gerekçe

<!-- … -->

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| … | … |

## Sonuçlar

**Olumlu:** …

**Olumsuz / kabul edilen maliyet:** …

**Geri dönüş maliyeti:** …

## İlgili task'lar

`NEN-095` (karar) · `NEN-096` · `NEN-098`

## Notlar
