---
id: NEN-100
title: Translation FFI surface with progress, cancel and log guard
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-099]
blocks: [NEN-101]
adr: []
---

# NEN-100 — Translation FFI surface with progress, cancel and log guard

## Sonuç

Bir çeviri işi FFI sınırından başlatılıp ilerlemesi izlenebiliyor ve iptal
edilebiliyor; çevrilen metin bu sınırdan hiçbir log yüzeyine sızmıyor.

## Bağlam

`nen-ffi` **tek dış kapıdır** (`docs/architecture.md` → crate tablosu) ve olay
teslimat yönü `ADR-0033` ile sabitlenmiş: core sahibidir, kabuğa iter.
`NEN-045`'in playback oturumu ve `NEN-080`'in handoff gate'i bu deseni zaten
kuruyor — çeviri de aynı deseni izler, yeni bir yön açılmaz.

K23 #4 (subtitle diyaloğu) burada en yüksek riskli sınır: cue metni FFI'dan
geçtiği için `Debug`/`Display` türevleri elle yazılmalı. Emsal: `NEN-083`'ün
handoff guard'ları ve `NEN-080`'in elle yazılmış locator `Debug`'ı.

## Kapsam

- Çeviri işini başlatan, ilerlemesini ileten ve iptal eden FFI yüzeyi
- Tipli hataların FFI taksonomisine bağlanması (`ADR-0005` deseni)
- Cue metni taşıyan her tipin `Debug`'ının shape-only olması

## YAPILMAYACAK

- macOS kabuğu — `NEN-101` · `NEN-102`
- Kotlin/Android bağlaması — M10
- İkinci bir olay kanalı; `ADR-0033`'ün yönü değişmez

## Kanıt (DoD)

- [ ] FFI testi: iş başlatılıyor, ilerleme olayları sırayla geliyor, sonuç teslim ediliyor
- [ ] Negatif: iptal sonrası hiçbir sonuç olayı gelmiyor (late commit yok)
- [ ] Guard: FFI yüzeyinden çıkan hiçbir `String`/`Debug`/hata gösterimi cue metni,
      medya URL'si veya özel yol taşımıyor — negatif kontrolle (K23 #1, #3, #4)
- [ ] Negatif: guard geçici olarak kaldırıldığında tarama testi kırmızıya dönüyor
- [ ] `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
