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

`nen-ffi` **tek dış kapıdır** (`docs/architecture.md` → crate tablosu; ADR-0006
kural 2). `ADR-0033` Karar 3, playback olay akışının teslimat yönünü **pull**
olarak sabitliyor — core kabuğa itmiyor, kabuk `drain_events()` çağırıyor;
push (`EventSink`) o kararda açıkça reddedilen alternatif. Bu, playback'in
~30 Hz'lik sürekli olay akışına özgü bir tercih (ADR-0033 gerekçesi). Bir
çeviri işinin ilerlemesi aynı profile girmiyor — blok başına birkaç kesikli
olay — ve `ADR-0004` Karar 1/2/5 zaten kendi `JobHandle` + tek delivery gate
desenini tanımlıyor (`tasks:` alanında bu task adıyla anılıyor). Planlama
sırasında kullanıcı kararıyla netleşti: çeviri ilerlemesi bir foreign
`ForeignTranslationProgressSink` ile **push** taşınır; ADR-0033'e bu ayrımı
kaydeden bir Notlar girdisi eklenir, yeni bir ADR açılmaz (ADR-0043/`NEN-081`
emsali).

K23 #4 (subtitle diyaloğu) burada en yüksek riskli sınır: cue metni FFI'dan
geçtiği için `Debug`/`Display` türevleri elle yazılmalı. Emsal: `NEN-083`'ün
handoff guard'ları ve `NEN-080`'in elle yazılmış locator `Debug`'ı.

## Kapsam

- Çeviri işini başlatan, ilerlemesini ileten ve iptal eden FFI yüzeyi
- Yardımcıların (provider/store/index) `nen-app` tarafında bir kompozisyon
  kökünde kurulması — `nen-ffi` yalnız ilkel değer alır, yeni crate'e
  bağlanmaz (ADR-0006 kural 3)
- Biten işin sonucunun `SubtitleLibrary::add_translation` ile kataloğa
  eklenmesi için FFI çağrısı (`NEN-099`'un ayrı ve açık adımı)
- Tipli hataların FFI taksonomisine bağlanması — bugün `nen-ffi`'de zaten var
  olan kod deseni (ADR-0006 kural 2 + K23); `docs/adr/0005-*` henüz yazılmadı
  (`docs/adr/README.md` → "Planlanan"), bu task onu kararlaştırmıyor
- Cue metni taşıyan her tipin `Debug`'ının shape-only olması

## YAPILMAYACAK

- macOS kabuğu — `NEN-101` · `NEN-102`
- Kotlin/Android bağlaması — M10
- Çeviri ilerlemesi için birden fazla teslimat yolu (yalnız push, drain
  yok) — mevcut `TranslationCall` gate'i tek mekanizma kalır
- Playback olay akışının pull yönü — `ADR-0033`'ün kendi kapsamı değişmez,
  yalnız notla genişler

## Kanıt (DoD)

- [ ] FFI testi: iş başlatılıyor, ilerleme olayları sırayla geliyor, sonuç teslim ediliyor
- [ ] Negatif: iptal sonrası hiçbir sonuç olayı gelmiyor (late commit yok)
- [ ] Guard: FFI yüzeyinden çıkan hiçbir `String`/`Debug`/hata gösterimi cue metni,
      medya URL'si veya özel yol taşımıyor — negatif kontrolle (K23 #1, #3, #4)
- [ ] Negatif: guard geçici olarak kaldırıldığında tarama testi kırmızıya dönüyor
- [ ] `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
