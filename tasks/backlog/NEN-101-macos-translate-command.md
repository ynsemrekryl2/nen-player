---
id: NEN-101
title: macOS translate command and target language setting
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-100, NEN-037]
blocks: [NEN-102]
adr: []
---

# NEN-101 — macOS translate command and target language setting

## Sonuç

Kullanıcı macOS'ta hedef çeviri dilini bir ayardan seçebiliyor ve seçili
altyazı kaynağı için "AI ile çevir" komutunu verebiliyor.

## Bağlam

Şartname §9: "Kullanıcı ayrıca **açıkça** 'AI ile <hedef dile> çevir' komutunu
verir. Hedef dil kullanıcı ayarıdır; varsayılan Türkçe olabilir. Kaynak zaten
hedef dildeyse translation başlatılmaz."

`NEN-037` Settings sahnesini ve `SubtitlePreferenceStore` + `LanguageCatalog`
desenini zaten kurdu; hedef dil ayarı bu desenin üzerine gelir, ikinci bir dil
tablosu açılmaz. Komut yeri `PlayerCommands` (`NEN-042`'nin "Son Açılanları
Temizle" maddesini eklediği yer).

## Kapsam

- Settings'e hedef çeviri dili tercihi (varsayılan: sistem dili veya Türkçe)
- `PlayerCommands`'e "AI ile çevir" komutu
- Komutun etkin/devre dışı koşulları: kaynak seçili değilse, kaynak zaten hedef
  dildeyse, ya da iş zaten koşuyorsa devre dışı
- Kaynak seçmenin komutu **tetiklememesi**

## YAPILMAYACAK

- İlerleme ve iptal yüzeyi — `NEN-102`
- Provider veya model seçtirme — M6
- Glossary düzenleme yüzeyi — M6
- Teknik ID girişi — non-goal

## Kanıt (DoD)

- [ ] Swift testi: hedef dil ayarı yazılıp okunuyor ve uygulama yeniden başlatıldığında kalıcı
- [ ] Swift testi: kaynak seçiliyken komut etkin, seçili değilken devre dışı
- [ ] Negatif: kaynak zaten hedef dildeyken komut devre dışı ve iş başlatılamıyor
- [ ] Negatif: kaynak seçmek tek başına hiçbir çeviri işi başlatmıyor
- [ ] Gerçek `.app` checklist: ayar değiştirilip komut veriliyor, iş başlıyor — `evidence/M5/NEN-101-checklist.md`

## Kanıt kaydı

<!-- done olurken doldurulacak -->
