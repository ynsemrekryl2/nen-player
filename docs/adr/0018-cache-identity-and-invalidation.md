---
adr: 0018
title: Cache identity bileşenleri, versiyonlama ve invalidasyon
status: proposed
milestone: M5
tasks: [NEN-097]
date: —
---

# ADR-0018 — Cache identity bileşenleri, versiyonlama ve invalidasyon

## Durum

`proposed`

## Bağlam

`docs/product-spec.md` §11 cache identity'nin bileşenlerini sayıyor: source
fingerprint · source language · target language · provider/model · media
context · glossary · block size/overlap · pipeline version · prompt version ·
schema version · block-layout version · translation-session version. Ve şunu
şart koşuyor: "Prompt/schema/pipeline semantiği değişince **uyumsuz cache
kullanılmamalıdır**."

M5'in dördüncü çıkış kriteri tam olarak budur: "Cache identity bileşenlerinden
biri değişince eski artifact **kullanılmıyor**".

Kararlaştırılması gerekenler:

1. **Bileşenlerin kanonik sırası ve serileştirmesi.** Kimlik bir hash ise, aynı
   girdinin her makinede aynı değeri vermesi buna bağlı.
2. **Versiyon alanları ne zaman artar?** `pipeline version`, `prompt version`,
   `schema version`, `block-layout version`, `translation-session version` —
   hangisi elle artırılır, hangisi türetilir?
3. **Glossary boşken.** M5'te glossary yazma yüzeyi yok (M6); alan boşken
   kimlik nasıl hesaplanır ve ileride glossary eklendiğinde eski cache ne olur?
4. **Çoklu artifact'in görünürlüğü.** Kullanıcı kararı (2026-09-08 — S9):
   farklı provider/model/glossary ile üretilen artifact'ler **diskte yan yana
   durur**, ama M5'te hedef dil grubunda yalnız **en yeni** olan gösterilir.
   Bu kararın kimliğe ve projeksiyona (`NEN-098`) yansıması yazılmalı.
5. **`media context` nedir?** ADR-0009'un evidence katmanından hangi alan
   kimliğe girer — ve girmesi gerçekten gerekli mi?

## Karar

<!-- Kullanıcı onayıyla doldurulacak. -->

## Gerekçe

<!-- … -->

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| … | … |

## Sonuçlar

**Olumlu:** …

**Olumsuz / kabul edilen maliyet:** …

**Geri dönüş maliyeti:** …

## İlgili task'lar

`NEN-097` · tüketici: `NEN-098`

## Notlar
