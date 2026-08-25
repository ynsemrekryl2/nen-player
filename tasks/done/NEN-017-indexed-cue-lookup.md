---
id: NEN-017
title: Indexed cue lookup
milestone: M2
size: M
state: done
depends_on: [NEN-016]
blocks: [NEN-027]
adr: []
---

# NEN-017 — Indexed cue lookup

## Sonuç

Zamandan cue'ya erişim, doküman büyüklüğünden bağımsız olarak logaritmik sürede
doğru sonucu verir.

## Kapsam

- `CueIndex`: binary search veya boundary-event yapısı
- `activeCue(at:)` ve `cues(range:)`
- Kenar durumlar: boş doküman, tek cue, örtüşen cue'lar, boşluk (cue yok) anları
- Benchmark: 50k cue

**Eşik:** lookup logaritmik; lineer taramaya karşı ölçüm tablosu zorunlu.

## YAPILMAYACAK

- Renderer entegrasyonu → NEN-027
- Sync offset uygulaması → M7 (index ham zamanlar üzerinde çalışır)

## Kanıt (DoD)

- [x] 10.000 rastgele seek'te index sonucu lineer tarama sonucuyla **birebir** aynı
- [x] Benchmark: 50k cue'da lookup süresi + lineer taramayla karşılaştırma tablosu
- [x] Kenar durumlar: boş doküman, tek cue, örtüşen cue, cue'suz an

## Kanıt kaydı

`core/crates/nen-subtitle/src/index.rs` — `CueIndex<'a>`, ödünç alınmış bir
`SubtitleDocument` üzerinde `build` · `active_cues(at)` · `active_cue(at)` ·
`cues_in(range)`. Yeni bağımlılık yok.

### Üç karar

**1. API çakışan cue'ları düşürmez.** NEN-013 çakışan cue'ları kabul ettiği
için (farklı konuşmacı SRT'leri meşru) bir `t` anında birden fazla cue aktif
olabilir. Birincil sorgu `active_cues(at)` → hepsi, doküman sırasında.
`active_cue(at)` yalnız kolaylık sarmalayıcısı; task metnindeki tekil
`activeCue(at:)` bu adla korundu ama artık *ilk* cue'yu döndürdüğü ve
diğerlerini attığı doküman yorumunda açıkça yazıyor — NEN-027 istiflenmiş
konuşmacıyı sessizce kaybetmesin diye.

**2. Yapı: başlangıca göre sıralı dizi + `max_end_prefix` artırımı.**
`SubtitleDocument` NEN-013'ün `NonMonotonicCue` kuralı sayesinde zaten
`start_ms`'e göre sıralı; yanına tek bir monoton yardımcı dizi ekleniyor
(`max_end_prefix[i] = max(cues[0..=i].end_ms)`). Sorgu iki
`partition_point` + döndürdüğü pencerenin filtresi.

**Reddedilen alternatif — boundary-event segment dizisi.** Sorgusu en kötü
durumda da `O(log n + k)` olurdu, ama belleği değil: segment başına aktif cue
listesi tuttuğu için tamamı birbiriyle çakışan `n` cue'da `O(n²)` giriş
üretir. Bu crate güvenilmeyen girdi ayrıştırıyor (`docs/security-policy.md`
§2) ve `encoding`'in 10 MiB sınırı ~100k cue'ya izin veriyor — kötü niyetli
bir dosyanın belleği tüketebilmesi, patolojik bir dosyanın tek bir sorguyu
yavaşlatmasından çok daha kötü bir başarısızlık biçimi. Seçilen yapı girdi ne
olursa olsun `O(n)` bellekte kalıyor.

Bedeli açıkça kayda geçiyor: sorgunun filtrelediği aday penceresi çakışma
derinliğiyle büyür, yani dokümanın tamamını kaplayan tek bir cue o sorguyu
taramaya yaklaştırır. Bu şekil (`generated/one-long-cue`) parity korpusunda
var — doğruluk bozulmuyor.

**3. ADR yazılmadı.** Yapı `nen-subtitle` içinde kalıyor, crate sınırı veya
bağımlılık grafiği değişmiyor, yeni dış crate yok; `docs/product-spec.md` §14
zaten "lineer bütün-liste taraması olmasın" diyor. Karar burada, kanıt
kaydında. `adr:` alanı `[7]` → **`[]`** düzeltildi — NEN-013/014/015 ile aynı
precedent: ADR-0007'nin konusu (cue kimliği ve fingerprint algoritması)
NEN-016'nın kararıydı, bu task onu kararlaştırmıyor.

### DoD #1 — 10 000 seek, lineer taramayla birebir

`core/crates/nen-subtitle/tests/lookup_parity.rs`. Sabit tohumlu
(`0x4E45_4E30_3137`) xorshift64* üreteci — `fuzz_smoke.rs`'in deseni.
Referans implementasyon (`support::linear_active_cues`) kasten mümkün olan en
aptal hali: tüm cue'ları filtreleyen tek bir `iter().filter()`.

Korpus 14 doküman: `fixtures/subtitles/valid/` içindeki 8 gerçek fixture +
6 sentetik şekil (boş · tek cue · 200 cue boşluklu · derinlik 3 · derinlik 8 ·
dokümanın tamamını kaplayan tek cue). Seek uzayı dokümanın sınırlarının
dışını da kapsıyor.

```
$ cargo test --manifest-path core/Cargo.toml -p nen-subtitle --test lookup_parity
test ten_thousand_random_seeks_match_a_linear_scan ... ok   (10 000 seek)
test random_range_queries_match_a_linear_scan ... ok        (10 000 aralık)
test every_cue_boundary_is_sampled_exactly ... ok
test result: ok. 3 passed; 0 failed
```

Üçüncü test rastgeleliğin kapatamadığı yeri kapatıyor: rastgele seek'ler
sınır milisaniyelerine ancak kazara düşer, bu yüzden korpustaki her cue'nun
`start_ms - 1` · `start_ms` · `end_ms - 1` · `end_ms` anları ayrıca
karşılaştırılıyor.

**Negatif kontrol (kanıtın boşta dönmediği):** parity testinin gerçek bir
hatayı yakaladığı, `index.rs`'e iki yönde kasıtlı hata sokularak doğrulandı —
NEN-006/NEN-010'daki bozuk-fixture tekniğiyle aynı ruh, kalıcı iz bırakmadan:

| Sabotaj | Etkisi | Sonuç |
|---|---|---|
| Sondaki `end_ms > from` filtresi kaldırıldı | fazla cue döner | 3/3 parity testi FAILED (`overlapping-cues.srt: active cues at 3958 ms disagree with a linear scan`) |
| `max_end_prefix` sınırı bir ms fazla ilerletildi | eksik cue döner | 3/3 parity testi FAILED (`crlf.srt: boundary 1999 ms of cue 1 disagrees with a linear scan`) |

İkisinden sonra dosya orijinaline geri alındı.

### DoD #2 — 50k cue benchmark

`core/crates/nen-subtitle/tests/lookup_bench.rs`, `#[ignore]`'lu — normal
`cargo test` ve CI etkilenmiyor. `bash scripts/bench-cue-lookup.sh` (yeni)
release'de koşturup `--nocapture` ile tabloyu basıyor. **Bu bir baseline'dır,
eşik değil** — test hiçbir süre iddia etmiyor (NEN-008/009 precedent'i).

Yöntem: tek bir lookup saatin kendi okuma maliyeti mertebesinde olduğu için
her örnek bir *batch* lookup'ı ölçüp bölüyor (index batch 100, lineer batch
10 — lineer tarama yüzlerce kat yavaş); 100 örnek; yüzdelikler batch
ortalamaları üzerinden. Her sonuçtan bir checksum toplanıp basılıyor, yani
optimizer ölçülen işi silemiyor.

Apple M5 · macOS 27.0 · rustc 1.98.0 · release (3 koşunun aralığı):

| doküman | build | index p50 | index p95 | lineer p50 | hızlanma |
|---|---|---|---|---|---|
| 50k cue, çakışma yok | 20–28 µs | **48–53 ns** | 52–57 ns | 16.4–17.6 µs | **315–344×** |
| 50k cue, derinlik 3 | 20–24 µs | **50–57 ns** | 54–69 ns | 15.6–16.5 µs | **288–318×** |

Debug (karşılaştırma için): index p50 420–457 ns, lineer p50 ~212 µs —
yani release'e göre index ~8×, lineer tarama ~13× yavaş; oran korunuyor.

**Ölçüm yöntemi bir kusur bulup düzeltti.** İlk sürümde tablo satırlarının
sırası sonucu değiştiriyordu: hangi satır önce koşarsa ~2.5× yavaş çıkıyordu
(sırayı ters çevirince sapma da tersine döndü — 134 ns / 57 ns → 209 ns /
76 ns). Sebep lookup değil, taze ayrılmış 50 000 cue'luk dokümanın ilk
dokunuş page-fault'ları. Düzeltme: her satır iki kez ölçülüyor, yalnız
ikincisi raporlanıyor. Düzeltmeden sonra iki satır da 48–57 ns'de birleşti ve
sıra etkisi kayboldu.

### DoD #3 — kenar durumlar

`index.rs` içinde 10 unit test:

```
test index::tests::empty_document_has_nothing_active ... ok
test index::tests::single_cue_is_active_only_inside_its_span ... ok
test index::tests::boundaries_are_half_open ... ok
test index::tests::moments_before_after_and_between_cues_are_empty ... ok
test index::tests::overlapping_cues_are_all_returned_in_document_order ... ok
test index::tests::a_long_cue_stays_visible_behind_later_ones ... ok
test index::tests::range_query_returns_every_overlapping_cue ... ok
test index::tests::empty_and_reversed_ranges_return_nothing ... ok
test index::tests::lookup_at_the_maximum_timestamp_does_not_overflow ... ok
test index::tests::prefix_maximum_is_non_decreasing ... ok
```

Semantik: cue `t`'de aktiftir ⟺ `start_ms <= t < end_ms` (yarı açık,
`TimeSpan`'in kendi yorumuyla ve NEN-008 spike'ının `active_cue`'suyla
tutarlı) — bir cue'nun bittiği ms'de artık ekranda değildir. Aralık dışı
sorgu, boş aralık ve ters aralık **hata değil**, boş sonuç.
`active_cues(u32::MAX)` taşmıyor (`saturating_add`).

### Doğrulama

```
$ cargo test --manifest-path core/Cargo.toml --workspace
108 passed → 121 passed, 0 failed, 1 ignored (benchmark)
  (+10 index unit · +3 lookup_parity)

$ cargo clippy --workspace --all-targets --manifest-path core/Cargo.toml -- -D warnings
                                                        → exit 0, uyarı yok
$ cargo fmt --all --check --manifest-path core/Cargo.toml → exit 0
$ (cd core && cargo deny check)
advisories ok, bans ok, licenses ok, sources ok         → exit 0 (yeni bağımlılık yok)

$ cargo tree -p nen-subtitle --edges normal
nen-subtitle → blake3 · encoding_rs · nen-domain        → kenarlar DEĞİŞMEDİ
$ cargo tree -p nen-domain --edges normal
nen-domain v0.1.0                                       → tek düğüm, sıfır bağımlılık

$ bash scripts/bench-cue-lookup.sh            → yukarıdaki tablo (release)
$ bash scripts/bench-cue-lookup.sh --debug    → debug karşılaştırması
$ bash scripts/check-docs.sh                  → 8/8 denetim geçti, exit 0
$ bash scripts/test.sh                        → exit 0
```

Dokunulan dosyalar: yeni `core/crates/nen-subtitle/src/index.rs`,
`core/crates/nen-subtitle/src/lib.rs` (+`pub mod index`), yeni
`core/crates/nen-subtitle/tests/lookup_parity.rs`, yeni
`core/crates/nen-subtitle/tests/lookup_bench.rs`,
`core/crates/nen-subtitle/tests/support/mod.rs` (`Rng` · deterministik
doküman üreteci · lineer referans implementasyonlar), yeni
`scripts/bench-cue-lookup.sh`.
