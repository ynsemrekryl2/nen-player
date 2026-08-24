---
adr: 0002
title: Shared core dili
status: accepted
milestone: M1
tasks: [NEN-008, NEN-009, NEN-010, NEN-011, NEN-012, NEN-029]
date: 2026-08-24
---

# ADR-0002 — Shared core dili

## Durum

`accepted`

## Bağlam

`docs/product-spec.md` §3, shared core için varsayılan aday olarak Rust'ı
belirledi ve karardan önce "Swift ve Kotlin binding, async progress,
cancellation, typed error ve büyük cue listeleri için teknik spike
yapılmalı" dedi. M1'in amacı tam olarak bu: dili **ölçüme dayanarak**
kilitlemek (`docs/milestones/M1-core-spike.md`).

Beş spike task'ı bu ölçümü üretti:

| Task | Ölçüm |
|---|---|
| NEN-008 | 50 000 cue'luk dokümanın FFI üzerinden erişimi (tam liste vs. pencereli) |
| NEN-009 | Async progress ve cooperative cancellation |
| NEN-010 | Typed error mapping (Rust → Swift) |
| NEN-011 | Aynı üç ölçümün Kotlin/JVM paritesi |
| NEN-029 | Reverse-FFI playback ownership (core-owned vs. shell-owned) |

Bu ADR onaylanmadan M2 dışı hiçbir milestone başlamaz
(`docs/roadmap.md` → "Sert kilit").

**NEN-029'un sonucu bu kararın doğrudan girdisidir:** M1'in no-go
kriterlerinden biri "reverse-FFI'ın her iki yönü de (A ve B) kabul
edilemez çıkıyorsa" idi. NEN-029 A yönünü (core-owned session, reverse
callback) ölçülebilir ve kabul edilebilir maliyetli buldu; `ADR-0026`
(accepted, 2026-08-24) bu yönü zaten kilitledi. Yani bu no-go koşulu
tetiklenmedi — Rust'ın playback ownership'i FFI sınırı üzerinden
yönetebildiği ayrı bir kararla kanıtlandı.

Tüm ölçümler `core/spikes/*` altında, `ADR-0028`'in açtığı spike-local FFI
kapısıyla yapıldı; hiçbiri ürün koduna terfi etmedi.

## Karar

**Rust, shared core dili olarak kabul edilecektir.** Application katmanı,
domain katmanı ve platformlar-arası paylaşılan iş mantığı Rust'ta yazılır;
Swift ve Kotlin/JVM'e UniFFI-üretimi binding üzerinden bağlanır (binding
teknolojisinin kendisi bu ADR'nin kapsamı dışında — `ADR-0003`).

## Gerekçe

### Ölçüm sonuçları (baseline, eşik değil)

Ortam: Apple M5 · macOS 27.0 · rustc/cargo 1.98.0 · Swift 6.4 ·
OpenJDK 26.0.2.1 · uniffi 0.32.0 · release build (aksi belirtilmedikçe).

| Ölçüm | Sonuç | Task |
|---|---|---|
| 50k cue, tam liste erişimi | p50 **37.7 ms**, en uzun main-thread bloğu **48.4 ms**, peak RSS **19.5 MiB** | NEN-008 |
| 50k cue, pencereli erişim (40 cue) | p50 **44.3 µs**, en uzun blok **0.08 ms**, peak RSS **11.7 MiB** | NEN-008 |
| `activeCue` arama maliyeti | **1.50 µs** | NEN-008 |
| Cancellation latency (checkpoint_every=1) | p50 **33 µs** | NEN-009 |
| Cancellation latency (checkpoint_every=100) | p50 **2.02 ms** | NEN-009 |
| Typed error eşleme maliyeti (5 varyant) | p50 **2.04 µs**, p95 **2.17 µs** | NEN-010 |
| Kotlin/JVM typed error eşleme maliyeti | Swift p50'sinin **~5.5×**'i, p95'in **~23×**'ü (JIT ısınması + GC, sayısal hata değil) | NEN-011 |
| Cue-transfer checksum (50k cue) | Swift ve Kotlin/JVM'de **birebir aynı** (`75001045577800` / `34337591381145`) | NEN-011 |
| Reverse-FFI per-call maliyeti, 60 Hz (yön A) | p50 **36.67 µs** + MainActor-hop p50 **~40 µs** | NEN-029 |
| Reverse-FFI per-call maliyeti, 60 Hz (yön B) | p50 **1.33 µs**, hop yok | NEN-029 |
| Yön A'nın 60 Hz'de toplam FFI payı | **~4.6 ms/sn** (1000 ms/sn kare bütçesinin binde biri mertebesinde) | NEN-029 |

### Invariant'lar (pass/fail — M1-core-spike.md)

| # | Invariant | Durum | Task |
|---|---|---|---|
| I1 | Cancellation sonrası late callback yok | ✅ sağlandı (release+debug, Swift ve Kotlin/JVM; NEN-029'un A yönünde de) | NEN-009, NEN-011, NEN-029 |
| I2 | Cancellation sonrası late commit yok | ✅ sağlandı | NEN-009 |
| I3 | Typed error string parse gerektirmiyor | ✅ sağlandı — derleme-zamanı exhaustive switch garantisi (4. varyant eklenince `swift build` "switch must be exhaustive" ile kırılıyor) | NEN-010, NEN-011 |
| I4 | Resource/thread sızıntısı yok | ✅ sağlandı (tekrarlı iptal + shutdown, her iki dilde, NEN-029'un iki yönünde de) | NEN-009, NEN-011, NEN-029 |
| I5 | Semantic sonuçlar Swift/Kotlin arasında aynı | ✅ sağlandı — cue-transfer checksum'ları birebir | NEN-011 |

Beş invariant'ın hepsi kanıtlandı; hiçbiri ölçüm sonucuna göre gevşetilmedi.

### No-go kontrolü

`M1-core-spike.md`'nin tanımladığı üç no-go koşulu tek tek değerlendirildi:

1. **Bir invariant sağlanamıyorsa** — sağlanmadı, I1–I5 hepsi kanıtlı. Tetiklenmedi.
2. **Baseline hiçbir makul tasarıma izin vermiyorsa** (ör. pencereli erişim de UI'ı donduruyorsa) — pencereli erişim p50 44.3 µs, main-thread'i 60 fps bütçesinin (16.7 ms) çok altında bloke ediyor. Tetiklenmedi.
3. **Reverse-FFI'ın her iki yönü de kabul edilemezse** — yön A ölçülebilir ve kabul edilebilir çıktı, `ADR-0026` ile kilitlendi. Tetiklenmedi.

**Sonuç: go.** Ölçülen hiçbir sayı veya invariant ihlali Rust adayını
geçersiz kılmadı; Kotlin/JVM tarafındaki tek gözlemlenen sapma (typed-error
eşleme maliyetinin 5.5–23×'i) JIT ısınması ve GC kaynaklı, mutlak
büyüklüğü (düşük mikrosaniye mertebesi) hiçbir kullanıcı senaryosunda fark
edilmez.

## Reddedilen alternatifler

`M1-core-spike.md`'nin no-go durumunda değerlendirilecek alternatifler
olarak listelediği üç seçenek; hiçbiri için ayrı bir spike koşulmadı çünkü
no-go tetiklenmedi.

| Alternatif | Neden reddedildi |
|---|---|
| **Kotlin Multiplatform core** (Kotlin/Native → iOS, JVM/Android native) | Bu alternatif yalnız Rust adayı no-go alırsa değerlendirilecekti — hiçbir invariant ihlal edilmediğinden ve ölçülen maliyet hiçbir makul UI bütçesini zorlamadığından, zaten çalışan ve beş ölçümde kanıtlanmış bir adaydan, hiç spike koşulmamış bir alternatife geçmenin ölçülmüş bir gerekçesi yok. Kotlin/Native'in iOS tarafındaki FFI/tooling olgunluğu bu spike'ın kapsamında hiç test edilmedi — bilinmeyen risk, bilinen (ve kabul edilebilir) maliyetin yerini almaz. |
| **Swift core + Android'de ayrı implementasyon** | İş mantığının (subtitle sync, translation orchestration, source catalog dedup) iki platformda **bağımsız olarak** yazılıp senkron tutulması gerekir — `product-spec.md`'nin "Shared core içinde bulunacaklar" hedefiyle doğrudan çelişir. I5 (Swift/Kotlin arası semantik eşitlik) tek bir Rust implementasyonuyla FFI sınırında **bir kere** kanıtlandı (NEN-011: checksum birebir); iki ayrı implementasyonla bu garanti her yeni özellikte elle yeniden doğrulanması gereken sürekli bir yüke dönüşür. |
| **C++ core** | Bellek güvenliği derleyici tarafından garanti edilmiyor — I4'ün (kaynak/thread sızıntısı yok) Rust'ta ownership modeliyle doğrudan desteklendiği yerde, C++'ta aynı garanti elle disiplinle sağlanmalı. Ayrıca bu proje log redaction (`NEN-006`, K23) gibi güvenlik-duyarlı bir yüzeye sahip; bellek güvenliği olmayan bir dil ek risk taşır. Hiçbir invariant ihlal edilmediğinden bu riski üstlenmenin ölçülmüş bir gerekçesi yok. |

## Sonuçlar

**Olumlu:** Beş spike'ın kanıtladığı zemin üzerine `NEN-013` (strict SRT
parser) ve M2'nin geri kalanı doğrudan başlayabilir. `ADR-0003` (binding
teknolojisi — UniFFI/C ABI) artık yalnız "hangi binding" sorusuna
odaklanabilir, "hangi dil" sorusu kapandı. `docs/architecture.md`'nin
"Karar statüsü" tablosundaki Rust satırı "aday" işaretini kaybeder.

**Olumsuz / kabul edilen maliyet:** Kotlin/JVM tarafında typed-error
eşleme maliyeti Swift'e göre gözle görülür şekilde daha yüksek (5.5–23×,
mutlak değeri hâlâ ihmal edilebilir); M10 (Android) bu farkı gerçek cihazda
yeniden ölçmeli — JVM/desktop ölçümü Android/ART için doğrudan geçerli
değil.

**Geri dönüş maliyeti: pahalı.** Bu karardan sonra M2 (subtitle core),
M3 (macOS vertical slice) ve sonrası Rust API yüzeyi üzerine inşa edilir.
Spike kodu terfi etmediği için (`core/spikes/` altında kalır) bugüne kadar
yazılan kod kaybı sıfır, ama karardan dönmek M2+'nin tüm ürün kodunun
yeniden yazılması anlamına gelir.

## İlgili task'lar

`NEN-008` · `NEN-009` · `NEN-010` · `NEN-011` · `NEN-012` · `NEN-029`

## Notlar

<!-- Karar sonrası gözlemler -->
