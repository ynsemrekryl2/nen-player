---
id: NEN-059
title: Restore the Rust CI gates
milestone: M3
size: S
state: done
closed: 2026-08-27
depends_on: [NEN-025, NEN-051]
blocks: []
adr: []
---

# NEN-059 — Restore the Rust CI gates

## Sonuç

Sabitlenmiş Rust toolchain'inin biçimlendirme ve lint kapıları yeniden yeşildir.

## Kapsam

- `cargo fmt` ile CI'ın raporladığı Rust dosyalarını biçimlendir
- Değişikliklerin yalnız whitespace/satır düzeni olduğunu mekanik olarak doğrula
- Rust 1.98'in sabit assertion lint'ini önerdiği `const` blokla gider
- CI workflow'undaki yerel kapıların tamamını çalıştır

## YAPILMAYACAK

- Runtime davranışı, public API, tip veya veri akışı değişikliği
- Alakasız refactor ya da bağımlılık güncellemesi
- Force push veya Git geçmişini değiştirme

## Kanıt (DoD)

- [x] `cargo fmt --check` çıkış 0
- [x] Rustfmt dosyalarının whitespace dışı byte'ları aynı; tek fark belgelenen `const` wrapper
- [x] `cargo clippy --workspace --all-targets -- -D warnings` çıkış 0
- [x] CI'ın yerel eşdeğeri bütünüyle geçiyor

## Kanıt kaydı

Rust 1.98.0 (`rustfmt 1.9.0-stable`) ile ilk `cargo fmt --check`, GitHub
Actions'taki aynı beş dosyada kırmızıydı. `cargo fmt` sonrasında tam olarak bu
beş dosya değişti ve biçim kapısı çıkış 0 verdi.

Dört dosyada HEAD ile çalışma ağacının whitespace dışı byte'ları `cmp` ile
birebir aynıydı. Beşinci dosyada aynı karşılaştırma, Clippy'nin önerdiği
`const { assert!(...) }` wrapper'ı normalize edilerek geçti; başka token farkı
yoktu.

İlk tam yerel CI koşusu biçim kapısından geçti ve sıradaki kapıda
`clippy::assertions_on_constants` ile kırmızı oldu. Assertion `const` bloka
alındıktan sonra workflow'un yerel eşdeğeri bütünüyle çıkış 0 verdi:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` — tüm testler geçti
- `cargo deny check` — `advisories ok, bans ok, licenses ok, sources ok`
- `bash scripts/test.sh` — `2 test dosyasının hepsi geçti`
- `bash scripts/task-index.sh --check` — `OK: tasks/INDEX.md güncel.`
- `bash scripts/check-docs.sh` — `SONUÇ: tüm denetimler geçti.`

Runtime davranışı veya public API değişmedi. Task commit'i `0e434d6` için
GitHub Actions `CI` run **33042520202**, bütün adımları **1 dk 57 sn** içinde
geçerek `success` tamamlandı:
https://github.com/ynsemrekryl2/nen-player/actions/runs/33042520202
