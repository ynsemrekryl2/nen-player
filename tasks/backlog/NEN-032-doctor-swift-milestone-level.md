---
id: NEN-032
title: Move swift to M1 in doctor's milestone levels
milestone: M1
size: S
state: backlog
depends_on: [NEN-030]
blocks: []
adr: []
---

# NEN-032 — `doctor.sh`'ta `swift`in milestone seviyesi

## Sonuç

`bash scripts/doctor.sh M1` Swift'i M1 gereksinimi olarak raporlar; Swift'i
olmayan bir makinede M1'e başlanamayacağı **kapıda** görünür, ilk kırmızı
derlemede değil.

## Bağlam

`doctor.sh` `swift`i **M3** seviyesinde tutuyor, ama M1 Swift'e iki yerden
bağımlı hale geldi:

- `NEN-007` — `scripts/test-apple.sh`, Swift test target'ı FFI iskeletini
  doğruluyor (kanıt kaydı, not 2 bunu zaten yazmıştı)
- `NEN-008` — `scripts/spike-cues.sh`, ölçüm harness'ı Swift executable'ı

İki task da kural 5 gereği doctor'a dokunmadı. Sonuç: `doctor.sh M1` Swift'siz
bir makinede **çıkış 0** verir, sonra `test-apple.sh` çalışmaz. Doctor'ın işi
tam olarak bunu önlemek.

## Kapsam

- `swift` girdisinin seviyesi M3 → M1 (`blocker`)
- Seviye tablosunun kümülatiflik davranışının bozulmadığının doğrulanması
  (M3 çağrısı Swift'i istemeye devam etmeli)
- Tam Xcode ayrımının korunması: **M1 blocker'ı değil** — CommandLineTools
  yetiyor, eksik yolları `test-apple.sh` telafi ediyor (NEN-007 notu)
- `docs/STATUS.md` toolchain tablosundaki seviye satırının güncellenmesi

## YAPILMAYACAK

- Tam Xcode'u herhangi bir milestone'da blocker yapmak → M3 kararı
- libmpv / Android SDK seviyelerine dokunmak
- `doctor.sh`'ın kurulum yapması — script hiçbir şey kurmaz (`doctor.test.sh` S7)

## Kanıt (DoD)

- [ ] `scripts/tests/doctor.test.sh` içinde: `swift` yokken `doctor.sh M1`
      **çıkış 1**, varken çıkış 0
- [ ] Parametresiz `doctor.sh` hâlâ **daima çıkış 0**
- [ ] `bash scripts/test.sh` geçiyor
- [ ] `docs/STATUS.md` toolchain tablosu güncel

## Kanıt kaydı

<!-- done olurken doldurulacak -->
