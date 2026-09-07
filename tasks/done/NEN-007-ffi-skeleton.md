---
id: NEN-007
title: Rust workspace and UniFFI skeleton
milestone: M1
size: M
state: done
closed: 2026-08-24
depends_on: [NEN-004]
blocks: [NEN-005, NEN-006, NEN-008, NEN-009, NEN-010]
adr: [6]
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

- [x] macOS'ta Swift test target'ı core fonksiyonunu çağırıp geçiyor
- [x] `cargo test --workspace` geçiyor
- [x] Binding üretimi tek komutla tekrarlanabilir (adım kaydedildi)
- [x] `nen-domain` hiçbir crate'e bağımlı değil (bağımlılık grafiği kanıtı)

## Kanıt kaydı

**Bağlam** (M1 ölçüm bağlamı kuralı — `docs/milestones/M1-core-spike.md`):
Apple Silicon (arm64) · macOS 27.0 · `rustc` 1.98.0 (88d9e12ae 2026-08-18) ·
`cargo` 1.98.0 · Apple Swift 6.4 (swiftlang-6.4.0.30.4) · `uniffi` 0.32.0 ·
**debug build** · 2026-08-24. Toolchain `core/rust-toolchain.toml` ile 1.98.0'a
pinli. Bu task'ta benchmark **yok** — ölçüm task'ları NEN-008/009/010/029.

### 1. Swift test target'ı core fonksiyonunu çağırıyor

```
$ bash scripts/test-apple.sh
◇ Test run started.
↳ Testing Library Version: 2077
↳ Target Platform: arm64e-apple-macos14.0
◇ Suite "Core FFI bridge" started.
✔ Test "version() carries a value across the whole Rust chain" passed after 0.001 seconds.
✔ Test "version() is stable across repeated calls" passed after 0.001 seconds.
✔ Test run with 2 tests in 1 suite passed after 0.001 seconds.
```

Geçen testler: `CoreBridgeTests.versionCrossesTheFfiBoundary` ·
`CoreBridgeTests.versionIsStableAcrossCalls`
(`platforms/apple-shared/Tests/NenCoreTests/CoreBridgeTests.swift`).

İlk test `version() == "nen-core 0.1.0"` bekliyor. Bu değer sabit string değil:
`nen-domain::CORE_NAME` ile `nen-app`'in `CARGO_PKG_VERSION`'ından birleşiyor,
yani zincirin tamamı — **nen-domain → nen-app → nen-ffi → Swift** — kanıtlanıyor.

### 2. `cargo test --workspace`

```
$ cd core && cargo test --workspace
test tests::version_combines_domain_constant_and_crate_version ... ok   (nen-app)
test tests::exported_version_matches_app_layer ... ok                   (nen-ffi)
test result: ok. 1 passed; 0 failed   ×2
test result: ok. 0 passed; 0 failed   ×21   (henüz testi olmayan crate'ler ve doc-test'ler)
```

Toplam **2 passed, 0 failed**; hiçbir target hata vermedi.

### 3. Binding üretimi tek komutla tekrarlanabilir

```
$ rm -rf platforms/apple-shared/generated
$ bash scripts/build-apple.sh
▶ 1/3  cargo build -p nen-ffi (debug)
▶ 2/3  uniffi-bindgen generate --language swift
▶ 3/3  SwiftPM düzenine yerleştirme

Tamam. Üretilenler (debug):
  platforms/apple-shared/generated/NenCore/nen_ffi.swift
  platforms/apple-shared/generated/README.md
  platforms/apple-shared/generated/lib/libnen_ffi.a
  platforms/apple-shared/generated/nen_ffiFFI/module.modulemap
  platforms/apple-shared/generated/nen_ffiFFI/nen_ffiFFI.h
```

Üretilen dosyalar commit **edilmiyor** — `.gitignore:23` → `/platforms/**/generated/`,
`git check-ignore -v` ile doğrulandı. Binding üretici (`uniffi-bindgen`) workspace
içinde bir bin target'ı; ayrıca araç kurulumu gerekmiyor.

### 4. `nen-domain` hiçbir crate'e bağımlı değil

```
$ cd core && cargo tree -p nen-domain --edges normal
nen-domain v0.1.0 (/Users/ye/Developer/nen-player/core/crates/nen-domain)
```

Tek düğüm — ne iç ne dış bağımlılık. `cargo metadata` ile tüm grafiğin
ADR-0006 tablosuyla birebir aynı olduğu doğrulandı:

```
nen-domain     -> (yok)
nen-ports      -> nen-domain
nen-subtitle   -> nen-domain
nen-identity   -> nen-domain
nen-catalog    -> nen-domain, nen-identity, nen-subtitle
nen-translate  -> nen-domain, nen-ports, nen-subtitle
nen-sync       -> nen-domain, nen-subtitle
nen-persist    -> nen-domain, nen-ports
nen-providers  -> nen-domain, nen-ports
nen-app        -> (diğer dokuzu)
nen-ffi        -> nen-app
```

`nen-ffi`'ın tek dış kapı olduğu da mekanik olarak kontrol edildi:

```
$ grep -rn --include='*.rs' -E "uniffi::|extern \"C\"" core/crates | grep -v crates/nen-ffi/
  (çıktı yok)
```

### Notlar — sonraki task'ları ilgilendiren iki bulgu

**1. Swift testleri XCTest değil swift-testing kullanıyor.** Bu makinede tam
Xcode yok (`xcode-select -p` → `/Library/Developer/CommandLineTools`) ve
CommandLineTools `XCTest.framework` getirmiyor; `Testing.framework` getiriyor.
Ek olarak swift-testing'in macro plugin'i (`libTestingMacros.dylib`) ve
`lib_TestingInterop.dylib`, SwiftPM'in varsayılan arama yollarında değil. Düz
`swift test` bu yüzden **çalışmıyor**; `scripts/test-apple.sh` eksik yolları
tespit edip ekliyor ve tam Xcode kuruluysa hiçbir ek bayrak koymuyor.

**2. `scripts/doctor.sh` `swift`i M3 seviyesinde tutuyor**, ama Swift testi artık
M1'de (bu task) gerekiyor. Doctor'a dokunulmadı (kural 5). Sırası geldiğinde ayrı
bir task konusu.

**3. UniFFI hâlâ ADAY.** Bu task binding teknolojisini seçmedi; ADR-0003
NEN-011/NEN-012'de karara bağlanacak. Kabul edilen tek mimari karar **ADR-0006**
(monorepo + crate sınırları + tek FFI kapısı).
