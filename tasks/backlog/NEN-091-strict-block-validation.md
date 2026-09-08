---
id: NEN-091
title: Strict local validation of a translated block
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-090]
blocks: [NEN-092]
adr: [0016]
---

# NEN-091 — Strict local validation of a translated block

## Sonuç

Provider'ın çevrilmiş blok cevabı, structured output olsa bile, **yerel
doğrulama** geçmeden hiçbir yere teslim edilmiyor.

## Bağlam

Şartname §10: provider cevabı **untrusted**, structured output olsa bile
**local validation authoritative**. Doğrulama kümesi şartnamede sayılı: exact
cue count · yalnız izin verilen cue ID · unique cue ID · non-empty text ·
beklenen cue sırasına normalization.

Bu M5'in **yapısal doğruluk** ölçütünün kalbi (S3 kararı, 2026-09-08) ve
`tasks/README.md`'nin security/validation satırına girer — **negatif test
zorunlu**.

## Ön koşul — ADR-0016

`proposed` yazılır, kullanıcı onaylar, `accepted` olur. En az şunları
kararlaştırır: doğrulama hata taksonomisi · hangi ihlalin targeted repair ile
onarılabilir, hangisinin doğrudan blok başarısızlığı olduğu · sıra
normalizasyonunun sınırı (neyin düzeltme, neyin ihlal sayıldığı) · zaman ve ID
alanlarının provider tarafından **hiç** değiştirilemeyeceği.

## Kapsam

- Blok cevabı doğrulayıcısı ve tipli ihlal listesi
- Beklenen cue sırasına normalizasyon
- Cue ID / `TimeSpan` alanlarının girdiden aynen korunduğunun zorlanması

## YAPILMAYACAK

- Onarım denemesi — `NEN-092`
- Dilsel kalite değerlendirmesi (anlam, üslup) — M5'in ölçütü değil (S3)
- Belge seviyesinde birleştirme ve WebVTT — `NEN-094`

## Kanıt (DoD)

- [ ] ADR-0016 `accepted`
- [ ] Negatif: eksik cue (count uyuşmazlığı) reddediliyor
- [ ] Negatif: fazladan cue reddediliyor
- [ ] Negatif: bloğa ait olmayan cue ID reddediliyor
- [ ] Negatif: tekrar eden cue ID reddediliyor
- [ ] Negatif: boş veya yalnız boşluktan oluşan metin reddediliyor
- [ ] Negatif: provider'ın değiştirdiği bir `TimeSpan` veya cue ID reddediliyor
- [ ] Unit: karışık sırayla gelen geçerli cevap normalize ediliyor, reddedilmiyor
- [ ] Guard: ihlal mesajı **cue metnini taşımıyor** (K23 #4) — negatif kontrolle

## Kanıt kaydı

<!-- done olurken doldurulacak -->
