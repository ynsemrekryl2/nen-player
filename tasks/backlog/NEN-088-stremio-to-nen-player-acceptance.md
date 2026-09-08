---
id: NEN-088
title: Stremio to Nen Player acceptance
milestone: M4
size: S
state: backlog
closed:
depends_on: [NEN-087]
blocks: []
adr: [44]
---

# NEN-088 — Stremio to Nen Player acceptance

## Sonuç

Stremio 5.1.26'da tek bir "MPV içinde oynat" eylemi, gerçek Nen Player
uygulamasını soğuk ve sıcak açılışta medyayı oynatır hâlde gösterir.

## Kapsam

- Gerçek Stremio 5.1.26 + gerçek Nen Player `.app` kabul koşusu
- Stremio'nun gönderdiği başlangıç değerinin aynen uygulanması; mevcut sürümün
  gönderdiği `0` değerinin baştan oynatma olarak kaydı
- Metadata yokluğunda akışın bozulmadığı senaryo
- Argüman, locator ve özel yol için negatif log taraması
- Ekran ve güvenli adım kaydı

## YAPILMAYACAK

- Stremio'dan mevcut oynatma konumunu tahmin etmek veya geri almak
- Stremio'ya sonuç/pozisyon döndürmek
- Gerçek medya URL'sini veya özel metadata'yı kanıta yazmak

## Kanıt (DoD)

- [ ] Soğuk açılışta gerçek Stremio eylemi Nen Player'ı açıyor ve oynatıyor
- [ ] Uygulama zaten açıkken aynı eylem doğru handoff yolunu kullanıyor
- [ ] Gelen nonzero pozisyon varsa uygulanıyor; ölçülen `0` davranışı kayda
      geçiyor
- [ ] Metadata yokken oynatma sürüyor
- [ ] Negatif log taraması tüm yasak parçalar için sıfır eşleşme veriyor
- [ ] `evidence/M4/NEN-088-checklist.md` ekran/adım kaydını içeriyor

## Kanıt kaydı

<!-- NEN-088 kapanışında doldurulacak. -->
