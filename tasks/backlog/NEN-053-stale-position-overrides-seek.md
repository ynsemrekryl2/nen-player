---
id: NEN-053
title: A stale position event never overrides a seek that landed
milestone: M3
size: S
state: backlog
depends_on: [NEN-051]
blocks: [NEN-055]
adr: [11, 31]
---

# NEN-053 — A stale position event never overrides a seek that landed

## Sonuç

Kaydırıcı bırakıldığında top bırakıldığı yerde kalır. Seek inene kadar gelen
`positionChanged` olayları kabuğun gösterdiği konumu değiştiremez.

## Kapsam

- `PlayerModel`'de uçuştaki seek sayacı: `seek(to:)` başarıyla dönünce artar,
  `seekCompleted` geldikçe azalır
- `consume` içinde `positionChanged` ile `seekCompleted`'ın ayrılması — bugün
  ikisi aynı `case` dalında ve ayrımsız uygulanıyor
- Sayaç sıfırdan büyükken `positionChanged`'ın yok sayılması
- Kendi kendini iyileştiren emniyet süresi: son seek'ten sonra belirli bir süre
  geçtiyse sayaç dikkate alınmaz (kayıp bir cevap konumu kalıcı olarak
  dondurmasın). Süre ve saat, mevcut enjeksiyon kalıbıyla `init`'e girer
- Sayacın `openMedia`, `shutdown`, `resynchronize` ve seek reddinde sıfırlanması
- `commitSeek()` sırası: önce seek verilir, sonra önizleme silinir

## YAPILMAYACAK

- Poll aralığını veya pump aralığını değiştirmek → aralık ADR-0033 Karar 2
  gereği çekirdekte
- Komuttan sonra anında drain → `NEN-055`
- `EventQueue`'nun coalescing kurallarını değiştirmek → ADR-0011 Karar 1
- Sürükleme sırasında canlı önizleme (ara seek göndermek) — ayrı bir ürün
  kararı, bu task görüneni yalanlamayı bitirir

## Neden ayrı task

Bildirilen semptom: sürüklenip bırakılan top önce **eski** konuma ışınlanıyor,
sonra bırakılan yere geliyor. Kod okunduğunda üç neden çıktı ve üçü de kabukta:

1. `commitSeek()` önizlemeyi seek'i vermeden **önce** siliyor, yani gösterilen
   konum bir an sürükleme öncesine düşüyor.
2. `consume` bayat `positionChanged`'ı hiçbir korumasız uyguluyor. mpv seek'i
   servis etmeden önce `time-pos` yayınlamayı sürdürür; bu olaylar bir sonraki
   drain'de (33 ms pump + 50 ms poll) gelir ve iyimser yazılan hedefi ezer.
3. `EventQueue.push` coalescing olan `PositionChanged`'ı kuyruğun sonuna
   taşır, `SeekCompleted` ise kritik olduğu için yerinde kalır. Tek bir drain
   içinde bile bayat konum doğru cevabın üstüne yazabilir.

`NEN-051` seek'in **cevabını** doğru olaya bağladı; bu task cevabın kabukta
ezilmemesini sağlar. Depoda uçuştaki seek'i tutan hiçbir bayrak yok.

## Kanıt (DoD)

- [ ] Seek'ten sonra gelen bayat `positionChanged` gösterilen konumu
      değiştirmiyor (test)
- [ ] `seekCompleted` değiştiriyor ve sayacı düşürüyor (test)
- [ ] İki ardışık seek, iki `seekCompleted` ile temizleniyor (test)
- [ ] Emniyet süresi dolduktan sonra `positionChanged` yeniden kabul ediliyor
      (test)
- [ ] `commitSeek` bırakma anında eski konumu hiç göstermiyor (test)
- [ ] Negatif: guard kaldırılınca bu testlerden en az biri kırmızı
- [ ] Elle: gerçek medyada sürükle-bırak, geri sıçrama yok (checklist)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
