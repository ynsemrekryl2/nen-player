---
id: NEN-087
title: Install reversible Stremio MPV bridge
milestone: M4
size: M
state: backlog
closed:
depends_on: [NEN-086]
blocks: [NEN-088]
adr: [44]
---

# NEN-087 — Install reversible Stremio MPV bridge

## Sonuç

Kullanıcı onayıyla kurulan köprü, Stremio'nun sabit MPV launcher yolundan Nen
Player'a gider; yabancı mevcut executable sessizce ezilmez ve kurulum geri
alınabilir.

## Kapsam

- `/usr/local/bin/mpv` için açık onaylı, idempotent kurulum ve kaldırma akışı
- Yabancı dosyada varsayılan ret; açık değiştirme seçeneğinde atomik yedek
- Nen tarafından yönetilen wrapper için değişmez işaret ve bütünlük kontrolü
- Stremio argv şeklinin Nen handoff girişine güvenli aktarımı
- Tanınmayan doğrudan `mpv` çağrılarının önceki/gerçek MPV'ye devri
- Soğuk ve zaten açık Nen Player süreçlerinin aynı handoff yolunu kullanması

## YAPILMAYACAK

- Stremio uygulama paketini veya imzasını değiştirmek
- MPV/VLC bundle kimliğini taklit etmek
- Locator, token, query veya özel yolu loglamak
- Stremio'dan geri pozisyon/sonuç protokolü almak

## Kanıt (DoD)

- [ ] Temiz makinede kurulum, mevcut dosyada güvenli ret, açık değiştirme ve
      kaldırma sonrası birebir geri yükleme testleri
- [ ] Stremio biçimli argv'nin değişmeden handoff'a ulaştığı test
- [ ] Tanınmayan çağrının gerçek MPV'ye devredildiği test
- [ ] Negatif: wrapper stdout/stderr/log yüzeyine locator veya argv sızdırmıyor
- [ ] Soğuk ve sıcak uygulama açılışı platform testiyle doğrulanıyor

## Kanıt kaydı

<!-- NEN-087 kapanışında doldurulacak. -->
