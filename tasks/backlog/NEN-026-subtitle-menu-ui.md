---
id: NEN-026
title: Subtitle menu UI
milestone: M3
size: M
state: backlog
depends_on: [NEN-019, NEN-025]
blocks: [NEN-027]
adr: [10, 31]
---

# NEN-026 — Subtitle menu UI

## Sonuç

Tek bir altyazı düğmesi, katalog projeksiyonunu şartname §8'deki gruplu yapıda
gösterir.

## Kapsam

- Tek altyazı düğmesi; yüzeyi transport üzerinde **popover**
- `Kapalı` her zaman üstte
- `Kullanıcı Altyazıları` grubu, ardından dil grupları
- **Dil grubu başlıkları endonim** ("English", "Français", "Türkçe") —
  `nen-catalog`'un projeksiyonu yalnız `LanguageTag` döndürür (ADR-0010
  Karar 7), metni yazan bu task'tır; NEN-019'un `menu_projection_golden`
  testi §8'in **yapısını** kanıtlıyor, §8'in **görünen metnini** ilk kez
  üreten yer burasıdır
- Origin rozetleri `Gömülü` · `OpenSubtitles` · `AI` — satırın sağında, soluk
- `Dil Belirsiz` grubu
- Birinci/ikinci tercih edilen dil grupları listenin üstünde (ADR-0010 Karar 4)
- **Tarama beklenmez (ADR-0031 Karar 4):** menü açılır açılmaz `Kapalı` ve
  gömülü track'leri gösterir, sidecar bulundukça grup büyür, tarama sürerken
  bunu belirten bir işaret bulunur. Liste büyürken: seçili kaynak değişmez ·
  odak ve scroll sıfırlanmaz · otomatik seçim **yeniden tetiklenmez**
- **Hatalı kaynak (ADR-0031 Karar 5):** kataloğa girmiş ama kullanılamayan
  kaynak menüde **kalır**, soluk ve **seçilemez** gösterilir, yanında kapalı
  kümeden kısa sebep etiketi taşır — `okunamadı` · `biçim hatalı` · `çok büyük`
- Seçim → o anki gösterilen kaynak **ve** AI çeviri komutunun kaynağı olur

## YAPILMAYACAK

- Render → NEN-027
- "AI ile çevir" komutu → M5
- Ayrı "AI subtitle mode" — **yasak**
- Kaynak seçiminin çeviri başlatması — **yasak** (§9)
- Aynı listenin menü bar'a ikinci kez yansıtılması — M3'te tek yüzey
- Güvenlik kapısından dönen dosyanın menüde görünmesi — **yasak**; o dosya
  kaynak olmadı (ADR-0031 Karar 5, NEN-025)
- Sebep etiketinde teknik detay veya dosya yolu — **yasak**

## Kanıt (DoD)

- [ ] Şartname §8 örneğiyle **görünen metin dahil** (endonim başlıklar,
      Türkçe chrome) eşleşen ekran görüntüsü — M2'nin projeksiyon golden'ı bu
      metni üretmiyordu, ilk gerçek kanıt burada
- [ ] Aynı kaynak menüde iki kez görünmüyor
- [ ] `Kapalı` her koşulda mevcut
- [ ] Kaynak seçmek hiçbir çeviri/indirme işi **başlatmıyor** (log/çağrı kanıtı)
- [ ] Menü, sidecar taraması sürerken açılıyor ve gömülü track'ler seçilebiliyor
- [ ] Menü açıkken yeni kaynak eklendiğinde seçili kaynak, odak ve scroll
      değişmiyor (checklist)
- [ ] Oynatma başladıktan sonra bulunan tercih edilen dildeki sidecar otomatik
      **seçilmiyor**
- [ ] Bozuk `.srt` menüde soluk ve seçilemez, sebep etiketi görünüyor
- [ ] Negatif: symlink/traversal ile verilen dosya menüde **hiç görünmüyor**
- [ ] Ekran görüntüsü `fixtures/` medyasıyla üretilmiş, dosya yolu görünmüyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
