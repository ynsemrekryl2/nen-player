---
adr: 0015
title: Çeviri blok stratejisi — boyut, overlap ve belge bağlamı
status: proposed
milestone: M5
tasks: [NEN-089]
date: —
---

# ADR-0015 — Çeviri blok stratejisi — boyut, overlap ve belge bağlamı

## Durum

`proposed`

## Bağlam

`docs/product-spec.md` §10 üç sayıyı sabitliyor: varsayılan block size **40**,
izin verilen aralık **30–60**, varsayılan overlap **6**. Ayrıca "bütün subtitle
dokümanı için bağlam analizi" istiyor.

Sabitlenmeyen ve kod yazılmadan önce kararlaştırılması gereken şeyler:

1. **Blok sınırı nereye düşer?** Sabit cue sayısı mı, yoksa sahne/sessizlik
   boşluğu gibi bir sinyal mi sınırı kaydırabilir? Kaydırırsa blok düzeni
   belgeye bağlı olarak değişir ve `block-layout version`'ın anlamı değişir.
2. **Overlap'in sözü kimindir?** Overlap penceresindeki cue iki blokta da
   çevriliyor; sonuçta hangi bloğun çevirisi geçerli sayılır?
3. **Bağlam analizi ne üretir?** Belgenin tamamından çıkan bu şey ne kadar
   büyür, her bloğa aynen mi girer, ve provider cevabını doğrulayan tarafın
   (`NEN-091`) onunla bir işi var mıdır?
4. **Cue kimliği provider'a nasıl gösterilir?** `CueId(u32)` doğrudan mı geçer,
   yoksa blok içi bir indeks mi? Doğrulamanın "yalnız izin verilen cue ID"
   kuralı buna dayanacak.
5. **`block-layout version` nasıl türetilir?** Cache identity (§11) bunu bir
   bileşen olarak tüketiyor; blok düzeninin semantiği değiştiğinde eski cache
   kullanılamaz olmalı.

M5'in kalite ölçütü **yapısal doğruluktur** (kullanıcı kararı, 2026-09-08 — S3):
dilsel kalite çıtası gerçek model geldiğinde M6'da kapanır. Dolayısıyla bu ADR
bağlam analizinin *dilsel* faydasını iddia etmez, yalnız **deterministik ve
yeniden üretilebilir** olmasını şart koşar.

## Karar

<!-- Kullanıcı onayıyla doldurulacak. Yukarıdaki beş sorunun her biri için
     tek cümlelik, emir kipinde karar. -->

## Gerekçe

<!-- Ölçüm varsa sayı. Varsayım varsa açıkça. -->

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| … | … |

## Sonuçlar

**Olumlu:** …

**Olumsuz / kabul edilen maliyet:** …

**Geri dönüş maliyeti:** …

## İlgili task'lar

`NEN-089` · tüketiciler: `NEN-090`, `NEN-097`, `NEN-103`

## Notlar
