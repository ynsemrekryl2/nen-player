---
id: NEN-083
title: Handoff input never reaches a log surface
milestone: M4
size: S
state: done
closed: 2026-09-08
depends_on: [NEN-080]
blocks: [NEN-084]
adr: []
---

# NEN-083 — Handoff input never reaches a log surface

## Sonuç

Handoff'la gelen argüman, medya URL'si ve metadata hiçbir log, `Debug`,
`Display` veya hata mesajı yüzeyinde görünmez — ve bunu kapalı-küme bir denetim
korur.

## Bağlam

K23 ihlali **güvenlik hatasıdır** (CLAUDE.md). Yasak liste bu task'ın tam
merkezinde: medya URL'si · token içeren query · özel tam dosya yolu · raw
provider response. `security-policy.md` §1 aynı şeyi `Debug`/`Display`
kuralıyla bağlıyor: bu tipler türetilmez, elle yazılır.

M4'ün üç çıkış kriterinden biri doğrudan budur: *"Negatif: argüman ve medya
URL'si loglanmıyor (log denetimi)."*

Depoda taklit edilecek emsal hazır: `core/crates/nen-ports/tests/guard_playback_debug.rs`
bir `forbidden_fragments()` listesi tutuyor, her tipi basıp tarıyor ve
`a_derived_media_source_really_does_leak()` ile **kontrolün sağır olmadığını**
ayrıca kanıtlıyor. `guard_redaction.rs` ve `guard_subtitle_file_debug.rs` aynı
desende.

## Kapsam

- Handoff'un taşıdığı her tipin `Debug`/`Display` çıktısının taranması:
  locator, argüman, başlangıç pozisyonu taşıyıcısı, metadata
- Handoff yolundan çıkan **hata** tiplerinin de taranması — hata mesajı bir log
  yüzeyidir
- Kapalı küme: yeni bir handoff alanı eklendiğinde denetimin dışında kalmaması
- Kasıtlı sızdıran bir tiple negatif kontrol — denetimin gerçekten yakaladığı

## YAPILMAYACAK

- Yeni bir redaction yardımcısı yazmak — `nen-domain/src/redact.rs` var
- Loglamayı tümden kapatmak; güvenli alanlar (şema, durum, süre, hata sınıfı)
  loglanabilir kalır (`security-policy.md` §1 → "Loglanabilecekler")
- Diğer platformların log yüzeyleri → M9–M11

## Kanıt (DoD)

- [x] Guard testi: handoff tiplerinin hiçbiri yasak parça basmıyor
      (`guard_playback_debug.rs` deseninde)
- [x] Guard testi: handoff hata tipleri de temiz
- [x] **Negatif kontrol (zorunlu):** kasıtlı sızdıran bir tip eklendiğinde
      denetim kırmızıya dönüyor — kontrol sağır değil
- [x] Negatif kontrol: redaction geri alındığında yalnız bu denetim kırmızı
- [x] Güvenli alanların hâlâ basıldığı ayrıca doğrulanmış (denetim her şeyi
      susturarak geçmiyor)
- [x] Rust workspace ve macOS test kapıları yeşil; paralel script koşusundaki
      önceden bilinen NEN-049 sınıfı contention serial kapıda yok

## Kanıt kaydı

- Core `guard_handoff_debug`: 6/6 geçti; argv’den parse edilen token’lı uzak
  locator, yerel path, başlangıç pozisyonu ve `HandoffRejection` Debug/Display
  çıktıları temiz kaldı. FFI guard: 5/5 geçti; gate locator/request ve tüm
  `FfiHandoffRejection` varyantları temiz kaldı.
- `nen-identity::guard_evidence_debug`: 8/8 geçti; doğrudan
  `HandoffMetadata` debug çıktısı yalnız presence alanlarını gösterdi.
- macOS `handoff log surfaces`: 4/4 geçti. `HandoffOutcome`, FFI locator ve
  request için `String(describing:)`, `String(reflecting:)` ve `dump` çıktıları
  URL/path içermedi; şema, uzantı, süre ve hata sınıfı korundu. Rejection artık
  payload taşımıyor ve mevcut güvenli kullanıcı mesajını model gösteriyor.
- Negatif kontroller: türetilmiş Rust/Swift taşıyıcılar aynı fixture’ları
  sızdırdı; Rust `HandoffLocator` redaksiyonu geçici kaldırıldığında hedef
  guard 4 testte kırmızı oldu; Swift `HandoffOutcome` redaksiyonu geçici
  kaldırıldığında hedef guard 8 assertion ile kırmızı oldu. Üretim redaksiyonu
  geri yüklendi.
- `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --workspace
  --all-targets -- -D warnings` ve `cargo deny check` yeşil. macOS Swift paketi
  serial koşuda **239 test / 0 failure** (`swift test --no-parallel`).
- `bash scripts/test-macos.sh` paralel koşusu handoff suite’leri dahil 238/239
  testte yeşil kaldı; tek kırmızı, önceden belgelenmiş NEN-049 sınıfı
  `PicturelessSurfaceTests` main-actor contention’ıydı. Aynı paket serial
  kapıda 239/239 yeşildir.
- Ayrıntılı kayıt: `evidence/M4/NEN-083-checklist.md`.
