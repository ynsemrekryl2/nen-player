---
adr: 0007
title: Subtitle domain modeli — cue kimliği ve timeline fingerprint algoritması
status: accepted
milestone: M2
tasks: [NEN-016]
date: 2026-08-25
---

# ADR-0007 — Subtitle domain modeli — cue kimliği ve timeline fingerprint algoritması

## Durum

`accepted`

## Bağlam

`docs/product-spec.md` §11 (`ValidatedSubtitleArtifact`) ve §12 (`SyncProfile`)
"subtitle timeline fingerprint" ve "source fingerprint"ı birer kimlik bileşeni
olarak varsayıyor: aynı timeline'dan türeyen çeviriler aynı `SyncProfile`'ı
paylaşabilmeli (§12), cache identity ise kaynağın **içeriğine** duyarlı olmalı
(§11 — provider/model, block size, glossary vb. ile birlikte `source
fingerprint`). Bu iki gereksinim ayrı iki değeri gerektiriyor: yalnız
zamanlamaya duyarlı bir `TimelineFingerprint` ve zamanlama + metne duyarlı bir
`SourceFingerprint`.

`core/crates/nen-domain/src/subtitle.rs`'deki `CueId` ve `TimeSpan` doc
yorumları NEN-013'ten beri bu kararı NEN-016'ya erteliyor ("NEN-016 will
define the stable, cross-source cue identity on top of this"); NEN-013/014/015
kapanışlarında `adr: [7]` alanı hep aynı gerekçeyle `[]`e düzeltildi çünkü konu
buraya ait (`docs/DECISIONS.md`).

Karar verilmezse: her tüketici (catalog, translate, sync) kendi ad-hoc
fingerprint hesaplamasını yazar — tutarsız, cache/SyncProfile eşleşmesini
sessizce bozar (`docs/roadmap.md` R6: "Timeline fingerprint kararsızlığı →
SyncProfile/cache eşleşmez").

## Karar

1. **Crate yerleşimi:** fingerprint tipleri ve hesaplaması **`nen-subtitle`**
   crate'ine eklenecektir, `nen-domain`'e değil. `docs/architecture.md`'nin
   crate tablosu "timeline" sorumluluğunu zaten `nen-subtitle`'a veriyor;
   `nen-domain` her kapanışta doğrulanan sıfır-bağımlılık özelliğini korur.

2. **Hash algoritması:** **`blake3`** kullanılacaktır (`core/Cargo.toml`'a yeni
   workspace bağımlılığı, yalnız `nen-subtitle`'da kullanılır).

3. **Kanonik girdi kodlaması** (byte-exact, tekrar üretilebilir):
   - `TimelineFingerprint`: `u32 LE` cue sayısı, ardından doküman sırasıyla
     her cue için `start_ms: u32 LE` + `end_ms: u32 LE` → `blake3::hash(...)`.
   - `SourceFingerprint`: aynı önek, ardından her cue için ayrıca her satır
     uzunluk-önekli (`u32 LE` byte uzunluğu + UTF-8 byte'ları) olarak zamanlama
     byte'larından önce eklenir — farklı satır bölünmelerinin aynı
     concatenation'a düşmesini engeller.
   - `CueId` **hiçbir hash'e dahil değildir.** Sıra zaten doküman sırasıyla
     iterasyonla kodlandığı için (iki cue yer değiştirince id/metin aynı kalsa
     bile byte akışı değişir), DoD'un "cue sırası değişince fingerprint
     değişiyor" maddesi id byte'ı eklemeden karşılanır.

4. **Cue kimliği netleştirmesi:** `subtitle.rs`'deki "NEN-016 will define the
   stable, cross-source cue identity" yorumu şöyle çözülür: **yeni bir kimlik
   tipi eklenmez.** `CueId` doküman-yerel indeks olarak kalır; kaynaklar arası
   eşleştirme (SyncProfile, cache identity) doküman düzeyindeki fingerprint
   üzerinden yapılır (M5/M7), cue başına değil. Yorumlar bu kararla
   güncellenecektir.

5. **Tip biçimi:** `TimelineFingerprint([u8; 32])` / `SourceFingerprint([u8;
   32])`, `Copy + Clone + PartialEq + Eq + Hash`, `Display` küçük harf hex
   olarak (§11'in persistence/cache key ihtiyacı için).

## Gerekçe

**`blake3`:** saf Rust (mobil hedeflerde OpenSSL/sistem kütüphanesi linkleme
derdi yok), deterministic, hızlı; `encoding_rs` (NEN-015) ile aynı ruhta tek
küçük, denetlenmiş bağımlılık. Lisansı `CC0-1.0 OR Apache-2.0` — `Apache-2.0`
kolu `core/deny.toml`'ın izin listesinde zaten var, `cargo deny check`
muhtemelen `deny.toml` değişikliği gerektirmeyecek (uygulama adımında
doğrulanacak).

**`nen-subtitle` (`nen-domain` değil):** `nen-domain`'in sıfır bağımlılık
özelliği her kapanışta (`cargo tree` tek düğüm) ayrı bir kanıt maddesi olarak
tutuluyor; fingerprint hesaplaması saf değer tipi değil, bir hash bağımlılığı
gerektiren *davranış* — `nen-subtitle` zaten `encoding_rs` taşıyor, aynı
sınıfta ikinci bir dış bağımlılık eklemenin maliyeti `nen-domain`'in
invariant'ını bozmaktan düşük.

**Uzunluk-önekli satır kodlaması:** `["ab", "c"]` ile `["a", "bc"]` gibi farklı
satır bölünmeleri, düz concatenation'da aynı byte dizisine düşüp yanlışlıkla
aynı `SourceFingerprint`'i üretebilirdi; uzunluk öneki bunu yapısal olarak
imkânsız kılıyor.

**`CueId` hash'e dahil değil:** id'nin anlamı SRT kaynağının blok sırası
(`subtitle.rs`); iki farklı kaynaktan gelen ama zamanlama/metni birebir aynı
dokümanın aynı `SourceFingerprint`'i üretmesi isteniyor (§12'nin "aynı
timeline'dan türeyen çeviriler aynı SyncProfile'ı kullanabilmeli" gereksinimi)
— id'yi dahil etmek bunu gereksiz yere kırardı.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| `sha2` (SHA-256) | Adversarial preimage senaryosu yok (fingerprint bir cache/identity anahtarı, kriptografik imza değil); `blake3`'ten yavaş, kazanç yok |
| `std::hash::DefaultHasher` (SipHash) | Süreçler arası rastgele seed'li ve algoritma sürüm garantisi yok — persist edilecek/paylaşılacak bir cache key için (§11) uygun değil |
| Fingerprint tiplerini `nen-domain`'e koymak | `nen-domain`'in sıfır bağımlılık invariant'ını (her kapanışta doğrulanan) gereksiz yere bozar; `docs/architecture.md` zaten "timeline"ı `nen-subtitle`'a veriyor |
| `CueId`'yi hash'e dahil etmek | Aynı timeline'dan türeyen farklı-kaynaklı dokümanların aynı fingerprint'i üretmesini engeller — §12'nin SyncProfile paylaşım gereksinimini kırar |
| Satırları düz `\n` ile birleştirip hashlemek | Farklı satır bölünmelerinin aynı concatenation'a düşme riski (`["ab","c"]` vs `["a","bc"]`); uzunluk-önekli kodlama bunu bedavaya çözüyor |

## Sonuçlar

**Olumlu:** cache identity (§11) ve SyncProfile (§12) için tek, denetlenebilir
fingerprint kaynağı; `nen-domain` sıfır bağımlılık kalır; `CueId`'nin kapsamı
netleşir (doküman-yerel, kalıcı bir "cross-source id" değil).

**Olumsuz / kabul edilen maliyet:** `nen-subtitle`'a ikinci bir dış
bağımlılık (`blake3`) giriyor.

**Geri dönüş maliyeti:** ucuz — dış imza (`TimelineFingerprint::of(&doc)` /
`SourceFingerprint::of(&doc)` → `[u8; 32]` sarmalayıcı) sabit kalabilir, iç
hash algoritması bu ADR superseded edilerek değiştirilebilir.

## İlgili task'lar

`NEN-016`

## Notlar

Uygulamada `SourceFingerprint` kodlamasına, Karar §3'te belirtilen uzunluk-
önekli satırların **önüne** bir `line_count: u32 LE` de eklendi. Yalnız
uzunluk-önekleme satır içi ayrışmayı çözüyordu; cue sınırını (satırların
nerede bitip bir sonraki cue'nun zamanlama byte'larının nerede başladığını)
ayrıca sabitlemek için cue başına satır sayısı gerekiyordu — aksi halde farklı
satır/cue sınırı kombinasyonları teorik olarak aynı byte dizisine
düşebilirdi. Kararın amacını değiştirmiyor, yalnız "belirsizlik yok" hedefini
tamamlıyor.
