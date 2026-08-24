---
id: NEN-016
title: SubtitleDocument and timeline fingerprint
milestone: M2
size: M
state: done
depends_on: [NEN-013]
blocks: [NEN-017, NEN-019]
adr: [7]
---

# NEN-016 — SubtitleDocument and timeline fingerprint

## Sonuç

Bir altyazının **zamanlaması** ile **içeriği** ayrı ayrı parmak izlenir; aynı
timeline'dan türeyen çeviriler aynı timeline fingerprint'ini paylaşır.

## Kapsam

- `SubtitleDocument` modeli
- `TimelineFingerprint` — yalnız cue zamanlarından
- `SourceFingerprint` — içerik (metin dahil) dahil
- Fingerprint'lerin kararlılık ve duyarlılık testleri

## YAPILMAYACAK

- Cache identity'nin tamamı → M5 / ADR-0018
- SyncProfile → M7 (yalnız fingerprint burada üretilir)
- Media fingerprint → NEN-018

## Kanıt (DoD)

- [x] Aynı timeline → aynı `TimelineFingerprint`
- [x] Tek cue 1 ms kayınca `TimelineFingerprint` **değişiyor**
- [x] Metin değişip zamanlar aynı kalınca `TimelineFingerprint` **değişmiyor**,
      `SourceFingerprint` **değişiyor**
- [x] Cue sırası değişince fingerprint değişiyor

## Kanıt kaydı

**Ön koşul: `ADR-0007` kabul edildi** ("Subtitle domain modeli — cue kimliği
ve timeline fingerprint algoritması"). Karar: hash `blake3`; tipler
`nen-domain`'e değil `nen-subtitle`'a eklendi (`nen-domain`'in sıfır
bağımlılık özelliği korunuyor, `docs/architecture.md` "timeline"ı zaten
`nen-subtitle`'a veriyor); `TimelineFingerprint` = cue sayısı + sıralı
`start_ms`/`end_ms`; `SourceFingerprint` = aynısı + cue başına satır sayısı +
uzunluk-önekli satır byte'ları; `CueId` hiçbir hash'e dahil değil. Ayrıca
`subtitle.rs`'deki "NEN-016 will define the stable, cross-source cue
identity" yorumları ADR'nin kararıyla güncellendi: yeni bir kimlik tipi
eklenmiyor, kaynaklar arası eşleştirme doküman düzeyindeki fingerprint
üzerinden yapılıyor.

`core/crates/nen-subtitle/src/fingerprint.rs` — `TimelineFingerprint::of`
ve `SourceFingerprint::of`, ikisi de `[u8; 32]` sarmalayıcı,
`Display`/`Debug` küçük harf hex basıyor. 8 unit test, DoD'un dört maddesini
birebir karşılıyor + üç ek durum (`CueId` fingerprint'i etkilemiyor, farklı
satır bölünmesi `SourceFingerprint`'i değiştiriyor, boş doküman panik
vermiyor):

```
$ cargo test --manifest-path core/Cargo.toml -p nen-subtitle fingerprint
running 8 tests
test fingerprint::tests::empty_document_does_not_panic ... ok
test fingerprint::tests::different_line_split_changes_source_fingerprint ... ok
test fingerprint::tests::text_change_changes_source_not_timeline ... ok
test fingerprint::tests::same_timeline_same_timeline_fingerprint ... ok
test fingerprint::tests::one_ms_shift_changes_timeline_fingerprint ... ok
test fingerprint::tests::display_is_lowercase_hex_of_64_chars ... ok
test fingerprint::tests::cue_order_change_changes_both_fingerprints ... ok
test fingerprint::tests::cue_id_does_not_affect_either_fingerprint ... ok
test result: ok. 8 passed; 0 failed
```

```
$ cargo test --manifest-path core/Cargo.toml --workspace
100 passed, 0 failed → 108 passed, 0 failed (NEN-016 ile +8)

$ cargo fmt --all --check --manifest-path core/Cargo.toml       → exit 0
$ cargo clippy --workspace --all-targets --manifest-path core/Cargo.toml -- -D warnings
                                                                  → exit 0, uyarı yok
$ (cd core && cargo deny check)
advisories ok, bans ok, licenses ok, sources ok                 → exit 0
  (blake3 = "CC0-1.0 OR Apache-2.0"; Apache-2.0 kolu deny.toml'ın
  izin listesinde zaten vardı, deny.toml'a dokunulmadı — ADR-0007'nin
  öngörüsü doğrulandı)

$ bash scripts/check-docs.sh   → 8/8 denetim geçti, exit 0
```

Dokunulan dosyalar: `core/Cargo.toml` (+blake3 workspace dependency),
`core/crates/nen-subtitle/Cargo.toml` (+blake3), yeni
`core/crates/nen-subtitle/src/fingerprint.rs`,
`core/crates/nen-subtitle/src/lib.rs` (+`pub mod fingerprint`, modül dokümanı
güncellendi), `core/crates/nen-domain/src/subtitle.rs` (üç doc yorumu
ADR-0007'nin kararıyla güncellendi, davranış değişmedi), yeni
`docs/adr/0007-subtitle-domain-model.md` (`accepted`), `docs/adr/README.md`
("Yazılmış" tablosuna taşındı, "Planlanan"dan çıkarıldı).
