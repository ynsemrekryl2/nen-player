# Durum

> **Bu dosya yalnız doğrulanmış bugünü anlatır.** Plan `roadmap.md`'de, kararlar
> `DECISIONS.md`'de, task ayrıntısı `tasks/INDEX.md`'de. Burada tekrar edilmez.
>
> Son güncelleme: **2026-08-24**

## Nerede duruyoruz

| | |
|---|---|
| **Mevcut milestone** | **M1 — Core Technical Spike** (M0 kapandı) |
| **Aktif task** | *yok* — `tasks/active/` boş |
| **Son tamamlanan** | `NEN-004` — toolchain doctor |
| **Sıradaki READY** | **`NEN-030`** (toolchain gerektirmez) · `NEN-007` (Rust ister) |
| **Task sayısı** | 30 · done 4 · active 0 · blocked 0 · backlog 26 |
| **Commit** | **yok** — repo hâlâ commit'siz, tüm dosyalar untracked |

`NEN-030` önce öneriliyor: hiçbir kurulum gerektirmiyor ve `doctor.sh`'ı
milestone-aware yaparak M1'e temiz bir giriş kapısı açıyor.

## Toolchain

`bash scripts/doctor.sh` (2026-08-24, bu makine — macOS 27.0):

| Araç | Durum | Gereken yer |
|---|---|---|
| `swift` | ✅ Apple Swift 6.4 | M3 |
| `cargo` / `rustc` | ❌ **eksik** | **M1 — NEN-007'nin ön koşulu** |
| `cargo-deny` | ❌ eksik | M1 / NEN-005 öncesi |
| Tam Xcode | ❌ yalnız `/Library/Developer/CommandLineTools` | M3 |
| libmpv | ❌ eksik | M3 |
| JDK | ❌ eksik | M1 / NEN-011 öncesi *(NEN-030 sonrası doctor bunu M1 altında gösterecek)* |
| Gradle | ❌ eksik | Wrapper tercih edilir — blocker değil |

Kurulum komutları `doctor.sh` çıktısında. Script **hiçbir şey kurmaz**.

## Blocker'lar

| # | Blocker | Kimi durduruyor | Çözüm |
|---|---|---|---|
| B1 | **Rust kurulu değil** | `NEN-007` ve tüm M1 spike zinciri | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| B2 | Tam Xcode + libmpv yok | M3 (henüz sıra gelmedi) | App Store'dan Xcode + `brew install mpv` — M3 öncesi |

`NEN-030` **hiçbir blocker'a takılmıyor** — bugün yapılabilir.

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

Hiçbiri `NEN-030` veya `NEN-007`'yi bloke etmiyor.

## Son doğrulama

2026-08-24, tümü bu makinede çalıştırıldı:

```
$ bash scripts/check-docs.sh
  ok  aktif task: 0
  ok  tüm state alanları dizinleriyle uyumlu
  ok  tüm done task'ların kanıt kaydı dolu
  ok  tüm depends_on/blocks hedefleri mevcut
  ok  döngü yok
  ok  done task'ların ADR'leri accepted
  ok  tasks/INDEX.md güncel
SONUÇ: tüm denetimler geçti.          → exit 0

$ bash scripts/task-index.sh --check
OK: tasks/INDEX.md güncel.            → exit 0

$ bash scripts/doctor.sh
SONUÇ: 5 zorunlu araç eksik, 2 opsiyonel eksik.   → exit 1
```

> `doctor.sh`'ın çıkış kodu 1 vermesi **beklenen** — milestone-aware davranış
> henüz yok, `NEN-030` ile gelecek. O zamana kadar parametresiz çağrı gelecek
> milestone eksiklerini de zorunlu sayıyor.

## Repository'nin gerçek durumu

- **Kod yok.** `core/` ve `platforms/` yalnız dizin iskeleti + `.gitkeep`.
- **Dependency yok.** Cargo/Gradle/Xcode projesi kurulmadı.
- Var olan: 6 ana doküman · 12 milestone dosyası · ADR sistemi (1 accepted) ·
  30 task · 3 script · `fixtures/` iskeleti.
- Depo kökünde **`LICENSE` dosyası bilerek yok** — bkz. [`licensing.md`](licensing.md).
- Git: `main` branch, **hiç commit yok**.

## Bu dosyayı kim günceller

Her task kapanışında (`/finish-task`) ve her milestone geçişinde. `NEN-030`
sonrası `scripts/check-docs.sh` denetim 8, buradaki "Sıradaki READY" satırının
`tasks/INDEX.md` ile uyuşmasını mekanik olarak zorlayacak.
