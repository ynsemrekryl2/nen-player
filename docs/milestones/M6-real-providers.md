# M6 — Real Providers

> **İskelet.** Task kırılımı, bir önceki milestone kapanırken üretilir
> (bkz. `tasks/README.md` → "Yeni task açma").

## Amaç

OpenSubtitles, OpenAI ve OpenRouter'ın gerçek entegrasyonu ve kullanıcı API anahtarlarının platform secure storage'ında saklanması.

## Kapsam

- OpenSubtitles resmi API (yalnız metadata kataloglama; indirme seçimde)
- OpenAI Responses API ve OpenRouter provider port implementasyonları
- OpenRouter structured-output capability preflight
- SecureCredentialStore: Keychain (bu milestone'da macOS)
- İndirme güvenliği: approved HTTPS host, bounded redirect, boyut sınırı, archive reddi

## Kapsam dışı

- Scraping — **yasak**
- Seçilmeden indirme — **yasak**
- Testlerde gerçek kredi/kota kullanımı — **yasak**
- Diğer platformların secure storage'ı → M9–M11

## Çıkış kriterleri

- [ ] Katalogda gerçek OpenSubtitles adayları görünüyor, indirme yalnız seçimde
- [ ] Negatif: private file ID / hash / filename dışarı **sızmıyor** (opaque public ID)
- [ ] Negatif: approved liste dışı host'a istek atılmıyor
- [ ] Negatif: archive ve boyut aşımı reddediliyor
- [ ] API key Keychain'de; log/artifact/crash içinde **yok**
- [ ] Testler kayıtlı fixture ile çalışıyor, ağa çıkmıyor

## Task'lar

_(henüz kırılmadı)_

## Bağımlılıklar

M5

## Retro

<!-- kapanışta doldurulacak -->
