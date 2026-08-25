---
id: NEN-042
title: Recent media list in the empty state
milestone: M3
size: S
state: backlog
depends_on: [NEN-024]
blocks: []
adr: [31]
---

# NEN-042 — Recent media list in the empty state

## Sonuç

Boş durum, son açılan tek medya yerine son N medyayı listeler ve her satır
security-scoped bookmark'ıyla yeniden açılabilir.

## Kapsam

- Son açılan medyaların sıralı listesi (N sabit, küçük)
- Her kayıt için security-scoped bookmark saklama ve tazeleme
- Erişilemeyen kaydın listeden düşürülmesi (taşınmış/silinmiş dosya)
- Listeyi temizleme komutu

## YAPILMAYACAK

- Pozisyon hatırlama / kaldığı yerden devam — ayrı iş, M5 persistence'a bağlı
- Küçük resim / poster üretimi — kapsam dışı
- Tam yolun listede gösterilmesi — **yasak** (ADR-0031 Karar 2); yalnız dosya adı
- Uzak medyanın (http/https) geçmişe yazılması — URL token taşıyabilir;
  ayrı karar gerektirir

## Neden ayrı task

`NEN-024` boş durumda **tek** son açılan satırını üretiyor (ADR-0031
beyin fırtınası, B7). Tam liste + bookmark tazeleme + erişilemeyen kayıt
temizliği ayrı bir yaşam döngüsü işi; M3 slice'ının kabul kriterlerinden
hiçbiri buna bağlı değil (CLAUDE.md kural 5).

## Kanıt (DoD)

- [ ] N medya açıldıktan sonra liste doğru sırada görünüyor
- [ ] Yeniden başlatmada tüm kayıtlar bookmark ile açılabiliyor
- [ ] Taşınan/silinen dosya listeden düşüyor, uygulama hata vermiyor
- [ ] Listede tam yol görünmüyor (yalnız dosya adı)
- [ ] Temizleme komutu listeyi ve saklanan bookmark'ları siliyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
