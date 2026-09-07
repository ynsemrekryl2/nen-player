---
id: NEN-001
title: Repository skeleton
milestone: M0
size: S
state: done
closed: 2026-08-24
depends_on: []
blocks: [NEN-002, NEN-003, NEN-004, NEN-006]
adr: []
---

# NEN-001 — Repository skeleton

## Sonuç

Repo, `docs/architecture.md`'de tanımlı dizin yapısına ve temel proje
dosyalarına sahiptir; yeni gelen biri nereye ne koyacağını dosya ağacından
anlayabilir.

## Kapsam

- Dizin ağacı: `docs/ tasks/ core/ platforms/ fixtures/ scripts/`
- `.gitignore` (Rust, Xcode, Gradle, macOS, secrets, generated bindings)
- `.editorconfig`
- `README.md` — giriş, mimari özet, "nereden başlamalı" tablosu
- `CLAUDE.md` — çalışma protokolü, değişmez kurallar, commit formatı
- `LICENSE.md` — lisans kararının **neden ertelendiği**
- `docs/product-spec.md`, `docs/roadmap.md`, `docs/architecture.md`,
  `docs/glossary.md`, `docs/security-policy.md`, `docs/testing-strategy.md`
- Boş dizinlerde `.gitkeep`

## YAPILMAYACAK

- Lisans metni seçimi — S1 (dağıtım modeli) ve ADR-0012 (libmpv linkleme)
  kararlarına bağlı, M3 öncesi verilecek
- Cargo/Gradle/Xcode proje dosyaları → M1
- Herhangi bir ürün kodu

## Kanıt (DoD)

- [x] Dizin ağacı `docs/architecture.md` §"Crate sınırları" ve repo yapısıyla uyumlu
- [x] `README.md` tüm ana dokümanlara çalışan link veriyor
- [x] `.gitignore` secrets ve generated binding dizinlerini kapsıyor
- [x] `LICENSE.md` boş bir lisans yerine gerekçe içeriyor

## Kanıt kaydı

Tarih: 2026-08-24 · Kanıt tipi: dosya ağacı + link denetimi

- Dizin ağacı `docs/architecture.md` §"Crate sınırları" ile uyumlu oluşturuldu:
  `docs/ tasks/ core/{crates,spikes} platforms/{macos,apple-shared,android,windows,linux} fixtures/{subtitles,media,providers} scripts/`
- `docs/` altında 21 markdown dosyası (6 ana doküman + 12 milestone + 3 ADR dosyası)
- README.md link denetimi — 10/10 hedef mevcut, kırık link yok:
  `docs/architecture.md · docs/product-spec.md · tasks/INDEX.md · docs/roadmap.md ·
   docs/security-policy.md · docs/testing-strategy.md · docs/glossary.md ·
   CLAUDE.md · LICENSE.md`
- `.gitignore` secrets (`\.env`, `secrets/`, `*.local.toml`) ve
  `platforms/**/generated/` dizinlerini kapsıyor
- `LICENSE.md` lisans metni yerine erteleme gerekçesini içeriyor
  (S1 dağıtım modeli + ADR-0012 libmpv linkleme; karar M3 öncesi)
