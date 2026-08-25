---
id: NEN-018
title: Media evidence, OS-compatible hash, release name parser
milestone: M2
size: L
state: active
depends_on: [NEN-012]
blocks: [NEN-019, NEN-033, NEN-034, NEN-035, NEN-036]
adr: [9]
---

# NEN-018 — Media evidence, OS-compatible hash, release name parser

## Sonuç

Bir medya dosyasından kimlik kanıtları toplanır ve dosya adından başlık, yıl,
sezon, bölüm çıkarılır; çıkarılamadığında hata değil "bilinmiyor" döner.

## Kapsam

- `MediaEvidence`: filename, sunucunun beyan ettiği ad, boyut,
  OpenSubtitles-uyumlu hash, container metadata, release name, `.nfo` sidecar,
  klasör/kardeş/URL ipuçları, opsiyonel handoff metadata
- Release-name parser: film (title + year) ve dizi (series + S/E) desenleri
- ADR-0009'un katmanlı kanıt sırası (`resolve`) ve §6 fallback'i
  (`identity_candidates`)
- ≥30 gerçekçi dosya adı için golden

**ADR-0009 ile eklenen katmanlar** (`size: M` → `L` bu yüzden):

| Katman | Bu task'ta yapılan |
|---|---|
| Handoff metadata | Alan + sıralamadaki yeri |
| `.nfo` sidecar | Parser (XML + tek satır URL), IMDb/TMDb ID, başlık/yıl/S-E |
| Container metadata | Şekil + başlık/yıl çıkarımı (gerçek demuxer M3) |
| Dosya adı beyanı | Yerel ad **veya** sunucu beyanı; sanitization + testi (dolduran adapter NEN-036) |
| Üst klasör adları | `dir_hints`, yakından uzağa |
| Kardeş mutabakatı | Doğrular, karar üretmez |
| URL path segmentleri | Tüm segmentler, basename'den köke; query/fragment/host hariç |

## YAPILMAYACAK

- OpenSubtitles sorgusu → NEN-033 (M6)
- AI ile release-name normalizasyonu → NEN-034
- Güven skoru / aday sıralama → NEN-035
- Uzak kanıtları fiilen toplayan HTTP portu → NEN-036 (M3); burada yalnız alanlar
- Kullanıcıya düzeltme UI'ı → M3+ (burada yalnız model)
- Herhangi bir I/O — dosya okuma, ağ (ADR-0009 Karar 2)
- Media URL query'sinin kimlik kaynağı olarak kullanılması — **yasak** (§6)
- `.torrent` metadata'sı — **non-goal** (CLAUDE.md; ADR-0009 → Notlar)

## Kanıt (DoD)

- [ ] Hash, bağımsız referans implementasyonuyla eşleşiyor (sentetik korpus +
      sınır vakaları: minimum altı → `Err`, tam eşik, eşik+1, tek byte değişimi)
- [ ] ≥30 dosya adı golden: başlık/yıl/sezon/bölüm doğru
- [ ] Çözümlenemeyen ad için hata değil `Unknown` dönüyor
- [ ] ADR-0009 Karar 6'nın katman sırası test edildi: her katman tek başına
      kazanıyor, üstteki varken alttakiler yok sayılıyor, hiçbiri yokken `Unknown`
- [ ] Negatif: URL query'si evidence'a **girmiyor** (test)
- [ ] Negatif: sunucu beyanı adı sanitize ediliyor — `../`, kontrol karakteri,
      RFC 5987 `filename*` yüzde-encode'u yola dönüşmüyor
- [ ] Negatif: evidence'ın `{:?}` çıktısı tam yol/hash/filename/NFO içeriği
      sızdırmıyor
- [ ] Negatif kontrol: `#[derive(Debug)]`'lı kasıtlı bozuk fixture aynı deseni
      gerçekten sızdırıyor (denetim boşta dönmüyor)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
