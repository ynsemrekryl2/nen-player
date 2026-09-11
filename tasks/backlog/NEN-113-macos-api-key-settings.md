---
id: NEN-113
title: macOS API key entry in settings
milestone: M6
size: M
state: backlog
closed:
depends_on: [NEN-112]
blocks: [NEN-118, NEN-126]
adr: [20, 31]
---

# NEN-113 — macOS API key entry in settings

## Sonuç

Kullanıcı OpenSubtitles, OpenAI ve OpenRouter anahtarlarını macOS ayarlarından
giriyor ve silebiliyor; alan maskeli, kayıtlı anahtar bir daha tam
gösterilmiyor (yalnız "kayıtlı" göstergesi), anahtar Keychain dışında hiçbir
yere yazılmıyor.

## Kapsam

- `SubtitlePreferencesSettingsView` / `TranslationPreferenceStore` emsalinde
  yeni ayar satırları (ADR-0031 Karar 6 + `NEN-110`'un Notlar girdisi): üç
  `SecureField`, her biri "kayıtlı · değiştir · sil" durumlarıyla
- Kaydetme `NEN-112` adapter'ı üzerinden; alan temizlendiğinde bellekteki
  değer sıfırlanıyor (`security-policy.md` §5 "bellekte kısa")
- Boş/whitespace giriş kaydedilmiyor; geçersiz giriş (kontrol karakteri, aşırı
  uzunluk) `NEN-111`'in newtype reddiyle ADR-0031 hata sunumuna düşüyor
- `LanguageCatalog`/`SubtitleMenuPresentation` emsali: metinler Türkçe, ikinci
  bir string tablosu açılmıyor

## YAPILMAYACAK

- Provider/model **seçici** — `NEN-118`
- Anahtarın doğrulanması için ağa çıkmak ("test connection") — bu task
  kaydeder, doğrulamaz; geçersiz anahtar ilk gerçek istekte tipli hata verir
- Genel bir "Ayarlar" ekranı — ADR-0031 Karar 6 yasaklıyor; yalnız satır ekle
- Anahtarı `UserDefaults`/plist'e yazmak — **yasak**

## Kanıt (DoD)

- [ ] Swift testleri: kaydet → "kayıtlı" göstergesi; sil → alan boş; boş giriş
      kaydedilmiyor (fake `ForeignSecureCredentialStore` ile)
- [ ] Negatif (zorunlu): kaydettikten sonra `UserDefaults` ve uygulama
      Application Support dizininde anahtar sentinel'ı **yok** (`grep -r`
      ölçümü checklist'te)
- [ ] Negatif: kayıtlı anahtar UI'da tam metin olarak hiçbir görünümde yok
      (view model'in dışa verdiği string'ler taranıyor)
- [ ] Gerçek `.app` checklist: üç anahtar girildi/silindi, uygulama yeniden
      açılınca "kayıtlı" göstergesi kalıcı (`evidence/M6/NEN-113-checklist.md`)
- [ ] `bash scripts/test-macos.sh` yeşil, baseline + bu task'ın testleri

## Kanıt kaydı

<!-- done olurken doldurulacak -->
