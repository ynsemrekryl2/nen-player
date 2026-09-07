---
id: NEN-041
title: doctor.sh reports an installed-but-unrunnable tool as missing
milestone: M3
size: S
state: done
closed: 2026-09-06
depends_on: [NEN-032]
blocks: []
adr: []
---

# NEN-041 — doctor.sh reports an installed-but-unrunnable tool as missing

## Sonuç

PATH'te bulunan ama çalıştırıldığında hata dönen bir araç (`swift --version`
çıkış kodu ≠ 0) `doctor.sh` tarafından ✓ raporlanmaz; ilgili milestone kapısı
kapalı kalır.

## Bağlam

2026-08-25'te bu makinede gözlendi: tam Xcode kuruluydu ama lisans kabul
edilmemişti, dolayısıyla `swift --version` çıkış 69 ile
`You have not agreed to the Xcode license agreements.` dönüyordu — yani `swift`
**çalışmıyordu**. `doctor.sh M3` yine de `✓ swift (sürüm okunamadı)` yazıp
kapıyı **açık** raporladı (çıkış 0).

Sebep `detect_swift`: `command -v swift` guard'ı geçiyor, `run_timeout`'un
çıktısından sürüm okunamıyor, fonksiyon yine de `echo "swift (sürüm
okunamadı)"` ile çıkış 0 dönüyor.

Bu, **NEN-005'te `detect_jdk`'da bulunan yanlış-pozitifin aynısı** (orada
`command -v` guard'ı hiç yoktu) ve tam olarak **NEN-032'nin önlemek istediği
hata sınıfı**: kapı sahte biçimde açık görünüyor, hata ilk gerçek Swift
komutuna (NEN-022 / NEN-024) erteleniyor.

## Kapsam

- `detect_swift` — çalıştırma başarısızlığında (`run_timeout` çıkış kodu ≠ 0
  veya sürüm okunamadı) `return 1`
- Aynı desendeki diğer `detect_*` fonksiyonları gözden geçirilir: sürüm
  okunamadığında ✓ raporlayan başka bir yer var mı
- `scripts/tests/doctor.test.sh`'e yeni senaryo — NEN-032'nin S7'sinde
  kullanılan shadow-PATH tekniğinin eşi: PATH'te hata dönen bir `swift` stub'ı
  varken doctor'ın M1 ve M3 kapılarını **kapalı** raporladığı doğrulanır

## YAPILMAYACAK

- Lisans kabulünü (`sudo xcodebuild -license accept`) script'in kendisinin
  yapması — `doctor.sh` hiçbir şey kurmaz veya değiştirmez, bu özellik
  NEN-032'de mekanik olarak kanıtlanmış durumda
- Araç sürümlerine minimum eşik koymak — bu task yalnız "çalışıyor mu"
  ayrımını düzeltir, sürüm politikası değil

## Kanıt (DoD)

- [x] Negatif: hata dönen `swift` stub'ıyla `doctor.sh M1` ve `doctor.sh M3`
      çıkış 1 veriyor, `swift` blocker listesinde görünüyor
- [x] Sağlam kurulumda yanlış pozitif yok — bu makinede `doctor.sh M3` çıkış 0
- [x] `bash scripts/test.sh` yeşil

## Kanıt kaydı

`detect_swift`, `swift --version` çıktısını filtrelemeden önce komutun çıkış
kodunu koruyor; komut sıfırdan farklı dönerse veya başarılı dönmesine rağmen
`Apple Swift version …` satırı üretmezse artık `return 1` veriyor. Böylece
PATH'te bulunmak tek başına hazır sayılmaya yetmiyor.

Aynı boru hattı yanlış pozitifi için bütün `detect_*` fonksiyonları gözden
geçirildi. `cargo`, `rustc`, `cargo-deny`, JDK, Gradle ve Xcode sürüm
komutları da önce gerçek çıkış kodunu, sonra boş/okunamayan çıktıyı denetliyor.
Başarısız olup yine de metin basan `pkg-config` çıktısı libmpv sürümü olarak
kabul edilmiyor; varsa dylib fallback'i kullanılmaya devam ediyor. Android SDK
tespiti komut çalıştırmadığı için değişmedi.

Deterministik shadow-PATH kanıtı (`scripts/tests/doctor.test.sh`):

```
S8: swift PATH'te ama çalışmıyor
  ok   S8 M1 blocker (swift exit 69) → exit 1
  ok   S8 M1 Swift'i blocker listesinde gösteriyor
  ok   S8 M3 blocker (kümülatif) → exit 1
S9: swift çalışıyor ama sürümü okunamıyor
  ok   S9 M1 blocker (sürüm okunamadı) → exit 1
  ok   S9 M1 Swift'i blocker listesinde gösteriyor
S10: diğer sürüm komutlarının hataları başarı sayılmıyor
  cargo · rustc · Xcode · cargo-deny · JDK · Gradle → hazır gösterilmiyor
  hatalı pkg-config → dylib fallback
```

Test dosyası toplam **49 doğrulamayla** çıkış 0 verdi; parametresiz doctor
bilgilendirici olarak çıkış 0 kalıyor ve hiçbir kurulum komutu çalıştırılmıyor.

Gerçek makine kanıtı:

```
$ bash scripts/doctor.sh M3
✓ cargo 1.98.0 · ✓ rustc 1.98.0 · ✓ Xcode 26.6
✓ Apple Swift version 6.3.3 · ✓ libmpv 2.5.0
SONUÇ: M3 için tüm blocker'lar hazır. → exit 0

$ bash scripts/test.sh
check-docs.test.sh ✓ · doctor.test.sh ✓
SONUÇ: 2 test dosyasının hepsi geçti. → exit 0
```
