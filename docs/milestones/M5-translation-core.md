# M5 — Translation Core

> Task kırılımı 2026-09-08'de üretildi (M4 kapanış ritüelinin gecikmiş son
> maddesi — `docs/roadmap.md` → "Milestone kapanış ritüeli").

## Amaç

AI çeviri pipeline'ının tamamı: bağlam analizi, overlapping blok çevirisi, sıkı yerel doğrulama, checkpoint/cancellation ve kalıcı artifact. Bu milestone **mock provider** ile tamamlanır — gerçek sağlayıcılar M6'da.

## Kapsam

- Blok stratejisi (varsayılan 40, izin 30–60, overlap 6)
- Strict validation: exact cue count, izin verilen/unique ID, non-empty text, sıra normalizasyonu
- En fazla 2 targeted repair + 1 full-block retry
- Yalnız doğrulanmış blok checkpoint
- ValidatedSubtitleArtifact + cache identity + atomik commit
- SQLite index + content-addressed store (ADR-0017)
- Deterministic mock provider

## Kapsam dışı

- Gerçek provider entegrasyonu → M6
- Secure credential storage → M6
- Progressive/yarım subtitle yayını — **yasak**
- Kaynak seçiminin çeviri başlatması — **yasak**

## Çıkış kriterleri

- [ ] Uçtan uca: kaynak seçimi → açık çeviri komutu → doğrulanmış artifact
- [ ] Yarım/progressive çıktı hiçbir koşulda yayınlanmıyor
- [ ] İptal sonrası **late commit yok**
- [ ] Cache identity bileşenlerinden biri değişince eski artifact **kullanılmıyor**
- [ ] Cue ID/sıra/zamanlar girdiyle birebir aynı
- [ ] Çeviri sırasında kaynak değişimi işi retarget **etmiyor**

## Kırılım öncesi kapanan açık sorular

Kırılım üretilmeden önce, milestone'u bekleten üç açık soru kullanıcı kararıyla
kapandı (2026-09-08):

| Soru | Karar | M5'e etkisi |
|---|---|---|
| **S3** — çeviri kalite hedefi | M5'in ölçütü **yapısal doğruluk**: cue sayısı birebir, ID'ler izinli ve tekil, metin boş değil, sıra/zamanlar korunuyor. Dilsel kalite çıtası gerçek model geldiğinde M6'da kapanır | `NEN-091`'in DoD'u yapısal; ADR-0015 bağlam analizinin *dilsel* faydasını iddia etmez |
| **S4** — offline/uçak modu | Kaydedilmiş artifact ağ olmadan açılır; çeviri komutu ağ yokken tipli hata verir. **Ayrı bir offline modu yok** | `NEN-098`'in DoD'una ağsız okuma negatif testi girdi |
| **S9** — çoklu AI çevirisi | Farklı provider/model/glossary ile üretilen artifact'ler **diskte yan yana** durur; M5'te hedef dil grubunda yalnız **en yeni** gösterilir | ADR-0018 ve `NEN-098`'in projeksiyonu; ayrı bir menü seçim task'ı açılmadı |

## Task'lar

`NEN-072` (done) · `NEN-089` · `NEN-090` · `NEN-091` · `NEN-092` · `NEN-093` ·
`NEN-094` · `NEN-095` · `NEN-096` · `NEN-097` · `NEN-098` · `NEN-099` ·
`NEN-100` · `NEN-101` · `NEN-102` · `NEN-103` · `NEN-044` · `NEN-104`

```
089 blok düzeni ─┬─▶ 090 provider+mock ─▶ 091 doğrulama ─▶ 092 repair ─▶ 093 checkpoint/iptal ─▶ 094 artifact ─┬─▶ 095 ADR ─▶ 096 CAS ──┐
                 │                                                                                             └─▶ 097 kimlik ─────────┴─▶ 098 index
                 │                                                                                                                          │
                 └─▶ 103 ADR ─▶ 044 gömülü metin ──────────────────────────────────────────────────────────────────┐                        ▼
                                                                                                                    │        099 orkestrasyon ─▶ 100 FFI ─▶ 101 macOS komut ─▶ 102 macOS ilerleme
                                                                                                                    └────────────────────────────────────────────────────────────────────┴─▶ 104 acceptance
```

### Çıkış kriteri → kanıt eşlemesi

| Çıkış kriteri | Kanıtlayan task |
|---|---|
| Uçtan uca: kaynak seçimi → açık çeviri komutu → doğrulanmış artifact | `NEN-099` (integration) · `NEN-104` (acceptance) |
| Yarım/progressive çıktı hiçbir koşulda yayınlanmıyor | `NEN-092` · `NEN-093` · `NEN-094` · `NEN-102` (negatif) |
| İptal sonrası **late commit yok** | `NEN-093` (negatif) · `NEN-100` (FFI negatif) |
| Cache identity bileşenlerinden biri değişince eski artifact **kullanılmıyor** | `NEN-097` (bileşen başına negatif) · `NEN-098` |
| Cue ID/sıra/zamanlar girdiyle birebir aynı | `NEN-091` (negatif) · `NEN-094` (golden) |
| Çeviri sırasında kaynak değişimi işi retarget **etmiyor** | `NEN-099` (negatif) |

### ADR'ler

| ADR | Konu | Karar veren task |
|---|---|---|
| **0015** | Blok stratejisi: sınır kuralı, overlap'in sözü, belge bağlamı, `block-layout version` | `NEN-089` |
| **0016** | Doğrulama ve onarım politikası; yerel doğrulama authoritative | `NEN-091` |
| **0017** | Persistence adapter (aday: SQLite index + content-addressed store), atomik commit | `NEN-095` |
| **0018** | Cache identity bileşenleri, versiyonlama, invalidasyon | `NEN-097` |
| **0045** | Gömülü altyazı metninin demux yolu | `NEN-103` |

Beşi de bugün `proposed`. Kural 4 gereği ilgili ADR `accepted` olmadan o
task'ın implementasyonuna geçilmez ve teknoloji adları **aday** sayılır.

## Bağımlılıklar

M2 + M3

## Retro

<!-- kapanışta doldurulacak -->
