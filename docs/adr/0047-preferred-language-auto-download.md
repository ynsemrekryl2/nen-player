---
adr: 0047
title: Tercih dilinde otomatik altyazı indirme sözleşmesi
status: accepted
milestone: M6
tasks: [NEN-125, NEN-038]
date: 2026-09-14
---

# ADR-0047 — Tercih dilinde otomatik altyazı indirme sözleşmesi

## Durum

`accepted` (2026-09-14, kullanıcı onayı, `NEN-125`)

## Bağlam

ADR-0010 Karar 9, `embedded` → `user` → `opensubtitles` tür sırasını
tanımladı; OpenSubtitles basamağını ölçülmüş kimlik doğruluğu ve ayrı bir
karara erteledi. NEN-033, NEN-035 ve NEN-036 tamamlandı. NEN-035'in 10 vakalık
fixture ölçümünde 8 vaka otomatik (%80), 1 vaka aday seçimi, 1 vaka manuel
seçim oldu ve yanlış sessiz otomatik karar 0 kaldı. Mevcut otomatik güven kapısı
`ConfidenceScore >= 60` olarak kalibre edilmiştir.

Karar, açılışta ek ağ isteği ve indirme yaratacağı için yalnız teknik bir
sıralama değişikliği değildir. Kullanıcı ayarı, tercih dili, günlük kota,
başarısızlık ve yanlış kimlikte sessiz indirme riski birlikte sınırlandırılmalıdır.

## Karar

Kullanıcı ayarı açıkça etkinleştirmedikçe otomatik OpenSubtitles indirmesi
**kapalı** tutulacaktır; etkin olduğunda medya açılışında önce birinci, sonra
yalnız onun grubunda `embedded` veya `user` kaynak bulunmayan ikinci tercih
dili denenecektir. Aday ancak identity assessment `Automatic`, çakışmasız ve
`ConfidenceScore >= 60` ise otomatik seçilebilir; her medya için günde en fazla
bir OpenSubtitles indirme denemesi yapılacak, hata/kota durumunda seçim
`Kapalı` kalacak ve aynı açılışta yeniden denenmeyecektir. Başarılı indirme
görünen kaynak olur; kullanıcı seçimini sessizce değiştirmez. `ai` kaynağı
otomatik seçime hiçbir koşulda girmez.

## Gerekçe

NEN-035 ölçümü, mevcut 60/100 kapısının fixture üzerinde %80 otomatik karar
ürettiğini ve yanlış sessiz karar üretmediğini gösteriyor; bu nedenle yeni,
ölçülmemiş bir eşik icat edilmiyor. Varsayılan kapalı ayar, açılış başına ağ
isteği/kota maliyetini ve kullanıcı tarafından istenmeyen indirmeyi açık
onaya bağlıyor. Önce hazır yerel kaynakları tüketmek, ADR-0010'un lazy ve
"seçilmeden download yok" sınırlarını koruyor. Günlük tek deneme ve retry
yok politikası kota ve yanlış eşleşme etkisini sınırlıyor.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Varsayılan olarak her açılışta otomatik indirme | Kullanıcı onayı olmadan ağ/kota tüketir ve yanlış kimlikte yanlış altyazıyı sessizce indirir |
| Güven skoruna bakmadan ilk OpenSubtitles adayını seçmek | NEN-035'in ölçtüğü otomatik karar kapısını ve çakışma/tie güvenliğini deler |
| `ai` kaynağını otomatik seçime eklemek | ADR-0010 §9 ve ADR-0046 untrusted AI çıktısının otomatik seçilmesini yasaklar |
| Her hata sonrası sınırsız yeniden deneme | Kota, açılış gecikmesi ve tekrarlanan yanlış indirme riskini büyütür |

## Sonuçlar

**Olumlu:** Hazır yerel altyazı varsa ağ isteği yapılmaz; etkin kullanıcılar
ölçülmüş güven kapısıyla tercih dilinde otomatik indirme alır; indirme ve
başarısızlık davranışı görünür ve sınırlıdır.

**Olumsuz / kabul edilen maliyet:** Varsayılan kapalı olduğu için kullanıcı
ayarı açmadan otomatik indirme yoktur; düşük güvenli veya çakışmalı kimlikler
otomatik indirmeyi atlar ve kullanıcı seçimine kalır. Birinci tercih başarısız
olduğunda ikinci tercih denenmesi ek gecikme yaratabilir, fakat her medya için
günlük tek deneme sınırı içinde kalır.

**Geri dönüş maliyeti:** orta — ayar ve otomatik seçim policy'si kaldırılabilir;
ancak günlük deneme kaydı ve parameterized catalog policy'si geriye dönük
uyumluluk için korunmalıdır.

## İlgili task'lar

`NEN-125`, `NEN-038`, `NEN-033`, `NEN-035`, `NEN-036`

## Notlar

Kullanıcı onayıyla ADR-0010 Karar 9'un ertelenmiş OpenSubtitles basamağı
açılmış sayılır; `NEN-038` implementasyon task'ı olarak başlayabilir.
