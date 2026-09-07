---
id: NEN-082
title: Treat handoff metadata as optional evidence
milestone: M4
size: M
state: backlog
closed:
depends_on: [NEN-080]
blocks: [NEN-084]
adr: [9]
---

# NEN-082 — Treat handoff metadata as optional evidence

## Sonuç

Handoff'la gelen metadata kimlik kanıtı olarak değerlendirilir; gelmezse veya
bozuksa akış hiç bozulmaz — medya yine oynar, kimlik diğer kanıtlardan çıkar.

## Bağlam

Şartname §5: *"Stremio canonical media identity gönderirse optional güçlü kanıt
olarak kullanılabilir. Her zaman geleceği varsayılmamalıdır."* §6 aynı şeyi
kanıt listesinde tekrar ediyor: *"optional handoff metadata"* zaten listenin ilk
maddesi. Yani model tarafı ADR-0009'da hazır; eksik olan handoff'un o listeye
bağlanması.

`security-policy.md` §2 "handoff extras"ı **adıyla** düşman girdi sayıyor:
doğrulanmadan kullanılmaz, parse eden kod `unwrap`/`expect` kullanmaz, untrusted
metin doğrudan bir yola/komuta/format string'ine gömülmez.

Metadata'nın biçimi `NEN-079`'un ADR'sinde kararlaştırılır.

## Kapsam

- Handoff metadata'sının ayrıştırılması ve `MediaEvidence`'a taşınması
- Kanıt katmanlarının ADR-0009'daki değerlendirme sırasının korunması —
  handoff yeni bir sıra icat etmez, var olan sıraya bir katman olarak girer
- Eksik, boş, bozuk ve düşmanca metadata'nın tipli hataya bağlanması
- Kimlik çıkarılamasa da medyanın oynaması (şartname §6)

## YAPILMAYACAK

- Kimlik çıkarımının kendisini değiştirmek — `nen-identity` mevcut haliyle
  kullanılır
- Kullanıcıya teknik ID göstermek veya sordurmak — şartname §6 ve non-goal
- Medya URL'sinin query/fragment/host'unu kimliğe katmak — ADR-0009 ve §6'nın
  kesin yasağı
- Metadata'yı loglamak → `NEN-083`
- Provider sorgusu tetiklemek → M6

## Kanıt (DoD)

- [ ] Metadata taşıyan handoff `MediaEvidence`'a beklenen alanları koyuyor
      (unit + golden)
- [ ] **Metadata yokken akış bozulmuyor** — medya oynuyor, kimlik diğer
      kanıtlardan çıkıyor (M4 çıkış kriteri #3)
- [ ] Negatif: bozuk/eksik/aşırı büyük/düşmanca metadata tipli hatayla
      reddediliyor, panik yok, `unwrap` yok
- [ ] Negatif: query · fragment · host kimliğe hiç girmiyor
- [ ] Negatif kontrol: metadata katmanı geri alındığında yalnız bu task'ın
      testleri kırmızı

## Kanıt kaydı

<!-- done olurken doldurulacak -->
