---
id: NEN-026
title: Subtitle menu UI
milestone: M3
size: M
state: backlog
depends_on: [NEN-019, NEN-025]
blocks: [NEN-027]
adr: [10]
---

# NEN-026 — Subtitle menu UI

## Sonuç

Tek bir altyazı düğmesi, katalog projeksiyonunu şartname §8'deki gruplu yapıda
gösterir.

## Kapsam

- Tek altyazı düğmesi
- `Kapalı` her zaman üstte
- `Kullanıcı Altyazıları` grubu, ardından dil grupları
- Origin rozetleri (Gömülü / OpenSubtitles / AI)
- `Dil Belirsiz` grubu
- Seçim → o anki gösterilen kaynak **ve** AI çeviri komutunun kaynağı olur

## YAPILMAYACAK

- Render → NEN-027
- "AI ile çevir" komutu → M5
- Ayrı "AI subtitle mode" — **yasak**
- Kaynak seçiminin çeviri başlatması — **yasak** (§9)

## Kanıt (DoD)

- [ ] Şartname §8 örneğiyle eşleşen ekran görüntüsü
- [ ] Aynı kaynak menüde iki kez görünmüyor
- [ ] `Kapalı` her koşulda mevcut
- [ ] Kaynak seçmek hiçbir çeviri/indirme işi **başlatmıyor** (log/çağrı kanıtı)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
