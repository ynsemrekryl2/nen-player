---
id: NEN-057
title: Read the language a subtitle filename declares
milestone: M3
size: S
state: backlog
depends_on: [NEN-025]
blocks: []
adr: [29, 30]
---

# NEN-057 — Read the language a subtitle filename declares

## Sonuç

`Film.tr.srt` gibi bir dosya adı dilini söylüyorsa, o dil `resolve_language`'a
**metadata** olarak verilir; içerik tespiti yalnız teyit/çelişki için kullanılır.

## Bağlam

`NEN-025` uygularken ayrıldı. Bugün kullanıcı altyazısının dili yalnız
**içerikten** tespit ediliyor (`nen_subtitle::language::resolve_language`
`metadata: None` ile çağrılıyor). Bu doğru çalışıyor ama eldeki bilginin bir
kısmını kullanmıyor: `Film.tr.srt`, `Film.en.srt` gibi adlar kullanıcı
kütüphanelerinde yaygın ve dili **açıkça** söylüyor.

`resolve_language`'ın imzası bunu zaten bekliyor: metadata verildiğinde nihai
dili o belirliyor, metin yine de incelenip güvenilir bir çelişki
raporlanabiliyor (`MetadataLanguageConflict`). Yani eksik olan tek şey, dosya
adındaki alt-uzantıyı bir `LanguageTag`'e çevirmek.

`NEN-025`'in Kapsam listesinde yoktu, Kural 5 gereği oraya eklenmedi.

## Kapsam

- Dosya adının son alt-uzantısını dil etiketi olarak ayrıştırma denemesi
  (`Film.tr.srt` → `tr`, `Film.pt-BR.srt` → `pt-BR`)
- Ayrıştırılamayan alt-uzantı sessizce yok sayılır — `Film.forced.srt`,
  `Film.2019.srt` dil değildir ve hata değildir
- Bulunan etiket `resolve_language`'a metadata olarak verilir
- Gruplama ADR-0030'a göre primary subtag ile — `pt-BR` kaynağı bölgesini
  korur, grubu `pt` olur

## YAPILMAYACAK

- İçerik tespitini kaldırmak — çelişki raporu için metin yine incelenir
- Dosya adından başlık/yıl/release adı çıkarmak — o `nen-identity`'nin işi
  (`NEN-018`)
- Çelişki durumunda kullanıcıya soru sormak — bu milestone'da yüzey yok

## Kanıt (DoD)

- [ ] `Film.tr.srt` Türkçe grubuna giriyor, içerik tespitine bakılmaksızın
- [ ] Metin başka dilse `MetadataLanguageConflict` üretiliyor ama dil yine
      metadata'nın dediği
- [ ] Negatif: `Film.forced.srt` ve `Film.2019.srt` dil ipucu üretmiyor
- [ ] Negatif: dosya adı hiçbir log'a girmiyor (K23 #8)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
