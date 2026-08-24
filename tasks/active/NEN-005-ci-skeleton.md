---
id: NEN-005
title: CI skeleton
milestone: M1
size: S
state: active
depends_on: [NEN-004, NEN-007]
blocks: []
adr: []
---

# NEN-005 — CI skeleton

## Sonuç

Her push'ta format, lint, test, lisans/güvenlik denetimi ve doküman tutarlılığı
otomatik çalışır; kural ihlali içeren bir commit CI'ı kırar.

## Kapsam

macOS runner üzerinde:

```
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo deny check
bash scripts/test.sh
bash scripts/task-index.sh --check
bash scripts/check-docs.sh
```

- Cache: cargo registry + target
- Concurrency: aynı branch'te eski koşu iptal edilir

## YAPILMAYACAK

- Platform/cihaz testleri CI'da çalıştırılmaz (kanıtları elle kaydedilir)
- Release/imzalama/notarization pipeline'ı → dağıtım kararı sonrası (S1)
- Android/Windows/Linux runner'ları → ilgili milestone'larda

## Kanıt (DoD)

- [ ] Kasıtlı `cargo fmt` ihlali içeren commit'te CI kırmızı, düzeltince yeşil
- [ ] Kasıtlı clippy uyarısı CI'ı kırıyor
- [ ] Bayat `INDEX.md` CI'ı kırıyor
- [ ] `scripts/test.sh` CI'da koşuyor; kasıtlı bozulan bir shell testi CI'ı kırıyor
- [ ] Yeşil koşunun süresi kaydedildi

## Kanıt kaydı

<!-- done olurken doldurulacak -->
