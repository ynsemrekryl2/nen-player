---
id: NEN-035
title: Identity confidence scoring and candidate ranking
milestone: M6
size: M
state: backlog
depends_on: [NEN-018]
blocks: [NEN-121, NEN-125]
adr: []
---

# NEN-035 — Identity confidence scoring and candidate ranking

## Sonuç

Kanıt katmanları çeliştiğinde veya hiçbiri kesin sonuç vermediğinde adaylar
skorlanır ve sıralanır; **yalnız eşiğin altında kalındığında** kullanıcıya aday
listesi gösterilir.

## Kapsam

- Skor modeli: her kanıt katmanının ağırlığı, çelişki ve teyit etkisi
- Eşik: üstünde sessizce karar, altında aday listesi
- Aday listesi projeksiyonu (UI çizimi değil, model)
- Manuel giriş **aday listesinden sonra** gelir (ADR-0009 Karar 7)

## YAPILMAYACAK

- Teknik ID giriş alanı — **non-goal**; kullanıcıya yalnız başlık/yıl/sezon/bölüm
- Aday listesini varsayılan akış haline getirmek
- Skorları ölçüm olmadan sabitlemek — gerçek aday kümesi M6'da gelir

## Kanıt (DoD)

- [ ] Fixture korpusunda aday listesi gösterim oranı **ölçülüyor ve raporlanıyor**
      (ADR-0009 Karar 7'nin hedefi: büyük çoğunlukta hiç gösterilmemesi)
- [ ] Çelişen kanıtlarda tanımlı ve test edilmiş sıralama
- [ ] Eşik üstünde kullanıcıya hiç sorulmuyor (test)
- [ ] Eşik altında aday listesi üretiliyor, manuel giriş ondan sonra

## Kanıt kaydı

<!-- done olurken doldurulacak -->
