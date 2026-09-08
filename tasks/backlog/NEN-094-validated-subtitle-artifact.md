---
id: NEN-094
title: ValidatedSubtitleArtifact and its WebVTT output
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-093, NEN-014]
blocks: [NEN-095, NEN-097]
adr: []
---

# NEN-094 — ValidatedSubtitleArtifact and its WebVTT output

## Sonuç

Çeviri sonucu, **yalnız belgenin tamamı doğrulandıktan sonra**, cue ID/sıra/
zamanları girdiyle birebir aynı olan bir `ValidatedSubtitleArtifact` ve onun
UTF-8 WebVTT çıktısı olarak var oluyor.

## Bağlam

Şartname §11 artifact'in taşıması gereken alanları sayıyor: artifact ID ·
source fingerprint · subtitle timeline fingerprint · source/target language ·
normalized cues · WebVTT · provider/model · pipeline versions · glossary
identity · media identity fingerprint · createdAt.

Bunların çoğu **zaten var**: `nen_subtitle::fingerprint::{SourceFingerprint,
TimelineFingerprint}` (NEN-016), `nen_subtitle::webvtt::write` (NEN-014),
`SubtitleSourceKind::Ai` (NEN-019). Bu task yeni bir fingerprint veya ikinci bir
WebVTT yazıcısı açmaz, var olanları birleştirir.

## Kapsam

- `ValidatedSubtitleArtifact` tipi ve alanlarının doldurulması
- Belge seviyesi son doğrulama: bütün blokların birleşimi girdiyle aynı cue
  kümesini veriyor mu
- WebVTT üretimi (`nen_subtitle::webvtt::write` yeniden kullanılır)
- Glossary identity için **alan** — değeri M5'te boş olabilir

## YAPILMAYACAK

- Diske yazma — `NEN-096`
- Cache identity'nin hesaplanması ve invalidasyonu — `NEN-097`
- Glossary yazma/düzenleme yüzeyi — M6; burada yalnız kimlik alanı durur
- İkinci bir WebVTT yazıcısı veya yeni fingerprint algoritması

## Kanıt (DoD)

- [ ] Golden: sabit fixture'ın artifact WebVTT çıktısı snapshot ile byte düzeyinde eşleşiyor
- [ ] Unit: artifact'in cue ID'leri, sırası ve `TimeSpan`'leri **girdi belgesiyle birebir aynı**
- [ ] Unit: `timeline fingerprint` girdi ve artifact için aynı değeri veriyor
- [ ] Negatif: bir blok doğrulanmamışken artifact **üretilmiyor** (yarım yayın yasağı)
- [ ] Negatif: cue metni dışında bir alanı değiştirilmiş sahte bir sonuçtan artifact üretilemiyor
- [ ] Guard: artifact'in `Debug`/`Display` gösterimi cue metnini taşımıyor (K23 #4)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
