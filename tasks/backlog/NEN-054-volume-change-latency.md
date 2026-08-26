---
id: NEN-054
title: A volume change is heard when it is made
milestone: M3
size: M
state: backlog
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

- [ ] Ölçüm raporu: yöntem, öncesi/sonrası gecikme, elenen adaylar
- [ ] Seçilen düzeltmenin testi (ret durumunda değerin geri alınması ve/veya
      bir sürüklemenin motora tek yazma göndermesi)
- [ ] `transportCommands` testindeki kırpma davranışı korunuyor
- [ ] `ao-volume`/`audio-buffer` seçilirse `bash scripts/test-macos.sh` yeşil
- [ ] Elle: ↑/↓ ve kaydırıcı, kaydırıcı takılmadan hareket ediyor (checklist)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
