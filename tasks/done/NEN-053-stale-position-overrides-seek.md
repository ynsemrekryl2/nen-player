---
id: NEN-053
title: A stale position event never overrides a seek that landed
milestone: M3
size: S
state: done
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

- [x] Seek'ten sonra gelen bayat `positionChanged` gösterilen konumu
      değiştirmiyor (test)
- [x] `seekCompleted` değiştiriyor ve sayacı düşürüyor (test)
- [x] İki ardışık seek, iki `seekCompleted` ile temizleniyor (test)
- [x] Emniyet süresi dolduktan sonra `positionChanged` yeniden kabul ediliyor
      (test)
- [x] ~~`commitSeek` bırakma anında eski konumu hiç göstermiyor (test)~~ —
      **testle ayırt edilemiyor**, gerekçesi kanıt kaydında; değişiklik yapıldı
      ama kanıtı yok
- [x] Negatif: guard kaldırılınca bu testlerden en az biri kırmızı
- [x] Elle: gerçek medyada sürükle-bırak, geri sıçrama yok (checklist)

## Kanıt kaydı

Tam kayıt: `evidence/M3/NEN-053-checklist.md`.

**Teşhis ölçümle düzeltildi.** Task açılırken yazılan neden — "mpv seek'i
servis edene kadar bayat `time-pos` yayınlar" — gerçek libmpv'ye karşı ölçünce
**yanlış çıktı**: mpv, seek komutunu alır almaz `time-pos`'u hedefe taşıyor ve
kuyruğun coalescing'i eski değeri zaten siliyor. Kusur bakma anındaydı:
kaydırıcı tutulurken AppKit iç içe izleme döngüsü çalıştırdığı için kabuğun
50 ms'lik poll'u aç kalıyor, bırakma anında **aynı runloop turunda** uyanıyor —
yani seek komutundan mikrosaniyeler sonra — ve kuyrukta o an duran en yeni
konum hâlâ sürükleme öncesinin oynatma başı oluyor.

Ölçüm bu sırayı birebir kurdu (geçici test, commit edilmedi): 1 sn drain
edilmedi, seek verildi, aynı turda drain edildi. Eski kural bırakma anında
`2800 ms` gösterdi ve 8 ms sonra `20000`'e sıçradı — bildirilen semptomun
kendisi. Guard aynı kayıtta `20000`'den hiç ayrılmadı.

`bash scripts/test-macos.sh` çıkış 0: **47 test / 6 suite**, 0 failure. Altı
yeni test `PlayerModelTests` içinde.

**Negatif kontrol:** `consume`'daki tek satırlık guard kaldırılınca **4 test
kırmızı**, ilki semptomun birebir kendisi
(`positionMilliseconds → 4080` beklenen `20000`). Guard geri konunca 47/47
yeşil.

**Kapsanmayan:** `commitSeek()`'teki sıra değişikliği testle ayırt edilemiyor —
iki sıra da aynı turda tamamlanıyor, SwiftUI ikisinde de tek render yapıyor.
Değişiklik korundu ama kanıtı yok; kanıt kaydı §4 bunu açıkça söylüyor.

**Elle acceptance:** ekran kontrolü reddedildiği için geçişi kullanıcı
kendisi yaptı ve düzeltilmiş yapıda "düzgün görünüyor" diye bildirdi. Tek bir
onaydır; ileri/geri ve oynarken/duraklatılmışken ayrımları ayrı ayrı
bildirilmedi (kanıt kaydı §5).
