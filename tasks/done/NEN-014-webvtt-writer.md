---
id: NEN-014
title: WebVTT writer
milestone: M2
size: S
state: done
depends_on: [NEN-013]
blocks: []
adr: []
---

# NEN-014 — WebVTT writer

## Sonuç

`SubtitleDocument`, byte düzeyinde kararlı UTF-8 WebVTT olarak yazılır.

## Kapsam

- UTF-8 WebVTT çıktısı (şartname §10 gereği final format)
- Zaman formatı, cue ayırıcı, metin kaçışları
- SRT → doc → WebVTT round-trip golden snapshot

## YAPILMAYACAK

- WebVTT **okuma** (parse) — şu an ihtiyaç yok, gerekirse ayrı task
- Stil/konum (cue settings) — kaynakta yoksa üretilmez

## Kanıt (DoD)

- [x] Round-trip golden: SRT → WebVTT snapshot byte-eşit
- [x] Çıktı UTF-8 ve BOM'suz
- [x] Cue ID/sıra/zamanlar girdiyle **aynen** aynı

## Kanıt kaydı

`core/crates/nen-subtitle/src/webvtt.rs` — `write(&SubtitleDocument) -> String`:
`WEBVTT` header, cue başına identifier (`CueId`) + `HH:MM:SS.mmm --> HH:MM:SS.mmm`
zaman satırı + metin satırları (`&`/`<`/`>` kaçışlı), boş satır ayırıcı. BOM hiç
yazılmıyor. `src/lib.rs`'e `pub mod webvtt;` eklendi.

Fixture korpusu genişletildi: mevcut 7 `valid/*.srt`'e ek olarak
`html-special-chars.srt` (kaçış kapsamı için `&`/`<`/`>` içeren 2 cue) eklendi
— kendi `.golden` (SRT parse) snapshot'ı da üretildi. 8 fixture'ın hepsi için
`webvtt::write` çıktısı `.vtt` golden'ı olarak commit edildi.

Yeni test dosyası `core/crates/nen-subtitle/tests/webvtt_roundtrip.rs`:
`every_valid_fixture_round_trips_to_its_vtt_golden` (DoD #1), `output_is_utf8_and_bom_free`
(DoD #2), `cue_identity_and_order_survive_the_round_trip` (DoD #3, WebVTT okuma
kapsam dışı olduğu için reparse değil, yazılan metnin id/zaman satırlarının
doğru sırada bulunmasıyla doğrulandı). `webvtt.rs` içinde ayrıca 3 birim testi:
zaman formatı (0 ve `MAX_TIMESTAMP_MS`), `&` önce kaçışlanıp `<`/`>`'nin tekrar
kaçışlanmadığı, sıradan unicode metnin dokunulmadan geçtiği.

`adr:` alanı `[7]` → **`[]`** düzeltildi — NEN-013'teki aynı gerekçe: ADR-0007
dosyası `docs/adr/` altında henüz yok (subtitle domain modeli, cue kimliği ve
timeline fingerprint algoritması konusu NEN-016'nın kararı), bu task onu
kararlaştırmıyor.

Doğrulama (2026-08-24, Apple M5 · arm64 · macOS 27.0 26A5416b · rustc/cargo 1.98.0):

```
$ cargo test -p nen-subtitle --manifest-path core/Cargo.toml
unittests   10 passed (srt 7 + webvtt 3)
fuzz_smoke   4 passed
golden_valid 3 passed
guard_error_debug 4 passed
malformed    3 passed
webvtt_roundtrip 3 passed             → NEN-014'ün üç DoD testi
                                       → toplam 27 passed, 0 failed

$ cargo test --manifest-path core/Cargo.toml --workspace  → tümü yeşil (73 → 82,
  NEN-013'ün 72'sine bu task'ın 10 testi eklendi)

$ cargo clippy --workspace --all-targets --manifest-path core/Cargo.toml -- -D warnings
                                       → exit 0, uyarı yok
$ (cd core && cargo fmt --check)      → exit 0

$ cargo tree -p nen-subtitle --edges normal --manifest-path core/Cargo.toml
nen-subtitle → nen-domain             → tek kenar, ADR-0006 sınırı korunuyor
```

Üretilen fixture'lar: `fixtures/subtitles/valid/html-special-chars.{srt,golden,vtt}`
ve mevcut 7 fixture'ın her biri için `.vtt` golden'ı
(`UPDATE_GOLDEN=1 cargo test -p nen-subtitle --test webvtt_roundtrip` ile üretildi).
