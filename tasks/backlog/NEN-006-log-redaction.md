---
id: NEN-006
title: Log redaction helpers and guard test
milestone: M1
size: M
state: backlog
depends_on: [NEN-001, NEN-007]
blocks: []
adr: [5]
---

# NEN-006 — Log redaction helpers and guard test

## Sonuç

Gizli veri taşıyan tiplerin hata, log ve `Debug` çıktısı yasaklı bilgilerin
hiçbirini içermez ve bu, test ile mekanik olarak korunur.

## Kapsam

- `nen-domain` içinde redaction yardımcıları: `Redacted<T>`, güvenli türev
  yardımcıları (`extension()`, `size_class()`, host allowlist eşlemesi)
- Hassas tiplerde `#[derive(Debug)]` **yasağı**, elle yazılmış `Debug`
- `docs/security-policy.md` §1'deki 8 yasak için guard test: hassas tiplerin
  `{:?}` çıktısı yasaklı desen içermiyor
- Typed error payload'larının aynı kurala uyması

## YAPILMAYACAK

- Platform log sink adapter'ları → ilgili platform milestone'ları
- Telemetri/crash raporlama → S10 yanıtlanmadan tasarlanmaz

## Kanıt (DoD)

- [ ] Guard test: URL, tam yol, cue metni, API key, private file ID içeren
      örnek tiplerin `{:?}` çıktısında yasaklı desen **yok**
- [ ] Negatif: `derive(Debug)` eklenmiş bir tip guard testi **kırıyor**
      (testin gerçekten koruduğunun kanıtı)
- [ ] Typed error varyantları payload'sız ayrıştırılabiliyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
