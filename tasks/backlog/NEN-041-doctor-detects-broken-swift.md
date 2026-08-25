---
id: NEN-041
title: doctor.sh reports an installed-but-unrunnable tool as missing
milestone: M3
size: S
state: backlog
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

- [ ] Negatif: hata dönen `swift` stub'ıyla `doctor.sh M1` ve `doctor.sh M3`
      çıkış 1 veriyor, `swift` blocker listesinde görünüyor
- [ ] Sağlam kurulumda yanlış pozitif yok — bu makinede `doctor.sh M3` çıkış 0
- [ ] `bash scripts/test.sh` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
