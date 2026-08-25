---
id: NEN-037
title: Subtitle language preference setting (primary and secondary)
milestone: M3
size: S
state: backlog
depends_on: [NEN-019, NEN-024]
blocks: []
adr: [10]
---

# NEN-037 — Subtitle language preference setting (primary and secondary)

## Sonuç

Kullanıcı birinci ve ikinci tercih edilen altyazı dilini ayarlayabilir; ayar
oturumlar arasında korunur ve menü sırası ile otomatik seçim bu ayara uyar.

## Kapsam

- macOS ayar yüzeyi: iki dil seçici (birinci / ikinci tercih), ikisi de
  boş bırakılabilir
- Ayarın kalıcılığı (M5'ten önce platform-yerel kalıcılık yeterli;
  cloud sync **non-goal**)
- `SubtitlePreferences` değerinin `nen-catalog` projeksiyonuna ve otomatik
  seçim politikasına (NEN-019) girdi olarak verilmesi
- Dil listesinde adların **endonim** gösterilmesi (ADR-0010 Karar 7)

## YAPILMAYACAK

- Üçüncü tercih veya sıralanabilir tam dil listesi — ADR-0010 iki slotta karar
  kıldı
- Gruplama/sıralama mantığını UI'da yeniden yazmak — çekirdek zaten sıralı
  projeksiyon döndürüyor (NEN-019)
- Cihazlar arası senkronizasyon — **non-goal** (roadmap S8)
- Otomatik indirme davranışı → NEN-038

## Kanıt (DoD)

- [ ] İki tercih ayarlanıp uygulama yeniden başlatıldığında korunuyor
- [ ] Tercih değişince altyazı menüsünün grup sırası değişiyor (checklist)
- [ ] İkinci tercih birinciyle aynı seçilirse yok sayılıyor
- [ ] Tercih boşken menü, ADR-0010 Karar 10'un "tercih ayarlanmamış" örneğiyle
      aynı sırada

## Kanıt kaydı

<!-- done olurken doldurulacak -->
