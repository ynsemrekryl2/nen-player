# Durum

> **Bu dosya yalnız doğrulanmış bugünü anlatır.** Plan `roadmap.md`'de, kararlar
> `DECISIONS.md`'de, task ayrıntısı `tasks/INDEX.md`'de, geçmiş kapanışlar
> `tasks/done/` ve `history/`'de. Burada tekrar edilmez; **150 satırı aşamaz**
> (`check-docs.sh` denetim 10).
>
> Son güncelleme: **2026-09-14** (**`NEN-113` kapandı** — macOS API key entry in
> settings.)

## Nerede duruyoruz

| | |
|---|---|
| **Mevcut milestone** | **M6 — Real Providers** (M5 2026-09-11'de kapandı — `docs/milestones/M6-real-providers.md`) |
| **Aktif task** | — |
| **Son tamamlanan** | **`NEN-113`** — macOS API key entry in settings. Ondan önce: `NEN-112` |
| **Sıradaki READY** | `NEN-116`, `NEN-119`, `NEN-120`, `NEN-124`, `NEN-128` |
| **Task sayısı** | 128 · done 111 · active 0 · blocked 0 · canceled 2 · backlog 15 |

**Son kapanış — `NEN-113` (2026-09-14):** macOS Ayarlar'a üç maskeli API key
satırı, FFI-only lifecycle, typed validation/error surface ve plaintext negatif
kanıt eklendi. Gerçek `.app` smoke akışı üç fixture için save → yeniden açılışta
kayıtlı → delete olarak tamamlandı. Kanıt: `evidence/M6/NEN-113-checklist.md`
ve `tasks/done/NEN-113-macos-api-key-settings.md`.

## Toolchain

`bash scripts/doctor.sh M3` (2026-08-25, bu makine — macOS 27.0 26A5416b):

| Araç | Durum | M1'deki seviyesi |
|---|---|---|
| `cargo` / `rustc` | ✅ 1.98.0 (2026-08-18) | blocker — **karşılandı** |
| `cargo-deny` | ✅ 0.20.2 (Homebrew, NEN-005) | soon — **karşılandı** |
| JDK | ✅ OpenJDK 26.0.2.1 (Homebrew `openjdk`, NEN-011) | soon — **karşılandı** |
| `swift` | ✅ Apple Swift 6.3.3 (Xcode bundled) | M1 (blocker) — NEN-007 testi ve NEN-008 harness'ı kullanıyor |
| Tam Xcode | ✅ Xcode 26.6 (build 17F113, `/Applications/Xcode.app`) | M3 (blocker) — **karşılandı** |
| libmpv | ✅ 2.5.0 (Homebrew mpv 0.41.0_8, pkg-config) | M3 (blocker) — **karşılandı** |
| Gradle | ✅ 9.7.1 (Homebrew, yalnız wrapper bootstrap için — NEN-011) | hiçbir milestone'da blocker değil (wrapper) |
| Android SDK | ❌ eksik | M10 |

Rust `rustup` ile kuruldu (2026-08-24). Workspace `core/rust-toolchain.toml` ile

Rust `rustup` ile kuruldu, workspace `core/rust-toolchain.toml` ile **1.98.0'a
pinli**. `doctor.sh M1` ve `doctor.sh M3` → **çıkış 0**; script hiçbir şey
kurmaz (`scripts/tests/doctor.test.sh` S7). Kurulum geçmişi:
`history/status-archive-2026-09.md`.

## Blocker'lar

**Gerçek blocker yok.** M1 ve M3 kapıları açık. Çözülmüş B1–B4 kaydı:
`history/status-archive-2026-09.md`.

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

2026-09-14'te `NEN-113` kapandı. `cargo test --workspace`, fmt/clippy/deny,
`bash scripts/test-macos.sh` (**281/281**), credential suite'i, gerçek Keychain
suite'i, ad-hoc `.app` build + strict codesign, `bash scripts/test.sh` (**6/6**),
M3 doctor ve `check-docs.sh` yeşil.

Önceki doğrulama girdileri ve toolchain kapısı geçmişi:
`history/status-archive-2026-09.md`.

## Repository

`core/` Cargo workspace (Rust 1.98.0, UniFFI) · `platforms/` (macOS SwiftPM; diğerleri iskelet) ·
`scripts/` shell tooling + testleri · `fixtures/` · `docs/adr/` (38 ADR).
Depo kökünde **`LICENSE` bilerek yok** — bkz. `licensing.md`. Remote:
`github.com/ynsemrekryl2/nen-player` (private). Bu dosya commit hash'i tutmaz;
git geçmişi kanoniktir.

## Bu dosyayı kim günceller

Her task kapanışında (`/finish-task`) ve her milestone geçişinde.
**Ekleme değil değiştirme:** kapanışta önceki "Son kapanış" özeti ve "Son
doğrulama" girdisi silinir, dosyada her zaman yalnız son kapanış bulunur.
Geçmiş `tasks/done/` Kanıt kaydında ve git'tedir; buraya kopyalanmaz.
`scripts/check-docs.sh` **denetim 8** "Sıradaki READY" satırının
`tasks/INDEX.md` ile uyuşmasını, **denetim 9** "Son doğrulama" tarihinin
en yeni done task'tan geri kalmamasını, **denetim 10** dosyanın 150 satırı
aşmamasını mekanik olarak zorlar — bayatlarsa veya şişerse CI kırılır.
