# Durum

> **Bu dosya yalnız doğrulanmış bugünü anlatır.** Plan `roadmap.md`'de, kararlar
> `DECISIONS.md`'de, task ayrıntısı `tasks/INDEX.md`'de. Burada tekrar edilmez.
>
> Son güncelleme: **2026-08-24** (NEN-007 kapanışı)

## Nerede duruyoruz

| | |
|---|---|
| **Mevcut milestone** | **M1 — Core Technical Spike** (M0 kapandı) |
| **Aktif task** | *yok* — `tasks/active/` boş |
| **Son tamamlanan** | `NEN-007` — Rust workspace + UniFFI iskeleti (**ilk kod**) |
| **Sıradaki READY** | `NEN-005` `NEN-006` `NEN-008` `NEN-009` `NEN-010` `NEN-031` |
| **Task sayısı** | 31 · done 6 · active 0 · blocked 0 · backlog 25 |

`NEN-007` kapandı: **repository artık kod içeriyor.** Cargo workspace, ADR-0006'nın
tarif ettiği 11 crate ve `nen-ffi` üzerinden Swift'e geçen bir `version()`
fonksiyonu ayakta. Milestone sırası önerisi (`M1-core-spike.md`) NEN-008/009/010
ile devam ediyor; NEN-031 (test fixture) ve NEN-006 (redaction) da açıldı.

**UniFFI hâlâ aday.** NEN-007 binding teknolojisini seçmedi; ADR-0003
NEN-011/NEN-012'de karara bağlanacak. Kabul edilen mimari karar **ADR-0006** —
monorepo yapısı, crate sınırları ve `nen-ffi`'ın tek dış kapı olması.

NEN-007'nin `adr:` alanı `[3, 6]` → **`[6]`** olarak düzeltildi: ADR-0003 spike
ölçümleri olmadan `accepted` olamaz, yani task'ın kapanışını kilitliyordu.
Aynı kusur `NEN-008` ve `NEN-011`'de duruyor — sırası gelince ele alınacak.

## Toolchain

`bash scripts/doctor.sh M1` (2026-08-24, bu makine — macOS 27.0):

| Araç | Durum | M1'deki seviyesi |
|---|---|---|
| `cargo` / `rustc` | ✅ 1.98.0 (2026-08-18) | blocker — **karşılandı** |
| `cargo-deny` | ❌ eksik | soon — NEN-005 öncesi |
| JDK | ❌ eksik | soon — NEN-011 (Kotlin/JVM parity) öncesi |
| `swift` | ✅ Apple Swift 6.4 | M3 — ama NEN-007 testi zaten kullanıyor |
| Tam Xcode | ❌ yalnız `/Library/Developer/CommandLineTools` | M3 — **M1 blocker'ı değil** |
| libmpv | ❌ eksik | M3 — **M1 blocker'ı değil** |
| Gradle | ❌ eksik | hiçbir milestone'da blocker değil (wrapper) |
| Android SDK | ❌ eksik | M10 |

Rust `rustup` ile kuruldu (2026-08-24). Workspace `core/rust-toolchain.toml` ile
**1.98.0'a pinli** — M1 ölçümlerinin başka makinede karşılaştırılabilmesi için.

`doctor.sh M1` → **çıkış 0** · `doctor.sh` (parametresiz) →
**çıkış 0** (bilgilendirici). Kurulum komutları çıktıda; script **hiçbir şey
kurmaz** — bu, `scripts/tests/doctor.test.sh` S7 ile mekanik olarak kanıtlanıyor.

## Blocker'lar

| # | Blocker | Kimi durduruyor | Çözüm |
|---|---|---|---|
| ~~B1~~ | ~~Rust kurulu değil~~ | — | ✅ **çözüldü** 2026-08-24 — rustup, 1.98.0 |
| B2 | JDK yok | `NEN-011` (M1 içinde, sıra gelmedi) | `brew install --cask temurin` |
| B3 | Tam Xcode + libmpv yok | M3; Swift testleri CommandLineTools'ta ek bayrak istiyor (`scripts/test-apple.sh` hallediyor) | App Store'dan Xcode + `brew install mpv` |
| B4 | `check-docs.test.sh` T1/T3/T6, **bir task `active` olduğu anda** kırılıyor | hiçbir task'ı **bloke etmiyor**; CI iskeleti (NEN-005) öncesi kapanmalı | `NEN-031` |

**Gerçek blocker kalmadı.** B2 ve B3 sıradaki task'ları engellemiyor. B4 bir test
fixture kusuru — `check-docs.sh`'ın kendisi her durumda exit 0; `tasks/active/`
boşken `test.sh` de yeşil. Sebebi ve kapsamı `NEN-031`'de.

## Kullanıcı kararı bekleyenler

| # | Konu | Ne zaman gerekiyor |
|---|---|---|
| **S3** | Çeviri kalite hedefinin operasyonel ölçütü | M5 |
| **S4** | Offline/uçak modu birinci sınıf mı? S8'in "cloud sync non-goal" cevabı bunu doğrudan etkiliyor — sync yoksa offline davranış tamamen yerel cache'e bağlı | M5–M6 |
| **S6** | Local ASR modeli ve cihaz kaynak bütçesi *(privacy kısmı cevaplandı)* | M8 |
| **S7** | Android TV minimum API seviyesi ve hedef cihaz sınıfı | M10 |
| **S9** | Birden fazla AI artifact'in UI'da gösterimi | M5 |
| **S11** | İleride public dağıtım | M3 sonrası |
| **S12** | Gerçek lisans seçimi | ADR-0012 sonrası |

Hiçbiri sıradaki task'ları bloke etmiyor.

## Son doğrulama

2026-08-24, tümü bu makinede çalıştırıldı (macOS 27.0 · arm64 · rustc/cargo
1.98.0 · Swift 6.4 · uniffi 0.32.0 · debug build):

```
$ bash scripts/doctor.sh M1
SONUÇ: M1 için tüm blocker'lar hazır.            → exit 0

$ cd core && cargo test --workspace
nen-app::tests::version_combines_domain_constant_and_crate_version ... ok
nen-ffi::tests::exported_version_matches_app_layer ... ok
2 passed, 0 failed (23 target)                   → exit 0

$ cd core && cargo tree -p nen-domain --edges normal
nen-domain v0.1.0                                → tek düğüm, sıfır bağımlılık

$ bash scripts/build-apple.sh                     → binding temiz üretildi
$ bash scripts/test-apple.sh
✔ Test run with 2 tests in 1 suite passed        → exit 0

$ bash scripts/check-docs.sh
  8/8 denetim geçti                              → exit 0

$ bash scripts/test.sh
  doctor.test.sh     ✓ (24 doğrulama)
  check-docs.test.sh ✓ ( 7 doğrulama)            → exit 0
```

**B4 uyarısı — `test.sh` yalnız şu anki durumda yeşil.** `check-docs.test.sh`,
`READY` listesini canlı `INDEX.md`'den türetiyor; liste boşaldığı anda — yani
**bir task `active` olduğu anda** — T1/T3/T6 kırılıyor. NEN-007 `active`ken
gözlendi, `done`a geçince kendiliğinden yeşile döndü. Yani paket bir sonraki
task başlatıldığında yeniden kırmızıya dönecek. `check-docs.sh`'ın kendisi her
iki durumda da doğru çalışıyor (exit 0). `NEN-031` bunu kapatacak.

## Repository'nin gerçek durumu

- **Kod var** (NEN-007 ile): `core/` altında Cargo workspace + 11 crate,
  `core/rust-toolchain.toml`, `Cargo.lock`. `core/spikes/` hâlâ boş.
- `platforms/apple-shared/` — SwiftPM paketi (`Package.swift` + swift-testing
  test target'ı). Üretilen binding `generated/` altında ve **commit edilmiyor**.
- Diğer `platforms/*` dizinleri hâlâ boş iskelet.
- Var olan: 6 ana doküman · 12 milestone dosyası · **2 accepted ADR**
  (0001, 0006) · 31 task · 6 script + 2 shell testi · `fixtures/` iskeleti.
- Depo kökünde **`LICENSE` dosyası bilerek yok** — bkz. [`licensing.md`](licensing.md).
- Git: `main` branch. **Bu dosya commit hash'i tutmaz** — commit geçmişi
  kanonik kayıttır ve elle tutulan hash satırı her kapanışta bayatlar
  (CLAUDE.md → "Commit politikası").

## Bu dosyayı kim günceller

Her task kapanışında (`/finish-task`) ve her milestone geçişinde.
`scripts/check-docs.sh` **denetim 8**, buradaki "Sıradaki READY" satırının
`tasks/INDEX.md` ile uyuşmasını mekanik olarak zorlar — bayatlarsa CI kırılır.
