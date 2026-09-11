---
id: NEN-123
title: macOS subtitle menu shows OpenSubtitles candidates and downloads on select
milestone: M6
size: M
state: backlog
closed:
depends_on: [NEN-122]
blocks: [NEN-126]
adr: [21, 31]
---

# NEN-123 — macOS subtitle menu shows OpenSubtitles candidates and downloads on select

## Sonuç

macOS altyazı menüsünde OpenSubtitles adayları kendi grubunda görünüyor;
bir adayı seçmek indirmeyi başlatıyor, geçici durum gösteriliyor, başarılı
indirme o altyazıyı seçili yapıyor, başarısız indirme mevcut seçimi
**değiştirmiyor** ve ADR-0031 hata sunumuyla bildiriliyor.

## Kapsam

- `SubtitleMenuPresentation`/`SubtitleMenuView`: `openSubtitles` satırları
  (etiket zaten "OpenSubtitles"), rozet/release adı ADR-0021'e göre;
  `NEN-062`/`NEN-067`'nin paneli, yeni krom yok
- `PlayerModel.selectSubtitle` yolu: kind `OpenSubtitles` ve belge yoksa
  `download` → başarıda `selectSubtitle`, başarısızlıkta mevcut seçim ve
  geçici hata mesajı (`NEN-102`'nin geçici-mesaj yuvası)
- İndirme sırasında geçici durum (`TranslationStatusPill` emsali, "İndiriliyor…")
  ve medya değişiminde iptal
- Anahtar yoksa: aday grubu görünmez ya da satır "Ayarlardan anahtar girin"
  ile devre dışı — ADR-0021'in kararı

## YAPILMAYACAK

- İndirme mantığı/kapıları — `NEN-122`
- Aday sıralaması — `NEN-035`
- Otomatik indirme — `NEN-038`
- Kalıcı "indirilenler" listesi

## Kanıt (DoD)

- [ ] Swift testleri (fake FFI/katalog ile): seçim → indirme → seçili;
      başarısız indirme → önceki seçim korunuyor, mesaj gösteriliyor;
      medya değişimi → gelen sonuç uygulanmıyor
- [ ] Negatif: kind `OpenSubtitles` satırı, belge inmeden **seçili**
      olarak işaretlenmiyor (§9 "zorla geçilmez" emsali)
- [ ] Gerçek `.app` checklist (`evidence/M6/NEN-123-checklist.md`, kullanıcı
      onayıyla tek gerçek koşu `NEN-126`'ya bırakılabilir; burada fake FFI
      ile UI checklist yeterli)
- [ ] `bash scripts/test-macos.sh` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
