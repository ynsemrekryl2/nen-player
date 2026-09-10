---
id: NEN-107
title: Document-wide translation progress across the FFI boundary
milestone: M6
size: M
state: backlog
closed:
depends_on: [NEN-102]
blocks: []
adr: []
---

# NEN-107 — Document-wide translation progress across the FFI boundary

## Sonuç

Koşan bir çeviri işinin ilerlemesi kullanıcıya blok sırası değil, belgenin
tamamına göre bir yüzde/sayaç olarak görünüyor — ve bu sayaç hiçbir koşulda
geri gitmiyor.

## Bağlam

`NEN-102`'nin ölçtüğü kısıt: `FfiTranslationProgress` bugün **blok-yereldir**.
`core/crates/nen-translate/src/checkpoint.rs` her blok sınırında
`TranslationCall::fork` kullanır, `done` sıfıra döner ve `total` yalnız o
bloğun çıktı cue sayısıdır (`checkpoint.rs`'in kendi yorumu, satır ~722). FFI
belgenin toplam cue sayısını hiç geçirmiyor — `FfiTranslationProgress`'te
böyle bir alan yok, `FfiMenuEntry`'de de yok. Kabuk bugün dürüst bir payda
hesaplayamıyor; `NEN-102` bu yüzden yalnız blok sırası + blok içi determinate
çubuk gösterdi.

M5'in mock provider'ı ağsız ve anlık olduğu için bu eksik görünmüyordu. M6'nın
gerçek sağlayıcılarıyla bir iş dakikalarca sürecek — kullanıcı o sırada
"3. blok" değil, okunabilir bir ilerleme ister. Bu yüzden bu task M6'nın gerçek
sağlayıcı entegrasyonundan **önce** planlanıyor.

**Ölçülmüş tuzak (NEN-106'nın kusuruyla aynı aile):** `TranslationCall`
ilerlemenin geri gitmesini ADR-0004 Karar 4 gereği `Permanent` ile reddeder.
`NEN-092`'nin repair/full-block retry'ı aynı bloğun cue'larını yeniden sürer
— blok-yerel `done`'ı olduğu gibi "offset + done" ile belge geneline
çevirmek, bir retry sırasında toplamı geri düşürür ve iş **kendi ilerleme
raporu yüzünden** düşer (`NEN-106`'nın tam olarak ölçtüğü kusur, farklı bir
yerde). Çözüm ya adaptörün gördüğü en yüksek değeri kelepçelemesi (`max(seen,
offset+done)`) ya da `nen-translate` tarafında retry'ların ayrı bir progress
kanalından geçip yalnız checkpoint anında belge-geneli sayaca yansıması —
hangisi seçilirse ADR-0004 Karar 4'ün monotonluk kuralı bozulmadan.

## Kapsam

- Belge-geneli, monoton ilerleme hesabı (`nen-translate` veya `nen-app`
  seviyesinde — implementasyon task'ının kendi kararı)
- `FfiTranslationProgress`'in belge-geneli `done`/`total` taşıması (veya yeni
  bir alan/varyantla genişlemesi — geriye uyumluluk implementasyon task'ının
  kararı)
- `NEN-102`'nin kabuk tarafının yeni payda ile beslenmesi (`TranslationProgressState`
  içindeki `block`/blok-içi fraction'ın belge-geneli fraction ile
  değiştirilmesi ya da ikisinin birlikte gösterilmesi)

## YAPILMAYACAK

- Retry/repair bütçesinin kendisi — `NEN-092` (zaten done)
- Checkpoint/cancellation mekanizması — `NEN-093` (zaten done, üstüne inşa
  edilir, değiştirilmez)
- Gerçek provider entegrasyonu — M6'nın kendi task'ları

## Kanıt (DoD)

- [ ] Unit: çok bloklu bir belgede ilerleme raporları **belge geneline göre**
      monoton artıyor (blok sınırında sıfırlanmıyor)
- [ ] Negatif (zorunlu): bir bloğun repair/full-block retry'ı sırasında
      belge-geneli `done` hiçbir koşulda **geri gitmiyor** — `NEN-106`'nın
      emsali, elle mutasyonla kırmızıya döndürülüp kanıtlanır
- [ ] Kontrat testi: `nen-ports`'un mevcut monotonluk kuralı
      (`TranslationCall::progress`) bu yeni yol için de geçerli — gevşetilmedi
- [ ] `NEN-102`'nin Swift testleri yeni paydayla güncellenip yeşil kalıyor
- [ ] Gerçek `.app` checklist: ilerleme yüzdesi büyük bir belgede okunabilir
      şekilde artıyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
