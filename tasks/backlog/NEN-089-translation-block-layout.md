---
id: NEN-089
title: Translation block layout and whole-document context
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-016]
blocks: [NEN-090, NEN-103]
adr: [0015]
---

# NEN-089 — Translation block layout and whole-document context

## Sonuç

Bir `SubtitleDocument` her zaman aynı, yeniden üretilebilir overlapping bloklara
ayrılıyor ve belgenin tamamından çıkarılan bağlam her bloğa aynı biçimde giriyor.

## Bağlam

`core/crates/nen-translate/src/lib.rs` bugün üç satırlık boş bir iskelet.
Şartname (`docs/product-spec.md` §10) blok boyutunu (varsayılan **40**, izin
verilen **30–60**) ve overlap'i (**6**) sabitliyor, ama blok sınırının nereye
düştüğünü, overlap'in sonuçta nasıl birleştiğini ve "bütün subtitle dokümanı
için bağlam analizi"nin ne ürettiğini söylemiyor. Bunlar ADR-0015 ile
kararlaştırılır.

## Ön koşul — ADR-0015

`proposed` yazılır, kullanıcı onaylar, `accepted` olur. En az şunları
kararlaştırır: blok sınırı kuralı · overlap penceresinin çakışan cue'larda hangi
bloğun sözünün geçerli olduğu · bağlam analizinin çıktısı (ne tutulur, ne kadar
büyür) · cue kimliğinin provider'a hangi biçimde gösterildiği · blok düzeni
versiyonunun (`block-layout version`, §11) nasıl türetildiği.

## Kapsam

- `nen-translate` içinde blok düzeni: deterministik, aynı belge → aynı düzen
- Belge tümü üzerinden bağlam çıkarımı (yalnız yerel, provider çağrısı yok)
- `block-layout version` üretimi — cache identity bunu tüketecek (`NEN-097`)
- Blok boyutunun izin verilen aralık dışına çıkmasının reddi

## YAPILMAYACAK

- Provider çağrısı veya prompt metni — `NEN-090`
- Doğrulama, repair, checkpoint — `NEN-091` · `NEN-092` · `NEN-093`
- Bağlam kalitesinin dilsel ölçümü — M5'in ölçütü yapısal doğruluk (S3, 2026-09-08)

## Kanıt (DoD)

- [ ] ADR-0015 `accepted`
- [ ] Golden: sabit bir fixture belgesinin blok düzeni snapshot'la byte düzeyinde eşleşiyor
- [ ] Unit: aynı belge iki kez bölündüğünde düzen birebir aynı (determinizm)
- [ ] Unit: overlap penceresindeki cue'lar her iki blokta da görünüyor ve sınır kuralı ADR-0015'in yazdığı gibi çözülüyor
- [ ] Negatif: aralık dışı blok boyutu (29 ve 61) tipli hata ile reddediliyor
- [ ] Negatif: cue sayısı bir bloktan küçük belge tek blok üretiyor, boş blok üretmiyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
