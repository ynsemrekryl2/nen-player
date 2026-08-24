---
adr: 0000
title: Şablon — kopyala ve doldur
status: template
milestone: —
tasks: []
date: —
---

# ADR-0000 — Başlık

> Bu dosya şablondur. Yeni ADR için kopyala:
> `cp docs/adr/0000-template.md docs/adr/0007-subtitle-domain-model.md`
> Numara sıradaki boş numaradır; asla yeniden kullanılmaz.

## Durum

`proposed` · `accepted` · `superseded by ADR-XXXX` · `rejected`

Yeni ADR **`proposed`** olarak açılır. Kullanıcı onayı olmadan `accepted`
yapılmaz.

## Bağlam

Hangi problem bu kararı gerektirdi? Hangi kısıtlar var (şartname maddesi,
platform sınırı, güvenlik kuralı)? Karar verilmezse ne olur?

İlgili şartname maddesi varsa referans ver: `docs/product-spec.md` §N.

## Karar

Tek paragrafta, net ve emir kipinde. "X kullanılacaktır." Belirsiz ifade yok.

## Gerekçe

Neden bu seçenek? Ölçüm varsa **sayı** ver. Varsayım varsa açıkça yaz.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| … | … |

Bu bölüm boş bırakılamaz. Alternatifi olmayan karar, karar değildir.

## Sonuçlar

**Olumlu:** …

**Olumsuz / kabul edilen maliyet:** …

**Geri dönüş maliyeti:** bu karardan dönmek ne kadar pahalı? (ucuz / orta /
pahalı — ve neden)

## İlgili task'lar

`NEN-0XX`, `NEN-0YY`

## Notlar

Karar sonrası ortaya çıkan gözlemler buraya eklenir. Karar değişiyorsa bu ADR
`superseded` yapılır ve **yeni bir ADR** yazılır — eski ADR düzenlenmez.
