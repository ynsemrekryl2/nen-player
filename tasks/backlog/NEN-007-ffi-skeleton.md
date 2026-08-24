---
id: NEN-007
title: Rust workspace and UniFFI skeleton
milestone: M1
size: M
state: backlog
depends_on: [NEN-004]
blocks: [NEN-005, NEN-006, NEN-008, NEN-009, NEN-010]
adr: [3, 6]
---

# NEN-007 — Rust workspace and UniFFI skeleton

## Sonuç

Swift tarafındaki bir test, cihazda çalışan Rust core'daki bir fonksiyonu
çağırıp doğru sonucu alır — yani **Rust + UniFFI adayı** ölçülebilir hale gelir.

> Bu task bir **aday doğrulaması**dır, karar değil. Rust ve UniFFI, ADR-0002 ve
> ADR-0003 kabul edilene kadar adaydır (`docs/architecture.md` → "Karar
> statüsü"). Task'ın kurduğu iskelet, NEN-008/009/010/029 ölçümlerinin üzerinde
> koşacağı zemindir.

## Kapsam

- Cargo workspace (`core/Cargo.toml`) ve `docs/architecture.md`'deki crate iskeleti
- `nen-ffi` — **tek dış kapı**, binding kurulumu (aday: UniFFI)
- Tek örnek fonksiyon (`version()` gibi) ve Swift test target'ı
- Binding üretim adımının build'e bağlanması

## YAPILMAYACAK

- Gerçek domain modeli → M2
- Kotlin binding → NEN-011
- Async/cancel/error ölçümleri → NEN-008/009/010

## Kanıt (DoD)

- [ ] macOS'ta Swift test target'ı core fonksiyonunu çağırıp geçiyor
- [ ] `cargo test --workspace` geçiyor
- [ ] Binding üretimi tek komutla tekrarlanabilir (adım kaydedildi)
- [ ] `nen-domain` hiçbir crate'e bağımlı değil (bağımlılık grafiği kanıtı)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
