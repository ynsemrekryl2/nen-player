---
id: NEN-033
title: OpenSubtitles hash-based identity lookup
milestone: M6
size: M
state: done
depends_on: [NEN-018]
blocks: []
adr: [40]
---

# NEN-033 — OpenSubtitles hash-based identity lookup

## Sonuç

NEN-018'in hesapladığı OpenSubtitles-uyumlu hash, OpenSubtitles'ın resmi
API'sine sorulur ve birebir dosya eşleşmesi varsa kimlik **tahmin edilmeden**
çözülür.

## Kapsam

- Hash → resmi API sorgusu, typed sonuç (`Match` · `NoMatch` · `Ambiguous`)
- Sonucun `MediaEvidence` katman sırasına (ADR-0009 Karar 6) nasıl girdiği
- Hash'i olmayan medyada (boyut sınırı altı, `Range` desteklemeyen sunucu)
  akışın bozulmaması
- Deterministic fake client (CLAUDE.md kural 8 — gerçek kota kullanılamaz)

## YAPILMAYACAK

- Scraping — **non-goal**, yalnız resmi API
- Private file ID / hash / filename metadata loglama — K23 #7, #8
- Aday skorlama ve kullanıcıya gösterim → NEN-035

## Kanıt (DoD)

- [x] Fake client'la birebir eşleşme kimliği çözüyor
- [x] `NoMatch` durumunda alt katmanlara düşülüyor, hata üretilmiyor
- [x] Negatif: private file ID ve hash hiçbir log/`Debug` çıktısında yok
- [x] Negatif: testler gerçek provider kotası kullanmıyor

## Kanıt kaydı

Ortam: Apple M5 · arm64 · macOS 27.0 · rustc/cargo 1.98.0 · Swift 6.3.3 ·
Xcode 26.6 · libmpv 2.5.0.
Tarih: 2026-09-06.

### DoD #1 — fake ve redakte fixture ile birebir eşleşme

`cargo test -p nen-providers -p nen-app -p nen-ports -p nen-identity` geçti.
OpenSubtitles adapter'ı `moviehash` + `moviehash_match=only` sorgusunu, `Api-Key`
ve `User-Agent` header'larını gönderiyor; film ve bölüm fixture'ları tekil
`Match` üretiyor. Aynı kimliğin tekrarları tek eşleşme, farklı kimlikler
`Ambiguous`, boş/exact olmayan sonuç `NoMatch`.

### DoD #2 — yerel fallback ve oynatma güvenliği

`nen-app` testlerinde `NoMatch`, `Ambiguous` ve `Transport` sonuçları mevcut
`MediaEvidence::resolve` akışını değiştirmiyor; yalnız exact `Match` yeni en
 güçlü katman oluyor. Hash yokken provider çağrısı sayacı **0**. Provider
sonucu oynatma akışına bağlanmadı.

### DoD #3 — redaction ve negatif ağ sınırları

Provider integration testleri API key, hash, private `file_id`, filename ve ham
cevabın `Debug` çıktısına girmediğini kanıtlıyor. Bozuk JSON,
1 MiB üstü gövde, HTTP 429, transport hatası, HTTP dışı host, güvensiz HTTP
redirect ve redirect sınırı typed, payload taşımayan hata olarak dönüyor.
`HttpRequest`/FFI request header değerleri ve URL'leri redacted; `URLSession`
adapter'ı header'ları yalnız gerçek request'e aktarıyor.

### DoD #4 — gerçek provider kotası kullanılmadı

Tüm provider testleri `FakeHttpClient`, deterministic `SequenceClient` ve
depodaki redakte JSON fixture'larını kullanıyor; OpenSubtitles'a gerçek ağ
çağrısı, hesap veya kota kullanılmadı.

### Tam doğrulama

```
cargo fmt --all -- --check                         exit 0
cargo clippy --workspace --all-targets -- -D warnings  exit 0
cargo test --workspace                         passed (1 benchmark ignored)
cargo deny check                               exit 0
bash scripts/test.sh                            exit 0
bash scripts/doctor.sh                          exit 0 (bilgilendirici)
bash scripts/doctor.sh M3                       exit 0
bash scripts/test-macos.sh                       182 test / 22 suite / 0 failure
bash scripts/build-macos-app.sh                  exit 0
codesign --verify --deep --strict                 valid on disk
```
