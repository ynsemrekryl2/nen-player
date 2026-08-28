---
id: NEN-064
title: Verified media identity in player chrome
milestone: M6
size: L
state: backlog
depends_on: [NEN-033, NEN-061]
blocks: []
adr: [9, 31]
---

# NEN-064 — Oynatıcı kromunda doğrulanmış medya kimliği

## Sonuç

Resmî API'den kesin hash eşleşmesiyle doğrulanan başlık, yıl ve varsa
sezon/bölüm bilgisi playback'i bekletmeden üst medya şeridinde görünür;
doğrulanmış kimlik yoksa mevcut basename kalır.

## Kapsam

- Yalnız `NEN-033`'teki resmî OpenSubtitles API'sinin kesin hash `Match`
  sonucu zengin kimlik sayılır. `NoMatch`, `Ambiguous` ve hata sonucu kimlik
  değişikliği üretmez.
- Application katmanı optional
  `VerifiedMediaIdentity { title, year, season, episode }` sunar; tek
  `nen-ffi` kapısından Swift'e geçer. Playback portu genişletilmez.
- Kimlik sorgusu playback'i bloklamaz. Sonuç gelene kadar basename görünür;
  medya değişmişse eski revision'a ait geç sonuç yok sayılır.
- Sunum tek satırdır: film `Inception (2010)`, dizi
  `Breaking Bad (2008) · S01E02`. Eksik optional alanlar sessizce atlanır.
- Kimlik değişimi mevcut 0,24 sn ease-out geçişini kullanır; shutdown ve
  yeni medya stale sonucu etkisiz kılar.

## YAPILMAYACAK

- Filename/klasör tahminini "doğrulanmış" kimlik gibi göstermek
- Aday seçimi veya manuel başlık/yıl/sezon/bölüm düzeltme UI'ı
- Çözünürlük, codec, bitrate, SDR/HDR veya engine metadata'sı
- Tam yol, URL/query, hash, private provider ID veya filename metadata'sını
  log ya da kanıt yüzeyine çıkarmak

## Kanıt (DoD)

- [ ] Kesin fake hash eşleşmesi film ve dizi alanlarını FFI'dan eksiksiz
      geçiriyor ve kompakt etiketi üretiyor
- [ ] `NoMatch`, `Ambiguous` ve typed hata basename fallback'ini koruyor;
      playback kesilmiyor ve kullanıcıya hata gösterilmiyor
- [ ] Medya revision'ı değiştikten sonra tamamlanan eski sorgu şeridi
      değiştirmiyor; shutdown sonrası late update yok
- [ ] Negatif redaction testinde yol, query, hash, private ID ve özel filename
      hiçbir `Debug`/log/kanıt çıktısında yok
- [ ] Basename → doğrulanmış film/dizi etiketi 0,24 sn geçişle gerçek
      `.app` üzerinde ve fixture medyayla doğrulanıyor
- [ ] Workspace, macOS testleri ve doküman denetimleri yeşil

## Kanıt kaydı

<!-- done olurken gerçek test ve acceptance çıktısıyla doldurulur. -->
