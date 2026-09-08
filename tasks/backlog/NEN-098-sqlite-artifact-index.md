---
id: NEN-098
title: Artifact metadata index and offline lookup
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-096, NEN-097]
blocks: [NEN-099]
adr: [0017]
---

# NEN-098 — Artifact metadata index and offline lookup

## Sonuç

Daha önce üretilmiş bir çeviri, uygulama yeniden başlatıldıktan sonra ve **ağ
olmadan** cache identity'siyle bulunup açılıyor.

## Bağlam

Şartname §11: "restart sonrası reuse". S4 kararı (2026-09-08) bunu M5'in
DoD'una alıyor: kaydedilmiş bir artifact ağ olmadan açılıp oynatılır, ayrı bir
"offline modu" anahtarı yoktur.

S9 kararı (2026-09-08): farklı provider/model/glossary ile üretilmiş artifact'ler
diskte yan yana durur (kimlikleri zaten ayırıyor), ama hedef dil grubunda M5'te
yalnız **en yeni** olan gösterilir.

## Kapsam

- Metadata index (ADR-0017'nin kararlaştırdığı biçimde) ve kimliğe göre arama
- Hedef dil başına "en yeni artifact" projeksiyonu
- Index ile içerik deposu arasındaki tutarsızlığın (kayıt var, içerik yok)
  sessizce değil tipli hatayla karşılanması

## YAPILMAYACAK

- Katalog/menü entegrasyonu — `NEN-099`
- Artifact silme veya cache temizleme komutu — M6
- Çoklu artifact'in kullanıcıya seçtirilmesi — S9 gereği M5'te yok

## Kanıt (DoD)

- [ ] Unit: yazılan artifact yeni bir process/store örneğinden kimliğiyle bulunuyor (restart reuse)
- [ ] Negatif: **ağ erişimi olmayan** bir ortamda arama ve okuma çalışıyor — HTTP portu hiç çağrılmıyor (çağrı sayacı 0)
- [ ] Negatif: cache identity bileşenlerinden biri değişmiş bir sorgu eski artifact'i **bulmuyor**
- [ ] Negatif: index'te kayıtlı ama içeriği silinmiş artifact tipli hata veriyor, boş/yarım belge döndürmüyor
- [ ] Unit: aynı hedef dil için iki artifact varken projeksiyon en yenisini veriyor, ikisi de diskte kalıyor
- [ ] Guard: index sorgusu ve dosya yolu loglanmıyor (K23 #3, #7)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
