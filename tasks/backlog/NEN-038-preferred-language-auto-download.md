---
id: NEN-038
title: Auto-download subtitles for the preferred language
milestone: M6
size: M
state: backlog
depends_on: [NEN-019, NEN-033, NEN-035, NEN-036]
blocks: []
adr: []
---

# NEN-038 — Auto-download subtitles for the preferred language

## Sonuç

Tercih edilen dilde yerel kaynak yoksa OpenSubtitles adayı otomatik indirilir —
ancak kimlik çıkarımının doğruluğu **ölçülmüş** biçimde yeterliyse.

## Kapsam

- ADR-0010 Karar 9'un tür önceliğindeki **üçüncü basamağın** açılması:
  `embedded` → `user` → `opensubtitles`
- Birinci tercih dilinde aday yoksa ikinci tercihe düşme
- Kullanıcının bu davranışı kapatabilmesi
- Yanlış altyazı indiğinde davranış (geri alma / sessizce değiştirmeme)

## Ön koşul — kendi ADR'si

ADR-0010 Karar 9 bu basamağı **reddetmedi, erteledi**. Açılması iki şeye bağlı:

1. Kimlik çıkarımının ölçülmüş doğruluğu — `NEN-033` · `NEN-035` (ölçülebilir
   kabul kriteri) · `NEN-036`
2. Açılışta otomatik ağ isteği ve indirme kullanıcı-görünür bir davranış
   sözleşmesidir → **yeni bir ADR** gerekir. En az şunları karara bağlamalı:
   güven eşiğinin sayısal değeri · kullanıcının kapatabilmesi · başarısızlık
   davranışı.

**ADR yazılıp kabul edilmeden bu task `active`'e alınmaz.**

## YAPILMAYACAK

- `ai` kaynağını otomatik seçime sokmak — §9 açık komut şartı, ADR-0010 Karar 9
- Tercih edilmeyen dillerde otomatik seçim veya indirme
- Kullanıcı seçimini indirilen altyazıyla sessizce değiştirmek

## Kanıt (DoD)

- [ ] Eşik altı güvende indirme **yapılmıyor** (negatif test)
- [ ] Kullanıcı kapattığında hiçbir otomatik istek çıkmıyor (negatif test)
- [ ] Birinci tercihte aday yoksa ikinciye düşülüyor, ikisi de yoksa `Kapalı`
- [ ] Deterministic fake provider ile — gerçek kota kullanılmıyor (Kural 8)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
