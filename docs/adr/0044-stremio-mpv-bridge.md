---
adr: 0044
title: Stremio macOS MPV launcher için geri alınabilir Nen Player köprüsü
status: accepted
milestone: M4
tasks: [NEN-086, NEN-087, NEN-088]
date: 2026-09-08
---

# ADR-0044 — Stremio macOS MPV launcher için geri alınabilir Nen Player köprüsü

## Durum

`accepted`

## Bağlam

Stremio 5.1.26 macOS'ta MPV dış oynatıcısını serbest uygulama seçimiyle değil,
sabit executable yollarını sırayla kontrol ederek keşfeder. Kurulu shell'in
`server.js` kaydında ilk yol `/usr/local/bin/mpv`, sonra `/opt/local/bin/mpv`
ve `/sw/bin/mpv`'dir; ilk mevcut yol `--start=<saniye> --no-terminal <locator>`
şeklinde çalıştırılır. Aynı sözleşme Stremio'nun resmî kaynak tartışmasında da
yer alır: [stremio-shell #351](https://github.com/Stremio/stremio-shell/issues/351).

Önceki M4 ölçümünde başlayan uygulama, Stremio'nun keşfettiği özel bir bundle
değil, bu sabit yollardan birinde önceden kurulmuş bir wrapper'ın hedefiydi.
Nen Player'ın gerçek Stremio akışına girebilmesi için launcher sözleşmesine
bağlanması gerekir; Stremio paketini yamamak veya bundle kimliği taklit etmek
güvenilir bir ürün sınırı değildir.

## Karar

Stremio paketi ve Nen Player bundle kimliği değiştirilmeyecektir. Kullanıcı
açıkça istediğinde, Stremio'nun ilk sabit MPV yolu olan `/usr/local/bin/mpv`
üzerine Nen Player'a yönlendiren geri alınabilir bir wrapper kurulacaktır.
Kurulum mevcut yabancı dosyayı varsayılan olarak reddedecek; yalnız açık bir
değiştirme seçimiyle önce atomik ve birebir geri yüklenebilir yedek alacaktır.
Wrapper Nen tarafından yönetildiğini değişmez bir işaret ve bütünlük kontrolüyle
belirtecek, kaldırma sırasında değiştirilmiş bir dosyayı sessizce silmeyecek ve
önceki dosyayı geri yükleyecektir. Stremio'nun ölçülmüş MPV argv biçimi güvenli
biçimde mevcut handoff girişine aktarılacak; diğer doğrudan `mpv` çağrıları
önceki/gerçek MPV'ye devredilecektir. Hiçbir locator, argüman, token veya özel
yol loglanmayacak; shell metin değerlendirmesi kullanılmayacaktır. Köprü soğuk
ve zaten açık Nen Player süreçlerini aynı `application(_:open:)`/argv handoff
yoluna ulaştıracaktır. Gelen başlangıç değeri aynen uygulanacak; Stremio'nun
mevcut sürümünde ölçülen `0`, baştan oynatma anlamına gelecektir.

## Gerekçe

Stremio'nun sabit yol sözleşmesi kullanıcı tarafından kurulmuş bir wrapper ile
zaten gözlenmiştir; köprü bu ölçülmüş yüzeye bağlanır ve Nen Player'ın kendi
bundle kimliğini yanlış bildirmez. `/usr/local/bin/mpv` seçimi, Stremio'nun
macOS arama sırasındaki ilk yoldur. Açık onay, yedek ve bütünlük kapıları
sistem genelindeki `mpv` çağrılarını sessizce ele geçirme riskini sınırlar.

Stremio'nun mevcut akışının başlangıç değerini `0` göndermesi, alıcı tarafın
olmayan bir konumu üretmesini gerektirmez. Nen Player gelen değeri doğru uygular;
mevcut Stremio sürümünde gerçek devam konumu sağlanmadığı kabul kabul kaydında
açıkça belirtilir.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Stremio paketindeki `server.js` dosyasını yamamak | İmzayı ve güncelleme güvenilirliğini bozar; yerel değişiklik kalıcı ürün entegrasyonu değildir. |
| Nen Player'ı MPV/VLC bundle kimliğiyle sunmak | Uygulama kimliği taklididir; başka kurulumları bozabilir ve Stremio güncellemesine kırılgan biçimde bağlanır. |
| Upstream özel oynatıcı desteğini beklemek | Bugün gerçek Stremio akışı sağlamaz; mevcut kullanıcı senaryosunu doğrulanamaz bırakır. |
| Mevcut `/usr/local/bin/mpv` dosyasını sessizce ezmek | Kullanıcının gerçek MPV veya başka wrapper kurulumunu geri döndürülemez biçimde bozar. |
| Her `mpv` çağrısını Nen Player'a yönlendirmek | Stremio dışı kullanıcı çağrılarını beklenmedik biçimde değiştirir; yalnız ölçülmüş argv şekli yönlendirilmelidir. |

## Sonuçlar

**Olumlu:** Gerçek Stremio eylemi, Stremio paketini değiştirmeden Nen Player'a
ulaşabilir; kurulum ve kaldırma denetlenebilir, geri alınabilir ve K23 sınırları
korunur.

**Olumsuz / kabul edilen maliyet:** Köprü kullanıcı onayı ve sistem executable
erişimi ister; `/usr/local/bin/mpv` başka araçlar tarafından da kullanılabilir.
Stremio'nun `0` gönderdiği sürümde gerçek devam konumu sağlanamaz.

**Geri dönüş maliyeti:** düşük-orta — yönetilen wrapper kaldırılıp yedek geri
yüklendiğinde Nen Player kodu değişmeden eski launcher davranışı döner.

## İlgili task'lar

`NEN-086` · `NEN-087` · `NEN-088` · `NEN-078` · `NEN-084`

## Notlar

Bu karar alıcı yüzey kararını (`ADR-0043`) supersede etmez; yeni karar,
Stremio'nun ölçülmüş sabit launcher yoluna bağlanan yerel köprünün kurulum ve
geri alma politikasını tanımlar.
