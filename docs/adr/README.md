# Mimari Kararlar (ADR)

Süreç: [`0001-adr-process.md`](0001-adr-process.md) · Şablon:
[`0000-template.md`](0000-template.md)

## Yazılmış

| ADR | Başlık | Durum | Milestone |
|---|---|---|---|
| [0001](0001-adr-process.md) | ADR süreci | ✅ accepted | M0 |
| [0006](0006-monorepo-and-crate-boundaries.md) | Monorepo yapısı ve crate sınırları | ✅ accepted | M1 |
| [0026](0026-playback-renderer-ownership.md) | Playback/renderer ownership yönü | ✅ accepted | M1 |
| [0028](0028-spike-ffi-surface.md) | Spike'ların geçici FFI yüzeyi | ✅ accepted | M1 |
| [0008](0008-encoding-detection-and-sanitization.md) | Encoding tespiti ve sanitization politikası | ✅ accepted | M2 |
| [0007](0007-subtitle-domain-model.md) | Subtitle domain modeli — cue kimliği ve timeline fingerprint algoritması | ✅ accepted | M2 |
| [0009](0009-media-evidence-and-identity.md) | Media evidence / identity çözümleme ve fallback sırası | ✅ accepted | M2 |
| [0010](0010-subtitle-source-catalog.md) | SubtitleSourceCatalog gruplama, dedup ve menü projeksiyon kuralları | ✅ accepted | M2 |
| [0029](0029-subtitle-language-detection.md) | Subtitle dili tespiti, güven eşiği ve metadata çelişki politikası | ✅ accepted | M2 |
| [0030](0030-language-group-granularity.md) | Menü gruplaması ve tercih eşleşmesi primary subtag üzerinden | ✅ accepted | M3 |
| [0011](0011-playback-port-contract.md) | `PlaybackEngine` capability modeli ve contract test yaklaşımı | ✅ accepted | M3 |
| [0031](0031-macos-shell-interaction-model.md) | macOS kabuk etkileşim modeli — hata sunumu, ekran gizliliği ve ayar yüzeyi | 🟡 proposed | M3 |

## Planlanan

Bu liste bir taahhüt değil, haritadır. Yeni ihtiyaç çıkarsa araya ADR girer;
numaralar sıradan verilir, aşağıdaki numaralar rezerve **değildir**.

Başlıklardaki teknoloji adları **aday**dır — ilgili ADR kabul edilene kadar
karar verilmiş sayılmaz (bkz. `docs/architecture.md` → "Karar statüsü").

| ADR | Konu | Ne zaman |
|---|---|---|
| 0002 | **Shared core dili kararı** (aday: Rust) — go/no-go kapısı | M1 |
| 0003 | FFI binding stratejisi (aday: UniFFI + C ABI); büyük veri geçiş modeli | M1 |
| 0004 | Async / cancellation / progress kontratı ve late-commit yasağı | M1 |
| 0005 | Typed error taksonomisi ve redaction kuralları | M1 |
| 0012 | **macOS motor seçimi** (aday: libmpv) — dağıtım, linkleme, lisans | M3 |
| 0013 | `SubtitleRenderer` stratejisi: engine-native vs. custom overlay | M3 |
| 0014 | Stremio handoff kontratı (Android Intent + macOS argüman) ve log yasakları | M4 |
| 0015 | Translation blok stratejisi: boyut, overlap, context analizi | M5 |
| 0016 | Validation ve repair politikası; authoritative local validation | M5 |
| 0017 | Persistence adapter (aday: SQLite index + content-addressed store), atomik commit | M5 |
| 0018 | Cache identity bileşenleri ve versiyonlama/invalidasyon | M5 |
| 0019 | Provider port soyutlaması ve capability preflight | M6 |
| 0020 | Secure credential storage haritası | M6 |
| 0021 | OpenSubtitles entegrasyon sınırları: opaque public ID, indirme güvenliği | M6 |
| 0022 | SyncProfile modeli ve anahtarlama; artifact ile paylaşım | M7 |
| 0023 | Audio auto-sync pipeline ve confidence eşikleri | M8 |
| 0024 | Audio privacy modeli: localOnly varsayılanı, remote izin akışı | M8 |
| 0025 | Android motor kararı: Media3 vs. alternatif | M10 |
| **0027** | **Performans bütçeleri** — M1 baseline'ları ve gerçek kullanım sonrası kabul | **M1 sonrası** |

## Kritik üçü

**ADR-0002** M1'in çıkış kapısıdır — core dili burada kilitlenir. `no-go`
çıkarsa M2 dışındaki tüm plan yeniden yazılır.

**ADR-0026** playback/renderer ownership yönünü belirler. Tüm M3 ve M7'nin
position çözünürlüğü buna bağlı; `NEN-021` bu ADR kabul edilmeden başlamaz.

**ADR-0012** libmpv'nin lisans ve linkleme modelini belirler; lisans seçimi
(`docs/licensing.md`, S12) ve public dağıtım (S11) buna bağlıdır.
