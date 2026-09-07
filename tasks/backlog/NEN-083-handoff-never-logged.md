---
id: NEN-083
title: Handoff input never reaches a log surface
milestone: M4
size: S
state: backlog
closed:
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

- [ ] Guard testi: handoff tiplerinin hiçbiri yasak parça basmıyor
      (`guard_playback_debug.rs` deseninde)
- [ ] Guard testi: handoff hata tipleri de temiz
- [ ] **Negatif kontrol (zorunlu):** kasıtlı sızdıran bir tip eklendiğinde
      denetim kırmızıya dönüyor — kontrol sağır değil
- [ ] Negatif kontrol: redaction geri alındığında yalnız bu denetim kırmızı
- [ ] Güvenli alanların hâlâ basıldığı ayrıca doğrulanmış (denetim her şeyi
      susturarak geçmiyor)
- [ ] Rust workspace ve `bash scripts/test-macos.sh` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
