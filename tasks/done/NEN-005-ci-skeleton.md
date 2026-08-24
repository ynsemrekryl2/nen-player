---
id: NEN-005
title: CI skeleton
milestone: M1
size: S
state: done
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

- [x] Kasıtlı `cargo fmt` ihlali içeren commit'te CI kırmızı, düzeltince yeşil
- [x] Kasıtlı clippy uyarısı CI'ı kırıyor
- [x] Bayat `INDEX.md` CI'ı kırıyor
- [x] `scripts/test.sh` CI'da koşuyor; kasıtlı bozulan bir shell testi CI'ı kırıyor
- [x] Yeşil koşunun süresi kaydedildi

## Kanıt kaydı

Repo: `https://github.com/ynsemrekryl2/nen-player` (private, bu task için
kuruldu). Workflow: [.github/workflows/ci.yml](../../.github/workflows/ci.yml),
lisans/advisory kapısı: [core/deny.toml](../../core/deny.toml). Runner:
`macos-15`.

### Kurulum sırasında bulunan ve düzeltilen 3 gerçek kusur

CI ilk kez gerçek bir runner'da koşunca, hiçbiri yerel makinede tekrarlanamayan
üç ayrı kusur ortaya çıktı — üçü de düzeltildi, ayrı commit'lerde:

1. **Run [32744362719](https://github.com/ynsemrekryl2/nen-player/actions/runs/32744362719)** — `cargo clippy` `rustc 1.97.1` ile çalıştı (gereken 1.98).
   `rust-toolchain.toml`'ın pin'i yalnız cargo'nun cwd'si `core/` altındayken
   devreye giriyor; `--manifest-path core/Cargo.toml` bunu tetiklemiyor.
   Düzeltme: fmt/clippy/test adımları `working-directory: core` ile
   sınırlandı (commit `608f43b`).
2. **Run [32744517860](https://github.com/ynsemrekryl2/nen-player/actions/runs/32744517860)** — `cargo deny check`, `EmbarkStudios/cargo-deny-action`
   bir Docker container action olduğu için macOS runner'da çalışmadı
   ("Container action is only supported on Linux"). Düzeltme:
   `taiki-e/install-action` (prebuilt binary) + düz `cargo deny check`
   adımına geçildi (commit `87d5618`).
3. **Run [32744833474](https://github.com/ynsemrekryl2/nen-player/actions/runs/32744833474)** — `bash scripts/test.sh`, `doctor.test.sh`'ın S4
   senaryosunda kırıldı: `scripts/doctor.sh`'ın `detect_jdk`'sı, diğer tüm
   `detect_*` fonksiyonlarının aksine `command -v java` guard'ı içermiyordu;
   `run_timeout`'un dayandığı perl `exec LIST`, PATH'te hiç `java` yoksa
   sessizce başarılı dönüyor (exit 0) — yani "JDK yok" durumu "JDK var"
   raporlanıyordu. Yerel Mac'lerde bu gizli kaldı çünkü CLT'li bir Mac'te
   JDK kurulu olmasa da `/usr/bin/java` çalıştırılınca gerçek bir hata
   veren bir Apple stub'ı olarak var; GitHub'ın macOS runner image'ında ise
   `/usr/bin/java` gerçek ve çalışan bir JDK. Düzeltme: `detect_jdk`'ya
   `command -v java` guard'ı eklendi; `doctor.test.sh`'ın S4 senaryosu da
   S7'nin swift için kullandığı shadow-PATH tekniğiyle güncellendi (gerçek
   `/usr/bin/java`'nın stub-hata davranışına güvenmek yerine onu PATH'ten
   tamamen gizliyor) (commit `20ccb90`). **Ürün kodu değil, mevcut task'ların
   (`NEN-004`/`NEN-030`/`NEN-032`) tooling'inde bulunan bir doğruluk hatası —
   NEN-005'in kendi yeşil taban çizgisi için önkoşul olduğundan bu task
   kapsamında düzeltildi.**

Bu üç düzeltmeden sonra **Run [32745671071](https://github.com/ynsemrekryl2/nen-player/actions/runs/32745671071) tamamen yeşil** — 7 adımın hepsi
geçti. Süre: **3m58s** (soğuk cache — ilk koşu, `~/.cargo/registry` +
`core/target` henüz cache'lenmemişti).

### DoD kanıtı — 4 kasıtlı ihlal, sırayla (pipeline sıralı olduğu için her biri
bir öncekini düzeltip bir sonrakini bozan zincirleme commit'lerle)

| # | Push | İhlal | Sonuç | Kırılan adım | Süre |
|---|---|---|---|---|---|
| 1 | `df1b8d6` | Kasıtlı `cargo fmt` ihlali (`nen-domain/src/lib.rs`) | 🔴 [32746215244](https://github.com/ynsemrekryl2/nen-player/actions/runs/32746215244) | `cargo fmt --check` | 24s |
| 2 | `ae7a82f` | fmt düzeltildi + kasıtlı clippy ihlali (`x == true`, `bool_comparison`) | 🔴 [32746341541](https://github.com/ynsemrekryl2/nen-player/actions/runs/32746341541) | `cargo clippy` (fmt ✓) | 35s |
| 3 | `4eab528` | clippy düzeltildi + bayat `tasks/INDEX.md` (elle stale marker eklendi) | 🔴 [32746487549](https://github.com/ynsemrekryl2/nen-player/actions/runs/32746487549) | `task-index.sh --check` (fmt/clippy/test/deny/test.sh ✓) | 1m9s |
| 4 | `eaeefe6` | INDEX.md düzeltildi + `doctor.test.sh` S6'da kasıtlı bozuk assertion | 🔴 [32746748025](https://github.com/ynsemrekryl2/nen-player/actions/runs/32746748025) | `bash scripts/test.sh` (fmt/clippy/test/deny ✓) | 45s |
| 5 | `e15c424` | Tüm ihlaller geri alındı — temiz durum | 🟢 [32746937130](https://github.com/ynsemrekryl2/nen-player/actions/runs/32746937130) | — tümü geçti | **52s** (sıcak cache) |

Push #3, aynı zamanda `bash scripts/test.sh`'ın CI'da fiilen koştuğunu ve
geçtiğini de doğruluyor (adım kendisi ✓, yalnız sonraki `task-index --check`
kırıldı) — DoD'un "CI'da koşuyor" kısmı ayrıca kanıtlanmış oldu.

**Yeşil koşu süresi: 3m58s (soğuk cache) → 52s (sıcak cache).** Fark, cache
key'inin (`Cargo.lock` hash'i) aynı kalması sayesinde `~/.cargo/registry` ve
`core/target`'ın restore edilmesinden geliyor.

### Yerel doğrulama (push öncesi, workflow'un adımlarıyla birebir)

```
$ (cd core && cargo fmt --check)                                    → exit 0
$ (cd core && cargo clippy --workspace --all-targets -- -D warnings) → exit 0
$ (cd core && cargo test --workspace)                                → exit 0
$ (cd core && cargo deny check)                                      → advisories ok, bans ok, licenses ok, sources ok
$ bash scripts/test.sh                                                → 2/2 test dosyası geçti
$ bash scripts/task-index.sh --check                                  → OK
$ bash scripts/check-docs.sh                                          → 8/8 denetim geçti
```

### Kapsam dışı bırakılanlar (görev tanımına göre)

Platform/cihaz testleri (`test-apple.sh`), release/imzalama pipeline'ı,
Android/Windows/Linux runner'ları — hiçbiri bu workflow'a eklenmedi
("YAPILMAYACAK" bölümü).
