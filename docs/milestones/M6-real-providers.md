# M6 — Real Providers

> Task kırılımı 2026-09-11'de üretildi (M5 kapanış ritüelinin son maddesi —
> `docs/roadmap.md` → "Milestone kapanış ritüeli").

## Amaç

OpenSubtitles, OpenAI ve OpenRouter'ın gerçek entegrasyonu ve kullanıcı API anahtarlarının platform secure storage'ında saklanması.

## Kapsam

- OpenSubtitles resmi API (yalnız metadata kataloglama; indirme seçimde)
- Kesin hash eşleşmesinden gelen doğrulanmış medya kimliğinin macOS üst
  şeridinde sunulması
- OpenAI Responses API ve OpenRouter provider port implementasyonları
- Varsayılan gerçek çeviri yolu OpenRouter `openai/gpt-5.6-luna`; upstream
  yalnız OpenAI, doğrudan OpenAI adapter'ı ayrıca desteklenir
- OpenRouter structured-output capability preflight
- SecureCredentialStore: Keychain (bu milestone'da macOS)
- İndirme güvenliği: approved HTTPS host, bounded redirect, boyut sınırı, archive reddi
- Kalıcı blok checkpoint'i — yarıda kalan bir çeviri kaldığı yerden devam eder (ADR-0017 Karar 5, `NEN-095`'te ertelendi)
- Belge-geneli çeviri ilerleme yüzdesi — `NEN-102`'nin blok-yerel göstergesinin yerini alır (`NEN-095`/`NEN-102`'de ertelendi)
- Kimlik genişletmeleri: güven skoru/aday sıralaması, AI destekli release-name
  normalizasyonu (gizlilik ADR'siyle), tercih dilinde otomatik indirme (kendi
  ADR'siyle) — kullanıcı kararıyla M6'da (2026-09-11)
- Uzak medyada gömülü metin çıkarımı (`NEN-044`'ün kapsam dışı bıraktığı yol)

## Kapsam dışı

- Scraping — **yasak**
- Seçilmeden indirme — **yasak**
- Testlerde gerçek kredi/kota kullanımı — **yasak** (kabul task'ının kullanıcı
  onaylı gerçek koşusu test değil, checklist'tir; credential önkoşulu
  OpenSubtitles + seçili gerçek çeviri sağlayıcısıdır)
- Diğer platformların secure storage'ı → M9–M11
- User glossary (§10) — ADR-0019 erteler; `GlossaryIdentity::none()` kalır
- Altyazı yükleme (upload), kalıcı indirilen-altyazı önbelleği

## Çıkış kriterleri

- [ ] Katalogda gerçek OpenSubtitles adayları görünüyor, indirme yalnız seçimde
- [ ] Negatif: private file ID / hash / filename dışarı **sızmıyor** (opaque public ID)
- [ ] Negatif: approved liste dışı host'a istek atılmıyor
- [ ] Negatif: archive ve boyut aşımı reddediliyor
- [ ] API key Keychain'de; log/artifact/crash içinde **yok**
- [ ] Testler kayıtlı fixture ile çalışıyor, ağa çıkmıyor
- [ ] Belge-geneli çeviri ilerlemesi gerçek (ağ gecikmeli) bir koşuda okunabilir artıyor, geri gitmiyor
- [ ] Yarıda kesilen bir çeviri yeniden açılışta kaldığı bloktan devam ediyor; yarım artifact görünmüyor

## Kırılım öncesi kapanan kararlar

Kırılım üretilmeden önce dört soru kullanıcı kararıyla kapandı (2026-09-11):

| Soru | Karar | M6'ya etkisi |
|---|---|---|
| **S3** — dilsel kalite çıtası ve model seçimi | Yalnız EN→TR için yayın kalitesi: telif-temiz 24 cue; 24/24 zorunlu düzeltmesiz, anlam tersine dönmesi/atlama/ekleme/isim-terim tutarsızlığı 0. Varsayılan OpenRouter/Luna; başarısızlıkta ayrı task Terra yükseltmesi | `NEN-126` checklist'inde ölçümlü dilsel inceleme maddesi; [ADR-0019](../adr/0019-real-translation-provider-boundary.md) |
| **S9** — çoklu AI çevirisinin sunumu | Doğrulanmış artifact'ler hedef dil grubunda `AI · sağlayıcı · model profili` etiketiyle ayrı; en yeni başlangıçta seçili | `NEN-118` projeksiyona yansıtır; [ADR-0019](../adr/0019-real-translation-provider-boundary.md) |
| Gerçek sağlayıcıyla manuel kabul | Kullanıcının kendi anahtarıyla OpenRouter/Luna gerçek koşusu; credential yalnız OpenSubtitles + seçili çeviri sağlayıcısı; Luna geçmezse aynı korpusla ayrı Terra kabul koşusu | `NEN-126`'nın kapsamı; Kural 8 testler için değişmedi |
| `NEN-034` · `NEN-035` · `NEN-038` · `NEN-109` | Dördü de M6'da kalır | 034/038'in ön koşul ADR'leri ayrı task (`NEN-124`, `NEN-125`) |

## Task'lar

`NEN-033` (done) · `NEN-110` · `NEN-111` · `NEN-112` · `NEN-113` ·
`NEN-114` · `NEN-115` · `NEN-116` · `NEN-117` · `NEN-107` · `NEN-118` ·
`NEN-119` · `NEN-120` · `NEN-064` · `NEN-121` · `NEN-122` · `NEN-123` ·
`NEN-035` · `NEN-124` · `NEN-034` · `NEN-125` · `NEN-038` · `NEN-105` ·
`NEN-109` · `NEN-126`

```
110 ADR-0020 ─▶ 111 port+fake ─┬─▶ 112 Keychain ─▶ 113 ayarlar UI ─────────────────────┐
                               │                                                       │
                               ├─▶ 120 kimlik uygulamada ─▶ 064 kimlik şeridi ─────────┤
                               │        │                                              │
114 ADR-0019 ─▶ 115 HTTP POST ─┴─▶ 116 OpenAI ─▶ 117 OpenRouter ─┐                     │
                                                                  ├─▶ 118 gerçek env ─┬┴─▶ 126 kabul
102 ─▶ 107 belge-geneli ilerleme ─────────────────────────────────┘                   │
                                                                                      │
119 ADR-0021 ─┬─▶ 121 aday katalog ─▶ 122 güvenli indirme ─▶ 123 macOS menü ──────────┘
120 ──────────┤                              │
035 skor ─────┘                              └─▶ 125 ADR-0047 ─▶ 038 auto-download
124 ADR-0046 ─┬─▶ 034 AI release-name
116 ──────────┘
096 ─▶ 105 kalıcı checkpoint        044 ─▶ 109 uzak gömülü metin   (bağımsız)
```

Üç kol var ve üçü de credential kapısına (`NEN-110`/`NEN-111`) bağlanıyor:
**kimlik** (120 → 064 → 121), **OpenSubtitles** (119 → 121 → 122 → 123) ve
**çeviri** (114 → 115 → 116 → 117 → 118). `NEN-107` gerçek sağlayıcıdan
**önce** gelir — dakikalarca süren bir işte kullanıcı blok sırası değil yüzde
görmeli. `NEN-105`, `NEN-109`, `NEN-035` kritik yolun dışında.

### Çıkış kriteri → kanıt eşlemesi

| Çıkış kriteri | Kanıtlayan task |
|---|---|
| Katalogda gerçek adaylar, indirme yalnız seçimde | `NEN-121` (negatif: download çağrısı 0) · `NEN-122` · `NEN-123` · `NEN-126` |
| Private file ID / hash / filename sızmıyor | `NEN-120` · `NEN-121` (K23 guard) · `NEN-122` |
| Approved liste dışı host'a istek yok | `NEN-122` (negatif) · `NEN-116` / `NEN-117` (provider endpoint) · `NEN-115` |
| Archive ve boyut aşımı reddediliyor | `NEN-122` (negatif, zorunlu) |
| API key Keychain'de; log/artifact/crash'te yok | `NEN-112` · `NEN-113` (negatif) · `NEN-118` (artifact guard) · `NEN-126` |
| Testler fixture ile, ağa çıkmıyor | `NEN-116` / `NEN-117` / `NEN-121` / `NEN-122` (`fixtures/providers/` redakte replay); `NEN-126`'nın tek manuel koşusu test değil |
| Belge-geneli ilerleme okunabilir, geri gitmiyor | `NEN-107` (negatif: retry'da geri gitmez) · `NEN-126` (gerçek gecikmeyle) |
| Yarıda kesilen çeviri devam ediyor | `NEN-105` (negatif: yarım artifact yok) |

### ADR'ler

| ADR | Konu | Karar veren task |
|---|---|---|
| **0019** | Gerçek çeviri sağlayıcı sınırı: HTTP POST, structured output, retry, preflight, model; S3 · S9 | `NEN-114` |
| **0020** | Secure credential storage haritası: port, Keychain adapter, secret yaşam döngüsü | `NEN-110` |
| **0021** | OpenSubtitles entegrasyon sınırları: opaque public ID, arama, seçimde indirme, indirme güvenliği | `NEN-119` |
| **0040** | OpenSubtitles hash kimlik provider sınırı (`accepted`, 2026-09-06) | `NEN-033` |
| **0046** | Dosya adının AI normalizasyonuna gönderilmesi — gizlilik | `NEN-124` |
| **0047** | Tercih dilinde otomatik indirme sözleşmesi (ADR-0010 Karar 9'un üçüncü basamağı) | `NEN-125` |

Açılışta 0040 dışındakiler yazılmamış; her biri kendi task'ında `proposed`
yazılır, kullanıcı onayıyla `accepted` olur (Kural 4).

## Bağımlılıklar

M5

## Retro

<!-- kapanışta doldurulacak -->
