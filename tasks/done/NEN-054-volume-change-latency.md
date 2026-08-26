---
id: NEN-054
title: A volume change is heard when it is made
milestone: M3
size: M
state: done
depends_on: [NEN-024]
blocks: []
adr: [12, 31]
---

# NEN-054 — A volume change is heard when it is made

## Sonuç

Ses düzeyi değiştirildiğinde kaydırıcı anında hareket eder ve duyulan
değişimin gecikmesi **ölçülmüş** ve kaydedilmiştir; ölçümün gösterdiği neden
düzeltilmiştir.

## Kapsam

### Önce ölçüm

Düzeltme adayları ölçümden önce seçilmez. `NEN-051`'deki gibi geçici,
**commit edilmeyen** kayıtla:

1. **Ayırt edici deney:** ↑/↓ tuşuyla tek adım (tek property yazması) ile
   kaydırıcıyı sürükleme karşılaştırılır. Tek adım da geç duyuluyorsa neden
   ses hattıdır, sürükleme seli değildir.
2. Bir sürüklemede kaç `setVolume` çağrısı gittiği ve her FFI turunun kaç ms
   sürdüğü (p50/p99).

### Sonra düzeltme — ölçümün gösterdiği aday

- **(a) Kaydırıcı geriden geliyor:** `PlayerModel.volume` bugün ancak bloklayan
  çağrı döndükten sonra yazılıyor, yani kaydırıcının okuduğu değer gecikiyor.
  → değer çağrıdan önce yazılır, ret durumunda geri alınır.
- **(b) Sürükleme seli:** her örnek ayrı bir bloklayan FFI turuna gidiyor; ne
  `onEditingChanged` ne throttle var. → kabukta "son değer kazanır": istenen
  düzey saklanır, motora tik başına tek yazma gider.
- **(c) Ses tamponu:** yazılım `volume` filtresi mpv'nin ses tamponunun
  **önünde** uygulanır; cihaza yollanmış örnekler eski düzeyde çalar
  (`audio-buffer` ayarlanmamış, varsayılan ~200 ms). → adapter cihaz düzeyinde
  ayarlar (`ao-volume`) veya tampon küçültülür.

## YAPILMAYACAK

- Ölçmeden düzeltme seçmek
- `nen-ports` port yüzeyini değiştirmek (ör. asenkron `set_volume`) — bu
  mimari karardır, önce ADR yazılır (Kural 4)
- Poll/pump aralıklarına dokunmak
- Kullanıcıya ses hattı seçtirmek

## Karar notu — `ao-volume`

Kullanıcı ses hattına dokunma iznini **verdi**. `ao-volume` seçilirse macOS'ta
uygulamanın sistem karıştırıcısındaki düzeyini değiştirir; bu, kullanıcıya
görünür bir davranış farkıdır ve kanıt kaydında açıkça yazılır. `volume` UI'da
tek doğru olarak kalır: `resynchronize()` bugün ses düzeyini motordan geri
okumuyor, seçilen çözüm bunu bozmamalı.

## Neden ayrı task

Bildirilen semptom: ses düzeyi değişiminin duyulması "baya" gecikmeli. Kod üç
farklı yerde gecikme üretebiliyor (kaydırıcının okuduğu değer, bloklayan
senkron yazmaların birikmesi, ses tamponu) ve hangisinin baskın olduğu
okumayla belirlenemez. `NEN-053` ve `NEN-055` farklı kök nedenlere sahip,
kanıtları da farklı; Kural 5 gereği ayrı.

## Kanıt (DoD)

- [x] Ölçüm raporu: yöntem, elenen adaylar — **öncesi/sonrası gecikme sayısı
      yok**, gerekçesi kanıt kaydında
- [x] Seçilen düzeltmenin testi — cihaz yolu headless test edilemiyor, **yedek
      yol** test edildi; sınır kanıt kaydında
- [x] `transportCommands` testindeki kırpma davranışı korunuyor
- [x] `bash scripts/test-macos.sh` yeşil (49 test / 7 suite)
- [x] Elle: kullanıcı ↑/↓ ile denedi, gecikme gitti (checklist)

## Kanıt kaydı

Tam kayıt: `evidence/M3/NEN-054-checklist.md`.

**Üç adaydan ikisi ölçümle elendi.** Bir sürüklemenin tam yükü (120 ardışık
`setVolume`, pump 33 Hz'de çekerken) ölçüldü: tam FFI turu p50 **8.6 µs**,
120 örnek toplam **1.1 ms**. Sürükleme selinin birikmesi de kaydırıcının
geriden gelmesi de bu sayıyla elendi — bu yüzden kabukta ne throttle ne de
iyimser yazma eklendi (Kural 5: düzeltilecek gecikme yok).

**Kalan aday ses hattıydı** ve mpv'nin kendi kılavuzu mekanizmayı adıyla
yazıyor: `--audio-buffer` varsayılanı 0.2 s ve büyütmek "may make soft-volume
... react slower". Aynı madde bu seçeneğin "yalnız test için" olduğunu da
söylediğinden tampon **küçültülmedi**; düzey tamponun ötesine, cihaza taşındı.

**Kullanıcı ayırt edici deneyi yaptı:** düzeltilmemiş yapıda ↑/↓ ile tek adım
(tek property yazması) da belirgin geç duyuluyordu — sel değil, hat.

**`ao-volume` iki riski ayrıca ölçüldü** (mpv CLI + IPC): dosya yeniden
yüklenince düzey korunuyor (ses %100'e fırlamıyor), süreç yeniden başlayınca
korunmuyor (kabuğun %100 varsayılanı gerçek düzeyle uyuşuyor). Çıkışın yok olup
geri gelmesi ölçülmedi, bilinmiyor olarak kaydedildi.

`bash scripts/test-macos.sh` çıkış 0: **49 test / 7 suite**. Yeni test yalnız
**yedek yolu** kapsıyor — testler `ao=null` ile koşuyor. **Negatif kontrol:**
yedek kaldırılınca iki beklenti kırmızı.

**Elle acceptance:** kullanıcı düzeltilmiş yapıda "şu anda doğru çalışıyor
görünüyor" diye bildirdi. Gecikmenin sayısal öncesi/sonrası ölçümü yapılmadı.
