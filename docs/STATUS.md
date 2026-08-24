# Durum

> **Bu dosya yalnız doğrulanmış bugünü anlatır.** Plan `roadmap.md`'de, kararlar
> `DECISIONS.md`'de, task ayrıntısı `tasks/INDEX.md`'de. Burada tekrar edilmez.
>
> Son güncelleme: **2026-08-24** (NEN-030 kapanışı)

## Nerede duruyoruz

| | |
|---|---|
| **Mevcut milestone** | **M1 — Core Technical Spike** (M0 kapandı) |
| **Aktif task** | *yok* — `tasks/active/` boş |
| **Son tamamlanan** | `NEN-030` — milestone-aware doctor + STATUS denetimi |
| **Sıradaki READY** | **`NEN-007`** — Rust workspace + binding iskeleti (Rust kurulumu ister) |
| **Task sayısı** | 30 · done 5 · active 0 · blocked 0 · backlog 25 |
| **Commit** | `4de3bc4` — M0 temeli (82 dosya) |

`NEN-007` M1 spike zincirinin ilk halkası ve **tek blocker'ı Rust kurulumu**.
Kurulumdan sonra `bash scripts/doctor.sh M1` çıkış 0 vermeli.

## Toolchain

`bash scripts/doctor.sh M1` (2026-08-24, bu makine — macOS 27.0):

| Araç | Durum | M1'deki seviyesi |
|---|---|---|
| `cargo` / `rustc` | ❌ eksik | **blocker** — NEN-007'nin ön koşulu |
| `cargo-deny` | ❌ eksik | soon — NEN-005 öncesi |
| JDK | ❌ eksik | soon — NEN-011 (Kotlin/JVM parity) öncesi |
| `swift` | ✅ Apple Swift 6.4 | M3 |
| Tam Xcode | ❌ yalnız `/Library/Developer/CommandLineTools` | M3 — **M1 blocker'ı değil** |
| libmpv | ❌ eksik | M3 — **M1 blocker'ı değil** |
| Gradle | ❌ eksik | hiçbir milestone'da blocker değil (wrapper) |
| Android SDK | ❌ eksik | M10 |

`doctor.sh M1` → **çıkış 1** (2 blocker) · `doctor.sh` (parametresiz) →
**çıkış 0** (bilgilendirici). Kurulum komutları çıktıda; script **hiçbir şey
kurmaz** — bu, `scripts/tests/doctor.test.sh` S7 ile mekanik olarak kanıtlanıyor.

## Blocker'lar

| # | Blocker | Kimi durduruyor | Çözüm |
|---|---|---|---|
| B1 | **Rust kurulu değil** | `NEN-007` ve tüm M1 spike zinciri | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| B2 | JDK yok | `NEN-011` (M1 içinde, sıra gelmedi) | `brew install --cask temurin` |
| B3 | Tam Xcode + libmpv yok | M3 (henüz sıra gelmedi) | App Store'dan Xcode + `brew install mpv` |

**B1 tek gerçek blocker.** B2 ve B3 sıradaki task'ı engellemiyor.

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

Hiçbiri `NEN-007`'yi bloke etmiyor.

## Son doğrulama

2026-08-24, tümü bu makinede çalıştırıldı:

```
$ bash scripts/test.sh
▶ check-docs.test.sh   (7 doğrulama)  ✓
▶ doctor.test.sh       (24 doğrulama) ✓
SONUÇ: 2 test dosyasının hepsi geçti.            → exit 0

$ bash scripts/check-docs.sh
  ok  aktif task: 0
  ok  tüm state alanları dizinleriyle uyumlu
  ok  tüm done task'ların kanıt kaydı dolu
  ok  tüm depends_on/blocks hedefleri mevcut
  ok  döngü yok
  ok  done task'ların ADR'leri accepted
  ok  tasks/INDEX.md güncel
  ok  STATUS.md ready listesi INDEX ile uyumlu (NEN-007)
SONUÇ: tüm denetimler geçti.                     → exit 0

$ bash scripts/task-index.sh --check
OK: tasks/INDEX.md güncel.                       → exit 0

$ bash scripts/doctor.sh                          → exit 0  (bilgilendirici)
$ bash scripts/doctor.sh M1                       → exit 1  (2 blocker: cargo, rustc)
$ bash scripts/doctor.sh M3                       → exit 1  (+ Xcode, libmpv)
```

## Repository'nin gerçek durumu

- **Kod yok.** `core/` ve `platforms/` yalnız dizin iskeleti + `.gitkeep`.
- **Dependency yok.** Cargo/Gradle/Xcode projesi kurulmadı.
- Var olan: 6 ana doküman · 12 milestone dosyası · ADR sistemi (1 accepted) ·
  30 task · 4 script + 2 shell testi · `fixtures/` iskeleti.
- Depo kökünde **`LICENSE` dosyası bilerek yok** — bkz. [`licensing.md`](licensing.md).
- Git: `main` branch, son commit `4de3bc4`.

## Bu dosyayı kim günceller

Her task kapanışında (`/finish-task`) ve her milestone geçişinde.
`scripts/check-docs.sh` **denetim 8**, buradaki "Sıradaki READY" satırının
`tasks/INDEX.md` ile uyuşmasını mekanik olarak zorlar — bayatlarsa CI kırılır.
