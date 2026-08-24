---
id: NEN-018
title: Media evidence, OS-compatible hash, release name parser
milestone: M2
size: M
state: backlog
depends_on: [NEN-012]
blocks: [NEN-019]
adr: [9]
---

# NEN-018 — Media evidence, OS-compatible hash, release name parser

## Sonuç

Bir medya dosyasından kimlik kanıtları toplanır ve dosya adından başlık, yıl,
sezon, bölüm çıkarılır; çıkarılamadığında hata değil "bilinmiyor" döner.

## Kapsam

- `MediaEvidence`: filename, boyut, OpenSubtitles-uyumlu hash, container
  metadata, release name, opsiyonel handoff metadata
- Release-name parser: film (title + year) ve dizi (series + S/E) desenleri
- Fallback sırası (şartname §6)
- ≥30 gerçekçi dosya adı için golden

## YAPILMAYACAK

- OpenSubtitles sorgusu → M6
- Kullanıcıya düzeltme UI'ı → M3+ (burada yalnız model)
- Media URL query'sinin kimlik kaynağı olarak kullanılması — **yasak** (§6)

## Kanıt (DoD)

- [ ] Hash, bilinen referans değerlerle eşleşiyor
- [ ] ≥30 dosya adı golden: başlık/yıl/sezon/bölüm doğru
- [ ] Çözümlenemeyen ad için hata değil `Unknown` dönüyor
- [ ] Negatif: URL query'si evidence'a **girmiyor** (test)
- [ ] Negatif: evidence'ın `{:?}` çıktısı tam yol/hash sızdırmıyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
