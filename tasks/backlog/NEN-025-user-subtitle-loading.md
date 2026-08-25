---
id: NEN-025
title: User subtitle loading and sidecar discovery
milestone: M3
size: M
state: backlog
depends_on: [NEN-013, NEN-015, NEN-024]
blocks: [NEN-026]
adr: [8]
---

# NEN-025 — User subtitle loading and sidecar discovery

## Sonuç

Kullanıcı altyazı dosyası yükleyebilir ve medyanın yanındaki aynı basename'li
SRT otomatik bulunur; güvenlik kurallarını ihlal eden dosyalar reddedilir.

## Kapsam

- "Altyazı Dosyası Yükle" akışı
- Sidecar tarama: medya ile aynı basename'e sahip `.srt`
- Güvenlik kapıları (`docs/security-policy.md` §4): regular-file doğrulaması,
  symlink reddi, path traversal reddi, boyut sınırı
- Encoding kontrolü + strict SRT parse
- **Yol digest'i:** `SubtitleSourceId::user(digest)`'in beklediği `[u8; 32]`'yi
  üreten yer burasıdır (ADR-0010 Karar 2). `nen-domain` bağımlılıksız olduğu
  için hash'i o crate hesaplayamaz ve ham yolu **hiçbir koşulda** almaz
  (K23 #3). Aynı dosya iki kez yüklendiğinde katalogda tek giriş kalması bu
  digest'e bağlıdır — kanıtı da burada üretilir.
- Hatalı dosya → kaynak "hatalı" işaretlenir, **playback durmaz**

## YAPILMAYACAK

- Menüde gösterim → NEN-026
- OpenSubtitles indirme → M6
- Klasör tarama / özyinelemeli arama — yalnız aynı basename

## Kanıt (DoD)

- [ ] Geçerli sidecar otomatik bulunuyor ve kullanıcı grubuna ekleniyor
- [ ] Negatif: symlink olarak verilen `.srt` **reddediliyor**
- [ ] Negatif: `../` içeren yol **reddediliyor**
- [ ] Negatif: dizin/FIFO **reddediliyor**
- [ ] Negatif: boyut sınırını aşan dosya okunmuyor
- [ ] Aynı dosya iki kez yüklendiğinde katalogda tek giriş kalıyor (digest eşit)
- [ ] Bozuk SRT'de playback **devam ediyor**, kaynak hatalı işaretleniyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
