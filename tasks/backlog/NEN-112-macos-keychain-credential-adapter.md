---
id: NEN-112
title: macOS Keychain credential adapter
milestone: M6
size: M
state: backlog
closed:
depends_on: [NEN-111]
blocks: [NEN-113]
adr: [20]
---

# NEN-112 — macOS Keychain credential adapter

## Sonuç

macOS'ta `ForeignSecureCredentialStore`'u implemente eden Keychain adapter'ı
`NEN-111`'in contract kitini **gerçek Keychain** üzerinde geçiyor; anahtar
yazılıyor, okunuyor, siliniyor ve uygulamanın kendi service adı dışındaki
hiçbir kayda erişilmiyor.

## Kapsam

- Yeni Swift modülü (ör. `NenCredentialsKeychain`) — `SecItemAdd` /
  `SecItemCopyMatching` / `SecItemUpdate` / `SecItemDelete`; kind → account,
  sabit service adı (ADR-0020); `kSecAttrAccessible` seçimi ADR'de
- Keychain hatalarının `FfiCredentialError`'a payload'sız eşlenmesi
  (`errSecItemNotFound` → yok, `errSecAuthFailed`/`errSecInteractionNotAllowed`
  → `Denied`, diğerleri → `Unavailable`)
- Test: contract kitinin Swift tarafı, **ayrı bir test service adı** ile
  gerçek Keychain'de koşar ve kendi kayıtlarını temizler (`NEN-049`'un gerçek
  libmpv testlerinin serileştirme emsali gerekiyorsa uygulanır)
- `PlaybackSessionClient` emsali: adapter'ın Rust'a verilişi tek yerde
  (kompozisyon kökü)

## YAPILMAYACAK

- Ayarlar UI'ı — `NEN-113`
- Anahtarı `UserDefaults`, plist veya dosyaya yazmak — **yasak** (§15)
- iCloud Keychain senkronu — cloud sync non-goal
- Diğer platform depoları — M9–M11

## Kanıt (DoD)

- [ ] Swift contract testleri gerçek Keychain'de geçiyor (`bash scripts/test-macos.sh`)
- [ ] Negatif (zorunlu): silinen anahtar okunmuyor; farklı kind'lar
      birbirine karışmıyor; test service adı dışındaki kayıtlara dokunulmuyor
      (test öncesi/sonrası `SecItemCopyMatching` sayımı)
- [ ] Negatif: Keychain hatası tipli, payload'sız dönüyor — Swift test
      `errSecItemNotFound`'ı "yok"a, bir hata kodunu `Unavailable`'a eşliyor
- [ ] K23: adapter'ın hiçbir `print`/log yolu anahtar değerini içermiyor
      (kod taraması + guard testi)
- [ ] `bash scripts/build-macos-app.sh` ve `codesign --verify` geçiyor
      (Keychain erişimi ad-hoc imzalı `.app`'te çalışıyor — gerçek `.app`'te
      tek set/get denemesi checklist'e)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
