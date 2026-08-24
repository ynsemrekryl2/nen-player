# Kararlar

Bu dosya **karar defteridir** — ne verildiği, ne zaman ve neden. Şartname veya
ADR metnini tekrarlamaz, onlara işaret eder.

- Ürün davranışının kanonik tanımı: [`product-spec.md`](product-spec.md)
- Mimari kararların tam metni: [`adr/`](adr/)
- Bugünkü durum: [`STATUS.md`](STATUS.md)

---

## 1. Kilitli ürün kararları

Şartnameden gelen, tartışmaya kapalı kararlar. Tam metin `product-spec.md`'de.

| Konu | Karar | Spec |
|---|---|---|
| Mimari model | Platform-native UI → application API → cihaz içi shared core → portlar. **Node/Express/localhost/browser/web dashboard yok** | §3 |
| Playback | `PlaybackEngine` **capability tabanlı port**. Kullanıcı motoru ne görür ne seçer | §4 |
| Playback önceliği | Medya; subtitle discovery ve AI çeviriyi **beklemez**. Subtitle hatası playback'i durdurmaz | §4 |
| Stremio | Nen Player add-on **değil**; external player olarak açılma ana özellik | §5 |
| Medya kimliği | Kullanıcıdan **teknik ID istenmez** (IMDb/Stremio yok) | §6 |
| Katalog | **Tek** `SubtitleSourceCatalog`, tek altyazı düğmesi, ayrı "AI subtitle mode" yok | §7, §8 |
| Çeviri tetikleme | Kaynak seçmek çeviri **başlatmaz**; yalnız açık kullanıcı komutu | §9 |
| Çeviri bütünlüğü | Blok 40 (30–60), overlap 6; cue ID/sıra/zaman **aynen korunur** | §10 |
| Provider güveni | Provider cevabı **untrusted**; local validation **authoritative** | §10 |
| Yayınlama | **Progressive/yarım subtitle yayını yok**; final yalnız tüm belge doğrulanınca, atomik commit | §10, §11 |
| Cache | Prompt/schema/pipeline semantiği değişince **uyumsuz cache kullanılmaz** | §11 |
| Senkronizasyon | Orijinal cue zamanları **değişmez**; düzeltme ayrı `SyncProfile` | §12 |
| Auto-sync onayı | **High confidence bile** kullanıcı onayı olmadan kalıcı uygulanmaz | §13 |
| Audio gizliliği | Varsayılan `localOnly`; raw audio loglanmaz; remote gönderim açık izinle | §13 |
| Cue lookup | Lineer tam-liste taraması **yasak**; seek sonrası doğru cue anında | §14 |
| Secret | Platform secure storage; plaintext config yasak; loglanmaz, artifact'e girmez | §15 |
| Yasaklar | Scraping · DRM bypass · torrent/debrid · path traversal · symlink · testlerde gerçek provider kotası | §16 |

## 2. Süreç kararları

| Karar | Tarih | Gerekçe |
|---|---|---|
| Task sistemi **repo içi markdown** (GitHub Issues değil) | 2026-08-24 | Agent ve insan aynı kaynağı okur, offline çalışır, PR diff'inde görünür |
| Doküman dili **Türkçe**; kod/tip/dosya adı/commit **İngilizce** | 2026-08-24 | İletişim diliyle tutarlı, kod evrensel |
| Milestone sırasında **subtitle core ile translation core ayrıldı**, macOS slice araya alındı | 2026-08-24 | FFI kontratı ve capability modeli, en ağır core işi yazılmadan gerçek bir UI'a karşı doğrulansın. Gerekçe: [`roadmap.md`](roadmap.md) → "Şartnameden sapma" |
| NEN-005 (CI) ve NEN-006 (redaction) M0'dan **M1'e taşındı** | 2026-08-24 | İkisi de Rust crate'i gerektiriyor; M0'ın "araç gerektirmez" özelliği korundu |
| `doctor.sh` parametresiz **daima çıkış 0**; kapı görevini milestone'lu çağrı yapar | 2026-08-24 | Gelecek milestone eksiği mevcut blocker gibi görünmesin. Uygulama: NEN-030 |
| Kanıt yükü **task tipine göre** belirlenir | 2026-08-24 | "Her task'a benchmark veya ekran kaydı" ya sahte kanıt ya kapanmayan task üretir. Güvenlik task'larında negatif test zorunluluğu **korundu** |
| Ölçülmemiş performans eşikleri **baseline'a çevrildi** | 2026-08-24 | `< 100 MB`, `< 250 ms` hiçbir ölçüme dayanmıyordu. Invariant'lar (I1–I5) pass/fail kaldı; bütçe ADR-0027 ile kabul edilecek |
| Kökte placeholder `LICENSE` **tutulmuyor** | 2026-08-24 | GitHub `LICENSE*`'ı hukuki metadata olarak okur; placeholder yanıltıcı. Bkz. [`licensing.md`](licensing.md) |
| Task'ın `adr:` alanı, kararın **verildiği** task'a yazılır — kullandığı task'a değil | 2026-08-24 | `NEN-007` hem ADR-0003'ü hem 0006'yı referans ediyordu; ADR-0003 spike ölçümleri olmadan `accepted` olamayacağı için task'ın kapanışını kilitliyordu. Alan `[6]`'ya indirildi, binding kararı NEN-011/NEN-012'de kaldı. `NEN-008`'de aynı kusur açılışında düzeltildi (`[3]` → `[28]`); `NEN-006`'da da aynı ilkeyle `[5]` → `[]` (ADR-0005 dosyası henüz yok, task onu kararlaştırmıyor); `NEN-011`'de duruyor |
| Swift testleri **swift-testing** ile yazılır (XCTest değil) | 2026-08-24 | CommandLineTools `XCTest.framework` getirmiyor, `Testing.framework` getiriyor. Tam Xcode zorunluluğu M3'e kadar ertelenmiş olsun. Uygulama: `scripts/test-apple.sh` (NEN-007) |
| Spike crate'i **kendi geçici FFI kapısını** açabilir; "tek kapı" kuralı ürün koduyla sınırlı | 2026-08-24 | ADR-0006 kural 2+3 ile CLAUDE.md kural 7 birlikte M1'in FFI ölçümlerini imkânsız kılıyordu. Kural 2'nin koruduğu şey K23 redaction'ının **ürün** yüzeyinde tek noktada denetlenmesi; ürüne linklenmeyen ölçüm binary'si o yüzeyi genişletmiyor. Sınırlar mekanik olarak grep/`cargo metadata` ile doğrulanıyor. Karar: [ADR-0028](adr/0028-spike-ffi-surface.md) |
| Commit **doğrulanmış kapanışta** otomatik atılır; onay kapısı push'a taşındı | 2026-08-24 | Eski kural (kullanıcı istemeden commit yok) her task kapanışında gereksiz bir el sıkışma üretiyordu. Korunmak istenen şey onay değil, **kayıt altına alınanın doğrulanmış olması**: kanıt dolu, testler yeşil, `check-docs.sh` çıkış 0. Geri döndürülmesi pahalı olan işlemler (push, amend, rebase, reset, force, tag) kullanıcıda kaldı |
| **Shared core dili Rust olarak kilitlendi** | 2026-08-24 | NEN-008/009/010/011/029'un beş ölçümü de I1–I5'i kanıtladı, M1'in üç no-go koşulundan hiçbiri tetiklenmedi (reverse-FFI dahil — bkz. ADR-0026). Kotlin Multiplatform/Swift+ayrı Android/C++ hiçbiri ayrı ölçülmedi; zaten kanıtlanmış bir adaydan spike'sız bir alternatife geçmenin gerekçesi yoktu. Karar: [ADR-0002](adr/0002-core-language.md) |
| Beş M1 baseline'ından **marjlı performans bütçesi** kabul edildi | 2026-08-24 | Tek cihaz (Apple M5) ve tek fixture boyutunda ölçülen p50/p95'in üzerine 4–11× marj eklendi; sıkı eşik değil, "bu aralığın dışına çıkarsan tasarımını gözden geçir" işareti. Karar: [ADR-0027](adr/0027-performance-budget.md) |
| **Playback/renderer ownership yönü A (core-owned session, reverse callback)** kabul edildi | 2026-08-24 | `NEN-029`'un fake-adapter ölçümleri A'nın mutlak maliyetinin (60 Hz'de bile ~5 ms/sn, 1000 ms/sn kare bütçesinin binde biri) hiçbir makul UI bütçesini zorlamadığını gösterdi; B'nin ~60× daha ucuz olması bu farkı kararı değiştirecek büyüklükte yapmadı. Asıl gerekçe zaten performans değildi — subtitle sync ve çeviri tetikleme gibi mantığın tek, platformdan bağımsız bir kaynaktan yönetilmesiydi. `NEN-021` port contract'ını bu yön üzerine kurar. Karar: [ADR-0026](adr/0026-playback-renderer-ownership.md) |

## 3. Cevaplanan açık sorular

| # | Soru | Karar | Tarih |
|---|---|---|---|
| **S1** | Dağıtım modeli | **Kişisel kullanım + side-loading.** Public dağıtım ertelendi → S11 | 2026-08-24 |
| **S2** | AI çeviri maliyeti | Kullanıcı **kendi** OpenSubtitles/OpenAI/OpenRouter anahtarını girer. **Hosted backend yok** | 2026-08-24 |
| **S5** | Uzak medya kapsamı | Genel **file/http/https** açma desteklenir. Stremio önemli bir giriş kaynağı, **tek remote kaynak değil** | 2026-08-24 |
| **S6** | Auto-sync gizliliği *(kısmen)* | Varsayılan **localOnly**; remote audio analizi **açık izin** ister. Model seçimi ve kaynak bütçesi hâlâ açık | 2026-08-24 |
| **S8** | Cihazlar arası tercih taşıma | **Cloud sync ilk ürün için non-goal** | 2026-08-24 |
| **S10** | Telemetri / crash raporlama | **İlk ürün için yok** | 2026-08-24 |

## 4. Ertelenmiş kararlar

| Konu | Ne zaman | Bağlı olduğu |
|---|---|---|
| Binding (aday: UniFFI + C ABI) | M1 | ADR-0003 |
| macOS motor + libmpv linkleme/lisans | M3 öncesi | ADR-0012 |
| Lisans ailesi ve public dağıtım | M3 öncesi / sonrası | S11, S12, ADR-0012 → [`licensing.md`](licensing.md) |
| Persistence adapter (aday: SQLite + CAS) | M5 | ADR-0017 |
| Android motor (aday: Media3) | M10 | ADR-0025 |

## 5. Teknoloji karar statüsü

**Kararlı** — ilgili ADR `accepted`:

| Teknoloji | Rol | Karar |
|---|---|---|
| Rust | shared core dili | [ADR-0002](adr/0002-core-language.md) |

**Aday** — ilgili ADR kabul edilene kadar karar sayılmaz:

| Aday | Rol | Kararı verecek |
|---|---|---|
| UniFFI | Swift/Kotlin binding | ADR-0003 |
| C ABI | Windows/Linux binding | ADR-0003 |
| SwiftUI | macOS UI | M3 dönemi |
| Rust HTTP / rustls | paylaşılan HTTP adapter | ADR-0019 dönemi |
| SQLite + content-addressed files | persistence adapter | ADR-0017 |
| libmpv | masaüstü playback motoru | ADR-0012 |
| Media3 | Android playback motoru | ADR-0025 |

**Aday olmayan** (mimari yönün kendisi, spike'tan bağımsız): port sınırları ·
capability modeli · HTTP/persistence **politika sahipliği** · `nen-domain`'in
I/O'suzluğu · tek FFI kapısı (`nen-ffi`).

## 6. ADR indeksi

| Durum | ADR |
|---|---|
| ✅ accepted | [0001](adr/0001-adr-process.md) — ADR süreci |
| 📋 planlanan | 0002–0027 — bkz. [`adr/README.md`](adr/README.md) |

En kritik üçü: **0002** (core dili, M1 kapısı) · **0026** (playback ownership,
NEN-021'in ön koşulu) · **0012** (libmpv linkleme, lisansın ön koşulu).
