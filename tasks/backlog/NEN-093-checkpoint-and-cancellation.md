---
id: NEN-093
title: Checkpoint only validated blocks and cancel without late commit
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-092]
blocks: [NEN-094]
adr: []
---

# NEN-093 — Checkpoint only validated blocks and cancel without late commit

## Sonuç

Yalnız tamamen doğrulanmış bir blok checkpoint'leniyor ve iptal edilen bir
çeviri işi **hiçbir koşulda** sonradan bir şey commit etmiyor.

## Bağlam

Şartname §10: "yalnız tamamen doğrulanmış block checkpoint" · "cancellation
kontrolleri" · "progressive veya yarım subtitle publication yok". M5'in iki
çıkış kriteri doğrudan buraya bakıyor.

Late-commit yasağının kontratı `ADR-0004`'ün konusu ve `NEN-009`
(`spike-async-cancel`) bunu M1'de ölçmüştü — o spike'ın kanıt deseni
(iptal sonrası commit sayacının **sıfır** kalması) burada ürün koduna taşınır.
Spike kodunun kendisi terfi etmez (Kural 7).

## Kapsam

- Blok başına checkpoint kaydı — yalnız doğrulama geçtikten sonra
- Blok sınırlarında iptal kontrolü ve işin sonlanma yolu
- Yarıda kalmış işin checkpoint'inden devam edebilmesi
- İptal sonrası hiçbir yazma/teslim yolunun açık kalmaması

## YAPILMAYACAK

- Checkpoint'in **diske** yazılması — `NEN-096`; burada checkpoint iş içi durumdur
- Artifact üretimi — `NEN-094`
- Kullanıcıya ilerleme gösterimi — `NEN-100` (FFI) · `NEN-102` (macOS)

## Kanıt (DoD)

- [ ] Unit: doğrulama başarısız olan blok checkpoint'lenmiyor (sayaç ile ayırt ediliyor)
- [ ] Unit: yarıda bırakılan iş, checkpoint'lenmiş bloklardan devam ediyor, onları yeniden çevirmiyor
- [ ] Negatif: iptalden sonra commit sayacı **0** — geç gelen provider cevabı hiçbir şey yazmıyor
- [ ] Negatif: iptal kontrolü kaldırıldığında bu test kırmızıya dönüyor (kontrol sağır değil)
- [ ] Negatif: yarısı doğrulanmış bir iş **hiçbir** kısmi belge yayımlamıyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
