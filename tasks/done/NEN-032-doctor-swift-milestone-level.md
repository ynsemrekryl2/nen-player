---
id: NEN-032
title: Move swift to M1 in doctor's milestone levels
milestone: M1
size: S
state: done
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

- [x] `scripts/tests/doctor.test.sh` içinde: `swift` yokken `doctor.sh M1`
      **çıkış 1**, varken çıkış 0
- [x] Parametresiz `doctor.sh` hâlâ **daima çıkış 0**
- [x] `bash scripts/test.sh` geçiyor
- [x] `docs/STATUS.md` toolchain tablosu güncel

## Kanıt kaydı

`requirement()`'ta `swift` artık kendi satırında: `swift:M1|swift:M3) echo
blocker`, `xcode:M3|libmpv:M3) echo blocker` (grup ayrıldı, Xcode/libmpv
davranışı değişmedi). `need_note()`'ta `swift) echo "M1"`.

Yeni senaryo `scripts/tests/doctor.test.sh` **S7: swift yok** — gerçek
makinede CLT kuruluyken `/usr/bin/swift` var olduğundan, `command -v swift`
onu bulmasın diye `/usr/bin`'in swift hariç geri kalanı bir gölge dizine
sembolik bağlanıp PATH ona yönlendiriliyor:

```
$ bash scripts/tests/doctor.test.sh
  ...
  S7: swift yok
    ok   S7 parametresiz (bilgilendirici) → exit 0
    ok   S7 M1 blocker (swift yok) → exit 1
    ok   S7 M1 çıktısı BLOCKER bölümü içeriyor
    ok   S7 M3 blocker (kümülatif) → exit 1
  S8: hiçbir kurulum komutu çalıştırılmadı
    ok   brew/curl/rustup/sdkmanager/cargo-install hiç çağrılmadı
```

```
$ bash scripts/test.sh
  check-docs.test.sh ✓ (11 doğrulama)
  doctor.test.sh     ✓ (26 doğrulama, S7 dahil)   → exit 0
```

Gerçek makinede (swift kurulu):

```
$ bash scripts/doctor.sh M1
...
HAZIR:
  ✓ cargo   ✓ rustc   ✓ swift   Apple Swift version 6.4
SONUÇ: M1 için tüm blocker'lar hazır.            → exit 0

$ bash scripts/doctor.sh M3
...  ✓ swift   Apple Swift version 6.4  (kümülatif olarak hâlâ isteniyor)

$ bash scripts/doctor.sh          → çıkış 0 (parametresiz, bilgilendirici, değişmedi)
```

`docs/STATUS.md` toolchain tablosu: `swift` satırı artık `M1 (blocker) —
NEN-007 testi ve NEN-008 harness'ı kullanıyor` diyor; eski "doctor'da M3,
gerçekte M1" tutarsızlık notu kaldırıldı.

`bash scripts/check-docs.sh` → 8/8 denetim geçti, exit 0.
