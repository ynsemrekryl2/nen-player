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

1. **Kanonik sıra ve serileştirme:** kimlik, `JSON.stringify` + genel amaçlı
   hash değil, `nen_subtitle::fingerprint` ile aynı desende **açık, sabit
   sıralı bir byte kodlaması** üzerinden BLAKE3 ile hesaplanır. Bileşenler bu
   ADR'nin 2. paragrafındaki sırayla art arda kodlanır; her string alan bir
   `u32` LE uzunluk öneki + UTF-8 bytes, her sayısal alan sabit genişlikte LE,
   her opsiyonel alan bir "var mı" byte'ı + (varsa) değer olarak yazılır.
2. **Bileşen sırası (kanonik):** `source fingerprint` · `source language` ·
   `target language` · `provider/model` · `media context` (madde 5) ·
   `glossary` (madde 3) · `block size` · `overlap` · `pipeline version` ·
   `prompt version` · `schema version` · `block-layout version` ·
   `translation-session version`.
3. **Glossary boşken:** boş bir koleksiyon, "0 eleman" olarak deterministik
   kodlanır; "glossary hiç yok" ile "glossary boş" ayrımı yapılmaz, ikisi de
   aynı temsille kodlanır. M6'da glossary dolduğunda kimlik doğal olarak
   değişir; eski (boş glossary'li) cache sessizce geçersiz kalır, ayrı bir
   migration adımı gerekmez.
4. **Versiyon alanları**, `nen-translate` crate kökünde tek bir yerde
   toplanan, elle artırılan sabitlerdir: `pipeline version` pipeline'ın adım
   sırası/anlamı değişince · `prompt version` prompt metni (M6) değişince ·
   `schema version` provider'a/dan giden veri şeklinin şekli değişince ·
   `block-layout version` ADR-0015'in blok sınırı/overlap kuralı değişince ·
   `translation-session version` dil çifti temsilinin şekli değişince.
5. **`media context`**, ADR-0009 evidence katmanının tamamı değil, yalnız
   content-addressed `MediaHash` (`nen-ports::identity`) olarak kimliğe girer.
   Başlık, yıl, sezon/bölüm gibi diğer evidence alanları kimliğe **girmez**.
6. **Çoklu artifact (S9):** bu ADR ayrı bir mekanizma eklemez — provider/model
   ve glossary zaten bileşen olduğundan, farklı üretimler doğal olarak farklı
   kimlik ve dolayısıyla farklı CAS anahtarı üretir; S9'un "diskte yan yana"
   davranışı kimlik hesaplamasının doğrudan sonucudur. "Menüde yalnız en
   yeni" projeksiyonu bu ADR'nin değil `NEN-098`'in kapsamındadır.

## Gerekçe

Madde 1 ve 2, ADR-0007'nin `nen_subtitle::fingerprint` için zaten kabul
ettiği ilkeyi (kanonik byte kodlaması, dile/derleyiciye bağımlı alan sırası
değil) cache identity'ye taşır — `serde_json`'ın alan sırası derive sırasına
bağlıdır ve platformlar arası garanti vermez; önceki bir projedeki
(Stremio AISubtitle) `JSON.stringify` + `sha256` deseni bu yüzden **taşınmaz**
(bkz. Notlar).

Madde 3, M5'in "glossary yazma yüzeyi M6" kararıyla (NEN-094, NEN-097
kapsamı) tutarlı: boş glossary'yi ayrı bir özel durum yapmak yerine sıradan
"0 eleman" olarak kodlamak, M6'da glossary eklendiğinde ek bir geçiş
mantığına ihtiyaç bırakmaz — kimlik zaten değişir.

Madde 5, ADR'nin kendi 5. sorusunun ("`media context` gerçekten gerekli mi?")
ölçülü bir cevabıdır: `SourceFingerprint` zaten kaynağın tam diyalog+zaman
içeriğini kapsıyor, dolayısıyla evidence'ın tamamını taşımanın (dosya yolu,
başlık metni) tek somut riski gereksiz redaction yüzeyi ve gereksiz cache
miss'tir (aynı diyalog farklı bir dosya adında/konumda tekrar çevrilir).
Yalnız `MediaHash`'i dahil etmek, spec'in §11'de saydığı "media context"
maddesini karşılarken bu riski taşımaz.

Madde 6, ayrı bir mekanizma gerektirmemesi bakımından en ucuz çözüm: S9
kararı zaten "provider/model/glossary farklıysa artifact'ler ayrışır" diyor,
ve bunlar zaten kimlik bileşeni — ek bir alan veya bayrak eklemek gereksiz.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| `JSON.stringify(...)` + genel amaçlı hash (önceki projenin yöntemi) | Alan sırası dilde/derleyicide garanti değil; platformlar arası tutarsızlık riski |
| `media context` olarak tüm evidence nesnesini (başlık, yıl, dosya yolu vb.) dahil etmek | `SourceFingerprint` zaten içerik-tabanlı benzersizliği sağlıyor; ek alan yalnız gereksiz cache miss ve redaction yüzeyi üretir |
| `media context`'i kimlikten tamamen çıkarmak | Spec §11'in saydığı bir bileşeni karşılamaz; `MediaHash` dahil etmenin maliyeti (bir alan) düşükken spec'e sadakati korur |
| Boş glossary için ayrı bir "glossary yok" durumu | Gereksiz özel durum; "0 eleman" kodlaması zaten aynı sonucu deterministik verir |

## Sonuçlar

**Olumlu:** kimlik hesaplaması platformdan bağımsız ve deterministik; S9'un
çoklu-artifact davranışı ayrı kod gerektirmeden kimlik hesaplamasının doğal
sonucu; boş glossary'den dolu glossary'ye geçiş sessizce ve doğru çalışır.

**Olumsuz / kabul edilen maliyet:** `media context`'in yalnızca `MediaHash`
olması, teorik olarak aynı diyalog+zamanlamaya sahip iki **farklı** medya
öğesinin (ör. aynı altyazı dosyasının iki ayrı filme yanlışlıkla eklenmesi)
aynı cache'i paylaşmasına izin verir; bu senaryo `SourceFingerprint`'in zaten
tam diyalog+zaman eşleşmesi gerektirmesi nedeniyle ihmal edilebilir kabul
edilir.

**Geri dönüş maliyeti:** `media context`'in kapsamı genişletilirse (ör. daha
fazla evidence alanı eklenirse) `translation-session version` veya yeni bir
bileşen alanı gerekir; bu doğal olarak eski cache'leri geçersiz kılar.

## İlgili task'lar

`NEN-097` · tüketici: `NEN-098`

## Notlar

Karar taslağı, önceki bir projedeki (Stremio AISubtitle, TypeScript) çalışan
bir çeviri hattının karşılaştırmalı incelemesinden türetildi — bkz. o hattın
`services/import-service.ts` içindeki `buildTranslationCacheIdentity`
fonksiyonunun bileşen listesi. Serileştirme yöntemi (`JSON.stringify` +
`sha256`) kasıtlı olarak taşınmadı; yerine `nen_subtitle::fingerprint`
(ADR-0007) ile aynı kanonik byte kodlaması deseni benimsendi.
