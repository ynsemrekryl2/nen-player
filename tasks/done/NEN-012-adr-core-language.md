---
id: NEN-012
title: Spike report and core language decision
milestone: M1
size: S
state: done
depends_on: [NEN-011, NEN-029]
blocks: [NEN-013, NEN-018, NEN-021]
adr: [2, 27]
---

# NEN-012 — Spike report and core language decision

## Sonuç

Shared core dili ADR-0002 ile **kilitlenmiştir**; kararın arkasında NEN-008/009/
010/011/029'dan gelen gerçek sayılar ve invariant kanıtları vardır.

## Kapsam

- Spike raporu: **beş** ölçümün baseline sonuçları (bağlamlarıyla) ve
  invariant'ların (I1–I5) durumu
- ADR-0002: karar, gerekçe, reddedilen alternatifler
  (Kotlin Multiplatform · Swift core + ayrı Android · C++ core), geri dönüş maliyeti
- **NEN-029'un ownership sonucu bu kararın girdisidir** — reverse-FFI her iki
  yönde de kabul edilemezse Rust adayı `no-go` alabilir
- ADR-0027 taslağı: baseline'lardan çıkan performans bütçesi önerisi
- `no-go` çıkarsa: yeni plan taslağı ve etkilenen milestone listesi

> `no-go`, "baseline beklenenden yavaş" demek **değildir** — bütçe ADR-0027 ile
> ayarlanabilir. `no-go` yalnız bir invariant sağlanamazsa, baseline hiçbir makul
> tasarıma izin vermiyorsa, veya reverse-FFI'ın **her iki yönü** de kabul
> edilemezse verilir.

## YAPILMAYACAK

- Spike kodunu ürüne terfi ettirmek — `core/spikes/` altında kalır
- Karar öncesi M2 dışındaki milestone'lara başlamak

## Kanıt (DoD)

- [x] ADR-0002 `accepted` ve beş ölçümün **sayıları + bağlamı** içinde
- [x] Invariant'lar I1–I5'in her birinin durumu tabloda
- [x] Reddedilen alternatifler bölümü dolu
- [x] NEN-029 ownership sonucunun karara etkisi yazılmış
- [x] go/no-go sonucu, yukarıdaki `no-go` tanımına göre açıkça gerekçelendirilmiş

## Kanıt kaydı

**ADR-0002 accepted (2026-08-24), kullanıcı onayı bu oturumda alındı.**
Karar: Rust shared core dili. Gerekçe ve tam sayı tablosu:
[`docs/adr/0002-core-language.md`](../../docs/adr/0002-core-language.md).

Beş ölçümün özeti (tam bağlam ADR-0002'de):

| Ölçüm | Sonuç | Task |
|---|---|---|
| 50k cue, pencereli erişim | p50 44.3 µs, peak RSS 11.7 MiB | NEN-008 |
| 50k cue, tam liste | p50 37.7 ms, en uzun blok 48.4 ms | NEN-008 |
| Cancellation latency | 33 µs (checkpoint=1) – 2.02 ms (checkpoint=100) | NEN-009 |
| Typed error eşleme | p50 2.04 µs, p95 2.17 µs (Swift) | NEN-010 |
| Kotlin/JVM parity | checksum'lar birebir aynı; eşleme maliyeti Swift'in 5.5–23×'i (JIT/GC, sayısal hata değil) | NEN-011 |
| Reverse-FFI (yön A, 60 Hz) | p50 36.67 µs + hop ~40 µs, ~4.6 ms/sn toplam | NEN-029 |

**Invariant tablosu (I1–I5): hepsi ✅ sağlandı** — tam tablo ADR-0002 →
"Invariant'lar" bölümünde.

**Reddedilen alternatifler:** Kotlin Multiplatform · Swift core + ayrı
Android implementasyonu · C++ core — üçü de M1-core-spike.md'nin no-go
fallback listesindendi; hiçbiri ayrıca spike edilmedi çünkü no-go
tetiklenmedi. Gerekçe: ADR-0002 → "Reddedilen alternatifler".

**NEN-029 ownership etkisi:** M1'in üçüncü no-go koşulu ("reverse-FFI'ın
her iki yönü de kabul edilemezse") NEN-029'un A yönünü kabul edilebilir
bulmasıyla tetiklenmedi; `ADR-0026` (accepted, 2026-08-24) bu yönü zaten
kilitlemişti — ADR-0002 bunu doğrudan girdi olarak kullandı.

**Go/no-go sonucu: GO.** `M1-core-spike.md`'nin üç no-go koşulu tek tek
kontrol edildi (invariant ihlali yok · baseline hiçbir tasarımı
engellemiyor · reverse-FFI'ın en az bir yönü kabul edilebilir) — hiçbiri
tetiklenmedi. Tam gerekçe: ADR-0002 → "No-go kontrolü".

**ADR-0027 (performans bütçesi) da bu task kapsamında `proposed` yazılıp
kullanıcı onayıyla `accepted` oldu** — dört bütçe (cue erişimi pencereli/tam,
cancellation latency, reverse-FFI per-call), ölçülen p50/p95'in üzerine
4–11× marj ile. Tam metin:
[`docs/adr/0027-performance-budget.md`](../../docs/adr/0027-performance-budget.md).

**Güncellenen dokümanlar:** `docs/architecture.md` (Rust "aday" işaretini
kaybetti), `docs/DECISIONS.md` (Rust "Kararlı" tablosuna taşındı, section 4
Ertelenmiş kararlar'dan iki satır çıktı), `docs/roadmap.md` (M1 → kapandı,
M2 → sıradaki).

**Yan bulgu (bu task kapsamı dışı, ayrı arka plan görevi işaretlendi):**
`docs/DECISIONS.md`'nin "Ertelenmiş kararlar" tablosu ADR-0026'yı hâlâ
bekleyen olarak listeliyor — NEN-029 kapanışında güncellenmemiş, bu task
onu düzeltmedi (kural 5: alakasız iyileştirme yeni task'a gider).
