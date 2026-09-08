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
| [0031](0031-macos-shell-interaction-model.md) | macOS kabuk etkileşim modeli — hata sunumu, ekran gizliliği ve ayar yüzeyi | ✅ accepted | M3 |
| [0012](0012-macos-playback-engine.md) | macOS playback motoru, linkleme modeli ve proje lisansı | ✅ accepted | M3 |
| [0013](0013-subtitle-renderer-strategy.md) | `SubtitleRenderer` stratejisi — engine-native çizim ve portun sınırları | ✅ accepted | M3 |
| [0032](0032-container-language-codes.md) | Konteynerin ISO 639-2 dil kodları tek kanonik etikete indirgenir | ✅ accepted | M3 |
| [0033](0033-playback-session-ffi-surface.md) | Çekirdek playback oturumunun FFI yüzeyi ve olay teslimat yönü | ✅ accepted | M3 |
| [0034](0034-macos-sandbox-and-sidecar-access.md) | macOS dağıtımı sandbox'sızdır; sidecar erişimi buna dayanır | ✅ accepted | M3 |
| [0035](0035-subtitle-source-reason-labels.md) | Altyazı kaynağı sebep etiketleri — kapalı küme iki elemanlıdır | ✅ accepted | M3 |
| [0036](0036-video-presentation-metadata.md) | Video presentation metadata playback portundan geçer | ❌ rejected | M3 |
| [0037](0037-subtitle-safe-area.md) | Kabuk kromunun örttüğü bant renderer'a bildirilir | ✅ accepted | M3 |
| [0038](0038-video-display-geometry.md) | Video display geometry playback portundan geçer | ✅ accepted | M3 |
| [0039](0039-remote-media-http-boundary.md) | Uzak medya HTTP sınırı ve macOS adapter'ı | ✅ accepted | M3 |
| [0040](0040-opensubtitles-identity-provider-boundary.md) | OpenSubtitles hash kimlik provider sınırı | ✅ accepted | M6 |
| [0041](0041-sidecar-discovery-scope.md) | Sidecar keşfi medyanın dizinini bir kez listeler | ✅ accepted | M3 |
| [0042](0042-seek-during-loading.md) | Yüklenirken verilen seek reddedilmez, ertelenir | ✅ accepted | M3 |
| [0043](0043-macos-handoff-surface.md) | macOS handoff alıcı yüzeyi — argv + open-with + custom scheme | ✅ accepted | M4 |
| [0044](0044-stremio-mpv-bridge.md) | Stremio macOS MPV launcher için geri alınabilir Nen Player köprüsü | ✅ accepted | M4 |
| [0015](0015-translation-block-strategy.md) | Çeviri blok stratejisi — boyut, overlap ve belge bağlamı | 🟡 proposed | M5 |
| [0016](0016-translation-validation-and-repair.md) | Çeviri doğrulama ve onarım politikası — yerel doğrulama authoritative | 🟡 proposed | M5 |
| [0017](0017-artifact-persistence-adapter.md) | Doğrulanmış artifact'lerin persistence adapter'ı | 🟡 proposed | M5 |
| [0018](0018-cache-identity-and-invalidation.md) | Cache identity bileşenleri, versiyonlama ve invalidasyon | 🟡 proposed | M5 |
| [0045](0045-embedded-text-demux-path.md) | Gömülü altyazı metninin demux yolu | 🟡 proposed | M5 |
| [0002](0002-core-language.md) | Shared core dili — go/no-go kapısı | ✅ accepted | M1 |
| [0027](0027-performance-budget.md) | Performans bütçeleri — M1 baseline'larından marjlı kabul | ✅ accepted | M1 sonrası |

## Planlanan

Bu liste bir taahhüt değil, haritadır. Yeni ihtiyaç çıkarsa araya ADR girer;
numaralar sıradan verilir, aşağıdaki numaralar rezerve **değildir**.

Başlıklardaki teknoloji adları **aday**dır — ilgili ADR kabul edilene kadar
karar verilmiş sayılmaz (bkz. `docs/architecture.md` → "Karar statüsü").

| ADR | Konu | Ne zaman |
|---|---|---|
| 0003 | FFI binding stratejisi (aday: UniFFI + C ABI); büyük veri geçiş modeli | M1 |
| 0004 | Async / cancellation / progress kontratı ve late-commit yasağı | M1 |
| 0005 | Typed error taksonomisi ve redaction kuralları | M1 |
| 0014 | Stremio handoff kontratı — Android Intent tarafı ve log yasakları (macOS yarısı ADR-0043 ile kapandı) | M10 |
| 0019 | Provider port soyutlaması ve capability preflight | M6 |
| 0020 | Secure credential storage haritası | M6 |
| 0021 | OpenSubtitles entegrasyon sınırları: opaque public ID, indirme güvenliği | M6 |
| 0022 | SyncProfile modeli ve anahtarlama; artifact ile paylaşım | M7 |
| 0023 | Audio auto-sync pipeline ve confidence eşikleri | M8 |
| 0024 | Audio privacy modeli: localOnly varsayılanı, remote izin akışı | M8 |
| 0025 | Android motor kararı: Media3 vs. alternatif | M10 |

## Kritik üçü

**ADR-0002** M1'in çıkış kapısıydı — core dili burada kilitlendi (`go`,
2026-08-24). Karar kapandı, aday değil: shared core Rust.

**ADR-0026** playback/renderer ownership yönünü belirledi (2026-08-24);
`NEN-021` bunun üzerine kuruldu ve kapandı.

**ADR-0012** M3'ün ikinci kapısıydı ve kapandı (2026-08-26): macOS motoru
**libmpv**, adapter Swift'te (`platforms/macos/`), geliştirmede dinamik link /
dağıtımda `.app` içine gömme (`NEN-043`), ve proje lisansı
**GPL-3.0-or-later**. Böylece `NEN-022` başlayabilir, `docs/licensing.md`
karara bağlandı ve roadmap **S12** kapandı; açık kalan tek şey **S11** (public
dağıtım) — App Store yolu GPL ile uyumsuz olduğu için bilerek kapalı.
