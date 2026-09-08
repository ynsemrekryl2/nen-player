---
adr: 0016
title: Çeviri doğrulama ve onarım politikası — yerel doğrulama authoritative
status: proposed
milestone: M5
tasks: [NEN-091]
date: —
---

# ADR-0016 — Çeviri doğrulama ve onarım politikası — yerel doğrulama authoritative

## Durum

`proposed`

## Bağlam

`docs/product-spec.md` §10 iki şeyi tartışmaya kapatıyor: provider cevabı
**untrusted**'dır, ve structured output kullanılsa bile **yerel doğrulama
authoritative**'dir. Doğrulama kümesi de sayılı: exact cue count · yalnız izin
verilen cue ID · unique cue ID · non-empty text · beklenen cue sırasına
normalization. Onarım bütçesi de sayılı: en fazla **2 targeted repair** + **1
full-block retry**.

Kararlaştırılması gereken, sayıların arasında kalan boşluk:

1. **İhlal taksonomisi.** Hangi ihlaller ayrı sınıflar? "Eksik cue" ile "bozuk
   zaman alanı" aynı muameleyi mi görür?
2. **Hangi ihlal onarılabilir?** Targeted repair yalnız ihlal eden cue'ları
   yeniden ister; ama örneğin provider bir `TimeSpan`'i değiştirdiyse bu
   onarılacak bir eksiklik mi, yoksa doğrudan blok başarısızlığı mı?
3. **Normalizasyonun sınırı nerede?** Karışık sırada gelen geçerli bir cevap
   normalize edilir. Peki tekrar eden ID? Fazladan whitespace? Nerede
   "düzeltilir", nerede "reddedilir"?
4. **Zaman ve ID dokunulmazlığı.** Provider'ın cue ID veya `TimeSpan`
   döndürmesi gerekiyor mu, yoksa bunlar hiç gönderilmeyip yerelde mi
   birleştirilir? İkinci seçenek bütün bir ihlal sınıfını imkânsız kılar.
5. **Hata mesajı ne taşıyabilir?** K23 #4 gereği cue diyaloğu hiçbir log
   yüzeyine düşemez — ihlal raporu bu yüzden metin taşıyamaz, yalnız şekil
   (hangi cue ID, hangi ihlal sınıfı).

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

`NEN-091` · tüketiciler: `NEN-092`, `NEN-093`, `NEN-094`

## Notlar
